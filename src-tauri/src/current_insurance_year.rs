use chrono::NaiveDate;
use rusqlite::{params, Connection, TransactionBehavior};
use std::{fs, path::Path};

pub struct CurrentInsuranceYear;

impl CurrentInsuranceYear {
    pub fn active(connection: &Connection) -> rusqlite::Result<i32> {
        connection.query_row(
            r#"SELECT "Rok" FROM "PojistnaObdobi"
               WHERE "Stav" = 'AKTIVNI'
               ORDER BY "Rok" DESC LIMIT 1"#,
            [],
            |row| row.get(0),
        )
    }

    pub fn initialize(
        connection: &mut Connection,
        database_path: &Path,
        calendar_year: i32,
    ) -> Result<i32, String> {
        Self::create_schema(connection).map_err(|_| "Pojistné období se nepodařilo načíst.")?;
        Self::seed_periods(connection).map_err(|_| "Pojistné období se nepodařilo načíst.")?;
        let active =
            Self::active(connection).map_err(|_| "Pojistné období se nepodařilo načíst.")?;
        if calendar_year <= active {
            return Ok(active);
        }
        Self::backup(database_path, calendar_year)?;
        Self::roll_forward(connection, active, calendar_year)
            .map_err(|_| "Nové pojistné období se nepodařilo vytvořit.")?;
        Ok(calendar_year)
    }

    fn create_schema(connection: &Connection) -> rusqlite::Result<()> {
        connection.execute_batch(
            r#"CREATE TABLE IF NOT EXISTS "PojistnaObdobi" (
                   "Rok" INTEGER PRIMARY KEY,
                   "Stav" TEXT NOT NULL CHECK ("Stav" IN ('AKTIVNI', 'UZAVRENO')),
                   "Vytvoreno" TEXT NOT NULL DEFAULT (datetime('now'))
               );
               CREATE TABLE IF NOT EXISTS "NeprevadetCleny" (
                   "RodneCislo" TEXT PRIMARY KEY,
                   "Duvod" TEXT
               );"#,
        )
    }

    fn seed_periods(connection: &Connection) -> rusqlite::Result<()> {
        let count: i64 =
            connection.query_row(r#"SELECT COUNT(*) FROM "PojistnaObdobi""#, [], |row| {
                row.get(0)
            })?;
        if count > 0 {
            return Ok(());
        }
        let active: i32 = connection.query_row(
            r#"SELECT MAX(pojisteni_rok("PojištěníOd")) FROM "Seznam"
               WHERE pojisteni_rok("PojištěníOd") BETWEEN 1900 AND 2999"#,
            [],
            |row| row.get(0),
        )?;
        connection.execute(
            r#"INSERT INTO "PojistnaObdobi" ("Rok", "Stav")
               SELECT DISTINCT pojisteni_rok("PojištěníOd"),
                      CASE WHEN pojisteni_rok("PojištěníOd") = ?1
                           THEN 'AKTIVNI' ELSE 'UZAVRENO' END
               FROM "Seznam"
               WHERE pojisteni_rok("PojištěníOd") BETWEEN 1900 AND 2999"#,
            [active],
        )?;
        Ok(())
    }

    fn backup(database_path: &Path, target_year: i32) -> Result<(), String> {
        let directory = database_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("backups");
        fs::create_dir_all(&directory)
            .map_err(|_| "Zálohu před převodem se nepodařilo vytvořit.")?;
        let destination = directory.join(format!(
            "dd-before-period-{target_year}-{}.sqlite",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        ));
        fs::copy(database_path, destination)
            .map_err(|_| "Zálohu před převodem se nepodařilo vytvořit.")?;
        Ok(())
    }

    fn roll_forward(
        connection: &mut Connection,
        source_year: i32,
        target_year: i32,
    ) -> rusqlite::Result<()> {
        let start = NaiveDate::from_ymd_opt(target_year, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(target_year, 12, 31).unwrap();
        let source_end = NaiveDate::from_ymd_opt(source_year, 12, 31).unwrap();
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let exists: i64 = transaction.query_row(
            r#"SELECT COUNT(*) FROM "PojistnaObdobi" WHERE "Rok" = ?1"#,
            [target_year],
            |row| row.get(0),
        )?;
        if exists > 0 {
            transaction.rollback()?;
            return Ok(());
        }
        let max_identifier: i64 = transaction.query_row(
            r#"SELECT COALESCE(MAX("Identifikátor"), 0) FROM "Seznam""#,
            [],
            |row| row.get(0),
        )?;
        let target_start = start.format("%Y-%m-%d").to_string();
        let missing_tariffs: i64 = transaction.query_row(
            r#"SELECT COUNT(*) FROM "Seznam" source
               WHERE pojisteni_rok(source."PojištěníOd") = ?1
                 AND NULLIF(TRIM(source."Ukončení"), '') IS NULL
                 AND date(source."PojištěníDo") >= date(?2)
                 AND NOT EXISTS (
                     SELECT 1 FROM "NeprevadetCleny" exclusion
                     WHERE NULLIF(TRIM(source."RodnéČíslo"), '') IS NOT NULL
                       AND exclusion."RodneCislo" = TRIM(source."RodnéČíslo")
                 )
                 AND NOT EXISTS (
                     SELECT 1 FROM "sazby_pojistneho" rate
                     WHERE rate."pojistna_castka" = source."RočPojistné"
                       AND rate."kategorie" = source."Kategorie"
                       AND rate."pojisteni_ztraty" = CASE WHEN source."Ztráta" <> 0 THEN 1 ELSE 0 END
                       AND rate."aktivni" = 1
                       AND date(rate."platnost_od") <= date(?3)
                       AND (rate."platnost_do" IS NULL OR date(rate."platnost_do") >= date(?3))
                 )"#,
            params![
                source_year,
                source_end.format("%Y-%m-%d").to_string(),
                target_start
            ],
            |row| row.get(0),
        )?;
        if missing_tariffs > 0 {
            return Err(rusqlite::Error::InvalidQuery);
        }
        transaction.execute(
            r#"INSERT INTO "Seznam" (
                   "Identifikátor", "PojištěníOd", "PojištěníDo", "RočPojistné",
                   "PojistnáČástka", "PojistNespotř", "Kategorie", "Ztráta",
                   "KódOC", "EvČíslo", "Titul", "Příjmení", "Jméno", "RodnéČíslo",
                   "Město", "Adresa", "PSČ", "Stát", "Poznámka", "OdbPříslušnost",
                   "ZO", "Ukončení", "SkutÚhrada", "Doklad", "e-mail", "Tisk", "DatumTisku"
               )
               SELECT
                   ?1 + ROW_NUMBER() OVER (ORDER BY rowid),
                   ?2, ?3, "RočPojistné",
                   (SELECT rate."rocni_pojistne" FROM "sazby_pojistneho" rate
                    WHERE rate."pojistna_castka" = source."RočPojistné"
                      AND rate."kategorie" = source."Kategorie"
                      AND rate."pojisteni_ztraty" = CASE WHEN source."Ztráta" <> 0 THEN 1 ELSE 0 END
                      AND rate."aktivni" = 1
                      AND date(rate."platnost_od") <= date(?6)
                      AND (rate."platnost_do" IS NULL OR date(rate."platnost_do") >= date(?6))
                    ORDER BY date(rate."platnost_od") DESC LIMIT 1),
                   "PojistNespotř",
                   "Kategorie", "Ztráta", "KódOC", "EvČíslo", "Titul", "Příjmení",
                   "Jméno", "RodnéČíslo", "Město", "Adresa", "PSČ", "Stát",
                   NULL, "OdbPříslušnost", "ZO", NULL, 0, 0, "e-mail", 0, NULL
               FROM "Seznam" AS source
               WHERE pojisteni_rok(source."PojištěníOd") = ?4
                 AND NULLIF(TRIM(source."Ukončení"), '') IS NULL
                 AND date(source."PojištěníDo") >= date(?5)
                 AND NOT EXISTS (
                     SELECT 1 FROM "NeprevadetCleny" exclusion
                     WHERE NULLIF(TRIM(source."RodnéČíslo"), '') IS NOT NULL
                       AND exclusion."RodneCislo" = TRIM(source."RodnéČíslo")
                 )"#,
            params![
                max_identifier,
                start.format("%Y-%m-%d 00:00:00").to_string(),
                end.format("%Y-%m-%d 00:00:00").to_string(),
                source_year,
                source_end.format("%Y-%m-%d").to_string(),
                target_start,
            ],
        )?;
        transaction.execute(
            r#"UPDATE "PojistnaObdobi" SET "Stav" = 'UZAVRENO'
               WHERE "Stav" = 'AKTIVNI'"#,
            [],
        )?;
        transaction.execute(
            r#"INSERT INTO "PojistnaObdobi" ("Rok", "Stav")
               VALUES (?1, 'AKTIVNI')"#,
            [target_year],
        )?;
        transaction.commit()
    }
}
