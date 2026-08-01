use crate::email_service::{self, EmailMessage};
use chrono::Local;
use printpdf::{
    path::{PaintMode, WindingOrder},
    Color, Image, ImageTransform, Mm, PdfDocument, Point, Polygon, Rgb,
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs::File, io::BufWriter, path::Path};

const TEMPLATE_1: &[u8] = include_bytes!("../resources/receipt-template-1.jpg");
const TEMPLATE_2: &[u8] = include_bytes!("../resources/receipt-template-2.jpg");
const TEMPLATE_3: &[u8] = include_bytes!("../resources/receipt-template-3.jpg");

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub id: i64,
    pub member_row_id: i64,
    pub member_identifier: String,
    pub payment_id: i64,
    pub registration_number: String,
    pub member_name: String,
    pub insurance_year: i32,
    pub paid_on: String,
    pub issued_on: String,
    pub amount: i64,
    pub contract_number: String,
    pub status: String,
    pub email_status: String,
    pub sent_at: Option<String>,
    pub recipient_email: Option<String>,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptSettings {
    pub automatic_creation: bool,
    pub automatic_sending: bool,
    pub email_subject: String,
    pub email_body: String,
    pub policyholder: String,
    pub contract_number: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentDocumentBasis {
    pub member_row_id: i64,
    pub member_name: String,
    pub registration_number: String,
    pub organization_code: String,
    pub insurance_year: i32,
    pub prescribed_premium: i64,
    pub paid_amount: i64,
    pub payment_dates: Vec<String>,
    pub contract_number: String,
    pub insurance_status: String,
    pub loss_insurance: bool,
    pub certificate_ready: bool,
}

#[derive(Debug)]
struct Snapshot {
    identifier: String,
    payment_id: i64,
    title: String,
    first_name: String,
    last_name: String,
    personal_id: String,
    registration: String,
    organization_code: String,
    organization: String,
    address: String,
    city: String,
    postal_code: String,
    country: String,
    category: String,
    insurance_from: String,
    insurance_to: String,
    insured_amount: i64,
    premium: i64,
    paid: i64,
    paid_on: String,
    email: String,
    loss_insurance: bool,
    termination: String,
}

pub fn ensure_schema(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        r#"CREATE TABLE IF NOT EXISTS "NastaveniDokladu" (
            "Id" INTEGER PRIMARY KEY CHECK("Id"=1),
            "AutomatickeVytvareni" INTEGER NOT NULL DEFAULT 1,
            "AutomatickeOdesilani" INTEGER NOT NULL DEFAULT 0,
            "PredmetEmailu" TEXT NOT NULL DEFAULT 'Doklad o pojištění',
            "TextEmailu" TEXT NOT NULL DEFAULT 'Přílohou zasíláme doklad o pojištění z odpovědnosti.',
            "Pojistnik" TEXT NOT NULL DEFAULT 'Federace vlakových čet - presidium',
            "CisloSmlouvy" TEXT NOT NULL DEFAULT '650 12 00002',
            "Aktualizovano" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        INSERT OR IGNORE INTO "NastaveniDokladu"("Id") VALUES(1);
        CREATE TABLE IF NOT EXISTS "DokladyOUhrade" (
            "Id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "PojistnyZaznamRowId" INTEGER NOT NULL,
            "IdentifikatorClena" TEXT NOT NULL,
            "IdPlatby" INTEGER NOT NULL,
            "EvidencniCislo" TEXT NOT NULL,
            "JmenoClena" TEXT NOT NULL,
            "PojistnyRok" INTEGER NOT NULL,
            "DatumUhrady" TEXT NOT NULL,
            "DatumVystaveni" TEXT NOT NULL,
            "Castka" INTEGER NOT NULL,
            "CisloSmlouvy" TEXT NOT NULL,
            "EmailPrijemce" TEXT,
            "Stav" TEXT NOT NULL DEFAULT 'Vytvořen',
            "StavEmailu" TEXT NOT NULL DEFAULT 'Neodeslán',
            "DatumOdeslani" TEXT,
            "Pdf" BLOB NOT NULL,
            "Sha256" TEXT NOT NULL,
            "Vytvoreno" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE("IdentifikatorClena","PojistnyRok")
        );
        CREATE INDEX IF NOT EXISTS "IX_DokladyOUhrade_Clen" ON "DokladyOUhrade"("IdentifikatorClena","PojistnyRok");
        CREATE TABLE IF NOT EXISTS "AuditDokladu" (
            "Id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "DatumCas" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "Uzivatel" TEXT NOT NULL,
            "IdDokladu" INTEGER,
            "IdentifikatorClena" TEXT NOT NULL,
            "Operace" TEXT NOT NULL,
            "Vysledek" TEXT NOT NULL
        );"#,
    )
}

pub fn load_settings(connection: &Connection) -> Result<ReceiptSettings, String> {
    ensure_schema(connection).map_err(|_| "Nastavení dokladů se nepodařilo načíst.".to_string())?;
    connection.query_row(
        r#"SELECT "AutomatickeVytvareni","AutomatickeOdesilani","PredmetEmailu","TextEmailu","Pojistnik","CisloSmlouvy" FROM "NastaveniDokladu" WHERE "Id"=1"#,
        [], |row| Ok(ReceiptSettings { automatic_creation: row.get::<_,i64>(0)? != 0, automatic_sending: row.get::<_,i64>(1)? != 0, email_subject: row.get(2)?, email_body: row.get(3)?, policyholder: row.get(4)?, contract_number: row.get(5)? })
    ).map_err(|_| "Nastavení dokladů se nepodařilo načíst.".to_string())
}

pub fn save_settings(connection: &Connection, settings: &ReceiptSettings) -> Result<(), String> {
    ensure_schema(connection).map_err(|_| "Nastavení dokladů se nepodařilo uložit.".to_string())?;
    if settings.contract_number.trim().is_empty() {
        return Err("Vyplňte číslo pojistné smlouvy.".into());
    }
    connection.execute(r#"UPDATE "NastaveniDokladu" SET "AutomatickeVytvareni"=?1,"AutomatickeOdesilani"=?2,"PredmetEmailu"=?3,"TextEmailu"=?4,"Pojistnik"=?5,"CisloSmlouvy"=?6,"Aktualizovano"=CURRENT_TIMESTAMP WHERE "Id"=1"#,
        params![settings.automatic_creation as i64,settings.automatic_sending as i64,settings.email_subject.trim(),settings.email_body.trim(),settings.policyholder.trim(),settings.contract_number.trim()])
        .map_err(|_| "Nastavení dokladů se nepodařilo uložit.".to_string())?;
    Ok(())
}

fn snapshot(connection: &Connection, row_id: i64, year: i32) -> Result<Snapshot, String> {
    let result = connection.query_row(
        r#"SELECT COALESCE("Identifikátor",''),COALESCE("Titul",''),COALESCE("Jméno",''),COALESCE("Příjmení",''),
          COALESCE("RodnéČíslo",''),COALESCE(CAST("EvČíslo" AS TEXT),''),COALESCE("ZO",''),COALESCE("Adresa",''),
          COALESCE("Město",''),COALESCE("PSČ",''),COALESCE("Stát",'Česká republika'),COALESCE("Kategorie",''),
          COALESCE("PojištěníOd",''),COALESCE("PojištěníDo",''),COALESCE("RočPojistné",0),COALESCE("PojistnáČástka",0),
          COALESCE((SELECT SUM("Castka") FROM "PlatbyClenu" WHERE "PojistnyZaznamRowId"=Seznam.rowid),COALESCE("SkutÚhrada",0)),
          COALESCE("e-mail",''),COALESCE(CAST("KódOC" AS TEXT),''),COALESCE("Ztráta",0),COALESCE("Ukončení",''),
          COALESCE((SELECT "Id" FROM "PlatbyClenu" WHERE "PojistnyZaznamRowId"=Seznam.rowid ORDER BY "DatumPrijeti" DESC,"Id" DESC LIMIT 1),0),
          COALESCE((SELECT "DatumPrijeti" FROM "PlatbyClenu" WHERE "PojistnyZaznamRowId"=Seznam.rowid ORDER BY "DatumPrijeti" DESC,"Id" DESC LIMIT 1),'')
          FROM "Seznam" WHERE rowid=?1 AND substr(CAST("PojištěníOd" AS TEXT),1,4)=CAST(?2 AS TEXT)"#,
        params![row_id,year], |row| Ok(Snapshot { identifier:row.get(0)?, title:row.get(1)?, first_name:row.get(2)?, last_name:row.get(3)?, personal_id:row.get(4)?, registration:row.get(5)?, organization:row.get(6)?, address:row.get(7)?, city:row.get(8)?, postal_code:row.get(9)?, country:row.get(10)?, category:row.get(11)?, insurance_from:row.get(12)?, insurance_to:row.get(13)?, insured_amount:row.get(14)?, premium:row.get(15)?, paid:row.get(16)?, email:row.get(17)?, organization_code:row.get(18)?, loss_insurance:row.get::<_,i64>(19)? != 0, termination:row.get(20)?, payment_id:row.get(21)?, paid_on:row.get(22)? })
    );
    match result {
        Ok(snapshot) => Ok(snapshot),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(format!(
            "Pro otevřeného člena nebylo nalezeno pojištění pro rok {year}."
        )),
        Err(error) => {
            eprintln!("payment_document_basis_load_failed command=get_payment_document_basis member_rowid={row_id} year={year} category=database_query database_error={error}");
            Err("Podklady dokladu se nepodařilo načíst.".to_string())
        }
    }
}

pub fn load_basis(
    connection: &Connection,
    row_id: i64,
    year: i32,
) -> Result<PaymentDocumentBasis, String> {
    let data = snapshot(connection, row_id, year)?;
    validate_required(&data)?;
    let settings = load_settings(connection)?;
    if settings.contract_number.trim().is_empty() {
        return Err("Doklad nelze vystavit: chybí číslo pojistné smlouvy.".into());
    }
    let mut statement = connection.prepare(
        r#"SELECT "DatumPrijeti" FROM "PlatbyClenu"
           WHERE "PojistnyZaznamRowId"=?1 ORDER BY "DatumPrijeti", "Id""#,
    ).map_err(|error| {
        eprintln!("payment_document_basis_load_failed command=get_payment_document_basis member_rowid={row_id} year={year} category=payment_query database_error={error}");
        "Podklady dokladu se nepodařilo načíst.".to_string()
    })?;
    let payment_dates = statement.query_map([row_id], |row| row.get(0))
        .and_then(|rows| rows.collect::<rusqlite::Result<Vec<String>>>())
        .map_err(|error| {
            eprintln!("payment_document_basis_load_failed command=get_payment_document_basis member_rowid={row_id} year={year} category=payment_mapping database_error={error}");
            "Podklady dokladu se nepodařilo načíst.".to_string()
        })?;
    let member_name = format!("{} {} {}", data.title, data.first_name, data.last_name)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    Ok(PaymentDocumentBasis {
        member_row_id: row_id,
        member_name,
        registration_number: data.registration,
        organization_code: data.organization_code,
        insurance_year: year,
        prescribed_premium: data.premium,
        paid_amount: data.paid,
        payment_dates,
        contract_number: settings.contract_number,
        insurance_status: if data.termination.trim().is_empty() {
            "Aktivní".into()
        } else {
            "Ukončené".into()
        },
        loss_insurance: data.loss_insurance,
        certificate_ready: data.paid >= data.premium,
    })
}

fn validate_required(snapshot: &Snapshot) -> Result<(), String> {
    let required = [
        ("interní identifikátor člena", snapshot.identifier.trim()),
        ("jméno", snapshot.first_name.trim()),
        ("příjmení", snapshot.last_name.trim()),
        ("rodné číslo", snapshot.personal_id.trim()),
        ("evidenční číslo", snapshot.registration.trim()),
        ("kód OC", snapshot.organization_code.trim()),
        ("kategorie", snapshot.category.trim()),
        ("datum počátku pojištění", snapshot.insurance_from.trim()),
        ("datum konce pojištění", snapshot.insurance_to.trim()),
    ];
    if let Some((name, _)) = required.into_iter().find(|(_, value)| value.is_empty()) {
        return Err(format!("Doklad nelze vystavit: chybí {name}."));
    }
    if snapshot.insured_amount <= 0 {
        return Err("Doklad nelze vystavit: chybí pojistná částka.".into());
    }
    if snapshot.premium <= 0 {
        return Err("Doklad nelze vystavit: chybí roční pojistné.".into());
    }
    Ok(())
}

fn font_file() -> Result<File, String> {
    File::open(r"C:\Windows\Fonts\arial.ttf")
        .map_err(|_| "Písmo pro doklad není dostupné.".to_string())
}

fn add_background(layer: &printpdf::PdfLayerReference, bytes: &[u8]) -> Result<(), String> {
    let source = image::load_from_memory(bytes)
        .map_err(|_| "Šablonu dokladu se nepodařilo načíst.".to_string())?;
    let image = Image::from_dynamic_image(&source);
    image.add_to_layer(
        layer.clone(),
        ImageTransform {
            translate_x: Some(Mm(0.0)),
            translate_y: Some(Mm(0.0)),
            dpi: Some(144.0),
            ..Default::default()
        },
    );
    Ok(())
}

fn cz_date(value: &str) -> String {
    chrono::NaiveDate::parse_from_str(value.get(0..10).unwrap_or(value), "%Y-%m-%d")
        .map(|date| date.format("%d.%m.%Y").to_string())
        .unwrap_or_default()
}

fn money(value: i64) -> String {
    let digits = value.abs().to_string();
    let grouped = digits
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(" ");
    format!("{}{grouped},00 Kč", if value < 0 { "-" } else { "" })
}

fn white_box(layer: &printpdf::PdfLayerReference, x: f32, y: f32, width: f32, height: f32) {
    layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None)));
    layer.add_polygon(Polygon {
        rings: vec![vec![
            (Point::new(Mm(x), Mm(y)), false),
            (Point::new(Mm(x + width), Mm(y)), false),
            (Point::new(Mm(x + width), Mm(y + height)), false),
            (Point::new(Mm(x), Mm(y + height)), false),
        ]],
        mode: PaintMode::Fill,
        winding_order: WindingOrder::NonZero,
    });
}

fn create_pdf(snapshot: &Snapshot, path: &Path) -> Result<Vec<u8>, String> {
    let (document, page1, layer1) =
        PdfDocument::new("Certifikát o pojištění", Mm(210.0), Mm(297.0), "Strana 1");
    let first = document.get_page(page1).get_layer(layer1);
    add_background(&first, TEMPLATE_1)?;
    let font = document
        .add_external_font(font_file()?)
        .map_err(|_| "Písmo pro doklad se nepodařilo načíst.".to_string())?;
    white_box(&first, 84.0, 139.0, 27.0, 7.0);
    white_box(&first, 153.0, 92.0, 38.0, 7.0);
    white_box(&first, 8.0, 29.0, 45.0, 7.0);
    first.set_fill_color(Color::Rgb(Rgb::new(0.18, 0.46, 0.55, None)));
    first.use_text(
        cz_date(&snapshot.insurance_from),
        10.0,
        Mm(87.0),
        Mm(141.0),
        &font,
    );
    first.use_text(
        cz_date(&snapshot.insurance_to),
        10.0,
        Mm(153.0),
        Mm(94.0),
        &font,
    );
    first.use_text("řádně", 10.0, Mm(176.0), Mm(94.0), &font);
    first.use_text(
        format!(
            "V Praze dne {}",
            Local::now().date_naive().format("%d.%m.%Y")
        ),
        10.0,
        Mm(9.0),
        Mm(31.0),
        &font,
    );
    let (page2, layer2) = document.add_page(Mm(210.0), Mm(297.0), "Strana 2");
    let layer = document.get_page(page2).get_layer(layer2);
    add_background(&layer, TEMPLATE_2)?;
    white_box(&layer, 7.0, 43.5, 188.0, 49.5);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    layer.add_polygon(Polygon {
        rings: vec![vec![
            (Point::new(Mm(4.5), Mm(43.5)), false),
            (Point::new(Mm(195.5), Mm(43.5)), false),
            (Point::new(Mm(195.5), Mm(43.8)), false),
            (Point::new(Mm(4.5), Mm(43.8)), false),
        ]],
        mode: PaintMode::Fill,
        winding_order: WindingOrder::NonZero,
    });
    layer.set_fill_color(Color::Rgb(Rgb::new(0.18, 0.46, 0.55, None)));
    let name = format!(
        "{} {} {}",
        snapshot.title, snapshot.first_name, snapshot.last_name
    )
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ");
    let address = format!(
        "{}; {}; {}",
        snapshot.city, snapshot.address, snapshot.postal_code
    );
    let registration = format!(
        "{}{:04}",
        snapshot.organization_code,
        snapshot.registration.parse::<i64>().unwrap_or(0)
    );
    let values = [
        (9.0, 86.3, "Jméno a příjmení:".into()),
        (45.0, 86.3, name),
        (135.0, 86.3, "Rodné číslo:".into()),
        (161.0, 86.3, snapshot.personal_id.clone()),
        (9.0, 79.8, "Evidenční číslo:".into()),
        (45.0, 79.8, registration),
        (102.0, 79.8, "Základní organizace:".into()),
        (151.0, 79.8, snapshot.organization.clone()),
        (9.0, 73.4, "Adresa bydliště:".into()),
        (45.0, 73.4, address),
        (9.0, 66.8, "Stát:".into()),
        (45.0, 66.8, snapshot.country.clone()),
        (9.0, 60.3, "Platnost pojištění od:".into()),
        (56.0, 60.3, cz_date(&snapshot.insurance_from)),
        (82.0, 60.3, "do:".into()),
        (95.0, 60.3, cz_date(&snapshot.insurance_to)),
        (9.0, 53.7, "Limit pojistného plnění:".into()),
        (57.0, 53.7, money(snapshot.insured_amount)),
        (112.0, 53.7, "Kategorie:".into()),
        (135.0, 53.7, snapshot.category.clone()),
        (9.0, 47.2, "Pojistné:".into()),
        (31.0, 47.2, money(snapshot.premium)),
        (74.0, 47.2, "Uhrazeno:".into()),
        (98.0, 47.2, money(snapshot.paid)),
    ];
    for (x, y, text) in values {
        layer.use_text(text, 9.0, Mm(x), Mm(y), &font);
    }
    let (page3, layer3) = document.add_page(Mm(210.0), Mm(297.0), "Strana 3");
    add_background(&document.get_page(page3).get_layer(layer3), TEMPLATE_3)?;
    document
        .save(&mut BufWriter::new(
            File::create(path).map_err(|_| "PDF se nepodařilo uložit.".to_string())?,
        ))
        .map_err(|_| "PDF se nepodařilo vytvořit.".to_string())?;
    std::fs::read(path).map_err(|_| "PDF se nepodařilo načíst.".to_string())
}

fn map_receipt(row: &rusqlite::Row<'_>) -> rusqlite::Result<Receipt> {
    Ok(Receipt {
        id: row.get(0)?,
        member_row_id: row.get(1)?,
        member_identifier: row.get(2)?,
        payment_id: row.get(3)?,
        registration_number: row.get(4)?,
        member_name: row.get(5)?,
        insurance_year: row.get(6)?,
        paid_on: row.get(7)?,
        issued_on: row.get(8)?,
        amount: row.get(9)?,
        contract_number: row.get(10)?,
        status: row.get(11)?,
        email_status: row.get(12)?,
        sent_at: row.get(13)?,
        recipient_email: row.get(14)?,
        checksum: row.get(15)?,
    })
}

pub fn list(
    connection: &Connection,
    member_row_id: Option<i64>,
    search: &str,
) -> Result<Vec<Receipt>, String> {
    ensure_schema(connection).map_err(|_| "Doklady se nepodařilo načíst.".to_string())?;
    let pattern = format!("%{}%", search.trim());
    let mut statement=connection.prepare(r#"SELECT "Id","PojistnyZaznamRowId","IdentifikatorClena","IdPlatby","EvidencniCislo","JmenoClena","PojistnyRok","DatumUhrady","DatumVystaveni","Castka","CisloSmlouvy","Stav","StavEmailu","DatumOdeslani","EmailPrijemce","Sha256" FROM "DokladyOUhrade" WHERE (?1 IS NULL OR "PojistnyZaznamRowId"=?1) AND (?2='' OR "JmenoClena" LIKE ?3 COLLATE NOCASE OR "EvidencniCislo" LIKE ?3 OR CAST("PojistnyRok" AS TEXT) LIKE ?3 OR COALESCE("EmailPrijemce",'') LIKE ?3 COLLATE NOCASE) ORDER BY "DatumVystaveni" DESC,"Id" DESC"#).map_err(|_| "Doklady se nepodařilo načíst.".to_string())?;
    let rows = statement
        .query_map(params![member_row_id, search.trim(), pattern], map_receipt)
        .map_err(|_| "Doklady se nepodařilo načíst.".to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "Doklady se nepodařilo načíst.".to_string())
}

pub fn create_if_eligible(
    database: &Path,
    user: &str,
    row_id: i64,
    year: i32,
    automatic: bool,
) -> Result<Option<i64>, String> {
    let mut connection =
        Connection::open(database).map_err(|_| "Doklad se nepodařilo vytvořit.".to_string())?;
    ensure_schema(&connection).map_err(|_| "Doklad se nepodařilo vytvořit.".to_string())?;
    let settings = load_settings(&connection)?;
    if automatic && !settings.automatic_creation {
        return Ok(None);
    }
    let data = snapshot(&connection, row_id, year)?;
    validate_required(&data)?;
    if data.paid < data.premium {
        return Ok(None);
    }
    if let Some(id)=connection.query_row(r#"SELECT "Id" FROM "DokladyOUhrade" WHERE "IdentifikatorClena"=?1 AND "PojistnyRok"=?2"#,params![data.identifier,year],|row|row.get(0)).optional().map_err(|_| "Doklad se nepodařilo ověřit.".to_string())? { return Ok(Some(id)); }
    let temp = std::env::temp_dir().join(format!("doklad-{}-{}.pdf", data.identifier, year));
    let pdf = match create_pdf(&data, &temp) {
        Ok(pdf) => pdf,
        Err(error) => {
            connection.execute(
                r#"INSERT INTO "AuditDokladu"("Uzivatel","IdentifikatorClena","Operace","Vysledek") VALUES(?1,?2,'CHYBA GENEROVÁNÍ','ERROR')"#,
                params![user,data.identifier],
            ).ok();
            return Err(error);
        }
    };
    let checksum = format!("{:x}", Sha256::digest(&pdf));
    let issued = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let name = format!("{} {} {}", data.title, data.first_name, data.last_name)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| "Doklad se nepodařilo uložit.".to_string())?;
    transaction.execute(r#"INSERT INTO "DokladyOUhrade"("PojistnyZaznamRowId","IdentifikatorClena","IdPlatby","EvidencniCislo","JmenoClena","PojistnyRok","DatumUhrady","DatumVystaveni","Castka","CisloSmlouvy","EmailPrijemce","Pdf","Sha256") VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,NULLIF(?11,''),?12,?13)"#,params![row_id,data.identifier,data.payment_id,data.registration,name,year,data.paid_on,issued,data.paid,settings.contract_number,data.email,pdf,checksum]).map_err(|_| "Doklad se nepodařilo uložit.".to_string())?;
    let id = transaction.last_insert_rowid();
    transaction.execute(r#"INSERT INTO "AuditDokladu"("Uzivatel","IdDokladu","IdentifikatorClena","Operace","Vysledek") VALUES(?1,?2,?3,?4,'OK')"#,params![user,id,data.identifier,if automatic{"AUTOMATICKÉ VYTVOŘENÍ"}else{"RUČNÍ VYTVOŘENÍ"}]).map_err(|_| "Doklad se nepodařilo zaznamenat.".to_string())?;
    transaction
        .commit()
        .map_err(|_| "Doklad se nepodařilo uložit.".to_string())?;
    if settings.automatic_sending {
        if data.email.trim().is_empty() {
            connection
                .execute(
                    r#"UPDATE "DokladyOUhrade" SET "StavEmailu"='Chybí e-mail' WHERE "Id"=?1"#,
                    [id],
                )
                .ok();
        } else {
            let _ = send(database, user, id);
        }
    }
    Ok(Some(id))
}

pub fn pdf(connection: &Connection, id: i64) -> Result<(String, Vec<u8>), String> {
    connection.query_row(r#"SELECT printf('Doklad_%d_%s.pdf',"PojistnyRok",replace("EvidencniCislo",' ','')),"Pdf" FROM "DokladyOUhrade" WHERE "Id"=?1"#,[id],|row|Ok((row.get(0)?,row.get(1)?))).map_err(|_|"Doklad se nepodařilo načíst.".to_string())
}

pub fn send(database: &Path, user: &str, id: i64) -> Result<(), String> {
    let connection =
        Connection::open(database).map_err(|_| "Doklad se nepodařilo odeslat.".to_string())?;
    ensure_schema(&connection).map_err(|_| "Doklad se nepodařilo odeslat.".to_string())?;
    let settings = load_settings(&connection)?;
    let (recipient,name,pdf,identifier):(String,String,Vec<u8>,String)=connection.query_row(r#"SELECT COALESCE("EmailPrijemce",''),printf('Doklad_%d_%s.pdf',"PojistnyRok",replace("EvidencniCislo",' ','')),"Pdf","IdentifikatorClena" FROM "DokladyOUhrade" WHERE "Id"=?1"#,[id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).map_err(|_|"Doklad se nepodařilo načíst.".to_string())?;
    if recipient.trim().is_empty() {
        return Err("Člen nemá vyplněný e-mail.".into());
    }
    let result = email_service::send(
        &connection,
        EmailMessage {
            recipient: &recipient,
            subject: &settings.email_subject,
            body: &settings.email_body,
            attachment_name: &name,
            attachment: pdf,
        },
    );
    let (status, operation) = if result.is_ok() {
        ("Odeslán", "ODESLÁNÍ")
    } else {
        ("Chyba", "CHYBA E-MAILU")
    };
    connection.execute(r#"UPDATE "DokladyOUhrade" SET "StavEmailu"=?1,"DatumOdeslani"=CASE WHEN ?1='Odeslán' THEN CURRENT_TIMESTAMP ELSE "DatumOdeslani" END WHERE "Id"=?2"#,params![status,id]).ok();
    connection.execute(r#"INSERT INTO "AuditDokladu"("Uzivatel","IdDokladu","IdentifikatorClena","Operace","Vysledek") VALUES(?1,?2,?3,?4,?5)"#,params![user,id,identifier,operation,if result.is_ok(){"OK"}else{"ERROR"}]).ok();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn receipt_database(
        email: Option<&str>,
        address: Option<&str>,
        first_name: Option<&str>,
        include_payment: bool,
    ) -> (tempfile::TempDir, std::path::PathBuf, i64) {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("receipt.sqlite");
        let connection = Connection::open(&database).unwrap();
        connection
            .execute_batch(
                r#"CREATE TABLE "Seznam" (
                "Identifikátor" TEXT, "Titul" TEXT, "Jméno" TEXT, "Příjmení" TEXT,
                "RodnéČíslo" TEXT, "EvČíslo" INTEGER, "ZO" TEXT, "Adresa" TEXT,
                "Město" TEXT, "PSČ" TEXT, "Stát" TEXT, "Kategorie" TEXT,
                "PojištěníOd" TEXT, "PojištěníDo" TEXT, "PojistnáČástka" INTEGER,
                "RočPojistné" INTEGER, "SkutÚhrada" INTEGER, "e-mail" TEXT, "KódOC" TEXT
                , "Ztráta" INTEGER, "Ukončení" TEXT
            );
            CREATE TABLE "PlatbyClenu" (
                "Id" INTEGER PRIMARY KEY AUTOINCREMENT,
                "PojistnyZaznamRowId" INTEGER NOT NULL,
                "DatumPrijeti" TEXT NOT NULL,
                "Castka" INTEGER NOT NULL
            );"#,
            )
            .unwrap();
        connection.execute(
            r#"INSERT INTO "Seznam" VALUES
               ('member-1','Ing.',?1,'Novák','780101/1234',53,'FVČ',?2,'Praha','110 00','Česká republika','B',
                '2026-01-01 00:00:00','2026-12-31 00:00:00',781,320000,781,?3,'2',-1,NULL)"#,
            params![first_name,address,email],
        ).unwrap();
        let inserted_row_id = connection.last_insert_rowid();
        let row_id = 42;
        connection
            .execute(
                "UPDATE \"Seznam\" SET rowid=?1 WHERE rowid=?2",
                params![row_id, inserted_row_id],
            )
            .unwrap();
        if include_payment {
            connection.execute(
                r#"INSERT INTO "PlatbyClenu"("PojistnyZaznamRowId","DatumPrijeti","Castka") VALUES(?1,'2026-07-31',781)"#,
                [row_id],
            ).unwrap();
        }
        ensure_schema(&connection).unwrap();
        (directory, database, row_id)
    }

    #[test]
    fn formats_czech_money() {
        assert_eq!(money(320_000), "320 000,00 Kč");
        assert_eq!(money(781), "781,00 Kč");
    }

    #[test]
    fn receipt_schema_prevents_duplicate_member_year() {
        let connection = Connection::open_in_memory().unwrap();
        ensure_schema(&connection).unwrap();
        let insert = |id: i64| {
            connection.execute(r#"INSERT INTO "DokladyOUhrade"("PojistnyZaznamRowId","IdentifikatorClena","IdPlatby","EvidencniCislo","JmenoClena","PojistnyRok","DatumUhrady","DatumVystaveni","Castka","CisloSmlouvy","Pdf","Sha256") VALUES(1,'member-1',?1,'1','Test',2026,'2026-01-01','2026-01-01',500,'650',X'25','hash')"#,[id])
        };
        assert!(insert(1).is_ok());
        assert!(insert(2).is_err());
    }

    #[test]
    fn settings_preserve_automatic_flags() {
        let connection = Connection::open_in_memory().unwrap();
        ensure_schema(&connection).unwrap();
        let settings = ReceiptSettings {
            automatic_creation: true,
            automatic_sending: true,
            email_subject: "Doklad".into(),
            email_body: "Text".into(),
            policyholder: "Federace".into(),
            contract_number: "650".into(),
        };
        save_settings(&connection, &settings).unwrap();
        let loaded = load_settings(&connection).unwrap();
        assert!(loaded.automatic_creation && loaded.automatic_sending);
    }

    #[test]
    fn creates_three_page_access_layout_pdf() {
        let path = std::env::temp_dir().join("pojisteni-receipt-layout-test.pdf");
        let snapshot = Snapshot {
            identifier: "test-member".into(),
            payment_id: 1,
            title: "Ing.".into(),
            first_name: "Jan".into(),
            last_name: "Novák".into(),
            personal_id: "780101/1234".into(),
            registration: "53".into(),
            organization_code: "2".into(),
            organization: "FVČ".into(),
            address: "Testovací 1".into(),
            city: "Praha".into(),
            postal_code: "110 00".into(),
            country: "Česká republika".into(),
            category: "B".into(),
            insurance_from: "2026-01-01".into(),
            insurance_to: "2026-12-31".into(),
            insured_amount: 320_000,
            premium: 781,
            paid: 781,
            paid_on: "2026-07-31".into(),
            email: "test@example.invalid".into(),
            loss_insurance: true,
            termination: String::new(),
        };
        let pdf = create_pdf(&snapshot, &path).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        assert!(pdf.len() > 100_000);
    }

    #[test]
    fn fully_paid_member_without_existing_receipt_can_create_one() {
        let (_directory, database, row_id) = receipt_database(
            Some("jan@example.invalid"),
            Some("Testovací 1"),
            Some("Jan"),
            false,
        );
        let connection = Connection::open(&database).unwrap();
        assert!(list(&connection, Some(row_id), "").unwrap().is_empty());
        drop(connection);
        let id = create_if_eligible(&database, "tester", row_id, 2026, false).unwrap();
        assert!(id.is_some());
    }

    #[test]
    fn existing_receipt_is_returned_without_duplicate() {
        let (_directory, database, row_id) = receipt_database(
            Some("jan@example.invalid"),
            Some("Testovací 1"),
            Some("Jan"),
            true,
        );
        let first = create_if_eligible(&database, "tester", row_id, 2026, false)
            .unwrap()
            .unwrap();
        let second = create_if_eligible(&database, "tester", row_id, 2026, false)
            .unwrap()
            .unwrap();
        assert_eq!(first, second);
        let connection = Connection::open(database).unwrap();
        assert_eq!(
            connection
                .query_row(r#"SELECT COUNT(*) FROM "DokladyOUhrade""#, [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }

    #[test]
    fn missing_email_is_optional() {
        let (_directory, database, row_id) =
            receipt_database(None, Some("Testovací 1"), Some("Jan"), true);
        let id = create_if_eligible(&database, "tester", row_id, 2026, false)
            .unwrap()
            .unwrap();
        let connection = Connection::open(database).unwrap();
        let email: Option<String> = connection
            .query_row(
                r#"SELECT "EmailPrijemce" FROM "DokladyOUhrade" WHERE "Id"=?1"#,
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(email.is_none());
    }

    #[test]
    fn null_optional_member_field_is_safe() {
        let (_directory, database, row_id) =
            receipt_database(Some("jan@example.invalid"), None, Some("Jan"), true);
        assert!(create_if_eligible(&database, "tester", row_id, 2026, false)
            .unwrap()
            .is_some());
    }

    #[test]
    fn missing_required_member_field_names_the_field() {
        let (_directory, database, row_id) =
            receipt_database(Some("jan@example.invalid"), Some("Testovací 1"), None, true);
        let error = create_if_eligible(&database, "tester", row_id, 2026, false).unwrap_err();
        assert_eq!(error, "Doklad nelze vystavit: chybí jméno.");
    }

    #[test]
    fn snapshot_uses_open_member_year_and_amounts() {
        let (_directory, database, row_id) = receipt_database(
            Some("jan@example.invalid"),
            Some("Testovací 1"),
            Some("Jan"),
            true,
        );
        let connection = Connection::open(database).unwrap();
        let data = snapshot(&connection, row_id, 2026).unwrap();
        assert_eq!(data.identifier, "member-1");
        assert_eq!(data.registration, "53");
        assert_eq!(data.premium, 781);
        assert_eq!(data.paid, 781);
        assert_eq!(data.insurance_from, "2026-01-01 00:00:00");
        assert!(snapshot(&connection, row_id, 2025)
            .unwrap_err()
            .contains("rok 2025"));
    }

    #[test]
    fn basis_uses_rowid_not_registration_number() {
        let (_directory, database, row_id) = receipt_database(
            Some("jan@example.invalid"),
            Some("Testovací 1"),
            Some("Jan"),
            true,
        );
        assert_eq!(row_id, 42);
        let connection = Connection::open(database).unwrap();
        let basis = load_basis(&connection, row_id, 2026).unwrap();
        assert_eq!(basis.member_row_id, 42);
        assert_eq!(basis.registration_number, "53");
        assert_eq!(basis.prescribed_premium, 781);
        assert_eq!(basis.paid_amount, 781);
        assert!(basis.certificate_ready && basis.loss_insurance);
    }

    #[test]
    fn transferred_payment_without_payment_rows_uses_stored_total() {
        let (_directory, database, row_id) = receipt_database(None, None, Some("Jan"), false);
        let connection = Connection::open(database).unwrap();
        let basis = load_basis(&connection, row_id, 2026).unwrap();
        assert_eq!(basis.paid_amount, 781);
        assert!(basis.payment_dates.is_empty());
        assert!(basis.certificate_ready);
    }

    #[test]
    fn multiple_payments_are_summed_from_payment_module() {
        let (_directory, database, row_id) = receipt_database(None, None, Some("Jan"), false);
        let connection = Connection::open(&database).unwrap();
        connection
            .execute(
                r#"UPDATE "Seznam" SET "SkutÚhrada"=0 WHERE rowid=?1"#,
                [row_id],
            )
            .unwrap();
        connection.execute(r#"INSERT INTO "PlatbyClenu"("PojistnyZaznamRowId","DatumPrijeti","Castka") VALUES(?1,'2026-03-01',400),(?1,'2026-04-01',381)"#,[row_id]).unwrap();
        let basis = load_basis(&connection, row_id, 2026).unwrap();
        assert_eq!(basis.paid_amount, 781);
        assert_eq!(basis.payment_dates, vec!["2026-03-01", "2026-04-01"]);
        assert!(basis.certificate_ready);
    }

    #[test]
    fn zero_payment_loads_basis_but_is_not_ready() {
        let (_directory, database, row_id) = receipt_database(None, None, Some("Jan"), false);
        let connection = Connection::open(&database).unwrap();
        connection
            .execute(
                r#"UPDATE "Seznam" SET "SkutÚhrada"=0 WHERE rowid=?1"#,
                [row_id],
            )
            .unwrap();
        let basis = load_basis(&connection, row_id, 2026).unwrap();
        assert_eq!(basis.paid_amount, 0);
        assert!(!basis.certificate_ready);
    }

    #[test]
    fn missing_contract_number_is_reported() {
        let (_directory, database, row_id) = receipt_database(None, None, Some("Jan"), false);
        let connection = Connection::open(&database).unwrap();
        connection
            .execute(
                r#"UPDATE "NastaveniDokladu" SET "CisloSmlouvy"='' WHERE "Id"=1"#,
                [],
            )
            .unwrap();
        assert_eq!(
            load_basis(&connection, row_id, 2026).unwrap_err(),
            "Doklad nelze vystavit: chybí číslo pojistné smlouvy."
        );
    }

    #[test]
    fn nonexistent_member_returns_safe_error() {
        let (_directory, database, _) = receipt_database(None, None, Some("Jan"), false);
        let connection = Connection::open(database).unwrap();
        assert!(load_basis(&connection, 999, 2026)
            .unwrap_err()
            .contains("rok 2026"));
    }

    #[test]
    fn kostal_ales_current_basis_loads_for_registration_one() {
        let (_directory, database, row_id) = receipt_database(
            Some("ales@example.invalid"),
            Some("Testovací 1"),
            Some("Aleš"),
            true,
        );
        let connection = Connection::open(&database).unwrap();
        connection.execute(r#"UPDATE "Seznam" SET "Příjmení"='Košťál',"EvČíslo"=1,"PojistnáČástka"=653,"SkutÚhrada"=653 WHERE rowid=?1"#,[row_id]).unwrap();
        connection
            .execute(
                r#"UPDATE "PlatbyClenu" SET "Castka"=653 WHERE "PojistnyZaznamRowId"=?1"#,
                [row_id],
            )
            .unwrap();
        let basis = load_basis(&connection, row_id, 2026).unwrap();
        assert_eq!(basis.member_name, "Ing. Aleš Košťál");
        assert_eq!(basis.registration_number, "1");
        assert_eq!((basis.prescribed_premium, basis.paid_amount), (653, 653));
        assert!(basis.certificate_ready);
    }
}
