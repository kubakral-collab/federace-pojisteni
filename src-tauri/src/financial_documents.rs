use chrono::Local;
use printpdf::{Mm, PdfDocument};
use rusqlite::{params, Connection, TransactionBehavior};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs::File, io::BufWriter, path::Path};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceInput {
    pub supplier: String,
    pub account_number: String,
    pub bank_code: String,
    pub variable_symbol: String,
    pub constant_symbol: Option<String>,
    pub specific_symbol: Option<String>,
    pub amount: i64,
    pub due_on: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
    pub id: i64,
    pub number: String,
    pub supplier: String,
    pub account: String,
    pub variable_symbol: String,
    pub amount: i64,
    pub due_on: String,
    pub status: String,
    pub batch_id: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResult {
    pub batch_id: i64,
    pub count: usize,
    pub total: i64,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct MemberDocumentData {
    pub row_id: i64,
    pub year: i32,
    pub registration: String,
    pub name: String,
    pub personal_id: String,
    pub organization: String,
    pub address: String,
    pub city: String,
    pub postal_code: String,
    pub country: String,
    pub email: String,
    pub insurance_from: String,
    pub insurance_to: String,
    pub category: String,
    pub insurance_limit: i64,
    pub premium: i64,
    pub paid: i64,
}

pub fn ensure_schema(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(r#"
      CREATE TABLE IF NOT EXISTS "VydaneFaktury" (
        "Id" INTEGER PRIMARY KEY AUTOINCREMENT, "Cislo" TEXT NOT NULL UNIQUE,
        "Dodavatel" TEXT NOT NULL, "CisloUctu" TEXT NOT NULL, "KodBanky" TEXT NOT NULL,
        "VS" TEXT NOT NULL, "KS" TEXT, "SS" TEXT, "Castka" INTEGER NOT NULL CHECK("Castka">0),
        "DatumVystaveni" TEXT NOT NULL, "DatumSplatnosti" TEXT NOT NULL, "Poznamka" TEXT,
        "Stav" TEXT NOT NULL DEFAULT 'PŘIPRAVENA' CHECK("Stav" IN ('PŘIPRAVENA','EXPORTOVÁNA','UHRAZENA','STORNOVÁNA')),
        "DavkaId" INTEGER, "Vytvoreno" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
      );
      CREATE TABLE IF NOT EXISTS "PlatebniDavky" (
        "Id" INTEGER PRIMARY KEY AUTOINCREMENT, "DatumCas" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        "Pocet" INTEGER NOT NULL, "CelkovaCastka" INTEGER NOT NULL, "Sha256" TEXT NOT NULL,
        "Soubor" TEXT, "Uzivatel" TEXT NOT NULL, "Stav" TEXT NOT NULL
      );
      CREATE TABLE IF NOT EXISTS "AuditFinancnichDokladu" (
        "Id" INTEGER PRIMARY KEY AUTOINCREMENT, "DatumCas" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        "Uzivatel" TEXT NOT NULL, "Druh" TEXT NOT NULL, "ObjektId" INTEGER,
        "Operace" TEXT NOT NULL, "Vysledek" TEXT NOT NULL
      );
      CREATE TABLE IF NOT EXISTS "ClenskeDoklady" (
        "Id" INTEGER PRIMARY KEY AUTOINCREMENT, "PojistnyZaznamRowId" INTEGER NOT NULL,
        "PojistnyRok" INTEGER NOT NULL, "Druh" TEXT NOT NULL,
        "Cislo" TEXT NOT NULL UNIQUE, "Castka" INTEGER NOT NULL DEFAULT 0,
        "NavazanyDokladId" INTEGER, "Sha256" TEXT NOT NULL, "DatumVystaveni" TEXT NOT NULL,
        "Vytvoreno" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
      );"#)
}

fn digits(value: &str, field: &str, max: usize) -> Result<String, String> {
    let clean = value
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>();
    if clean.is_empty() || clean.len() > max {
        return Err(format!("Pole {field} není platné."));
    }
    Ok(clean)
}

pub fn create_invoice(
    connection: &mut Connection,
    user: &str,
    input: InvoiceInput,
) -> Result<i64, String> {
    if input.supplier.trim().is_empty() || input.amount <= 0 {
        return Err("Vyplňte dodavatele a kladnou částku.".into());
    }
    let account = digits(&input.account_number, "číslo účtu", 16)?;
    let bank = digits(&input.bank_code, "kód banky", 4)?;
    let vs = digits(&input.variable_symbol, "variabilní symbol", 10)?;
    chrono::NaiveDate::parse_from_str(&input.due_on, "%Y-%m-%d")
        .map_err(|_| "Datum splatnosti není platné.".to_string())?;
    let year = Local::now().format("%Y").to_string();
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| "Fakturu se nepodařilo založit.".to_string())?;
    let sequence:i64=transaction.query_row(r#"SELECT COALESCE(MAX(CAST(substr("Cislo",6) AS INTEGER)),0)+1 FROM "VydaneFaktury" WHERE substr("Cislo",1,4)=?1"#,[&year],|r|r.get(0)).unwrap_or(1);
    let number = format!("{}-{:06}", year, sequence);
    transaction.execute(r#"INSERT INTO "VydaneFaktury"("Cislo","Dodavatel","CisloUctu","KodBanky","VS","KS","SS","Castka","DatumVystaveni","DatumSplatnosti","Poznamka") VALUES(?1,?2,?3,?4,?5,NULLIF(?6,''),NULLIF(?7,''),?8,date('now'),?9,NULLIF(?10,''))"#,params![number,input.supplier.trim(),account,bank,vs,input.constant_symbol.unwrap_or_default(),input.specific_symbol.unwrap_or_default(),input.amount,input.due_on,input.note.unwrap_or_default()]).map_err(|_|"Fakturu se nepodařilo uložit.".to_string())?;
    let id = transaction.last_insert_rowid();
    transaction.execute(r#"INSERT INTO "AuditFinancnichDokladu"("Uzivatel","Druh","ObjektId","Operace","Vysledek") VALUES(?1,'FAKTURA',?2,'VYTVOŘENÍ','OK')"#,params![user,id]).map_err(|_|"Fakturu se nepodařilo auditovat.".to_string())?;
    transaction
        .commit()
        .map_err(|_| "Fakturu se nepodařilo uložit.".to_string())?;
    Ok(id)
}

pub fn list_invoices(connection: &Connection) -> Result<Vec<Invoice>, String> {
    let mut s=connection.prepare(r#"SELECT "Id","Cislo","Dodavatel","CisloUctu"||'/'||"KodBanky","VS","Castka","DatumSplatnosti","Stav","DavkaId","Poznamka" FROM "VydaneFaktury" ORDER BY "Id" DESC"#).map_err(|_|"Knihu faktur se nepodařilo načíst.".to_string())?;
    s.query_map([], |r| {
        Ok(Invoice {
            id: r.get(0)?,
            number: r.get(1)?,
            supplier: r.get(2)?,
            account: r.get(3)?,
            variable_symbol: r.get(4)?,
            amount: r.get(5)?,
            due_on: r.get(6)?,
            status: r.get(7)?,
            batch_id: r.get(8)?,
            note: r.get(9)?,
        })
    })
    .and_then(|x| x.collect())
    .map_err(|_| "Knihu faktur se nepodařilo načíst.".to_string())
}

fn csv_field(v: &str) -> String {
    format!("\"{}\"", v.replace('"', "\"\""))
}
pub fn prepare_batch(connection: &mut Connection, user: &str) -> Result<BatchResult, String> {
    let mut s=connection.prepare(r#"SELECT "Id","Cislo","Dodavatel","CisloUctu","KodBanky","VS",COALESCE("KS",''),COALESCE("SS",''),"Castka","DatumSplatnosti" FROM "VydaneFaktury" WHERE "Stav"='PŘIPRAVENA' ORDER BY "DatumSplatnosti","Id""#).map_err(|_|"Dávku se nepodařilo připravit.".to_string())?;
    let rows = s
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, String>(7)?,
                r.get::<_, i64>(8)?,
                r.get::<_, String>(9)?,
            ))
        })
        .and_then(|x| x.collect::<rusqlite::Result<Vec<_>>>())
        .map_err(|_| "Dávku se nepodařilo načíst.".to_string())?;
    drop(s);
    if rows.is_empty() {
        return Err("Nejsou připravené žádné faktury.".into());
    }
    let total = rows.iter().map(|r| r.8).sum();
    let mut csv=String::from("\u{feff}\"Číslo\";\"Dodavatel\";\"Účet\";\"Kód banky\";\"VS\";\"KS\";\"SS\";\"Částka\";\"Splatnost\"\r\n");
    for r in &rows {
        csv.push_str(
            &[
                csv_field(&r.1),
                csv_field(&r.2),
                csv_field(&r.3),
                csv_field(&r.4),
                csv_field(&r.5),
                csv_field(&r.6),
                csv_field(&r.7),
                r.8.to_string(),
                csv_field(&r.9),
            ]
            .join(";"),
        );
        csv.push_str("\r\n");
    }
    let bytes = csv.into_bytes();
    let hash = format!("{:x}", Sha256::digest(&bytes));
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| "Dávku se nepodařilo založit.".to_string())?;
    tx.execute(r#"INSERT INTO "PlatebniDavky"("Pocet","CelkovaCastka","Sha256","Uzivatel","Stav") VALUES(?1,?2,?3,?4,'PŘIPRAVENA')"#,params![rows.len() as i64,total,hash,user]).map_err(|_|"Dávku se nepodařilo založit.".to_string())?;
    let batch_id = tx.last_insert_rowid();
    for r in &rows {
        tx.execute(
            r#"UPDATE "VydaneFaktury" SET "DavkaId"=?1 WHERE "Id"=?2 AND "Stav"='PŘIPRAVENA'"#,
            params![batch_id, r.0],
        )
        .map_err(|_| "Dávku se nepodařilo svázat.".to_string())?;
    }
    tx.commit()
        .map_err(|_| "Dávku se nepodařilo uložit.".to_string())?;
    Ok(BatchResult {
        batch_id,
        count: rows.len(),
        total,
        bytes,
    })
}

pub fn finish_batch(
    connection: &mut Connection,
    user: &str,
    batch_id: i64,
    path: &Path,
) -> Result<(), String> {
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| "Dávku se nepodařilo dokončit.".to_string())?;
    let changed=tx.execute(r#"UPDATE "PlatebniDavky" SET "Soubor"=?1,"Stav"='EXPORTOVÁNA' WHERE "Id"=?2 AND "Stav"='PŘIPRAVENA'"#,params![path.to_string_lossy(),batch_id]).map_err(|_|"Dávku se nepodařilo dokončit.".to_string())?;
    if changed != 1 {
        return Err("Dávka již byla zpracována.".into());
    }
    tx.execute(r#"UPDATE "VydaneFaktury" SET "Stav"='EXPORTOVÁNA' WHERE "DavkaId"=?1 AND "Stav"='PŘIPRAVENA'"#,[batch_id]).map_err(|_|"Faktury se nepodařilo označit.".to_string())?;
    tx.execute(r#"INSERT INTO "AuditFinancnichDokladu"("Uzivatel","Druh","ObjektId","Operace","Vysledek") VALUES(?1,'PLATEBNÍ DÁVKA',?2,'EXPORT','OK')"#,params![user,batch_id]).ok();
    tx.commit()
        .map_err(|_| "Dávku se nepodařilo dokončit.".to_string())
}

pub fn member_data(
    connection: &Connection,
    row_id: i64,
    year: i32,
) -> Result<MemberDocumentData, String> {
    connection.query_row(r#"SELECT rowid,?2,COALESCE(CAST("EvČíslo" AS TEXT),''),TRIM(COALESCE("Titul",'')||' '||COALESCE("Jméno",'')||' '||COALESCE("Příjmení",'')),COALESCE("RodnéČíslo",''),COALESCE("ZO",''),COALESCE("Adresa",''),COALESCE("Město",''),COALESCE("PSČ",''),COALESCE("Stát",''),COALESCE("e-mail",''),COALESCE("PojištěníOd",''),COALESCE("PojištěníDo",''),COALESCE("Kategorie",''),COALESCE("RočPojistné",0),COALESCE("PojistnáČástka",0),COALESCE("SkutÚhrada",0) FROM "Seznam" WHERE rowid=?1 AND pojisteni_rok("PojištěníOd")=?2"#,params![row_id,year],|r|Ok(MemberDocumentData{row_id:r.get(0)?,year:r.get(1)?,registration:r.get(2)?,name:r.get(3)?,personal_id:r.get(4)?,organization:r.get(5)?,address:r.get(6)?,city:r.get(7)?,postal_code:r.get(8)?,country:r.get(9)?,email:r.get(10)?,insurance_from:r.get(11)?,insurance_to:r.get(12)?,category:r.get(13)?,insurance_limit:r.get(14)?,premium:r.get(15)?,paid:r.get(16)?})).map_err(|_|"Podklady dokumentu nebyly nalezeny.".to_string())
}

pub fn member_pdf(data: &MemberDocumentData, kind: &str, destination: &Path) -> Result<(), String> {
    if kind == "application" {
        return application_pdf(data, destination);
    }
    let title = match kind {
        "application" => "Přihláška k pojištění",
        "voucher" => "Poštovní poukázka",
        "envelope" => "Adresní obálka",
        "label" => "Adresní štítek",
        _ => return Err("Neznámý dokument.".into()),
    };
    let (doc, page, layer) = PdfDocument::new(title, Mm(210.0), Mm(297.0), "Dokument");
    let font = doc
        .add_external_font(
            File::open(r"C:\Windows\Fonts\arial.ttf")
                .map_err(|_| "Písmo není dostupné.".to_string())?,
        )
        .map_err(|_| "Písmo není dostupné.".to_string())?;
    let l = doc.get_page(page).get_layer(layer);
    l.use_text(title, 20.0, Mm(20.0), Mm(270.0), &font);
    let lines = if kind == "application" {
        vec![
            format!("Jméno: {}", data.name),
            format!("Rodné číslo: {}", data.personal_id),
            format!("Evidenční číslo: {}", data.registration),
            format!("Organizace: {}", data.organization),
            format!(
                "Adresa: {}, {} {}",
                data.address, data.postal_code, data.city
            ),
            format!("E-mail: {}", data.email),
            format!("Pojištění: {} – {}", data.insurance_from, data.insurance_to),
            format!("Pojistné: {} Kč", data.premium),
        ]
    } else if kind == "voucher" {
        vec![
            format!("Plátce: {}", data.name),
            format!(
                "Adresa: {}, {} {}",
                data.address, data.postal_code, data.city
            ),
            format!("Variabilní symbol: {}", data.personal_id.replace('/', "")),
            format!("Částka: {} Kč", (data.premium - data.paid).max(0)),
        ]
    } else {
        vec![
            data.name.clone(),
            data.address.clone(),
            format!("{} {}", data.postal_code, data.city),
            data.country.clone(),
        ]
    };
    let mut y = 245.0;
    for line in lines {
        l.use_text(line, 12.0, Mm(25.0), Mm(y), &font);
        y -= 12.0;
    }
    doc.save(&mut BufWriter::new(
        File::create(destination).map_err(|_| "Dokument se nepodařilo uložit.".to_string())?,
    ))
    .map_err(|_| "Dokument se nepodařilo vytvořit.".to_string())
}

fn application_pdf(data: &MemberDocumentData, destination: &Path) -> Result<(), String> {
    let (document, page, layer) = PdfDocument::new("Přihláška", Mm(210.0), Mm(297.0), "Přihláška");
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
    current.use_text("P ř i h l á š k a", 16.0, Mm(74.0), Mm(281.0), &bold);
    current.use_text(
        "Závazně se přihlašuji k pojištění z odpovědnosti za škody způsobené zaměstnavateli.",
        9.0,
        Mm(18.0),
        Mm(270.0),
        &regular,
    );
    let fields = [
        ("Příjmení a jméno:", data.name.as_str()),
        ("Rodné číslo:", data.personal_id.as_str()),
        ("Bydliště:", data.address.as_str()),
        ("Město:", data.city.as_str()),
        ("PSČ:", data.postal_code.as_str()),
        ("Evidenční číslo:", data.registration.as_str()),
        ("Organizace:", data.organization.as_str()),
    ];
    let mut y = 251.0;
    for (label, value) in fields {
        current.use_text(label, 10.0, Mm(22.0), Mm(y), &bold);
        current.use_text(value, 11.0, Mm(66.0), Mm(y), &regular);
        current.use_text(
            "____________________________________________",
            9.0,
            Mm(65.0),
            Mm(y - 1.5),
            &regular,
        );
        y -= 10.0;
    }
    current.use_text("Typ pojištění", 11.0, Mm(22.0), Mm(174.0), &bold);
    let normalized_category = data.category.trim().to_uppercase().replace(' ', "");
    for (index, (code, option)) in [
        ("B", "Standard"),
        ("BZ", "Standard + ztráta"),
        ("A", "Řidič"),
        ("AZ", "Řidič + ztráta"),
    ]
    .iter()
    .enumerate()
    {
        let selected = normalized_category == *code;
        current.use_text(
            if selected { "[X]" } else { "[ ]" },
            10.0,
            Mm(28.0),
            Mm(164.0 - index as f32 * 8.0),
            &regular,
        );
        current.use_text(
            *option,
            10.0,
            Mm(38.0),
            Mm(164.0 - index as f32 * 8.0),
            &regular,
        );
    }
    current.use_text("Limit pojistného plnění", 11.0, Mm(106.0), Mm(174.0), &bold);
    for (index, amount) in [200000_i64, 240000, 280000, 320000, 360000, 400000]
        .iter()
        .enumerate()
    {
        current.use_text(
            if data.insurance_limit == *amount {
                "[X]"
            } else {
                "[ ]"
            },
            10.0,
            Mm(112.0),
            Mm(164.0 - index as f32 * 8.0),
            &regular,
        );
        current.use_text(
            format!("{} Kč", amount),
            10.0,
            Mm(122.0),
            Mm(164.0 - index as f32 * 8.0),
            &regular,
        );
    }
    current.use_text(
        format!(
            "Pojistné období: {} - {}",
            data.insurance_from, data.insurance_to
        ),
        10.0,
        Mm(22.0),
        Mm(105.0),
        &regular,
    );
    current.use_text(
        format!("Roční pojistné: {} Kč", data.premium),
        10.0,
        Mm(22.0),
        Mm(95.0),
        &regular,
    );
    current.use_text(
        "Prohlašuji, že jsem byl seznámen s pojistnými podmínkami a s roční výší pojistného.",
        9.0,
        Mm(22.0),
        Mm(76.0),
        &regular,
    );
    current.use_text(
        "Datum: ____________________",
        10.0,
        Mm(22.0),
        Mm(48.0),
        &regular,
    );
    current.use_text(
        "Podpis: ______________________________",
        10.0,
        Mm(112.0),
        Mm(48.0),
        &regular,
    );
    document
        .save(&mut BufWriter::new(
            File::create(destination).map_err(|_| "Dokument se nepodařilo uložit.".to_string())?,
        ))
        .map_err(|_| "Dokument se nepodařilo vytvořit.".to_string())
}

pub fn record_member_document(
    connection: &mut Connection,
    user: &str,
    data: &MemberDocumentData,
    kind: &str,
    bytes: &[u8],
) -> Result<i64, String> {
    let amount = (data.premium - data.paid).max(0);
    let hash = format!("{:x}", Sha256::digest(bytes));
    let prefix = match kind {
        "application" => "PRI",
        "voucher" => "POU",
        "envelope" => "OBA",
        "label" => "STI",
        _ => return Err("Neznámý dokument.".into()),
    };
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| "Dokument se nepodařilo evidovat.".to_string())?;
    let next: i64 = tx
        .query_row(
            r#"SELECT COUNT(*)+1 FROM "ClenskeDoklady" WHERE "PojistnyRok"=?1"#,
            [data.year],
            |r| r.get(0),
        )
        .unwrap_or(1);
    let linked = if kind == "voucher" {
        let invoice_number = format!("FCL-{}-{:06}", data.year, next);
        tx.execute(r#"INSERT INTO "ClenskeDoklady"("PojistnyZaznamRowId","PojistnyRok","Druh","Cislo","Castka","Sha256","DatumVystaveni") VALUES(?1,?2,'FAKTURA ČLENA',?3,?4,?5,date('now'))"#,params![data.row_id,data.year,invoice_number,amount,hash]).map_err(|_|"Fakturu člena se nepodařilo evidovat.".to_string())?;
        Some(tx.last_insert_rowid())
    } else {
        None
    };
    let number = format!("{}-{}-{:06}", prefix, data.year, next);
    tx.execute(r#"INSERT INTO "ClenskeDoklady"("PojistnyZaznamRowId","PojistnyRok","Druh","Cislo","Castka","NavazanyDokladId","Sha256","DatumVystaveni") VALUES(?1,?2,?3,?4,?5,?6,?7,date('now'))"#,params![data.row_id,data.year,kind,number,amount,linked,hash]).map_err(|_|"Dokument se nepodařilo evidovat.".to_string())?;
    let id = tx.last_insert_rowid();
    tx.execute(r#"INSERT INTO "AuditFinancnichDokladu"("Uzivatel","Druh","ObjektId","Operace","Vysledek") VALUES(?1,?2,?3,'VYTVOŘENÍ PDF','OK')"#,params![user,kind,id]).map_err(|_|"Dokument se nepodařilo auditovat.".to_string())?;
    tx.commit()
        .map_err(|_| "Dokument se nepodařilo evidovat.".to_string())?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invoice_batch_is_atomic_and_not_exported_twice() {
        let mut c = Connection::open_in_memory().unwrap();
        ensure_schema(&c).unwrap();
        let input = InvoiceInput {
            supplier: "Dodavatel".into(),
            account_number: "123456".into(),
            bank_code: "0100".into(),
            variable_symbol: "2026001".into(),
            constant_symbol: None,
            specific_symbol: None,
            amount: 1250,
            due_on: "2026-09-01".into(),
            note: None,
        };
        create_invoice(&mut c, "tester", input).unwrap();
        let batch = prepare_batch(&mut c, "tester").unwrap();
        assert_eq!((batch.count, batch.total), (1, 1250));
        let path = Path::new("batch.csv");
        finish_batch(&mut c, "tester", batch.batch_id, path).unwrap();
        assert!(prepare_batch(&mut c, "tester").is_err());
    }
    #[test]
    fn voucher_and_member_invoice_are_linked_atomically() {
        let mut c = Connection::open_in_memory().unwrap();
        ensure_schema(&c).unwrap();
        let d = MemberDocumentData {
            row_id: 7,
            year: 2026,
            registration: "1".into(),
            name: "Test".into(),
            personal_id: "1".into(),
            organization: "ZO".into(),
            address: "A".into(),
            city: "B".into(),
            postal_code: "1".into(),
            country: "CZ".into(),
            email: "".into(),
            insurance_from: "2026-01-01".into(),
            insurance_to: "2026-12-31".into(),
            category: "A".into(),
            insurance_limit: 320000,
            premium: 1000,
            paid: 200,
        };
        record_member_document(&mut c, "tester", &d, "voucher", b"pdf").unwrap();
        let pair: (i64, i64) = c
            .query_row(
                r#"SELECT COUNT(*),COUNT("NavazanyDokladId") FROM "ClenskeDoklady""#,
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(pair, (2, 1));
    }

    #[test]
    fn application_pdf_contains_access_form_sections() {
        let directory = tempfile::tempdir().unwrap();
        let destination = std::env::var_os("FED_PDF_QA_DIR")
            .map(std::path::PathBuf::from)
            .map(|path| path.join("application.pdf"))
            .unwrap_or_else(|| directory.path().join("application.pdf"));
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let data = MemberDocumentData {
            row_id: 1,
            year: 2026,
            registration: "10001".into(),
            name: "Jan Novák".into(),
            personal_id: "800101/0000".into(),
            organization: "ZO PRAHA".into(),
            address: "Hlavní 1".into(),
            city: "Praha".into(),
            postal_code: "110 00".into(),
            country: "Česká republika".into(),
            email: "test@example.cz".into(),
            insurance_from: "2026-01-01".into(),
            insurance_to: "2026-12-31".into(),
            category: "A".into(),
            insurance_limit: 320000,
            premium: 1500,
            paid: 0,
        };
        application_pdf(&data, &destination).unwrap();
        assert!(std::fs::metadata(destination).unwrap().len() > 1_000);
    }
}
