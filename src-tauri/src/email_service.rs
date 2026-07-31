use lettre::{
    message::{header::ContentType, Attachment, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

const CREDENTIAL_USER: &str = "smtp-password";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailSettings {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub sender_email: String,
    pub encryption: String,
    pub credential_name: String,
    #[serde(default)]
    pub password_configured: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveEmailSettings {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub sender_email: String,
    pub encryption: String,
    pub credential_name: String,
    pub password: Option<String>,
}

pub struct EmailMessage<'a> {
    pub recipient: &'a str,
    pub subject: &'a str,
    pub body: &'a str,
    pub attachment_name: &'a str,
    pub attachment: Vec<u8>,
}

pub fn ensure_schema(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        r#"CREATE TABLE IF NOT EXISTS "EmailNastaveni" (
            "Id" INTEGER PRIMARY KEY CHECK ("Id" = 1),
            "Server" TEXT NOT NULL DEFAULT '',
            "Port" INTEGER NOT NULL DEFAULT 587,
            "UzivatelskeJmeno" TEXT NOT NULL DEFAULT '',
            "EmailOdesilatele" TEXT NOT NULL DEFAULT '',
            "Sifrovani" TEXT NOT NULL DEFAULT 'STARTTLS',
            "CredentialName" TEXT NOT NULL DEFAULT 'FederacePojisteni-SMTP',
            "Aktualizovano" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        INSERT OR IGNORE INTO "EmailNastaveni" ("Id") VALUES (1);"#,
    )
}

fn credential(name: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(name, CREDENTIAL_USER)
        .map_err(|_| "Přístup k zabezpečenému úložišti hesel se nezdařil.".to_string())
}

pub fn load(connection: &Connection) -> Result<EmailSettings, String> {
    ensure_schema(connection).map_err(|_| "Nastavení e-mailu se nepodařilo načíst.".to_string())?;
    let mut settings = connection
        .query_row(
            r#"SELECT "Server", "Port", "UzivatelskeJmeno", "EmailOdesilatele",
                      "Sifrovani", "CredentialName" FROM "EmailNastaveni" WHERE "Id"=1"#,
            [],
            |row| {
                Ok(EmailSettings {
                    server: row.get(0)?,
                    port: row.get::<_, i64>(1)? as u16,
                    username: row.get(2)?,
                    sender_email: row.get(3)?,
                    encryption: row.get(4)?,
                    credential_name: row.get(5)?,
                    password_configured: false,
                })
            },
        )
        .map_err(|_| "Nastavení e-mailu se nepodařilo načíst.".to_string())?;
    settings.password_configured = credential(&settings.credential_name)
        .and_then(|entry| entry.get_password().map_err(|_| String::new()))
        .is_ok();
    Ok(settings)
}

pub fn save(connection: &Connection, input: SaveEmailSettings) -> Result<(), String> {
    let server = input.server.trim();
    let username = input.username.trim();
    let sender = input.sender_email.trim();
    let credential_name = input.credential_name.trim();
    if server.is_empty() || input.port == 0 || username.is_empty() || sender.is_empty() {
        return Err("Vyplňte server, port, uživatelské jméno a e-mail odesílatele.".into());
    }
    if !matches!(
        input.encryption.as_str(),
        "STARTTLS" | "TLS" | "Bez šifrování"
    ) {
        return Err("Vyberte podporovaný typ šifrování.".into());
    }
    if credential_name.is_empty() {
        return Err("Vyplňte název zabezpečeného záznamu hesla.".into());
    }
    ensure_schema(connection).map_err(|_| "Nastavení e-mailu se nepodařilo uložit.".to_string())?;
    connection
        .execute(
            r#"UPDATE "EmailNastaveni" SET "Server"=?1,"Port"=?2,"UzivatelskeJmeno"=?3,
               "EmailOdesilatele"=?4,"Sifrovani"=?5,"CredentialName"=?6,
               "Aktualizovano"=CURRENT_TIMESTAMP WHERE "Id"=1"#,
            params![
                server,
                input.port,
                username,
                sender,
                input.encryption,
                credential_name
            ],
        )
        .map_err(|_| "Nastavení e-mailu se nepodařilo uložit.".to_string())?;
    if let Some(password) = input.password.filter(|value| !value.is_empty()) {
        credential(credential_name)?
            .set_password(&password)
            .map_err(|_| {
                "Heslo se nepodařilo uložit do Windows Credential Manageru.".to_string()
            })?;
    }
    Ok(())
}

fn transport(settings: &EmailSettings, password: String) -> Result<SmtpTransport, String> {
    let credentials = Credentials::new(settings.username.clone(), password);
    let builder = match settings.encryption.as_str() {
        "TLS" => SmtpTransport::relay(&settings.server),
        "STARTTLS" => SmtpTransport::starttls_relay(&settings.server),
        "Bez šifrování" => Ok(SmtpTransport::builder_dangerous(&settings.server)),
        _ => return Err("Nastavení šifrování SMTP není podporováno.".into()),
    }
    .map_err(|_| "Spojení s e-mailovým serverem se nepodařilo připravit.".to_string())?;
    Ok(builder.port(settings.port).credentials(credentials).build())
}

pub fn send(connection: &Connection, request: EmailMessage<'_>) -> Result<(), String> {
    let settings = load(connection)?;
    let password = credential(&settings.credential_name)?
        .get_password()
        .map_err(|_| "SMTP heslo není uložené ve Windows Credential Manageru.".to_string())?;
    let from: Mailbox = settings
        .sender_email
        .parse()
        .map_err(|_| "E-mail odesílatele není platný.".to_string())?;
    let to: Mailbox = request
        .recipient
        .parse()
        .map_err(|_| "E-mail příjemce není platný.".to_string())?;
    let attachment = Attachment::new(request.attachment_name.to_string()).body(
        request.attachment,
        ContentType::parse("application/pdf")
            .map_err(|_| "Přílohu se nepodařilo připravit.".to_string())?,
    );
    let message = Message::builder()
        .from(from)
        .to(to)
        .subject(request.subject)
        .multipart(
            MultiPart::mixed()
                .singlepart(SinglePart::plain(request.body.to_string()))
                .singlepart(attachment),
        )
        .map_err(|_| "E-mail se nepodařilo připravit.".to_string())?;
    transport(&settings, password)?
        .send(&message)
        .map_err(|_| "E-mail se nepodařilo odeslat. Doklad zůstává bezpečně uložen.".to_string())?;
    Ok(())
}
