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
            r#"SELECT "EvČíslo" AS "Evid. číslo", TRIM(COALESCE("Titul",'')||' '||COALESCE("Příjmení",'')||' '||COALESCE("Jméno",'')) AS "Příjmení a jméno", "RodnéČíslo" AS "Rodné číslo", TRIM(COALESCE("Adresa",'')||', '||COALESCE("Město",'')) AS "Bydliště", "PSČ", "Kategorie" AS "Pojištění", "PojistnáČástka" AS "Částka", CASE WHEN "Stát"='Slovenská republika' THEN 'SK' ELSE '' END AS "Stát" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 ORDER BY "KódOC", "Příjmení", "Jméno""#,
        )),
        "oc" => Ok((
            "Sestava OC",
            r#"SELECT "EvČíslo" AS "Evid. číslo", TRIM(COALESCE("Příjmení",'')||' '||COALESCE("Jméno",'')) AS "Příjmení a jméno", "RodnéČíslo" AS "Rod. číslo", "PojistnáČástka" AS "Částka", TRIM(COALESCE("Adresa",'')||', '||COALESCE("Město",'')) AS "Adresa trvalého bydliště", "RočPojistné" AS "Roč. pojistné", "Kategorie" AS "Kateg." FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND (?2='' OR CAST("KódOC" AS TEXT)=?2) ORDER BY "EvČíslo""#,
        )),
        "zo" => Ok((
            "Sestava ZO",
            r#"SELECT "EvČíslo" AS "Evid. číslo", TRIM(COALESCE("Příjmení",'')||' '||COALESCE("Jméno",'')) AS "Příjmení a jméno", "RodnéČíslo" AS "Rod. číslo", "PojistnáČástka" AS "Částka", TRIM(COALESCE("Adresa",'')||', '||COALESCE("Město",'')) AS "Adresa trvalého bydliště", "RočPojistné" AS "Roč. pojistné", "Kategorie" AS "Kateg.", CASE WHEN UPPER(COALESCE("Kategorie",'')) LIKE '%Z%' THEN 'Ano' ELSE '' END AS "Ztráta", "Poznámka" FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND (?3='' OR "ZO"=?3) ORDER BY "EvČíslo""#,
        )),
        "claims" => Ok((
            "Pojistné události / HVP",
            r#"SELECT claim."ID" AS "Poř.", TRIM(COALESCE(member."Titul",'')||' '||COALESCE(member."Příjmení",'')||' '||COALESCE(member."Jméno",'')) AS "Pojištěnec", member."EvČíslo" AS "Evid", 'Kategorie '||COALESCE(member."Kategorie",'')||'; roč. '||COALESCE(member."RočPojistné",0) AS "Typ pojištění", COALESCE(member."PojištěníOd",'')||' - '||COALESCE(member."PojištěníDo",'') AS "Pojistné období", claim."VznikPU" AS "PU ze dne", claim."ZjistenaSkoda" AS "Škoda", COALESCE(member."OdbPříslušnost",'')||CASE WHEN NULLIF(TRIM(COALESCE(member."ZO",'')),'') IS NULL THEN '' ELSE ' - ZO '||member."ZO" END AS "Odborová organizace", claim."Ukonceno" AS "Vypořádáno", claim."PopisUdalosti" AS "Popis události" FROM "PojistneUdalosti" claim JOIN "Seznam" member ON member.rowid=claim."PojistnyZaznamRowId" WHERE claim."PojistnyRok"=?1 AND (?2='' OR CAST(member."KódOC" AS TEXT)=?2) ORDER BY COALESCE(claim."PolohaVSestave", claim."ID"), claim."ID""#,
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
    if report.title == "Pojistné události / HVP" {
        return claims_pdf(report, destination);
    }
    if matches!(
        report.title.as_str(),
        "Sestava OC" | "Sestava ZO" | "Přehled pro pojišťovnu"
    ) {
        return access_table_pdf(report, destination);
    }
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

fn report_column_widths(title: &str) -> &'static [f32] {
    match title {
        "Sestava OC" => &[18.0, 39.0, 27.0, 20.0, 75.0, 25.0, 18.0],
        "Sestava ZO" => &[17.0, 36.0, 25.0, 19.0, 59.0, 23.0, 16.0, 16.0, 28.0],
        _ => &[19.0, 45.0, 29.0, 72.0, 17.0, 24.0, 22.0, 12.0],
    }
}

fn access_table_pdf(report: &ReportPreview, destination: &Path) -> Result<(), String> {
    let (document, first_page, first_layer) =
        PdfDocument::new(&report.title, Mm(297.0), Mm(210.0), "Sestava");
    let regular = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\arial.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let bold = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\arialbd.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let widths = report_column_widths(&report.title);
    let mut page = first_page;
    let mut layer = first_layer;
    let mut page_number = 1;
    let mut y = 178.0;
    let draw_header = |page, layer, page_number: usize| {
        let current = document.get_page(page).get_layer(layer);
        current.use_text(
            "S dokumentem je nutno nakládat v souladu s pravidly ochrany osobních údajů.",
            6.0,
            Mm(64.0),
            Mm(204.0),
            &regular,
        );
        current.use_text(report.title.to_uppercase(), 16.0, Mm(7.0), Mm(194.0), &bold);
        let mut x = 7.0;
        for (index, column) in report.columns.iter().enumerate() {
            let width = widths.get(index).copied().unwrap_or(25.0);
            current.use_text(
                clipped(column, (width / 2.0).max(5.0) as usize),
                7.0,
                Mm(x),
                Mm(184.0),
                &bold,
            );
            x += width;
        }
        current.use_text(
            format!("Strana {page_number}"),
            7.0,
            Mm(270.0),
            Mm(7.0),
            &regular,
        );
    };
    draw_header(page, layer, page_number);
    for row in &report.rows {
        if y < 18.0 {
            let next = document.add_page(Mm(297.0), Mm(210.0), "Sestava");
            page = next.0;
            layer = next.1;
            page_number += 1;
            y = 178.0;
            draw_header(page, layer, page_number);
        }
        let current = document.get_page(page).get_layer(layer);
        let mut x = 7.0;
        for (index, value) in row.iter().enumerate() {
            let width = widths.get(index).copied().unwrap_or(25.0);
            current.use_text(
                clipped(value, (width / 1.7).max(5.0) as usize),
                7.0,
                Mm(x),
                Mm(y),
                &regular,
            );
            x += width;
        }
        y -= 5.0;
    }
    let current = document.get_page(page).get_layer(layer);
    current.use_text(
        format!("Úhrnem počet: {}", report.total_rows),
        9.0,
        Mm(7.0),
        Mm(y.max(12.0)),
        &bold,
    );
    document
        .save(&mut BufWriter::new(
            File::create(destination).map_err(|_| "PDF se nepodařilo uložit.".to_string())?,
        ))
        .map_err(|_| "PDF se nepodařilo vytvořit.".to_string())
}

fn clipped(value: &str, limit: usize) -> String {
    let mut chars = value.chars();
    let text: String = chars.by_ref().take(limit).collect();
    if chars.next().is_some() {
        format!("{text}...")
    } else {
        text
    }
}

fn claims_pdf(report: &ReportPreview, destination: &Path) -> Result<(), String> {
    let (document, first_page, first_layer) = PdfDocument::new(
        "SESTAVA POJISTNÝCH UDÁLOSTÍ",
        Mm(297.0),
        Mm(210.0),
        "Sestava",
    );
    let regular = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\arial.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let bold = document
        .add_external_font(
            File::open(r"C:\Windows\Fonts\arialbd.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let mut page = first_page;
    let mut layer = first_layer;
    let mut y = 183.0;
    let mut page_number = 1;
    let draw_header = |page, layer, page_number: usize| {
        let current = document.get_page(page).get_layer(layer);
        current.use_text(
            "S dokumentem je nutno nakládat v souladu s pravidly ochrany osobních údajů.",
            6.0,
            Mm(64.0),
            Mm(204.0),
            &regular,
        );
        current.use_text(
            "SESTAVA POJISTNÝCH UDÁLOSTÍ",
            17.0,
            Mm(7.0),
            Mm(194.0),
            &bold,
        );
        current.use_text(
            format!("Strana {page_number}"),
            7.0,
            Mm(270.0),
            Mm(7.0),
            &regular,
        );
    };
    draw_header(page, layer, page_number);
    for row in &report.rows {
        if y < 34.0 {
            let next = document.add_page(Mm(297.0), Mm(210.0), "Sestava");
            page = next.0;
            layer = next.1;
            page_number += 1;
            y = 183.0;
            draw_header(page, layer, page_number);
        }
        let current = document.get_page(page).get_layer(layer);
        let at = |index: usize| row.get(index).map(String::as_str).unwrap_or("");
        current.use_text(
            format!("Poř.: {}   Pojištěnec: {}", at(0), clipped(at(1), 38)),
            7.0,
            Mm(7.0),
            Mm(y),
            &bold,
        );
        current.use_text(
            format!(
                "Evid: {}   Typ pojištění: {}   Období: {}",
                at(2),
                clipped(at(3), 28),
                clipped(at(4), 24)
            ),
            7.0,
            Mm(72.0),
            Mm(y),
            &regular,
        );
        current.use_text(
            format!(
                "PU: {}   Škoda: {}   Organizace: {}   Vypořádáno: {}",
                at(5),
                at(6),
                clipped(at(7), 24),
                at(8)
            ),
            7.0,
            Mm(184.0),
            Mm(y),
            &regular,
        );
        y -= 5.0;
        current.use_text("Popis události:", 7.0, Mm(7.0), Mm(y), &bold);
        y -= 4.0;
        let description = at(9).replace(['\r', '\n'], " ");
        let characters: Vec<char> = description.chars().collect();
        for chunk in characters.chunks(180).take(3) {
            current.use_text(
                chunk.iter().collect::<String>(),
                7.0,
                Mm(7.0),
                Mm(y),
                &regular,
            );
            y -= 4.0;
        }
        y -= 4.0;
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

    #[test]
    fn claims_pdf_uses_access_style_grouped_layout() {
        let directory = tempfile::tempdir().unwrap();
        let destination = std::env::var_os("FED_PDF_QA_DIR")
            .map(std::path::PathBuf::from)
            .map(|path| path.join("claims-report.pdf"))
            .unwrap_or_else(|| directory.path().join("claims.pdf"));
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let report = ReportPreview {
            title: "Pojistné události / HVP".into(),
            columns: vec![
                "Poř.".into(),
                "Pojištěnec".into(),
                "Evid".into(),
                "Typ pojištění".into(),
                "Pojistné období".into(),
                "PU ze dne".into(),
                "Škoda".into(),
                "Odborová organizace".into(),
                "Vypořádáno".into(),
                "Popis události".into(),
            ],
            rows: vec![vec![
                "1".into(),
                "Jan Novák".into(),
                "10001".into(),
                "Kategorie A; roč. 320000".into(),
                "01.01.2024 - 31.12.2024".into(),
                "10.06.2024".into(),
                "15000".into(),
                "FVČ - ZO PRAHA".into(),
                "".into(),
                "Popis testovací události".into(),
            ]],
            total_rows: 1,
        };
        pdf(&report, &destination).unwrap();
        let bytes = std::fs::read(destination).unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
        assert!(bytes.len() > 1_000);
    }

    #[test]
    fn access_table_pdf_has_headers_and_rows() {
        let directory = tempfile::tempdir().unwrap();
        let destination = std::env::var_os("FED_PDF_QA_DIR")
            .map(std::path::PathBuf::from)
            .map(|path| path.join("oc-report.pdf"))
            .unwrap_or_else(|| directory.path().join("oc.pdf"));
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let report = ReportPreview {
            title: "Sestava OC".into(),
            columns: vec![
                "Evid. číslo".into(),
                "Příjmení a jméno".into(),
                "Rod. číslo".into(),
                "Částka".into(),
                "Adresa trvalého bydliště".into(),
                "Roč. pojistné".into(),
                "Kateg.".into(),
            ],
            rows: vec![vec![
                "10001".into(),
                "Jan Novák".into(),
                "800101/0000".into(),
                "1500".into(),
                "Hlavní 1, Praha".into(),
                "320000".into(),
                "A".into(),
            ]],
            total_rows: 1,
        };
        pdf(&report, &destination).unwrap();
        assert!(std::fs::metadata(destination).unwrap().len() > 1_000);
    }
}
