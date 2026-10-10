use crate::{database_backup, migrations};
use chrono::{Local, NaiveDate};
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
        _database_path: &Path,
        _calendar_year: i32,
    ) -> Result<i32, String> {
        Self::seed_periods(connection).map_err(|_| "Pojistné období se nepodařilo načíst.")?;
        let active =
            Self::active(connection).map_err(|_| "Pojistné období se nepodařilo načíst.")?;
        Ok(active)
    }

    pub fn prepare(
        connection: &mut Connection,
        database_path: &Path,
        target_year: i32,
    ) -> Result<i32, String> {
        Self::seed_periods(connection).map_err(|_| "Pojistné období se nepodařilo načíst.")?;
        let source_year =
            Self::active(connection).map_err(|_| "Pojistné období se nepodařilo načíst.")?;
        let exists: bool = connection
            .query_row(
                r#"SELECT EXISTS(SELECT 1 FROM "PojistnaObdobi" WHERE "Rok"=?1)"#,
                [target_year],
                |row| row.get(0),
            )
            .map_err(|_| "Pojistný rok se nepodařilo ověřit.".to_string())?;
        if exists {
            return Err(format!("Pojistný rok {target_year} již existuje."));
        }
        if target_year != source_year + 1 {
            return Err("Lze připravit pouze bezprostředně následující pojistný rok.".into());
        }
        Self::backup(database_path, target_year)?;
        Self::roll_forward(connection, source_year, target_year)
            .map_err(|_| "Nové pojistné období se nepodařilo vytvořit.")?;
        Ok(target_year)
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
        let base = format!(
            "PredZalozenimRoku_{target_year}_{}",
            Local::now().format("%Y%m%d-%H%M%S")
        );
        let mut destination = directory.join(format!("{base}.fvcbackup"));
        let mut suffix = 2;
        while destination.exists() {
            destination = directory.join(format!("{base}_{suffix}.fvcbackup"));
            suffix += 1;
        }
        database_backup::create(database_path, &destination, false)
            .map_err(|error| format!("Zálohu před převodem se nepodařilo vytvořit: {error}"))?;
        database_backup::inspect(&destination, migrations::DB_SCHEMA_VERSION)
            .map_err(|error| format!("Zálohu před převodem se nepodařilo ověřit: {error}"))?;
        Ok(())
    }

    fn roll_forward(
        connection: &mut Connection,
        source_year: i32,
        target_year: i32,
    ) -> rusqlite::Result<()> {
        let start = NaiveDate::from_ymd_opt(target_year, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(target_year, 12, 31).unwrap();
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let exists: i64 = transaction.query_row(
            r#"SELECT COUNT(*) FROM "PojistnaObdobi" WHERE "Rok" = ?1"#,
            [target_year],
            |row| row.get(0),
        )?;
        if exists > 0 {
            transaction.rollback()?;
            return Err(rusqlite::Error::InvalidQuery);
        }
        let target_start = start.format("%Y-%m-%d").to_string();
        let missing_tariffs: i64 = transaction.query_row(
            r#"SELECT COUNT(*) FROM "Seznam" source
               WHERE pojisteni_rok(source."PojištěníOd") = ?1
                 AND NOT (NULLIF(TRIM(source."Ukončení"), '') IS NOT NULL
                          AND COALESCE(source."PojisteniUkonceno",0)=1)
                 AND NOT EXISTS (
                     SELECT 1 FROM "sazby_pojistneho" rate
                     WHERE rate."pojistna_castka" = source."RočPojistné"
                       AND rate."kategorie" = source."Kategorie"
                       AND rate."pojisteni_ztraty" = CASE WHEN source."Ztráta" <> 0 THEN 1 ELSE 0 END
                       AND rate."aktivni" = 1
                       AND date(rate."platnost_od") <= date(?2)
                       AND (rate."platnost_do" IS NULL OR date(rate."platnost_do") >= date(?2))
                 )"#,
            params![
                source_year,
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
                   "ZO", "Ukončení", "SkutÚhrada", "Doklad", "e-mail", "Telefon", "Tisk", "DatumTisku"
               )
               SELECT
                   source."Identifikátor",
                   ?1, ?2, "RočPojistné",
                   (SELECT rate."rocni_pojistne" FROM "sazby_pojistneho" rate
                    WHERE rate."pojistna_castka" = source."RočPojistné"
                      AND rate."kategorie" = source."Kategorie"
                      AND rate."pojisteni_ztraty" = CASE WHEN source."Ztráta" <> 0 THEN 1 ELSE 0 END
                      AND rate."aktivni" = 1
                      AND date(rate."platnost_od") <= date(?4)
                      AND (rate."platnost_do" IS NULL OR date(rate."platnost_do") >= date(?4))
                    ORDER BY date(rate."platnost_od") DESC LIMIT 1),
                   "PojistNespotř",
                   "Kategorie", "Ztráta", "KódOC", "EvČíslo", "Titul", "Příjmení",
                   "Jméno", "RodnéČíslo", "Město", "Adresa", "PSČ", "Stát",
                   NULL, "OdbPříslušnost", "ZO", NULL, 0, 0, "e-mail", "Telefon", 0, NULL
               FROM "Seznam" AS source
               WHERE pojisteni_rok(source."PojištěníOd") = ?3
                 AND NOT (NULLIF(TRIM(source."Ukončení"), '') IS NOT NULL
                          AND COALESCE(source."PojisteniUkonceno",0)=1)"#,
            params![
                start.format("%Y-%m-%d 00:00:00").to_string(),
                end.format("%Y-%m-%d 00:00:00").to_string(),
                source_year,
                target_start,
            ],
        )?;
        transaction.execute(
            r#"INSERT INTO "PojistnaObdobi" ("Rok", "Stav")
               VALUES (?1, 'AKTIVNI')"#,
            [target_year],
        )?;
        transaction.commit()
    }
}
