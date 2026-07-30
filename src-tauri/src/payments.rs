use chrono::{Duration, Local, NaiveDate};
use deunicode::deunicode;
use printpdf::{Mm, PdfDocument};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufWriter, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSettings {
    pub recipient_name: String,
    pub account_number: String,
    pub bank_code: String,
    pub iban: String,
    pub bic: String,
    pub constant_symbol: String,
    pub default_due_days: i64,
    pub message_template: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentOrderDraft {
    pub row_id: i64,
    pub payer_name: String,
    pub address: String,
    pub city: String,
    pub postal_code: String,
    pub registration_number: String,
    pub insurance_year: i32,
    pub insured_amount: i64,
    pub annual_premium: i64,
    pub actual_payment: i64,
    pub amount_due: i64,
    pub organization: String,
    pub variable_symbol: String,
    pub recipient_name: String,
    pub account: String,
    pub iban: String,
    pub bic: String,
    pub constant_symbol: String,
    pub issue_date: String,
    pub due_date: String,
    pub message: String,
    pub settings_complete: bool,
    pub validation_errors: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentOrderPdfInput {
    pub row_id: i64,
}

pub fn ensure_schema(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        r#"CREATE TABLE IF NOT EXISTS "PlatebniNastaveni" (
            "Id" INTEGER PRIMARY KEY CHECK ("Id" = 1),
            "NazevPrijemce" TEXT NOT NULL DEFAULT '',
            "CisloUctu" TEXT NOT NULL DEFAULT '',
            "KodBanky" TEXT NOT NULL DEFAULT '',
            "IBAN" TEXT NOT NULL DEFAULT '',
            "BIC" TEXT NOT NULL DEFAULT '',
            "KonstantniSymbol" TEXT NOT NULL DEFAULT '3558',
            "VychoziSplatnostDni" INTEGER NOT NULL DEFAULT 15,
            "ZpravaProPrijemce" TEXT NOT NULL DEFAULT 'Pojistné {rok} – Ev. č. {evidencni_cislo}',
            "Aktualizovano" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        INSERT OR IGNORE INTO "PlatebniNastaveni" ("Id") VALUES (1);"#,
    )
}

pub fn load_settings(connection: &Connection) -> rusqlite::Result<PaymentSettings> {
    connection.query_row(
        r#"SELECT "NazevPrijemce", "CisloUctu", "KodBanky", "IBAN", "BIC",
                  "KonstantniSymbol", "VychoziSplatnostDni", "ZpravaProPrijemce"
           FROM "PlatebniNastaveni" WHERE "Id" = 1"#,
        [],
        |row| {
            Ok(PaymentSettings {
                recipient_name: row.get(0)?,
                account_number: row.get(1)?,
                bank_code: row.get(2)?,
                iban: row.get(3)?,
                bic: row.get(4)?,
                constant_symbol: row.get(5)?,
                default_due_days: row.get(6)?,
                message_template: row.get(7)?,
            })
        },
    )
}

pub fn save_settings(connection: &Connection, settings: &PaymentSettings) -> Result<(), String> {
    let recipient = settings.recipient_name.trim();
    let account = settings.account_number.trim();
    let bank = settings.bank_code.trim();
    let iban = settings.iban.trim();
    if settings.default_due_days < 0 || settings.default_due_days > 365 {
        return Err("Výchozí splatnost musí být v rozmezí 0 až 365 dní.".into());
    }
    if !settings
        .constant_symbol
        .chars()
        .all(|character| character.is_ascii_digit())
    {
        return Err("Konstantní symbol může obsahovat pouze číslice.".into());
    }
    if (!account.is_empty() || !bank.is_empty()) && (account.is_empty() || bank.is_empty()) {
        return Err("Vyplňte číslo účtu i kód banky.".into());
    }
    connection
        .execute(
            r#"UPDATE "PlatebniNastaveni" SET
               "NazevPrijemce" = ?1, "CisloUctu" = ?2, "KodBanky" = ?3,
               "IBAN" = ?4, "BIC" = ?5, "KonstantniSymbol" = ?6,
               "VychoziSplatnostDni" = ?7, "ZpravaProPrijemce" = ?8,
               "Aktualizovano" = CURRENT_TIMESTAMP WHERE "Id" = 1"#,
            params![
                recipient,
                account,
                bank,
                iban,
                settings.bic.trim(),
                settings.constant_symbol.trim(),
                settings.default_due_days,
                settings.message_template.trim()
            ],
        )
        .map_err(|_| "Platební údaje se nepodařilo uložit.".to_string())?;
    Ok(())
}

pub fn variable_symbol(personal_id: &str) -> Result<String, String> {
    let symbol: String = personal_id
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect();
    if symbol.len() < 9 || symbol.len() > 10 {
        return Err("Variabilní symbol se nepodařilo vytvořit.".into());
    }
    Ok(symbol)
}

pub fn due_date(days: i64) -> NaiveDate {
    Local::now().date_naive() + Duration::days(days)
}

pub fn settings_complete(settings: &PaymentSettings) -> bool {
    !settings.recipient_name.trim().is_empty()
        && ((!settings.account_number.trim().is_empty() && !settings.bank_code.trim().is_empty())
            || !settings.iban.trim().is_empty())
}

pub fn render_message(template: &str, year: i32, registration_number: &str) -> String {
    template
        .replace("{rok}", &year.to_string())
        .replace("{evidencni_cislo}", registration_number)
}

pub fn record_audit(
    connection: &Connection,
    user: &str,
    member_identifier: &str,
    insurance_year: i32,
    output: &str,
) -> Result<(), String> {
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
        .map_err(|_| "Vytvoření příkazu se nepodařilo zaznamenat.".to_string())?;
    connection
        .execute(
            r#"INSERT INTO "AuditLog"
           ("DatumČas", "Uživatel", "Operace", "IdentifikátorPojištěnce", "Výsledek")
           VALUES (CURRENT_TIMESTAMP, ?1, ?2, ?3, 'OK')"#,
            params![
                user,
                format!("PAYMENT_ORDER_{output}:{insurance_year}"),
                member_identifier
            ],
        )
        .map_err(|_| "Vytvoření příkazu se nepodařilo zaznamenat.".to_string())?;
    Ok(())
}

pub fn safe_filename(year: i32, registration: &str, last_name: &str, first_name: &str) -> String {
    let raw = format!(
        "Prikaz_k_uhrade_{year}_{registration}_{}_{}.pdf",
        deunicode(last_name),
        deunicode(first_name)
    );
    raw.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

pub fn create_pdf(
    path: &Path,
    draft: &PaymentOrderDraft,
    amount: i64,
    due_date: &str,
    message: &str,
) -> Result<(), String> {
    let (document, page, layer) =
        PdfDocument::new("Příkaz k úhradě", Mm(210.0), Mm(297.0), "Příkaz");
    let layer = document.get_page(page).get_layer(layer);
    let font_file = File::open(r"C:\Windows\Fonts\arial.ttf")
        .map_err(|_| "PDF se nepodařilo vytvořit s českým písmem.".to_string())?;
    let font = document
        .add_external_font(font_file)
        .map_err(|_| "PDF se nepodařilo vytvořit.".to_string())?;
    let bold_file = File::open(r"C:\Windows\Fonts\arialbd.ttf")
        .map_err(|_| "PDF se nepodařilo vytvořit s českým písmem.".to_string())?;
    let bold = document
        .add_external_font(bold_file)
        .map_err(|_| "PDF se nepodařilo vytvořit.".to_string())?;
    layer.use_text("FEDERACE VLAKOVÝCH ČET", 11.0, Mm(20.0), Mm(278.0), &bold);
    layer.use_text("PŘÍKAZ K ÚHRADĚ", 22.0, Mm(20.0), Mm(260.0), &bold);
    layer.use_text("Plátce", 12.0, Mm(20.0), Mm(240.0), &bold);
    layer.use_text(&draft.payer_name, 11.0, Mm(20.0), Mm(232.0), &font);
    layer.use_text(&draft.address, 11.0, Mm(20.0), Mm(225.0), &font);
    layer.use_text(
        format!("{} {}", draft.postal_code, draft.city),
        11.0,
        Mm(20.0),
        Mm(218.0),
        &font,
    );
    layer.use_text("Příjemce", 12.0, Mm(110.0), Mm(240.0), &bold);
    layer.use_text(&draft.recipient_name, 11.0, Mm(110.0), Mm(232.0), &font);
    layer.use_text(
        format!("Číslo účtu: {}", draft.account),
        11.0,
        Mm(110.0),
        Mm(225.0),
        &font,
    );
    if !draft.iban.is_empty() {
        layer.use_text(
            format!("IBAN: {}", draft.iban),
            10.0,
            Mm(110.0),
            Mm(218.0),
            &font,
        );
    }
    let rows = [
        ("Částka", format!("{amount} Kč")),
        ("Variabilní symbol", draft.variable_symbol.clone()),
        ("Pojistný rok", draft.insurance_year.to_string()),
        ("Konstantní symbol", draft.constant_symbol.clone()),
        ("Splatnost", due_date.to_string()),
        ("Datum vytvoření", draft.issue_date.clone()),
        ("Zpráva pro příjemce", message.to_string()),
    ];
    for (index, (label, value)) in rows.iter().enumerate() {
        let y = 190.0 - index as f32 * 13.0;
        layer.use_text(*label, 11.0, Mm(20.0), Mm(y), &bold);
        layer.use_text(value, 11.0, Mm(75.0), Mm(y), &font);
    }
    layer.use_text("Prostor pro QR platbu", 9.0, Mm(142.0), Mm(72.0), &font);
    document
        .save(&mut BufWriter::new(
            File::create(path).map_err(|_| "PDF se nepodařilo uložit.".to_string())?,
        ))
        .map_err(|_| "PDF se nepodařilo uložit.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_symbol_uses_personal_id_without_slash() {
        assert_eq!(variable_symbol("000000/0000").unwrap(), "0000000000");
        assert!(variable_symbol("neuvedeno").is_err());
    }

    #[test]
    fn message_uses_supported_placeholders() {
        assert_eq!(
            render_message("Pojistné {rok} – Ev. č. {evidencni_cislo}", 2026, "107"),
            "Pojistné 2026 – Ev. č. 107"
        );
    }

    #[test]
    fn pdf_is_created_without_personal_identifier() {
        let path = std::env::temp_dir().join("pojisteni-payment-order-test.pdf");
        let draft = PaymentOrderDraft {
            row_id: 1,
            payer_name: "Jan Novák".into(),
            address: "Ulice 1".into(),
            city: "Praha".into(),
            postal_code: "110 00".into(),
            registration_number: "107".into(),
            insurance_year: 2026,
            insured_amount: 200_000,
            annual_premium: 594,
            actual_payment: 0,
            amount_due: 594,
            organization: "FVČ".into(),
            variable_symbol: "7803265536".into(),
            recipient_name: "Federace".into(),
            account: "123/0100".into(),
            iban: String::new(),
            bic: String::new(),
            constant_symbol: "3558".into(),
            issue_date: "2026-07-26".into(),
            due_date: "2026-08-10".into(),
            message: "Pojistné 2026".into(),
            settings_complete: true,
            validation_errors: Vec::new(),
        };
        create_pdf(&path, &draft, 594, "2026-08-10", "Pojistné 2026").unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
        assert!(bytes.len() > 500);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn payment_order_audit_contains_member_year_and_output_only() {
        let connection = Connection::open_in_memory().unwrap();
        record_audit(&connection, "test-user", "42", 2026, "PDF").unwrap();
        let audit: (String, String, String, String) = connection.query_row(
            r#"SELECT "Uživatel", "Operace", "IdentifikátorPojištěnce", "Výsledek" FROM "AuditLog""#,
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).unwrap();
        assert_eq!(
            audit,
            (
                "test-user".into(),
                "PAYMENT_ORDER_PDF:2026".into(),
                "42".into(),
                "OK".into()
            )
        );
    }
}
