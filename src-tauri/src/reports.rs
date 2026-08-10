use printpdf::{Mm, PdfDocument};
use rusqlite::{
    params,
    types::{Value, ValueRef},
    Connection,
};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufWriter, path::Path};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportFilter {
    pub kind: String,
    pub year: i32,
    pub organization_code: Option<String>,
    pub organization: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportPreview {
    pub title: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: usize,
}

pub fn ensure_schema(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        r#"CREATE TABLE IF NOT EXISTS "AuditSestav" (
        "Id" INTEGER PRIMARY KEY AUTOINCREMENT,
        "DatumCas" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        "Uzivatel" TEXT NOT NULL,
        "Druh" TEXT NOT NULL,
        "Format" TEXT NOT NULL,
        "PojistnyRok" INTEGER NOT NULL,
        "PocetRadku" INTEGER NOT NULL,
        "Vysledek" TEXT NOT NULL
    );"#,
    )
}

fn value(value: ValueRef<'_>) -> String {
    match value {
        ValueRef::Null => String::new(),
        ValueRef::Integer(v) => v.to_string(),
        ValueRef::Real(v) => format!("{v:.2}"),
        ValueRef::Text(v) => String::from_utf8_lossy(v).into_owned(),
        ValueRef::Blob(_) => "[binární data]".into(),
    }
}

fn definition(kind: &str) -> Result<(&'static str, &'static str), String> {
    match kind {
        "insurer" => Ok((
            "Přehled pro pojišťovnu",
            r#"SELECT "EvČíslo" AS "Ev. číslo", TRIM(COALESCE("Titul",'')||' '||COALESCE("Jméno",'')||' '||COALESCE("Příjmení",'')) AS "Pojištěnec", "KódOC" AS "OC", "ZO" AS "ZO", "Kategorie", "PojištěníOd" AS "Pojištění od", "Ukončení", "PojistnáČástka" AS "Pojistné", "SkutÚhrada" AS "Uhrazeno" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1"#,
        )),
        "oc" => Ok((
            "Sestava OC",
            r#"SELECT "KódOC" AS "OC", "EvČíslo" AS "Ev. číslo", "Příjmení", "Jméno", "ZO", "Kategorie", "RočPojistné" AS "Limit", "PojistnáČástka" AS "Pojistné", "SkutÚhrada" AS "Uhrazeno" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND (?2='' OR CAST("KódOC" AS TEXT)=?2)"#,
        )),
        "zo" => Ok((
            "Sestava ZO",
            r#"SELECT "ZO", "EvČíslo" AS "Ev. číslo", "Příjmení", "Jméno", "Kategorie", "PojištěníOd" AS "Od", "PojištěníDo" AS "Do", "PojistnáČástka" AS "Pojistné", "SkutÚhrada" AS "Uhrazeno" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND (?3='' OR "ZO"=?3)"#,
        )),
        "claims" => Ok((
            "Pojistné události / HVP",
            r#"SELECT claim."ID", member."EvČíslo" AS "Ev. číslo", member."Příjmení", member."Jméno", member."KódOC" AS "OC", claim."VznikPU" AS "Vznik", claim."OznameniPU" AS "Oznámení", claim."ZjistenaSkoda" AS "Škoda", claim."PojistnePlneni" AS "Plnění", CASE WHEN claim."Ukonceno" IS NULL OR TRIM(claim."Ukonceno")='' THEN 'Otevřená' ELSE 'Uzavřená' END AS "Stav" FROM "PojistneUdalosti" claim JOIN "Seznam" member ON member.rowid=claim."PojistnyZaznamRowId" WHERE claim."PojistnyRok"=?1 AND (?2='' OR CAST(member."KódOC" AS TEXT)=?2)"#,
        )),
        "starts" => Ok((
            "Počátky pojištění",
            r#"SELECT "PojištěníOd" AS "Počátek", "EvČíslo" AS "Ev. číslo", "Příjmení", "Jméno", "KódOC" AS "OC", "ZO", "Kategorie", "PojistnáČástka" AS "Pojistné" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND (?4='' OR date("PojištěníOd")>=date(?4)) AND (?5='' OR date("PojištěníOd")<=date(?5))"#,
        )),
        "terminations" => Ok((
            "Ukončení pojištění",
            r#"SELECT "Ukončení", "EvČíslo" AS "Ev. číslo", "Příjmení", "Jméno", "KódOC" AS "OC", "ZO", "Kategorie" FROM "Seznam" WHERE pojisteni_rok("Ukončení")=?1 AND (?4='' OR date("Ukončení")>=date(?4)) AND (?5='' OR date("Ukončení")<=date(?5))"#,
        )),
        "contacts" => Ok((
            "Kontaktní seznam",
            r#"SELECT "EvČíslo" AS "Ev. číslo", "Příjmení", "Jméno", "e-mail" AS "E-mail", "Adresa", "Město", "PSČ", "ZO", "KódOC" AS "OC" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND (?2='' OR CAST("KódOC" AS TEXT)=?2) AND (?3='' OR "ZO"=?3)"#,
        )),
        "duplicates" => Ok((
            "Kontrola duplicit",
            r#"SELECT "RodnéČíslo" AS "Rodné číslo", COUNT(*) AS "Počet", GROUP_CONCAT("EvČíslo", ', ') AS "Evidenční čísla", GROUP_CONCAT(TRIM(COALESCE("Jméno",'')||' '||COALESCE("Příjmení",'')), '; ') AS "Jména" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND NULLIF(TRIM("RodnéČíslo"),'') IS NOT NULL GROUP BY TRIM("RodnéČíslo") HAVING COUNT(*)>1"#,
        )),
        "quality" => Ok((
            "Kontrola kvality dat",
            r#"SELECT rowid AS "Row ID", "EvČíslo" AS "Ev. číslo", "Příjmení", "Jméno", CASE WHEN NULLIF(TRIM("EvČíslo"),'') IS NULL THEN 'Chybí evidenční číslo' WHEN NULLIF(TRIM("RodnéČíslo"),'') IS NULL THEN 'Chybí rodné číslo' WHEN NULLIF(TRIM("Příjmení"),'') IS NULL OR NULLIF(TRIM("Jméno"),'') IS NULL THEN 'Chybí jméno' WHEN COALESCE("PojistnáČástka",0)<=0 THEN 'Chybí pojistné' WHEN COALESCE("SkutÚhrada",0)>COALESCE("PojistnáČástka",0) THEN 'Přeplatek' ELSE 'Jiný nesoulad' END AS "Problém" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND (NULLIF(TRIM("EvČíslo"),'') IS NULL OR NULLIF(TRIM("RodnéČíslo"),'') IS NULL OR NULLIF(TRIM("Příjmení"),'') IS NULL OR NULLIF(TRIM("Jméno"),'') IS NULL OR COALESCE("PojistnáČástka",0)<=0 OR COALESCE("SkutÚhrada",0)>COALESCE("PojistnáČástka",0))"#,
        )),
        "member" => Ok((
            "Parametrická sestava člena",
            r#"SELECT "EvČíslo" AS "Ev. číslo","Příjmení","Jméno","RodnéČíslo" AS "Rodné číslo","KódOC" AS "OC","ZO","PojištěníOd" AS "Od","PojištěníDo" AS "Do","RočPojistné" AS "Limit","Kategorie","PojistnáČástka" AS "Pojistné","SkutÚhrada" AS "Uhrazeno","Ukončení" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND (?6='' OR "Příjmení" LIKE '%'||?6||'%' COLLATE NOCASE OR "Jméno" LIKE '%'||?6||'%' COLLATE NOCASE OR CAST("EvČíslo" AS TEXT)=?6 OR "RodnéČíslo" LIKE '%'||?6||'%')"#,
        )),
        _ => Err("Neznámý druh sestavy.".into()),
    }
}

pub fn preview(connection: &Connection, filter: &ReportFilter) -> Result<ReportPreview, String> {
    ensure_schema(connection).map_err(|_| "Sestavu se nepodařilo připravit.".to_string())?;
    let (title, sql) = definition(&filter.kind)?;
    let oc = filter.organization_code.as_deref().unwrap_or_default();
    let org = filter.organization.as_deref().unwrap_or_default();
    let from = filter.date_from.as_deref().unwrap_or_default();
    let to = filter.date_to.as_deref().unwrap_or_default();
    let search = filter.search.as_deref().unwrap_or_default();
    let mut statement = connection
        .prepare(sql)
        .map_err(|_| "Sestavu se nepodařilo připravit.".to_string())?;
    let columns = statement
        .column_names()
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>();
    let parameter_count = statement.parameter_count();
    let values = vec![
        Value::Integer(filter.year as i64),
        Value::Text(oc.into()),
        Value::Text(org.into()),
        Value::Text(from.into()),
        Value::Text(to.into()),
        Value::Text(search.into()),
    ];
    let rows = statement
        .query_map(
            rusqlite::params_from_iter(values.into_iter().take(parameter_count)),
            |row| {
                (0..columns.len())
                    .map(|index| row.get_ref(index).map(value))
                    .collect::<rusqlite::Result<Vec<_>>>()
            },
        )
        .and_then(|rows| rows.collect::<rusqlite::Result<Vec<_>>>())
        .map_err(|_| "Sestavu se nepodařilo načíst.".to_string())?;
    Ok(ReportPreview {
        title: title.into(),
        columns,
        total_rows: rows.len(),
        rows,
    })
}

fn csv_escape(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

pub fn csv(report: &ReportPreview) -> Vec<u8> {
    let mut output = String::from("\u{feff}");
    output.push_str(
        &report
            .columns
            .iter()
            .map(|v| csv_escape(v))
            .collect::<Vec<_>>()
            .join(";"),
    );
    output.push_str("\r\n");
    for row in &report.rows {
        output.push_str(
            &row.iter()
                .map(|v| csv_escape(v))
                .collect::<Vec<_>>()
                .join(";"),
        );
        output.push_str("\r\n");
    }
    output.into_bytes()
}

pub fn pdf(report: &ReportPreview, destination: &Path) -> Result<(), String> {
    let (document, page, layer) = PdfDocument::new(&report.title, Mm(297.0), Mm(210.0), "Sestava");
    let font = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\arial.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let mut current_page = page;
    let mut current_layer = layer;
    let mut y = 198.0;
    for (index, line) in std::iter::once(report.title.clone())
        .chain(std::iter::once(report.columns.join(" | ")))
        .chain(report.rows.iter().map(|r| r.join(" | ")))
        .enumerate()
    {
        if y < 12.0 {
            let next = document.add_page(Mm(297.0), Mm(210.0), "Sestava");
            current_page = next.0;
            current_layer = next.1;
            y = 198.0;
        }
        let text: String = line.chars().take(180).collect();
        document
            .get_page(current_page)
            .get_layer(current_layer)
            .use_text(
                text,
                if index == 0 { 14.0 } else { 7.0 },
                Mm(8.0),
                Mm(y),
                &font,
            );
        y -= if index == 0 { 10.0 } else { 5.0 };
    }
    document
        .save(&mut BufWriter::new(
            File::create(destination).map_err(|_| "PDF se nepodařilo uložit.".to_string())?,
        ))
        .map_err(|_| "PDF se nepodařilo vytvořit.".to_string())
}

pub fn audit(
    connection: &Connection,
    user: &str,
    filter: &ReportFilter,
    format: &str,
    count: usize,
) {
    let _=connection.execute(r#"INSERT INTO "AuditSestav"("Uzivatel","Druh","Format","PojistnyRok","PocetRadku","Vysledek") VALUES(?1,?2,?3,?4,?5,'OK')"#, params![user,filter.kind,format,filter.year,count as i64]);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csv_quotes_semicolons_and_quotes() {
        let report = ReportPreview {
            title: "Test".into(),
            columns: vec!["A".into()],
            rows: vec![vec!["x;y\"z".into()]],
            total_rows: 1,
        };
        let output = String::from_utf8(csv(&report)).unwrap();
        assert!(output.contains("\"x;y\"\"z\""));
    }
}
