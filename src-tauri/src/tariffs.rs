use chrono::NaiveDate;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

const SEED_AMOUNTS: [i64; 6] = [200_000, 240_000, 280_000, 320_000, 360_000, 400_000];
const SEED_TARIFFS: [[[i64; 6]; 2]; 3] = [
    [
        [950, 1030, 1242, 1361, 1509, 1674],
        [1146, 1311, 1427, 1641, 1856, 2276],
    ],
    [
        [495, 536, 578, 710, 849, 950],
        [594, 644, 743, 858, 981, 1146],
    ],
    [
        [1901, 2059, 2485, 2723, 3020, 3350],
        [2294, 2624, 2855, 3284, 3713, 4455],
    ],
];

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TariffResult {
    pub premium: i64,
    pub months: i64,
    pub insured_amount: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TariffRate {
    pub id: i64,
    pub insured_amount: i64,
    pub category: String,
    pub loss_insurance: bool,
    pub annual_premium: i64,
    pub valid_from: String,
    pub valid_to: Option<String>,
    pub active: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TariffRateInput {
    pub id: Option<i64>,
    pub insured_amount: i64,
    pub category: String,
    pub loss_insurance: bool,
    pub annual_premium: i64,
    pub valid_from: String,
    pub valid_to: Option<String>,
    pub active: bool,
    pub note: Option<String>,
}

pub fn ensure_schema(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        r#"CREATE TABLE IF NOT EXISTS "sazby_pojistneho" (
               "id" INTEGER PRIMARY KEY AUTOINCREMENT,
               "pojistna_castka" INTEGER NOT NULL CHECK ("pojistna_castka" > 0),
               "kategorie" TEXT NOT NULL CHECK ("kategorie" IN ('A', 'B', 'C')),
               "pojisteni_ztraty" INTEGER NOT NULL CHECK ("pojisteni_ztraty" IN (0, 1)),
               "rocni_pojistne" INTEGER NOT NULL CHECK ("rocni_pojistne" >= 0),
               "platnost_od" TEXT NOT NULL,
               "platnost_do" TEXT,
               "aktivni" INTEGER NOT NULL DEFAULT 1 CHECK ("aktivni" IN (0, 1)),
               "poznamka" TEXT,
               "created_at" TEXT NOT NULL DEFAULT (datetime('now')),
               "updated_at" TEXT NOT NULL DEFAULT (datetime('now'))
           );
           CREATE INDEX IF NOT EXISTS "idx_sazby_vyhledani"
             ON "sazby_pojistneho"
                ("pojistna_castka", "kategorie", "pojisteni_ztraty", "platnost_od");"#,
    )?;
    let count: i64 =
        connection.query_row(r#"SELECT COUNT(*) FROM "sazby_pojistneho""#, [], |row| {
            row.get(0)
        })?;
    if count == 0 {
        let categories = ["A", "B", "C"];
        for (category_index, category) in categories.iter().enumerate() {
            for loss_index in 0..=1 {
                for (amount_index, amount) in SEED_AMOUNTS.iter().enumerate() {
                    connection.execute(
                        r#"INSERT INTO "sazby_pojistneho"
                           ("pojistna_castka", "kategorie", "pojisteni_ztraty",
                            "rocni_pojistne", "platnost_od", "aktivni", "poznamka")
                           VALUES (?1, ?2, ?3, ?4, '1900-01-01', 1, 'Původní sazba z Accessu')"#,
                        params![
                            amount,
                            category,
                            loss_index,
                            SEED_TARIFFS[category_index][loss_index][amount_index]
                        ],
                    )?;
                }
            }
        }
        connection.execute_batch(
            r#"WITH ranked AS (
                   SELECT
                       "RočPojistné" AS amount,
                       "Kategorie" AS category,
                       CASE WHEN "Ztráta" <> 0 THEN 1 ELSE 0 END AS loss,
                       "PojistnáČástka" AS premium,
                       ROW_NUMBER() OVER (
                           PARTITION BY "RočPojistné", "Kategorie",
                                        CASE WHEN "Ztráta" <> 0 THEN 1 ELSE 0 END
                           ORDER BY "PojištěníOd" DESC, rowid DESC
                       ) AS position
                   FROM "Seznam"
                   WHERE "RočPojistné" > 0
                     AND "PojistnáČástka" >= 0
                     AND "Kategorie" IN ('A', 'B', 'C')
               )
               INSERT INTO "sazby_pojistneho"
               ("pojistna_castka", "kategorie", "pojisteni_ztraty", "rocni_pojistne",
                "platnost_od", "aktivni", "poznamka")
               SELECT
                   ranked.amount,
                   ranked.category,
                   ranked.loss,
                   ranked.premium,
                   '1900-01-01',
                   1,
                   'Doplněno z nejnovějšího záznamu v databázi'
               FROM ranked
               WHERE ranked.position = 1
                 AND NOT EXISTS (
                     SELECT 1 FROM "sazby_pojistneho" rate
                     WHERE rate."pojistna_castka" = ranked.amount
                       AND rate."kategorie" = ranked.category
                       AND rate."pojisteni_ztraty" = ranked.loss
                 );"#,
        )?;
    }
    Ok(())
}

pub fn calculate(
    connection: &Connection,
    category: &str,
    loss: bool,
    insured_amount: i64,
    insurance_date: NaiveDate,
    months: i64,
) -> rusqlite::Result<Option<TariffResult>> {
    let premium: Option<i64> = connection
        .query_row(
            r#"SELECT "rocni_pojistne" FROM "sazby_pojistneho"
               WHERE "pojistna_castka" = ?1
                 AND "kategorie" = ?2
                 AND "pojisteni_ztraty" = ?3
                 AND "aktivni" = 1
                 AND date("platnost_od") <= date(?4)
                 AND ("platnost_do" IS NULL OR date("platnost_do") >= date(?4))
               ORDER BY date("platnost_od") DESC
               LIMIT 1"#,
            params![
                insured_amount,
                category,
                i64::from(loss),
                insurance_date.format("%Y-%m-%d").to_string()
            ],
            |row| row.get(0),
        )
        .optional()?;
    Ok(premium.map(|premium| TariffResult {
        premium,
        months,
        insured_amount: premium as f64 / 12.0 * months as f64,
    }))
}

pub fn insured_amounts(connection: &Connection) -> rusqlite::Result<Vec<i64>> {
    let mut statement = connection.prepare(
        r#"SELECT DISTINCT "pojistna_castka" FROM "sazby_pojistneho"
           WHERE "aktivni" = 1 ORDER BY "pojistna_castka""#,
    )?;
    let amounts = statement
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>();
    amounts
}

pub fn list(connection: &Connection) -> rusqlite::Result<Vec<TariffRate>> {
    let mut statement = connection.prepare(
        r#"SELECT "id", "pojistna_castka", "kategorie", "pojisteni_ztraty",
                  "rocni_pojistne", "platnost_od", "platnost_do", "aktivni", "poznamka"
           FROM "sazby_pojistneho"
           ORDER BY "pojistna_castka", "kategorie", "pojisteni_ztraty", date("platnost_od") DESC"#,
    )?;
    let rates = statement
        .query_map([], |row| {
            Ok(TariffRate {
                id: row.get(0)?,
                insured_amount: row.get(1)?,
                category: row.get(2)?,
                loss_insurance: row.get::<_, i64>(3)? != 0,
                annual_premium: row.get(4)?,
                valid_from: row.get(5)?,
                valid_to: row.get(6)?,
                active: row.get::<_, i64>(7)? != 0,
                note: row.get(8)?,
            })
        })?
        .collect();
    rates
}

pub fn save(connection: &Connection, input: TariffRateInput) -> Result<i64, String> {
    if input.insured_amount <= 0
        || input.annual_premium < 0
        || !matches!(input.category.as_str(), "A" | "B" | "C")
    {
        return Err("Zkontrolujte údaje sazby.".into());
    }
    let valid_from = NaiveDate::parse_from_str(&input.valid_from, "%Y-%m-%d")
        .map_err(|_| "Zkontrolujte datum platnosti od.")?;
    let valid_to = input
        .valid_to
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d"))
        .transpose()
        .map_err(|_| "Zkontrolujte datum platnosti do.")?;
    if valid_to.is_some_and(|end| end < valid_from) {
        return Err("Platnost do nesmí být dříve než platnost od.".into());
    }
    let overlap: i64 = connection
        .query_row(
            r#"SELECT COUNT(*) FROM "sazby_pojistneho"
               WHERE "pojistna_castka" = ?1
                 AND "kategorie" = ?2
                 AND "pojisteni_ztraty" = ?3
                 AND "id" <> COALESCE(?4, -1)
                 AND date("platnost_od") <= date(COALESCE(?5, '9999-12-31'))
                 AND date(COALESCE("platnost_do", '9999-12-31')) >= date(?6)"#,
            params![
                input.insured_amount,
                input.category,
                i64::from(input.loss_insurance),
                input.id,
                valid_to.map(|date| date.format("%Y-%m-%d").to_string()),
                valid_from.format("%Y-%m-%d").to_string()
            ],
            |row| row.get(0),
        )
        .map_err(|_| "Sazbu se nepodařilo uložit.")?;
    if overlap > 0 {
        return Err("Pro tuto kombinaci již existuje sazba s překrývající se platností.".into());
    }
    let note = input.note.and_then(|value| {
        let value = value.trim().to_string();
        (!value.is_empty()).then_some(value)
    });
    let valid_to = valid_to.map(|date| date.format("%Y-%m-%d").to_string());
    if let Some(id) = input.id {
        let changed = connection
            .execute(
                r#"UPDATE "sazby_pojistneho"
                   SET "pojistna_castka" = ?1, "kategorie" = ?2, "pojisteni_ztraty" = ?3,
                       "rocni_pojistne" = ?4, "platnost_od" = ?5, "platnost_do" = ?6,
                       "aktivni" = ?7, "poznamka" = ?8, "updated_at" = datetime('now')
                   WHERE "id" = ?9"#,
                params![
                    input.insured_amount,
                    input.category,
                    i64::from(input.loss_insurance),
                    input.annual_premium,
                    valid_from.format("%Y-%m-%d").to_string(),
                    valid_to,
                    i64::from(input.active),
                    note,
                    id
                ],
            )
            .map_err(|_| "Sazbu se nepodařilo uložit.")?;
        if changed == 0 {
            return Err("Sazba nebyla nalezena.".into());
        }
        Ok(id)
    } else {
        connection
            .execute(
                r#"INSERT INTO "sazby_pojistneho"
                   ("pojistna_castka", "kategorie", "pojisteni_ztraty", "rocni_pojistne",
                    "platnost_od", "platnost_do", "aktivni", "poznamka")
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
                params![
                    input.insured_amount,
                    input.category,
                    i64::from(input.loss_insurance),
                    input.annual_premium,
                    valid_from.format("%Y-%m-%d").to_string(),
                    valid_to,
                    i64::from(input.active),
                    note
                ],
            )
            .map_err(|_| "Sazbu se nepodařilo uložit.")?;
        Ok(connection.last_insert_rowid())
    }
}
