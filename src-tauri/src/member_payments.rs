use rusqlite::{params, Connection, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberPayment {
    pub id: i64,
    pub received_on: String,
    pub amount: i64,
    pub insurance_year: i32,
    pub method: String,
    pub variable_symbol: String,
    pub note: Option<String>,
    pub status: String,
    pub imported_from_bank: bool,
    pub bank_transaction_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInput {
    pub id: Option<i64>,
    pub insurance_row_id: i64,
    pub received_on: String,
    pub amount: i64,
    pub method: String,
    pub note: Option<String>,
}

pub fn ensure_schema(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        r#"CREATE TABLE IF NOT EXISTS "PlatbyClenu" (
            "Id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "IdentifikatorClena" TEXT NOT NULL,
            "PojistnyZaznamRowId" INTEGER NOT NULL,
            "PojistnyRok" INTEGER NOT NULL,
            "DatumPrijeti" TEXT NOT NULL,
            "Castka" INTEGER NOT NULL CHECK ("Castka" > 0),
            "ZpusobUhrady" TEXT NOT NULL,
            "VariabilniSymbol" TEXT NOT NULL,
            "Poznamka" TEXT,
            "Stav" TEXT NOT NULL DEFAULT 'Zaúčtováno',
            "ImportovanoZBanky" INTEGER NOT NULL DEFAULT 0,
            "IdBankovniTransakce" TEXT UNIQUE,
            "Vytvoreno" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "Aktualizovano" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS "IX_PlatbyClenu_ClenRok"
          ON "PlatbyClenu" ("IdentifikatorClena", "PojistnyRok", "DatumPrijeti");
        CREATE TABLE IF NOT EXISTS "AuditPlateb" (
            "Id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "DatumCas" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "Uzivatel" TEXT NOT NULL,
            "IdentifikatorClena" TEXT NOT NULL,
            "PojistnyRok" INTEGER NOT NULL,
            "Castka" INTEGER NOT NULL,
            "Operace" TEXT NOT NULL
        );"#,
    )
}

pub fn bootstrap_legacy_payment(
    connection: &Connection,
    row_id: i64,
    member_identifier: &str,
    insurance_year: i32,
    variable_symbol: &str,
    existing_total: i64,
) -> rusqlite::Result<()> {
    if existing_total <= 0 {
        return Ok(());
    }
    connection.execute(
        r#"INSERT INTO "PlatbyClenu"
           ("IdentifikatorClena", "PojistnyZaznamRowId", "PojistnyRok", "DatumPrijeti",
            "Castka", "ZpusobUhrady", "VariabilniSymbol", "Poznamka", "Stav")
           SELECT ?1, ?2, ?3, printf('%04d-01-01', ?3),
                  ?4, 'Jiné', ?5, 'Převedeno z původní evidence', 'Zaúčtováno'
           WHERE NOT EXISTS (SELECT 1 FROM "PlatbyClenu" WHERE "PojistnyZaznamRowId" = ?2)"#,
        params![
            member_identifier,
            row_id,
            insurance_year,
            existing_total,
            variable_symbol
        ],
    )?;
    Ok(())
}

pub fn list(connection: &Connection, row_id: i64) -> rusqlite::Result<Vec<MemberPayment>> {
    let mut statement = connection.prepare(
        r#"SELECT "Id", "DatumPrijeti", "Castka", "PojistnyRok", "ZpusobUhrady",
                  "VariabilniSymbol", "Poznamka", "Stav", "ImportovanoZBanky", "IdBankovniTransakce"
           FROM "PlatbyClenu" WHERE "PojistnyZaznamRowId" = ?1
           ORDER BY "DatumPrijeti" DESC, "Id" DESC"#,
    )?;
    let payments = statement
        .query_map([row_id], |row| {
            Ok(MemberPayment {
                id: row.get(0)?,
                received_on: row.get(1)?,
                amount: row.get(2)?,
                insurance_year: row.get(3)?,
                method: row.get(4)?,
                variable_symbol: row.get(5)?,
                note: row.get(6)?,
                status: row.get(7)?,
                imported_from_bank: row.get::<_, i64>(8)? != 0,
                bank_transaction_id: row.get(9)?,
            })
        })?
        .collect();
    payments
}

fn validate(input: &PaymentInput) -> Result<(), String> {
    chrono::NaiveDate::parse_from_str(input.received_on.trim(), "%Y-%m-%d")
        .map_err(|_| "Zkontrolujte datum přijetí platby.".to_string())?;
    if input.amount <= 0 {
        return Err("Částka platby musí být vyšší než 0 Kč.".into());
    }
    if !matches!(input.method.as_str(), "Bankovní převod" | "Hotově" | "Jiné") {
        return Err("Vyberte platný způsob úhrady.".into());
    }
    Ok(())
}

fn recalculate(transaction: &rusqlite::Transaction<'_>, row_id: i64) -> rusqlite::Result<()> {
    transaction.execute(
        r#"UPDATE "Seznam" SET "SkutÚhrada" =
           COALESCE((SELECT SUM("Castka") FROM "PlatbyClenu" WHERE "PojistnyZaznamRowId" = ?1), 0)
           WHERE rowid = ?1"#,
        [row_id],
    )?;
    Ok(())
}

pub fn save(
    database: &Path,
    user: &str,
    member_identifier: &str,
    insurance_year: i32,
    variable_symbol: &str,
    input: PaymentInput,
) -> Result<i64, String> {
    validate(&input)?;
    let mut connection =
        Connection::open(database).map_err(|_| "Platbu se nepodařilo uložit.".to_string())?;
    ensure_schema(&connection).map_err(|_| "Platbu se nepodařilo uložit.".to_string())?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| "Platbu se nepodařilo uložit.".to_string())?;
    let (id, operation) = if let Some(id) = input.id {
        let changed = transaction
            .execute(
                r#"UPDATE "PlatbyClenu" SET "DatumPrijeti"=?1, "Castka"=?2, "Poznamka"=?3,
               "Aktualizovano"=CURRENT_TIMESTAMP
               WHERE "Id"=?4 AND "PojistnyZaznamRowId"=?5 AND "ImportovanoZBanky"=0"#,
                params![
                    input.received_on.trim(),
                    input.amount,
                    input.note,
                    id,
                    input.insurance_row_id
                ],
            )
            .map_err(|_| "Platbu se nepodařilo upravit.".to_string())?;
        if changed != 1 {
            return Err("Platbu se nepodařilo upravit.".into());
        }
        (id, "UPDATE")
    } else {
        transaction.execute(
            r#"INSERT INTO "PlatbyClenu" ("IdentifikatorClena", "PojistnyZaznamRowId", "PojistnyRok",
               "DatumPrijeti", "Castka", "ZpusobUhrady", "VariabilniSymbol", "Poznamka")
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8)"#,
            params![member_identifier, input.insurance_row_id, insurance_year, input.received_on.trim(),
                input.amount, input.method, variable_symbol, input.note],
        ).map_err(|_| "Platbu se nepodařilo uložit.".to_string())?;
        (transaction.last_insert_rowid(), "INSERT")
    };
    recalculate(&transaction, input.insurance_row_id)
        .map_err(|_| "Platbu se nepodařilo přepočítat.".to_string())?;
    transaction.execute(
        r#"INSERT INTO "AuditPlateb" ("Uzivatel","IdentifikatorClena","PojistnyRok","Castka","Operace")
           VALUES (?1,?2,?3,?4,?5)"#,
        params![user, member_identifier, insurance_year, input.amount, operation],
    ).map_err(|_| "Platbu se nepodařilo zaznamenat.".to_string())?;
    transaction
        .commit()
        .map_err(|_| "Platbu se nepodařilo uložit.".to_string())?;
    Ok(id)
}

pub fn delete(
    database: &Path,
    user: &str,
    member_identifier: &str,
    insurance_year: i32,
    row_id: i64,
    payment_id: i64,
) -> Result<(), String> {
    let mut connection =
        Connection::open(database).map_err(|_| "Platbu se nepodařilo odstranit.".to_string())?;
    ensure_schema(&connection).map_err(|_| "Platbu se nepodařilo odstranit.".to_string())?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| "Platbu se nepodařilo odstranit.".to_string())?;
    let amount: i64 = transaction.query_row(
        r#"SELECT "Castka" FROM "PlatbyClenu" WHERE "Id"=?1 AND "PojistnyZaznamRowId"=?2 AND "ImportovanoZBanky"=0"#,
        params![payment_id, row_id], |row| row.get(0),
    ).map_err(|_| "Platbu se nepodařilo odstranit.".to_string())?;
    transaction
        .execute(
            r#"DELETE FROM "PlatbyClenu" WHERE "Id"=?1 AND "PojistnyZaznamRowId"=?2"#,
            params![payment_id, row_id],
        )
        .map_err(|_| "Platbu se nepodařilo odstranit.".to_string())?;
    recalculate(&transaction, row_id)
        .map_err(|_| "Platbu se nepodařilo přepočítat.".to_string())?;
    transaction.execute(
        r#"INSERT INTO "AuditPlateb" ("Uzivatel","IdentifikatorClena","PojistnyRok","Castka","Operace") VALUES (?1,?2,?3,?4,'DELETE')"#,
        params![user,member_identifier,insurance_year,amount],
    ).map_err(|_| "Platbu se nepodařilo zaznamenat.".to_string())?;
    transaction
        .commit()
        .map_err(|_| "Platbu se nepodařilo odstranit.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn payment_crud_recalculates_total_and_writes_audit() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("pojisteni-member-payments-{unique}"));
        fs::create_dir_all(&directory).unwrap();
        let database = directory.join("payments.sqlite");
        let connection = Connection::open(&database).unwrap();
        connection
            .execute_batch(
                r#"CREATE TABLE "Seznam" ("SkutÚhrada" INTEGER); INSERT INTO "Seznam" VALUES (0);"#,
            )
            .unwrap();
        drop(connection);

        let id = save(
            &database,
            "tester",
            "42",
            2026,
            "7803265536",
            PaymentInput {
                id: None,
                insurance_row_id: 1,
                received_on: "2026-07-27".into(),
                amount: 300,
                method: "Bankovní převod".into(),
                note: Some("První platba".into()),
            },
        )
        .unwrap();
        save(
            &database,
            "tester",
            "42",
            2026,
            "7803265536",
            PaymentInput {
                id: Some(id),
                insurance_row_id: 1,
                received_on: "2026-07-28".into(),
                amount: 350,
                method: "Bankovní převod".into(),
                note: Some("Opraveno".into()),
            },
        )
        .unwrap();
        let connection = Connection::open(&database).unwrap();
        assert_eq!(
            connection
                .query_row(
                    r#"SELECT "SkutÚhrada" FROM "Seznam" WHERE rowid=1"#,
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            350
        );
        assert_eq!(list(&connection, 1).unwrap()[0].amount, 350);
        drop(connection);

        delete(&database, "tester", "42", 2026, 1, id).unwrap();
        let connection = Connection::open(&database).unwrap();
        assert_eq!(
            connection
                .query_row(
                    r#"SELECT "SkutÚhrada" FROM "Seznam" WHERE rowid=1"#,
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        assert_eq!(
            connection
                .query_row(r#"SELECT COUNT(*) FROM "AuditPlateb""#, [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            3
        );
        drop(connection);
        fs::remove_dir_all(directory).unwrap();
    }
}
