use rusqlite::{params, Connection, TransactionBehavior};
use serde::Serialize;
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Clone)]
struct ImportRow {
    first: String,
    last: String,
    personal: String,
    registration: i64,
    oc: String,
    organization: String,
    from: String,
    to: String,
    limit: i64,
    category: String,
    loss: bool,
    premium: i64,
    email: String,
    address: String,
    city: String,
    postal: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub path: String,
    pub total: usize,
    pub valid: usize,
    pub skipped_duplicates: usize,
    pub errors: Vec<String>,
    pub sample: Vec<Vec<String>>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub inserted: usize,
    pub skipped_duplicates: usize,
}

fn parse_line(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut value = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            if quoted && chars.peek() == Some(&'"') {
                value.push('"');
                chars.next();
            } else {
                quoted = !quoted;
            }
        } else if c == ';' && !quoted {
            out.push(value.trim().to_string());
            value.clear();
        } else {
            value.push(c);
        }
    }
    out.push(value.trim().to_string());
    out
}
fn key(value: &str) -> String {
    deunicode::deunicode(value)
        .to_lowercase()
        .replace([' ', '.', '_', '-'], "")
}
fn field<'a>(map: &HashMap<String, usize>, row: &'a [String], names: &[&str]) -> &'a str {
    names
        .iter()
        .find_map(|n| map.get(&key(n)).and_then(|i| row.get(*i)))
        .map(String::as_str)
        .unwrap_or("")
}
fn number(value: &str) -> i64 {
    value
        .replace(' ', "")
        .replace(',', ".")
        .split('.')
        .next()
        .unwrap_or("")
        .parse()
        .unwrap_or(0)
}
fn read(path: &Path) -> Result<(Vec<ImportRow>, Vec<String>, Vec<Vec<String>>), String> {
    let bytes = fs::read(path).map_err(|_| "Importní soubor se nepodařilo načíst.".to_string())?;
    let text = String::from_utf8(bytes).map_err(|_| "Import musí být CSV v UTF-8.".to_string())?;
    let mut lines = text.trim_start_matches('\u{feff}').lines();
    let headers = parse_line(lines.next().ok_or("Importní soubor je prázdný.")?);
    let map = headers
        .iter()
        .enumerate()
        .map(|(i, h)| (key(h), i))
        .collect::<HashMap<_, _>>();
    let mut rows = Vec::new();
    let mut errors = Vec::new();
    let mut sample = Vec::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let values = parse_line(line);
        let first = field(&map, &values, &["Jméno", "Jmeno"]).trim().to_string();
        let last = field(&map, &values, &["Příjmení", "Prijmeni"])
            .trim()
            .to_string();
        let personal = field(&map, &values, &["Rodné číslo", "Rodne cislo"])
            .trim()
            .to_string();
        let registration = number(field(
            &map,
            &values,
            &["Evidenční číslo", "Ev. číslo", "EvCislo"],
        ));
        let from = field(&map, &values, &["Pojištění od", "PojisteniOd"])
            .trim()
            .to_string();
        let to = field(&map, &values, &["Pojištění do", "PojisteniDo"])
            .trim()
            .to_string();
        let category = field(&map, &values, &["Kategorie"]).trim().to_uppercase();
        let limit = number(field(
            &map,
            &values,
            &["Pojistná částka", "Limit", "RocPojistne"],
        ));
        let premium = number(field(&map, &values, &["Pojistné", "PojistnaCastka"]));
        if first.is_empty()
            || last.is_empty()
            || personal.is_empty()
            || registration <= 0
            || from.len() < 10
            || to.len() < 10
            || !matches!(category.as_str(), "A" | "B" | "C")
            || limit <= 0
            || premium <= 0
        {
            errors.push(format!(
                "Řádek {} nemá všechna povinná platná pole.",
                index + 2
            ));
            continue;
        }
        let item = ImportRow {
            first,
            last,
            personal,
            registration,
            oc: field(&map, &values, &["OC", "Kód OC"]).trim().to_string(),
            organization: field(&map, &values, &["ZO", "Organizace"])
                .trim()
                .to_string(),
            from,
            to,
            limit,
            category,
            loss: matches!(
                field(&map, &values, &["Ztráta", "Ztrata"])
                    .to_lowercase()
                    .as_str(),
                "1" | "ano" | "true"
            ),
            premium,
            email: field(&map, &values, &["E-mail", "Email"])
                .trim()
                .to_string(),
            address: field(&map, &values, &["Adresa"]).trim().to_string(),
            city: field(&map, &values, &["Město", "Mesto"]).trim().to_string(),
            postal: field(&map, &values, &["PSČ", "PSC"]).trim().to_string(),
        };
        if sample.len() < 20 {
            sample.push(vec![
                item.registration.to_string(),
                format!("{} {}", item.first, item.last),
                item.personal.clone(),
                item.from.clone(),
                item.organization.clone(),
            ]);
        }
        rows.push(item)
    }
    Ok((rows, errors, sample))
}
fn year(date: &str) -> i32 {
    date.get(0..4).and_then(|v| v.parse().ok()).unwrap_or(0)
}
fn duplicate(connection: &Connection, row: &ImportRow) -> bool {
    connection.query_row(r#"SELECT EXISTS(SELECT 1 FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=?1 AND (TRIM(COALESCE("RodnéČíslo",''))=?2 OR CAST("EvČíslo" AS INTEGER)=?3))"#,params![year(&row.from),row.personal,row.registration],|r|r.get::<_,i64>(0)).unwrap_or(1)!=0
}
pub fn preview(connection: &Connection, path: &Path) -> Result<ImportPreview, String> {
    let (rows, errors, sample) = read(path)?;
    let duplicates = rows.iter().filter(|r| duplicate(connection, r)).count();
    Ok(ImportPreview {
        path: path.to_string_lossy().into_owned(),
        total: rows.len() + errors.len(),
        valid: rows.len() - duplicates,
        skipped_duplicates: duplicates,
        errors,
        sample,
    })
}
pub fn execute(
    connection: &mut Connection,
    path: &Path,
    user: &str,
) -> Result<ImportResult, String> {
    let (rows, errors, _) = read(path)?;
    if !errors.is_empty() {
        return Err("Import obsahuje neplatné řádky; nejprve opravte náhled.".into());
    }
    let skipped = rows.iter().filter(|r| duplicate(connection, r)).count();
    let insert = rows
        .into_iter()
        .filter(|r| !duplicate(connection, r))
        .collect::<Vec<_>>();
    let max: i64 = connection
        .query_row(
            r#"SELECT COALESCE(MAX("Identifikátor"),0) FROM "Seznam""#,
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| "Import se nepodařilo zahájit.".to_string())?;
    for (i, r) in insert.iter().enumerate() {
        tx.execute(r#"INSERT INTO "Seznam"("Identifikátor","Jméno","Příjmení","RodnéČíslo","EvČíslo","KódOC","ZO","Adresa","Město","PSČ","e-mail","PojištěníOd","PojištěníDo","RočPojistné","Kategorie","Ztráta","PojistnáČástka","SkutÚhrada","Doklad") VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,NULLIF(?11,''),?12,?13,?14,?15,?16,?17,0,0)"#,params![max+i as i64+1,r.first,r.last,r.personal,r.registration,r.oc,r.organization,r.address,r.city,r.postal,r.email,r.from,r.to,r.limit,r.category,r.loss as i64,r.premium]).map_err(|_|format!("Řádek {} se nepodařilo uložit.",i+2))?;
    }
    tx.execute(r#"INSERT INTO "AuditLog"("DatumČas","Uživatel","Operace","IdentifikátorPojištěnce","Výsledek") VALUES(CURRENT_TIMESTAMP,?1,'IMPORT',NULL,?2)"#,params![user,format!("OK:{}",insert.len())]).map_err(|_|"Import se nepodařilo auditovat.".to_string())?;
    tx.commit()
        .map_err(|_| "Import se nepodařilo dokončit.".to_string())?;
    Ok(ImportResult {
        inserted: insert.len(),
        skipped_duplicates: skipped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parser_handles_quoted_semicolon() {
        assert_eq!(parse_line("\"Novák; Jan\";1"), vec!["Novák; Jan", "1"]);
    }
}
