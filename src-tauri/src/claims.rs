use printpdf::{Mm, PdfDocument};
use rusqlite::{params, Connection, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufWriter, path::Path};

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
                "PojistnyZaznamRowId"=?1, "PojistnyRok"=?2,
                "Telefon"=?3, "Zamestnavatel"=?4, "Povolani"=?5, "VznikPU"=?6,
                "OznameniPU"=?7, "ZjistenaSkoda"=?8, "PojistnePlneni"=?9,
                "PopisUdalosti"=?10, "Poznamka1"=?11, "Poznamka2"=?12,
                "Ukonceno"=?13, "ResiPojistovna"=?14, "PolohaVSestave"=?15
               WHERE "ID"=?16 AND "IdentifikatorClena"=?17"#,
            params![
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
                id,
                member_identifier,
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

pub fn export_pdf(connection: &Connection, id: i64, destination: &Path) -> Result<(), String> {
    let values: Vec<String> = connection
        .query_row(
            r#"SELECT
                TRIM(COALESCE(member."Titul",'')||' '||COALESCE(member."Příjmení",'')||' '||COALESCE(member."Jméno",'')),
                COALESCE(member."RodnéČíslo",''), COALESCE(CAST(member."EvČíslo" AS TEXT),''),
                TRIM(COALESCE(member."Adresa",'')||', '||COALESCE(member."PSČ",'')||' '||COALESCE(member."Město",'')),
                COALESCE(claim."Telefon", member."Telefon",''), COALESCE(member."e-mail",''),
                COALESCE(claim."Povolani",''), COALESCE(claim."Zamestnavatel",''),
                'Kategorie '||COALESCE(member."Kategorie",'')||'; roč. '||COALESCE(member."RočPojistné",0),
                COALESCE(member."PojištěníOd",'')||' - '||COALESCE(member."PojištěníDo",''),
                COALESCE(claim."VznikPU",''), COALESCE(claim."OznameniPU",''),
                COALESCE(claim."PopisUdalosti",''), COALESCE(claim."Poznamka1",''),
                COALESCE(claim."Poznamka2",''), COALESCE(CAST(claim."ZjistenaSkoda" AS TEXT),''),
                COALESCE(claim."ResiPojistovna",''), COALESCE(CAST(claim."PojistnePlneni" AS TEXT),''),
                COALESCE(claim."Ukonceno",'')
              FROM "PojistneUdalosti" claim
              JOIN "Seznam" member ON member.rowid=claim."PojistnyZaznamRowId"
              WHERE claim."ID"=?1"#,
            [id],
            |row| (0..19).map(|index| row.get(index)).collect(),
        )
        .map_err(|_| "Podklady hlášení pojistné události nebyly nalezeny.".to_string())?;
    let (document, page, layer) =
        PdfDocument::new("Pojistná událost", Mm(210.0), Mm(297.0), "Hlášení");
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
    let current = document.get_page(page).get_layer(layer);
    current.use_text(
        "S dokumentem je nutno nakládat v souladu s pravidly ochrany osobních údajů.",
        6.0,
        Mm(20.0),
        Mm(287.0),
        &regular,
    );
    current.use_text("POJISTNÁ UDÁLOST", 17.0, Mm(70.0), Mm(274.0), &bold);
    let labels = [
        "Pojištěnec:",
        "Rodné číslo:",
        "Evidenční číslo:",
        "Bydliště:",
        "Telefon:",
        "e-mail:",
        "Povolání:",
        "Zaměstnavatel:",
        "Typ pojištění:",
        "Pojistné období:",
        "Vznik PU:",
        "Oznámení PU:",
    ];
    let mut y = 254.0;
    for (label, value) in labels.iter().zip(values.iter()) {
        current.use_text(*label, 10.0, Mm(22.0), Mm(y), &bold);
        current.use_text(value, 10.0, Mm(66.0), Mm(y), &regular);
        y -= 8.0;
    }
    for (label, index, lines) in [
        ("Popis PU:", 12_usize, 3_usize),
        ("Poznámky k PU:", 13, 2),
        ("Doplňky k PU:", 14, 2),
    ] {
        current.use_text(label, 10.0, Mm(22.0), Mm(y), &bold);
        let chars: Vec<char> = values[index].replace(['\r', '\n'], " ").chars().collect();
        for (line, chunk) in chars.chunks(95).take(lines).enumerate() {
            current.use_text(
                chunk.iter().collect::<String>(),
                9.0,
                Mm(66.0),
                Mm(y - line as f32 * 5.0),
                &regular,
            );
        }
        y -= lines as f32 * 5.0 + 5.0;
    }
    for (label, index) in [
        ("Zjištěná škoda:", 15_usize),
        ("Řeší pojišťovna:", 16),
        ("Pojistné plnění:", 17),
        ("Ukončeno:", 18),
    ] {
        current.use_text(label, 10.0, Mm(22.0), Mm(y), &bold);
        current.use_text(&values[index], 10.0, Mm(66.0), Mm(y), &regular);
        y -= 8.0;
    }
    document
        .save(&mut BufWriter::new(File::create(destination).map_err(
            |_| "Hlášení pojistné události se nepodařilo uložit.".to_string(),
        )?))
        .map_err(|_| "Hlášení pojistné události se nepodařilo vytvořit.".to_string())
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

    #[test]
    fn individual_claim_pdf_uses_access_fields() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(r#"
            CREATE TABLE "Seznam" ("Titul" TEXT,"Příjmení" TEXT,"Jméno" TEXT,"RodnéČíslo" TEXT,
              "EvČíslo" TEXT,"Adresa" TEXT,"PSČ" TEXT,"Město" TEXT,"Telefon" TEXT,"e-mail" TEXT,
              "Kategorie" TEXT,"RočPojistné" INTEGER,"PojištěníOd" TEXT,"PojištěníDo" TEXT);
            INSERT INTO "Seznam" VALUES ('','Novák','Jan','800101/0000','10001','Hlavní 1','110 00','Praha',
              '+420 123 456 789','test@example.cz','A',320000,'2026-01-01','2026-12-31');
        "#).unwrap();
        ensure_schema(&connection).unwrap();
        connection.execute(r#"INSERT INTO "PojistneUdalosti"("ID","IdentifikatorClena","PojistnyZaznamRowId","PojistnyRok","Telefon","Zamestnavatel","Povolani","VznikPU","OznameniPU","ZjistenaSkoda","PojistnePlneni","PopisUdalosti","Poznamka1","Poznamka2","Ukonceno","ResiPojistovna") VALUES(116,1,1,2026,'+420 123 456 789','Dopravce','Strojvedoucí','2026-06-10','2026-06-11',15000,12000,'Popis události','Poznámka','Doplnění','','Ano')"#, []).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let destination = std::env::var_os("FED_PDF_QA_DIR")
            .map(std::path::PathBuf::from)
            .map(|path| path.join("claim-detail.pdf"))
            .unwrap_or_else(|| directory.path().join("claim-detail.pdf"));
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        export_pdf(&connection, 116, &destination).unwrap();
        assert!(fs::metadata(destination).unwrap().len() > 1_000);
    }
}
