use rusqlite::{params, Connection, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Claim {
    pub id: i64,
    pub member_identifier: i64,
    pub insurance_row_id: i64,
    pub insurance_year: i32,
    pub occurred_on: Option<String>,
    pub reported_on: Option<String>,
    pub phone: Option<String>,
    pub employer: Option<String>,
    pub occupation: Option<String>,
    pub assessed_damage: Option<f64>,
    pub insurance_benefit: Option<f64>,
    pub description: Option<String>,
    pub note: Option<String>,
    pub additional_information: Option<String>,
    pub closed_on: Option<String>,
    pub handled_by: Option<String>,
    pub report_position: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimOverview {
    pub id: i64,
    pub member_row_id: i64,
    pub member_name: String,
    pub registration_number: String,
    pub organization_code: String,
    pub insurance_year: i32,
    pub occurred_on: Option<String>,
    pub reported_on: Option<String>,
    pub description: Option<String>,
    pub assessed_damage: Option<f64>,
    pub insurance_benefit: Option<f64>,
    pub status: String,
    pub last_changed: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewClaim {
    pub insurance_row_id: i64,
    pub occurred_on: Option<String>,
    pub reported_on: Option<String>,
    pub phone: Option<String>,
    pub employer: Option<String>,
    pub occupation: Option<String>,
    pub assessed_damage: Option<f64>,
    pub insurance_benefit: Option<f64>,
    pub description: Option<String>,
    pub note: Option<String>,
    pub additional_information: Option<String>,
    pub closed_on: Option<String>,
    pub handled_by: Option<String>,
    pub report_position: Option<String>,
}

pub fn ensure_schema(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        r#"CREATE TABLE IF NOT EXISTS "PojistneUdalosti" (
            "ID" INTEGER PRIMARY KEY,
            "IdentifikatorClena" INTEGER NOT NULL,
            "PojistnyZaznamRowId" INTEGER NOT NULL,
            "PojistnyRok" INTEGER NOT NULL,
            "Telefon" TEXT,
            "Zamestnavatel" TEXT,
            "Povolani" TEXT,
            "VznikPU" TEXT,
            "OznameniPU" TEXT,
            "ZjistenaSkoda" REAL,
            "PojistnePlneni" REAL,
            "PopisUdalosti" TEXT,
            "Poznamka1" TEXT,
            "Poznamka2" TEXT,
            "Ukonceno" TEXT,
            "ResiPojistovna" TEXT,
            "PolohaVSestave" TEXT,
            "Vytvoreno" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS "IX_PojistneUdalosti_Clen"
          ON "PojistneUdalosti" ("IdentifikatorClena", "PojistnyRok");"#,
    )
}

fn clean(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_string();
        (!value.is_empty()).then_some(value)
    })
}

pub fn list_for_member(connection: &Connection, identifier: i64) -> rusqlite::Result<Vec<Claim>> {
    let mut statement = connection.prepare(
        r#"SELECT "ID", "IdentifikatorClena", "PojistnyZaznamRowId", "PojistnyRok",
                  "VznikPU", "OznameniPU", "Telefon", "Zamestnavatel", "Povolani",
                  "ZjistenaSkoda", "PojistnePlneni", "PopisUdalosti", "Poznamka1",
                  "Poznamka2", "Ukonceno", "ResiPojistovna", "PolohaVSestave"
           FROM "PojistneUdalosti"
           WHERE "IdentifikatorClena" = ?1
           ORDER BY COALESCE("VznikPU", '') DESC, "ID" DESC"#,
    )?;
    let claims = statement
        .query_map([identifier], |row| {
            let closed_on: Option<String> = row.get(14)?;
            Ok(Claim {
                id: row.get(0)?,
                member_identifier: row.get(1)?,
                insurance_row_id: row.get(2)?,
                insurance_year: row.get(3)?,
                occurred_on: row.get(4)?,
                reported_on: row.get(5)?,
                phone: row.get(6)?,
                employer: row.get(7)?,
                occupation: row.get(8)?,
                assessed_damage: row.get(9)?,
                insurance_benefit: row.get(10)?,
                description: row.get(11)?,
                note: row.get(12)?,
                additional_information: row.get(13)?,
                status: if closed_on.as_deref().unwrap_or("").trim().is_empty() {
                    "Otevřená".into()
                } else {
                    "Uzavřená".into()
                },
                closed_on,
                handled_by: row.get(15)?,
                report_position: row.get(16)?,
            })
        })?
        .collect();
    claims
}

pub fn list_all(connection: &Connection) -> rusqlite::Result<Vec<ClaimOverview>> {
    let mut statement = connection.prepare(
        r#"SELECT claim."ID", member.rowid,
                  TRIM(COALESCE(member."Titul", '') || ' ' || COALESCE(member."Příjmení", '') || ' ' || COALESCE(member."Jméno", '')),
                  COALESCE(CAST(member."EvČíslo" AS TEXT), ''),
                  COALESCE(CAST(member."KódOC" AS TEXT), ''),
                  claim."PojistnyRok", claim."VznikPU", claim."OznameniPU",
                  claim."PopisUdalosti", claim."ZjistenaSkoda", claim."PojistnePlneni",
                  claim."Ukonceno", claim."Vytvoreno"
           FROM "PojistneUdalosti" claim
           JOIN "Seznam" member ON member.rowid = claim."PojistnyZaznamRowId"
           ORDER BY COALESCE(claim."VznikPU", claim."Vytvoreno") DESC, claim."ID" DESC"#,
    )?;
    let claims = statement
        .query_map([], |row| {
            let closed_on: Option<String> = row.get(11)?;
            Ok(ClaimOverview {
                id: row.get(0)?,
                member_row_id: row.get(1)?,
                member_name: row.get(2)?,
                registration_number: row.get(3)?,
                organization_code: row.get(4)?,
                insurance_year: row.get(5)?,
                occurred_on: row.get(6)?,
                reported_on: row.get(7)?,
                description: row.get(8)?,
                assessed_damage: row.get(9)?,
                insurance_benefit: row.get(10)?,
                status: if closed_on.as_deref().unwrap_or("").trim().is_empty() {
                    "Otevřená".into()
                } else {
                    "Uzavřená".into()
                },
                last_changed: row.get(12)?,
            })
        })?
        .collect();
    claims
}

pub fn create(
    database_path: &Path,
    member_identifier: i64,
    insurance_year: i32,
    input: NewClaim,
    user: &str,
) -> Result<i64, String> {
    if clean(input.occurred_on.clone()).is_none() {
        return Err("Vyplňte datum vzniku pojistné události.".into());
    }
    if clean(input.description.clone()).is_none() {
        return Err("Vyplňte popis pojistné události.".into());
    }
    let mut connection = Connection::open(database_path)
        .map_err(|_| "Pojistnou událost se nepodařilo uložit.".to_string())?;
    ensure_schema(&connection)
        .map_err(|_| "Pojistnou událost se nepodařilo uložit.".to_string())?;
    connection
        .execute_batch(
            r#"CREATE TABLE IF NOT EXISTS "AuditLog" (
            "Id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "DatumČas" TEXT NOT NULL,
            "Uživatel" TEXT NOT NULL,
            "Operace" TEXT NOT NULL,
            "IdentifikátorPojištěnce" TEXT,
            "Výsledek" TEXT NOT NULL
        );"#,
        )
        .map_err(|_| "Pojistnou událost se nepodařilo uložit.".to_string())?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| "Pojistnou událost se nepodařilo uložit.".to_string())?;
    let next_id: i64 = transaction
        .query_row(
            r#"SELECT MAX(COALESCE((SELECT MAX("ID") FROM "PojistneUdalosti"), 115), 115) + 1"#,
            [],
            |row| row.get(0),
        )
        .map_err(|_| "Pojistnou událost se nepodařilo uložit.".to_string())?;
    transaction
        .execute(
            r#"INSERT INTO "PojistneUdalosti" (
                "ID", "IdentifikatorClena", "PojistnyZaznamRowId", "PojistnyRok",
                "Telefon", "Zamestnavatel", "Povolani", "VznikPU", "OznameniPU",
                "ZjistenaSkoda", "PojistnePlneni", "PopisUdalosti", "Poznamka1",
                "Poznamka2", "Ukonceno", "ResiPojistovna", "PolohaVSestave"
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)"#,
            params![
                next_id,
                member_identifier,
                input.insurance_row_id,
                insurance_year,
                clean(input.phone),
                clean(input.employer),
                clean(input.occupation),
                clean(input.occurred_on),
                clean(input.reported_on),
                input.assessed_damage,
                input.insurance_benefit,
                clean(input.description),
                clean(input.note),
                clean(input.additional_information),
                clean(input.closed_on),
                clean(input.handled_by),
                clean(input.report_position),
            ],
        )
        .map_err(|_| "Pojistnou událost se nepodařilo uložit.".to_string())?;
    transaction
        .execute(
            r#"INSERT INTO "AuditLog"
               ("DatumČas", "Uživatel", "Operace", "IdentifikátorPojištěnce", "Výsledek")
               VALUES (CURRENT_TIMESTAMP, ?1, 'INSERT_CLAIM', ?2, 'OK')"#,
            params![user, member_identifier.to_string()],
        )
        .map_err(|_| "Pojistnou událost se nepodařilo uložit.".to_string())?;
    transaction
        .commit()
        .map_err(|_| "Pojistnou událost se nepodařilo uložit.".to_string())?;
    Ok(next_id)
}

pub fn update(
    database_path: &Path,
    id: i64,
    member_identifier: i64,
    insurance_year: i32,
    input: NewClaim,
    user: &str,
) -> Result<(), String> {
    if clean(input.occurred_on.clone()).is_none() {
        return Err("Vyplňte datum vzniku pojistné události.".into());
    }
    if clean(input.description.clone()).is_none() {
        return Err("Vyplňte popis pojistné události.".into());
    }
    let mut connection = Connection::open(database_path)
        .map_err(|_| "Pojistnou událost se nepodařilo upravit.".to_string())?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| "Pojistnou událost se nepodařilo upravit.".to_string())?;
    let changed = transaction
        .execute(
            r#"UPDATE "PojistneUdalosti" SET
                "Telefon"=?1, "Zamestnavatel"=?2, "Povolani"=?3, "VznikPU"=?4,
                "OznameniPU"=?5, "ZjistenaSkoda"=?6, "PojistnePlneni"=?7,
                "PopisUdalosti"=?8, "Poznamka1"=?9, "Poznamka2"=?10,
                "Ukonceno"=?11, "ResiPojistovna"=?12, "PolohaVSestave"=?13
               WHERE "ID"=?14 AND "IdentifikatorClena"=?15 AND "PojistnyRok"=?16"#,
            params![
                clean(input.phone),
                clean(input.employer),
                clean(input.occupation),
                clean(input.occurred_on),
                clean(input.reported_on),
                input.assessed_damage,
                input.insurance_benefit,
                clean(input.description),
                clean(input.note),
                clean(input.additional_information),
                clean(input.closed_on),
                clean(input.handled_by),
                clean(input.report_position),
                id,
                member_identifier,
                insurance_year,
            ],
        )
        .map_err(|_| "Pojistnou událost se nepodařilo upravit.".to_string())?;
    if changed != 1 {
        return Err("Pojistná událost nebyla nalezena.".into());
    }
    transaction
        .execute(
            r#"INSERT INTO "AuditLog"
               ("DatumČas", "Uživatel", "Operace", "IdentifikátorPojištěnce", "Výsledek")
               VALUES (CURRENT_TIMESTAMP, ?1, 'UPDATE_CLAIM', ?2, 'OK')"#,
            params![user, member_identifier.to_string()],
        )
        .map_err(|_| "Pojistnou událost se nepodařilo upravit.".to_string())?;
    transaction
        .commit()
        .map_err(|_| "Pojistnou událost se nepodařilo upravit.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn status_depends_only_on_closed_date() {
        assert_eq!(
            if Option::<String>::None.as_deref().unwrap_or("").is_empty() {
                "Otevřená"
            } else {
                "Uzavřená"
            },
            "Otevřená"
        );
        assert_eq!(
            if Some("2026-02-01").as_deref().unwrap_or("").is_empty() {
                "Otevřená"
            } else {
                "Uzavřená"
            },
            "Uzavřená"
        );
    }

    #[test]
    fn claim_creation_is_member_scoped_and_audited_without_description() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("pojisteni-claim-{unique}"));
        fs::create_dir_all(&directory).unwrap();
        let database = directory.join("claims.sqlite");
        let description = "Citlivý popis události";
        let id = create(
            &database,
            42,
            2026,
            NewClaim {
                insurance_row_id: 7,
                occurred_on: Some("2026-07-01".into()),
                reported_on: None,
                phone: None,
                employer: None,
                occupation: None,
                assessed_damage: None,
                insurance_benefit: None,
                description: Some(description.into()),
                note: None,
                additional_information: None,
                closed_on: None,
                handled_by: None,
                report_position: None,
            },
            "test-user",
        )
        .unwrap();
        assert_eq!(id, 116);
        let connection = Connection::open(&database).unwrap();
        assert_eq!(list_for_member(&connection, 42).unwrap().len(), 1);
        assert!(list_for_member(&connection, 43).unwrap().is_empty());
        let audit: (String, String, String) = connection
            .query_row(
                r#"SELECT "Operace", "IdentifikátorPojištěnce", "Výsledek" FROM "AuditLog""#,
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(audit, ("INSERT_CLAIM".into(), "42".into(), "OK".into()));
        let leaked: i64 = connection.query_row(
            r#"SELECT COUNT(*) FROM "AuditLog" WHERE CAST("IdentifikátorPojištěnce" AS TEXT) LIKE '%' || ?1 || '%'"#,
            [description],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(leaked, 0);
        drop(connection);
        fs::remove_dir_all(directory).unwrap();
    }
}
