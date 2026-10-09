mod claims;
mod applications;
mod current_insurance_year;
mod database_backup;
mod email_service;
mod financial_documents;
mod member_import;
mod member_payments;
mod migrations;
mod organization_payments;
mod payments;
mod receipts;
mod reports;
mod tariffs;

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime};
use current_insurance_year::CurrentInsuranceYear;
use rand_core::OsRng;
use rusqlite::types::ValueRef;
use rusqlite::{functions::FunctionFlags, params, Connection, OpenFlags, Row, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};
use tauri::{AppHandle, Manager, State};

const DATABASE_FILE: &str = "dd.sqlite";
const FRIENDLY_DATABASE_ERROR: &str =
    "Operaci se nepodařilo dokončit. Data nebyla změněna. Zkuste to prosím znovu.";

#[derive(Default)]
struct Session {
    authenticated: bool,
    user: String,
    role: String,
}

struct AppState {
    session: Mutex<Session>,
    database_maintenance: Mutex<()>,
    startup: Mutex<StartupStatus>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            session: Mutex::new(Session::default()),
            database_maintenance: Mutex::new(()),
            startup: Mutex::new(StartupStatus {
                ready: false,
                application_version: env!("CARGO_PKG_VERSION").into(),
                database_schema_version: None,
                supported_schema_version: migrations::DB_SCHEMA_VERSION,
                database_path: None,
                smoke_mode: cfg!(feature = "smoke-test"),
                failure: None,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StartupStatus {
    ready: bool,
    application_version: String,
    database_schema_version: Option<i64>,
    supported_schema_version: i64,
    database_path: Option<String>,
    smoke_mode: bool,
    failure: Option<migrations::MigrationFailure>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginResult {
    user: String,
    role: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthStatus {
    initialized: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FormOptions {
    organizations: Vec<String>,
    last_registration_number: i64,
    last_client: String,
    annual_amounts: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct RegistrationNumberOptions {
    next_number: i64,
    free_numbers: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewInsured {
    title: Option<String>,
    last_name: Option<String>,
    first_name: Option<String>,
    personal_id: Option<String>,
    organization: Option<String>,
    affiliation: String,
    city: Option<String>,
    address: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    note: Option<String>,
    insurance_from: Option<String>,
    insurance_to: Option<String>,
    annual_amount: i64,
    category: String,
    loss: bool,
    actual_payment: Option<i64>,
    code: i64,
    registration_year: i32,
    email: Option<String>,
    phone: Option<String>,
    registration_number: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemberUpdate {
    row_id: i64,
    title: Option<String>,
    last_name: Option<String>,
    first_name: Option<String>,
    personal_id: Option<String>,
    registration_number: Option<i64>,
    city: Option<String>,
    address: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    organization: Option<String>,
    affiliation: String,
    code: String,
    email: Option<String>,
    phone: Option<String>,
    note: Option<String>,
    actual_payment: Option<i64>,
    actual_termination: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveResult {
    identifier: i64,
    registration_number: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MemberRow {
    row_id: i64,
    identifier: Option<String>,
    code: Option<String>,
    registration_number: Option<String>,
    insured: String,
    personal_id: Option<String>,
    affiliation: Option<String>,
    insurance_from: Option<String>,
    insurance_to: Option<String>,
    actual_termination: Option<String>,
    category: Option<String>,
    loss: Option<String>,
    annual_premium: Option<String>,
    premium: Option<String>,
    actual_payment: Option<String>,
    note: Option<String>,
    title: Option<String>,
    last_name: Option<String>,
    first_name: Option<String>,
    city: Option<String>,
    address: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    organization: Option<String>,
    email: Option<String>,
    phone: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MemberPage {
    members: Vec<MemberRow>,
    total: i64,
    page: u32,
    page_size: u32,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemberFilters {
    affiliation: Option<String>,
    category: Option<String>,
    loss: Option<String>,
    status: Option<String>,
    premium: Option<String>,
    payment: Option<String>,
    payment_status: Option<String>,
    overdue: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ArchiveYear {
    year: i32,
    record_count: i64,
    unique_member_count: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LegacyTablePage {
    source: String,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    total: i64,
    page: u32,
    page_size: u32,
}

fn legacy_cell(value: ValueRef<'_>) -> String {
    match value {
        ValueRef::Null => String::new(),
        ValueRef::Integer(value) => value.to_string(),
        ValueRef::Real(value) => value.to_string(),
        ValueRef::Text(value) => String::from_utf8_lossy(value).into_owned(),
        ValueRef::Blob(_) => "[binární data]".to_string(),
    }
}

fn legacy_table_page(
    connection: &Connection,
    source: &str,
    search: Option<String>,
    page: u32,
    page_size: u32,
) -> Result<LegacyTablePage, String> {
    const ALLOWED: [&str; 10] = [
        "2002", "2003", "2004", "2005", "2006", "2007", "2008", "2009", "2010", "Faktura",
    ];
    if !ALLOWED.contains(&source) {
        return Err("Nepovolený historický zdroj.".to_string());
    }
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
            [source],
            |row| row.get(0),
        )
        .map_err(|_| "Historický zdroj se nepodařilo ověřit.".to_string())?;
    if !exists {
        return Err(format!("Historický zdroj {source} v databázi chybí."));
    }

    let quoted = format!("\"{}\"", source.replace('"', "\"\""));
    let probe = connection
        .prepare(&format!("SELECT * FROM {quoted} LIMIT 0"))
        .map_err(|_| "Historický zdroj se nepodařilo načíst.".to_string())?;
    let columns = probe
        .column_names()
        .iter()
        .map(|name| (*name).to_string())
        .collect::<Vec<_>>();
    let normalized_search = search.unwrap_or_default().trim().to_string();
    let where_clause = if normalized_search.is_empty() {
        String::new()
    } else {
        let searchable = columns
            .iter()
            .map(|column| format!("CAST(\"{}\" AS TEXT) LIKE ?1", column.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" OR ");
        format!(" WHERE {searchable}")
    };
    let total: i64 = if normalized_search.is_empty() {
        connection.query_row(&format!("SELECT COUNT(*) FROM {quoted}"), [], |row| {
            row.get(0)
        })
    } else {
        connection.query_row(
            &format!("SELECT COUNT(*) FROM {quoted}{where_clause}"),
            [format!("%{normalized_search}%")],
            |row| row.get(0),
        )
    }
    .map_err(|_| "Historické záznamy se nepodařilo spočítat.".to_string())?;
    let page_size = page_size.clamp(1, 200);
    let page = page.max(1);
    let offset = (i64::from(page) - 1) * i64::from(page_size);
    let sql = format!("SELECT * FROM {quoted}{where_clause} ORDER BY rowid LIMIT ? OFFSET ?");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|_| "Historické záznamy se nepodařilo načíst.".to_string())?;
    let column_count = columns.len();
    let map = |row: &Row<'_>| {
        (0..column_count)
            .map(|index| row.get_ref(index).map(legacy_cell))
            .collect::<Result<Vec<_>, _>>()
    };
    let rows = if normalized_search.is_empty() {
        statement.query_map(params![page_size, offset], map)
    } else {
        statement.query_map(
            params![format!("%{normalized_search}%"), page_size, offset],
            map,
        )
    }
    .map_err(|_| "Historické záznamy se nepodařilo načíst.".to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|_| "Historické záznamy se nepodařilo načíst.".to_string())?;

    Ok(LegacyTablePage {
        source: source.to_string(),
        columns,
        rows,
        total,
        page,
        page_size,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DashboardInfo {
    member_count: i64,
    last_registration_number: i64,
    database_date: String,
    program_version: &'static str,
    database_schema_version: i64,
    active_insurance_year: i32,
    commit_sha: &'static str,
    build_date: &'static str,
    git_tag: &'static str,
    overdue_count: i64,
    overdue_amount: i64,
    oldest_due_date: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InsurerOverview {
    insurance_year: i32,
    members: Vec<MemberRow>,
    total_premium: i64,
    total_paid: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchCertificateFilter {
    paid_from: Option<String>,
    organization_code: Option<String>,
    organization: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchCertificateResult {
    selected: usize,
    created: usize,
    existing: usize,
    skipped: usize,
    errors: Vec<String>,
    receipt_ids: Vec<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemDiagnostics {
    database_path: String,
    integrity: String,
    application_version: String,
    database_schema_version: i64,
    active_year: i32,
    members: i64,
    claims: i64,
    receipts: i64,
    invoices: i64,
    backups: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuditEntry {
    occurred_at: String,
    user: String,
    operation: String,
    result: String,
}

const MEMBER_SELECT: &str = r#"SELECT
    rowid,
    CAST("KódOC" AS TEXT),
    CAST("EvČíslo" AS TEXT),
    TRIM(COALESCE("Titul", '') || ' ' || COALESCE("Příjmení", '') || ' ' || COALESCE("Jméno", '')),
    CAST("RodnéČíslo" AS TEXT),
    CAST("OdbPříslušnost" AS TEXT),
    CAST("PojištěníOd" AS TEXT),
    CAST("PojištěníDo" AS TEXT),
    CAST("Ukončení" AS TEXT),
    CAST("Kategorie" AS TEXT),
    CAST("Ztráta" AS TEXT),
    CAST("RočPojistné" AS TEXT),
    CAST("PojistnáČástka" AS TEXT),
    CAST("SkutÚhrada" AS TEXT),
    CAST("Poznámka" AS TEXT),
    CAST("Identifikátor" AS TEXT),
    CAST("Titul" AS TEXT),
    CAST("Příjmení" AS TEXT),
    CAST("Jméno" AS TEXT),
    CAST("Město" AS TEXT),
    CAST("Adresa" AS TEXT),
    CAST("PSČ" AS TEXT),
    CAST("Stát" AS TEXT),
    CAST("ZO" AS TEXT),
    CAST("e-mail" AS TEXT),
    CAST("Telefon" AS TEXT)
FROM "Seznam""#;

#[cfg(test)]
const MEMBER_SEARCH: &str = r#"(?1 = '' OR
    COALESCE(CAST("KódOC" AS TEXT), '') LIKE ?2 ESCAPE '\' COLLATE NOCASE OR
    COALESCE(CAST("EvČíslo" AS TEXT), '') LIKE ?2 ESCAPE '\' COLLATE NOCASE OR
    COALESCE("Titul", '') LIKE ?2 ESCAPE '\' COLLATE NOCASE OR
    COALESCE("Příjmení", '') LIKE ?2 ESCAPE '\' COLLATE NOCASE OR
    COALESCE("Jméno", '') LIKE ?2 ESCAPE '\' COLLATE NOCASE OR
    COALESCE("RodnéČíslo", '') LIKE ?2 ESCAPE '\' COLLATE NOCASE OR
    COALESCE("OdbPříslušnost", '') LIKE ?2 ESCAPE '\' COLLATE NOCASE OR
    COALESCE(CAST("PojištěníOd" AS TEXT), '') LIKE ?2 ESCAPE '\' COLLATE NOCASE OR
    COALESCE(CAST("Ukončení" AS TEXT), '') LIKE ?2 ESCAPE '\' COLLATE NOCASE OR
    COALESCE("Kategorie", '') LIKE ?2 ESCAPE '\' COLLATE NOCASE OR
    COALESCE(CAST("Ztráta" AS TEXT), '') LIKE ?2 ESCAPE '\' COLLATE NOCASE OR
    COALESCE(CAST("RočPojistné" AS TEXT), '') LIKE ?2 ESCAPE '\' COLLATE NOCASE OR
    COALESCE(CAST("PojistnáČástka" AS TEXT), '') LIKE ?2 ESCAPE '\' COLLATE NOCASE OR
    COALESCE(CAST("SkutÚhrada" AS TEXT), '') LIKE ?2 ESCAPE '\' COLLATE NOCASE OR
    COALESCE("Poznámka", '') LIKE ?2 ESCAPE '\' COLLATE NOCASE
)"#;

const ARCHIVE_SEARCH: &str = r#"(?2 = '' OR
    COALESCE(CAST("KódOC" AS TEXT), '') LIKE ?3 ESCAPE '\' COLLATE NOCASE OR
    COALESCE(CAST("EvČíslo" AS TEXT), '') LIKE ?3 ESCAPE '\' COLLATE NOCASE OR
    COALESCE("Příjmení", '') LIKE ?3 ESCAPE '\' COLLATE NOCASE OR
    COALESCE("Jméno", '') LIKE ?3 ESCAPE '\' COLLATE NOCASE OR
    COALESCE("RodnéČíslo", '') LIKE ?3 ESCAPE '\' COLLATE NOCASE OR
    COALESCE("OdbPříslušnost", '') LIKE ?3 ESCAPE '\' COLLATE NOCASE OR
    COALESCE("ZO", '') LIKE ?3 ESCAPE '\' COLLATE NOCASE
)"#;

const MEMBER_FILTERS: &str = r#"AND (?4 = '' OR COALESCE(CAST("KódOC" AS TEXT), '') = ?4)
AND (?5 = '' OR COALESCE("Kategorie", '') = ?5)
AND (?6 = '' OR COALESCE(CAST("Ztráta" AS TEXT), '') = ?6)
AND (?7 = '' OR
    (?7 = 'aktivni' AND NULLIF(TRIM("Ukončení"), '') IS NULL) OR
    (?7 = 'ukonceny' AND NULLIF(TRIM("Ukončení"), '') IS NOT NULL))
AND (?8 = '' OR COALESCE(CAST("PojistnáČástka" AS TEXT), '') = ?8)
AND (?9 = '' OR COALESCE(CAST("SkutÚhrada" AS TEXT), '') = ?9)
AND (?10 = '' OR
    (?10 = 'uhrazeno' AND COALESCE("SkutÚhrada", 0) = COALESCE("PojistnáČástka", 0)) OR
    (?10 = 'neuhrazeno' AND COALESCE("SkutÚhrada", 0) <> COALESCE("PojistnáČástka", 0)))
AND (?11 = '' OR (?11 = 'po_splatnosti' AND EXISTS (
    SELECT 1 FROM "PrikazyKUhrade" prikaz
    WHERE prikaz."PojistnyZaznamRowId" = "Seznam".rowid
      AND prikaz."PojistnyRok" = pojisteni_rok("Seznam"."PojištěníOd")
      AND NULLIF(TRIM(prikaz."DatumSplatnosti"), '') IS NOT NULL
      AND date(prikaz."DatumSplatnosti") < date('now', 'localtime')
      AND COALESCE("Seznam"."SkutÚhrada", 0) < COALESCE("Seznam"."PojistnáČástka", 0)
)))"#;

#[cfg(not(feature = "smoke-test"))]
fn source_database_path(app: &AppHandle) -> Result<PathBuf, String> {
    let mut candidates = Vec::new();
    if let Ok(directory) = app.path().resource_dir() {
        candidates.push(directory.join(DATABASE_FILE));
        candidates.push(directory.join("_up_").join(DATABASE_FILE));
    }
    if let Ok(directory) = std::env::current_dir() {
        candidates.push(directory.join(DATABASE_FILE));
    }
    candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| "Databázi se nepodařilo načíst.".into())
}

fn resolve_database_path(
    smoke_mode: bool,
    override_path: Option<std::ffi::OsString>,
    standard_path: &Path,
    forbidden_roots: &[PathBuf],
) -> Result<PathBuf, String> {
    if !smoke_mode {
        return Ok(standard_path.to_path_buf());
    }

    let value = override_path.ok_or_else(|| {
        "Smoke režim vyžaduje FEDERACE_DB_PATH; pracovní databáze nebude otevřena.".to_string()
    })?;
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err("FEDERACE_DB_PATH musí být absolutní cesta.".into());
    }
    if !path.is_file() {
        return Err("FEDERACE_DB_PATH musí odkazovat na existující kopii databáze.".into());
    }

    let resolved = fs::canonicalize(&path)
        .map_err(|_| "FEDERACE_DB_PATH se nepodařilo bezpečně ověřit.".to_string())?;
    let standard = fs::canonicalize(standard_path).unwrap_or_else(|_| standard_path.to_path_buf());
    if resolved == standard {
        return Err("Smoke režim odmítl standardní pracovní databázi.".into());
    }
    for root in forbidden_roots {
        let root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
        if resolved.starts_with(&root) {
            return Err("Smoke databáze nesmí být umístěna v uživatelském AppData.".into());
        }
    }
    Ok(resolved)
}

fn working_database_path_with_state(app: &AppHandle) -> Result<(PathBuf, bool), String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|_| "Databázi se nepodařilo načíst.".to_string())?;

    #[cfg(feature = "smoke-test")]
    {
        let mut forbidden_roots = Vec::new();
        // app_data_dir comes from the Windows Known Folder API. Its parent is
        // therefore authoritative even when APPDATA was overridden in the
        // launching process (the incident this guard prevents).
        if let Some(path) = directory.parent() {
            forbidden_roots.push(path.to_path_buf());
        }
        if let Some(path) = std::env::var_os("APPDATA") {
            forbidden_roots.push(PathBuf::from(path));
        }
        if let Some(path) = std::env::var_os("LOCALAPPDATA") {
            forbidden_roots.push(PathBuf::from(path));
        }
        let destination = resolve_database_path(
            true,
            std::env::var_os("FEDERACE_DB_PATH"),
            &directory.join(DATABASE_FILE),
            &forbidden_roots,
        )?;
        println!("SMOKE DATABASE (absolute): {}", destination.display());
        return Ok((destination, false));
    }

    #[cfg(not(feature = "smoke-test"))]
    let destination = resolve_database_path(false, None, &directory.join(DATABASE_FILE), &[])?;

    #[cfg(not(feature = "smoke-test"))]
    {
    fs::create_dir_all(&directory).map_err(|_| "Databázi se nepodařilo načíst.".to_string())?;
    let newly_created = !destination.exists();
    if newly_created {
        fs::copy(source_database_path(app)?, &destination)
            .map_err(|_| "Databázi se nepodařilo načíst.".to_string())?;
    }
    Ok((destination, newly_created))
    }
}

fn working_database_path(app: &AppHandle) -> Result<PathBuf, String> {
    working_database_path_with_state(app).map(|(path, _)| path)
}

fn open_read_only(path: &Path) -> Result<Connection, String> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| "Databázi se nepodařilo načíst.".to_string())?;
    register_insurance_year(&connection)
        .map_err(|_| "Databázi se nepodařilo načíst.".to_string())?;
    connection
        .pragma_update(None, "query_only", true)
        .map_err(|_| "Databázi se nepodařilo načíst.".to_string())?;
    Ok(connection)
}

fn insurance_year(value: &str) -> Option<i32> {
    let value = value.trim();
    [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%d.%m.%Y %H:%M:%S",
    ]
    .iter()
    .find_map(|format| NaiveDateTime::parse_from_str(value, format).ok())
    .map(|date| date.year())
    .or_else(|| {
        ["%Y-%m-%d", "%d.%m.%Y", "%d. %m. %Y", "%d/%m/%Y"]
            .iter()
            .find_map(|format| NaiveDate::parse_from_str(value, format).ok())
            .map(|date| date.year())
    })
}

fn register_insurance_year(connection: &Connection) -> rusqlite::Result<()> {
    connection.create_scalar_function(
        "pojisteni_rok",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |context| {
            let value = context.get_raw(0);
            match value.as_str_or_null()? {
                Some(value) => Ok(insurance_year(value)),
                None => Ok(None),
            }
        },
    )
}

fn open_write(path: &Path) -> Result<Connection, String> {
    let connection = Connection::open(path).map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
    register_insurance_year(&connection).map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
    connection
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
    Ok(connection)
}

fn ensure_current_insurance_year(path: &Path) -> Result<i32, String> {
    let mut connection = open_write(path)?;
    CurrentInsuranceYear::initialize(&mut connection, path, Local::now().year())
}

#[cfg(test)]
fn ensure_member_contact_schema(connection: &Connection) -> rusqlite::Result<()> {
    for table in ["Seznam", "Editace"] {
        let sql = format!(r#"PRAGMA table_info("{table}")"#);
        let has_phone = connection
            .prepare(&sql)?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(Result::ok)
            .any(|name| name == "Telefon");
        if !has_phone {
            connection.execute(
                &format!(r#"ALTER TABLE "{table}" ADD COLUMN "Telefon" TEXT"#),
                [],
            )?;
        }
    }
    Ok(())
}

fn authenticated_user(state: &State<'_, AppState>) -> Result<String, String> {
    ensure_startup_ready(state)?;
    let session = state
        .session
        .lock()
        .map_err(|_| "Přihlášení již není platné.".to_string())?;
    if !session.authenticated {
        return Err("Přihlášení již není platné.".into());
    }
    Ok(session.user.clone())
}

fn require_admin(state: &State<'_, AppState>) -> Result<String, String> {
    ensure_startup_ready(state)?;
    let session = state
        .session
        .lock()
        .map_err(|_| "Přihlášení již není platné.".to_string())?;
    if !session.authenticated || session.role != "Správce" {
        return Err("Ke správě sazeb nemáte oprávnění.".into());
    }
    Ok(session.user.clone())
}

fn ensure_startup_ready(state: &State<'_, AppState>) -> Result<(), String> {
    let startup = state.startup.lock().map_err(|_| "Stav databáze není dostupný.".to_string())?;
    if startup.ready { return Ok(()); }
    Err(startup.failure.as_ref().map(ToString::to_string).unwrap_or_else(|| "Databáze ještě není připravena.".into()))
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn parse_date(value: &Option<String>) -> Result<Option<NaiveDate>, String> {
    match clean_optional(value.clone()) {
        Some(text) => NaiveDate::parse_from_str(&text, "%Y-%m-%d")
            .map(Some)
            .map_err(|_| "Zkontrolujte datum pojištění.".to_string()),
        None => Ok(None),
    }
}

fn access_month_count(start: Option<NaiveDate>, end: Option<NaiveDate>) -> i64 {
    match (start, end) {
        (Some(start), Some(end)) => i64::from(end.month()) - i64::from(start.month()) + 1,
        _ => 0,
    }
}

fn sqlite_date(value: Option<NaiveDate>) -> Option<String> {
    value.map(|date| format!("{date} 00:00:00"))
}

fn ensure_backup(connection: &Connection, database_path: &Path) -> Result<(), String> {
    let directory = database_path
        .parent()
        .ok_or_else(|| FRIENDLY_DATABASE_ERROR.to_string())?
        .join("backups");
    fs::create_dir_all(&directory).map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
    let backup_path = directory.join("dd-before-first-write.sqlite");
    if backup_path.exists() {
        return Ok(());
    }
    connection
        .execute_batch("PRAGMA wal_checkpoint(FULL);")
        .map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
    fs::copy(database_path, backup_path).map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
    Ok(())
}

#[cfg(test)]
fn create_audit_table(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        r#"CREATE TABLE IF NOT EXISTS "AuditLog" (
            "Id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "DatumČas" TEXT NOT NULL,
            "Uživatel" TEXT NOT NULL,
            "Operace" TEXT NOT NULL,
            "IdentifikátorPojištěnce" TEXT,
            "Výsledek" TEXT NOT NULL
        );"#,
    )
}

fn record_error(path: &Path, user: &str, identifier: Option<i64>) {
    if let Ok(connection) = open_write(path) {
        let _ = connection.execute(
                r#"INSERT INTO "AuditLog"
                   ("DatumČas", "Uživatel", "Operace", "IdentifikátorPojištěnce", "Výsledek")
                   VALUES (datetime('now'), ?1, 'INSERT', ?2, 'ERROR')"#,
                params![user, identifier.map(|value| value.to_string())],
            );
    }
}

fn next_identifier(transaction: &rusqlite::Transaction<'_>) -> rusqlite::Result<i64> {
    transaction.query_row(
        r#"SELECT COALESCE(MAX(CAST("Identifikátor" AS INTEGER)), 0) + 1 FROM "Seznam""#,
        [],
        |row| row.get(0),
    )
}

fn last_registration(connection: &Connection, year: i32) -> rusqlite::Result<(i64, String)> {
    connection
        .query_row(
            r#"SELECT
                   COALESCE(CAST("EvČíslo" AS INTEGER), 0),
                   ' - ' || COALESCE("Příjmení", '') || ' ' || COALESCE("Jméno", '')
               FROM "Seznam"
               WHERE substr(CAST("PojištěníOd" AS TEXT), 1, 4) = ?1
               ORDER BY CAST("EvČíslo" AS INTEGER) DESC
               LIMIT 1"#,
            [year.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .or_else(|error| {
            if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                Ok((0, String::new()))
            } else {
                Err(error)
            }
        })
}

fn registration_number_options(connection: &Connection) -> rusqlite::Result<RegistrationNumberOptions> {
    let mut statement = connection.prepare(
        r#"SELECT DISTINCT CAST("EvČíslo" AS INTEGER) FROM "Seznam"
           WHERE CAST("EvČíslo" AS INTEGER) > 0 ORDER BY 1"#,
    )?;
    let used = statement.query_map([], |row| row.get::<_, i64>(0))?
        .collect::<Result<BTreeSet<_>, _>>()?;
    let next_number = used.last().copied().unwrap_or(0) + 1;
    let free_numbers = (1..next_number).filter(|number| !used.contains(number)).collect();
    Ok(RegistrationNumberOptions { next_number, free_numbers })
}

fn allocate_registration_number(
    transaction: &rusqlite::Transaction<'_>,
    requested: i64,
) -> Result<i64, String> {
    let options = registration_number_options(transaction)
        .map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
    if requested == options.next_number || options.free_numbers.contains(&requested) {
        return Ok(requested);
    }
    let already_used: bool = transaction.query_row(
        r#"SELECT EXISTS(SELECT 1 FROM "Seznam" WHERE CAST("EvČíslo" AS INTEGER)=?1)"#,
        [requested],
        |row| row.get(0),
    ).map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
    if already_used {
        return Err(format!("Evidenční číslo {requested} již bylo použito. Vyberte jiné volné číslo."));
    }
    Err("Vybrané evidenční číslo již není dostupné. Obnovte nabídku a vyberte číslo znovu.".into())
}

fn clean_search(search: Option<String>) -> String {
    search
        .unwrap_or_default()
        .trim()
        .chars()
        .take(100)
        .collect()
}

#[cfg(test)]
fn member_page(
    connection: &Connection,
    search: Option<String>,
    page: u32,
    page_size: u32,
) -> rusqlite::Result<MemberPage> {
    let search = clean_search(search);
    let pattern = format!("%{}%", search.replace('%', "\\%").replace('_', "\\_"));
    let page = page.max(1);
    let page_size = page_size.clamp(10, 200);
    let offset = i64::from((page - 1) * page_size);
    let total = connection.query_row(
        &format!(r#"SELECT COUNT(*) FROM "Seznam" WHERE {MEMBER_SEARCH}"#),
        params![search, pattern],
        |row| row.get(0),
    )?;
    let sql = format!(
        r#"{MEMBER_SELECT}
           WHERE {MEMBER_SEARCH}
           ORDER BY CAST("EvČíslo" AS INTEGER), rowid
           LIMIT ?3 OFFSET ?4"#
    );
    let mut statement = connection.prepare(&sql)?;
    let members = statement
        .query_map(params![search, pattern, page_size, offset], map_member)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MemberPage {
        members,
        total,
        page,
        page_size,
    })
}

fn map_member(row: &Row<'_>) -> rusqlite::Result<MemberRow> {
    Ok(MemberRow {
        row_id: row.get(0)?,
        identifier: row.get(15)?,
        code: row.get(1)?,
        registration_number: row.get(2)?,
        insured: row.get(3)?,
        personal_id: row.get(4)?,
        affiliation: row.get(5)?,
        insurance_from: row.get(6)?,
        insurance_to: row.get(7)?,
        actual_termination: row.get(8)?,
        category: row.get(9)?,
        loss: row.get(10)?,
        annual_premium: row.get(11)?,
        premium: row.get(12)?,
        actual_payment: row.get(13)?,
        note: row.get(14)?,
        title: row.get(16)?,
        last_name: row.get(17)?,
        first_name: row.get(18)?,
        city: row.get(19)?,
        address: row.get(20)?,
        postal_code: row.get(21)?,
        country: row.get(22)?,
        organization: row.get(23)?,
        email: row.get(24)?,
        phone: row.get(25)?,
    })
}

fn current_member_record(
    connection: &Connection,
    row_id: i64,
    active_year: i32,
) -> rusqlite::Result<MemberRow> {
    connection.query_row(
        &format!(
            r#"{MEMBER_SELECT}
               WHERE rowid = ?1 AND pojisteni_rok("PojištěníOd") = ?2"#
        ),
        params![row_id, active_year],
        map_member,
    )
}

fn claim_member_record(
    connection: &Connection,
    selected_row_id: i64,
    occurred_on: &str,
) -> Result<(MemberRow, i32), String> {
    let date = NaiveDate::parse_from_str(occurred_on.trim(), "%Y-%m-%d")
        .map_err(|_| "Vyplňte platné datum vzniku pojistné události.".to_string())?;
    let year = date.year();
    let personal_id: String = connection
        .query_row(
            r#"SELECT TRIM(COALESCE("RodnéČíslo",'')) FROM "Seznam" WHERE rowid=?1"#,
            [selected_row_id],
            |row| row.get(0),
        )
        .map_err(|_| "Vybraný pojištěnec nebyl nalezen.".to_string())?;
    if personal_id.is_empty() {
        return Err("Historické pojištění nelze dohledat: člen nemá rodné číslo.".into());
    }
    let member = connection.query_row(
        &format!(r#"{MEMBER_SELECT} WHERE TRIM("RodnéČíslo")=?1 AND pojisteni_rok("PojištěníOd")=?2 ORDER BY rowid DESC LIMIT 1"#),
        params![personal_id, year], map_member,
    ).map_err(|_| format!("Historické údaje pojištění pro rok {year} nejsou dostupné."))?;
    Ok((member, year))
}

#[tauri::command]
fn resolve_claim_insurance(
    app: AppHandle,
    state: State<'_, AppState>,
    row_id: i64,
    occurred_on: String,
) -> Result<MemberRow, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    claim_member_record(&open_read_only(&path)?, row_id, &occurred_on).map(|result| result.0)
}

fn member_history_records(
    connection: &Connection,
    row_id: i64,
) -> rusqlite::Result<Vec<MemberRow>> {
    let personal_id: Option<String> = connection.query_row(
        r#"SELECT NULLIF(TRIM("RodnéČíslo"), '') FROM "Seznam" WHERE rowid = ?1"#,
        [row_id],
        |row| row.get(0),
    )?;
    let Some(personal_id) = personal_id else {
        return Ok(Vec::new());
    };
    let mut statement = connection.prepare(&format!(
        r#"{MEMBER_SELECT}
           WHERE TRIM("RodnéČíslo") = ?1 AND rowid <> ?2
           ORDER BY pojisteni_rok("PojištěníOd") DESC, rowid DESC"#
    ))?;
    let history = statement
        .query_map(params![personal_id, row_id], map_member)?
        .collect();
    history
}

fn update_current_member_record(
    path: &Path,
    user: &str,
    active_year: i32,
    member: MemberUpdate,
) -> Result<(), String> {
    let expected_code = if member.affiliation == "FVČ" {
        "1"
    } else if member.affiliation == "FV" {
        "2"
    } else {
        return Err("Zkontrolujte odborovou příslušnost.".into());
    };
    if member.code != expected_code {
        return Err("Zkontrolujte kód OC.".into());
    }
    let termination = parse_date(&member.actual_termination)?;
    let mut connection = open_write(path)?;
    ensure_backup(&connection, path)?;
    let result = (|| -> rusqlite::Result<()> {
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let stable_identifier: String = transaction.query_row(
            r#"SELECT CAST("Identifikátor" AS TEXT) FROM "Seznam"
               WHERE rowid = ?1 AND pojisteni_rok("PojištěníOd") = ?2"#,
            params![member.row_id, active_year],
            |row| row.get(0),
        )?;
        let changed = transaction.execute(
            r#"UPDATE "Seznam" SET
                   "Titul" = ?1, "Příjmení" = ?2, "Jméno" = ?3, "RodnéČíslo" = ?4,
                   "EvČíslo" = ?5, "Město" = ?6, "Adresa" = ?7, "PSČ" = ?8,
                   "Stát" = ?9, "ZO" = ?10, "OdbPříslušnost" = ?11, "KódOC" = ?12,
                   "e-mail" = ?13, "Telefon" = ?14, "Poznámka" = ?15, "SkutÚhrada" = ?16, "Ukončení" = ?17
               WHERE rowid = ?18 AND pojisteni_rok("PojištěníOd") = ?19"#,
            params![
                clean_optional(member.title),
                clean_optional(member.last_name),
                clean_optional(member.first_name),
                clean_optional(member.personal_id),
                member.registration_number,
                clean_optional(member.city),
                clean_optional(member.address),
                clean_optional(member.postal_code),
                clean_optional(member.country),
                clean_optional(member.organization),
                member.affiliation,
                member.code,
                clean_optional(member.email),
                clean_optional(member.phone),
                clean_optional(member.note),
                member.actual_payment.unwrap_or(0),
                sqlite_date(termination),
                member.row_id,
                active_year
            ],
        )?;
        if changed != 1 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        transaction.execute(
            r#"INSERT INTO "AuditLog"
               ("DatumČas", "Uživatel", "Operace", "IdentifikátorPojištěnce", "Výsledek")
               VALUES (datetime('now'), ?1, 'UPDATE', ?2, 'OK')"#,
            params![user, stable_identifier],
        )?;
        transaction.commit()
    })();
    result.map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())
}

fn archive_years(connection: &Connection) -> rusqlite::Result<Vec<ArchiveYear>> {
    let mut statement = connection.prepare(
        r#"SELECT
               pojisteni_rok("PojištěníOd") AS rok,
               COUNT(*) AS pocet_zaznamu,
               CASE WHEN COUNT(*) = COUNT(NULLIF(TRIM("RodnéČíslo"), ''))
                    THEN COUNT(DISTINCT TRIM("RodnéČíslo"))
                    ELSE NULL
               END AS pocet_clenu
           FROM "Seznam"
           INNER JOIN "PojistnaObdobi" period
             ON period."Rok" = pojisteni_rok("PojištěníOd")
            AND period."Stav" = 'UZAVRENO'
           WHERE pojisteni_rok("PojištěníOd") IS NOT NULL
           GROUP BY rok
           ORDER BY rok DESC"#,
    )?;
    let years = statement
        .query_map([], |row| {
            Ok(ArchiveYear {
                year: row.get(0)?,
                record_count: row.get(1)?,
                unique_member_count: row.get(2)?,
            })
        })?
        .collect();
    years
}

fn archive_member_page(
    connection: &Connection,
    year: i32,
    search: Option<String>,
    page: u32,
    page_size: u32,
    filters: MemberFilters,
) -> rusqlite::Result<MemberPage> {
    let search = clean_search(search);
    let pattern = format!("%{}%", search.replace('%', "\\%").replace('_', "\\_"));
    let page = page.max(1);
    let page_size = page_size.clamp(10, 200);
    let offset = i64::from((page - 1) * page_size);
    let affiliation = clean_search(filters.affiliation);
    let category = clean_search(filters.category);
    let loss = clean_search(filters.loss);
    let status = clean_search(filters.status);
    let premium = clean_search(filters.premium);
    let payment = clean_search(filters.payment);
    let payment_status = clean_search(filters.payment_status);
    let overdue = clean_search(filters.overdue);
    let total = connection.query_row(
        &format!(
            r#"SELECT COUNT(*) FROM "Seznam"
               WHERE pojisteni_rok("PojištěníOd") = ?1 AND {ARCHIVE_SEARCH}
               {MEMBER_FILTERS}"#
        ),
        params![
            year,
            search,
            pattern,
            affiliation,
            category,
            loss,
            status,
            premium,
            payment,
            payment_status,
            overdue
        ],
        |row| row.get(0),
    )?;
    let sql = format!(
        r#"{MEMBER_SELECT}
           WHERE pojisteni_rok("PojištěníOd") = ?1 AND {ARCHIVE_SEARCH}
           {MEMBER_FILTERS}
           ORDER BY CAST("EvČíslo" AS INTEGER), rowid
           LIMIT ?12 OFFSET ?13"#
    );
    let mut statement = connection.prepare(&sql)?;
    let members = statement
        .query_map(
            params![
                year,
                search,
                pattern,
                affiliation,
                category,
                loss,
                status,
                premium,
                payment,
                payment_status,
                overdue,
                page_size,
                offset
            ],
            map_member,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MemberPage {
        members,
        total,
        page,
        page_size,
    })
}

fn overdue_summary(
    connection: &Connection,
    insurance_year: i32,
) -> rusqlite::Result<(i64, i64, Option<String>)> {
    connection.query_row(
        r#"SELECT COUNT(*),
                  COALESCE(SUM(MAX(COALESCE(seznam."PojistnáČástka", 0) - COALESCE(seznam."SkutÚhrada", 0), 0)), 0),
                  MIN(prikaz."DatumSplatnosti")
           FROM "PrikazyKUhrade" prikaz
           INNER JOIN "Seznam" seznam ON seznam.rowid = prikaz."PojistnyZaznamRowId"
           WHERE prikaz."PojistnyRok" = ?1
             AND pojisteni_rok(seznam."PojištěníOd") = ?1
             AND NULLIF(TRIM(prikaz."DatumSplatnosti"), '') IS NOT NULL
             AND date(prikaz."DatumSplatnosti") < date('now', 'localtime')
             AND COALESCE(seznam."SkutÚhrada", 0) < COALESCE(seznam."PojistnáČástka", 0)"#,
        [insurance_year],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
}

fn validate_input(input: &NewInsured) -> Result<(Option<NaiveDate>, Option<NaiveDate>), String> {
    if !matches!(input.affiliation.as_str(), "FVČ" | "FV") {
        return Err("Zkontrolujte OdbPříslušnost.".into());
    }
    let expected_code = if input.affiliation == "FVČ" { 1 } else { 2 };
    if input.code != expected_code {
        return Err("Zkontrolujte KódOC.".into());
    }
    if !matches!(input.category.as_str(), "A" | "B" | "C") {
        return Err("Zkontrolujte Kategorie.".into());
    }
    if let Some(personal_id) = clean_optional(input.personal_id.clone()) {
        let valid = personal_id.len() == 11
            && personal_id.as_bytes().get(6) == Some(&b'/')
            && personal_id
                .chars()
                .enumerate()
                .all(|(index, character)| index == 6 || character.is_ascii_digit());
        if !valid {
            return Err("Zkontrolujte RodnéČíslo.".into());
        }
    }
    if let Some(postal_code) = clean_optional(input.postal_code.clone()) {
        let valid_character = |character: char| character.is_ascii_digit() || character == ' ';
        if postal_code.chars().filter(char::is_ascii_digit).count() != 5
            || !postal_code.chars().all(valid_character)
        {
            return Err("Zkontrolujte PSČ.".into());
        }
    }
    Ok((
        parse_date(&input.insurance_from)?,
        parse_date(&input.insurance_to)?,
    ))
}

fn save_to_database(path: &Path, user: &str, input: NewInsured) -> Result<SaveResult, String> {
    let (insurance_from, insurance_to) = validate_input(&input)?;
    let months = access_month_count(insurance_from, insurance_to);
    let mut connection = open_write(path)?;
    let tariff_date = insurance_from
        .or_else(|| NaiveDate::from_ymd_opt(input.registration_year, 1, 1))
        .ok_or_else(|| "Zkontrolujte datum pojištění.".to_string())?;
    let tariff = tariffs::calculate(
        &connection,
        &input.category,
        input.loss,
        input.annual_amount,
        tariff_date,
        months,
    )
    .map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?
    .ok_or_else(|| "Pro zadané údaje není platná sazba pojistného.".to_string())?;
    ensure_backup(&connection, path)?;

    let result = (|| -> Result<SaveResult, String> {
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
        transaction.execute(r#"DELETE FROM "Editace""#, [])
            .map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;

        let registration_number = allocate_registration_number(&transaction, input.registration_number)?;
        transaction.execute(
            r#"INSERT INTO "Editace" (
                "Titul", "Příjmení", "Jméno", "RodnéČíslo", "ZO", "OdbPříslušnost",
                "Město", "Adresa", "PSČ", "Stát", "Poznámka", "PojištěníOd",
                "PojištěníDo", "RočPojistné", "Kategorie", "Ztráta",
                "PojistnáČástka", "SkutÚhrada", "KódOC", "EvČíslo", "E-mail", "Telefon"
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22
            )"#,
            params![
                clean_optional(input.title),
                clean_optional(input.last_name),
                clean_optional(input.first_name),
                clean_optional(input.personal_id),
                clean_optional(input.organization),
                input.affiliation,
                clean_optional(input.city),
                clean_optional(input.address),
                clean_optional(input.postal_code),
                clean_optional(input.country),
                clean_optional(input.note),
                sqlite_date(insurance_from),
                sqlite_date(insurance_to),
                input.annual_amount,
                input.category,
                if input.loss { -1 } else { 0 },
                tariff.insured_amount.round() as i64,
                input.actual_payment.unwrap_or(0),
                input.code.to_string(),
                registration_number,
                clean_optional(input.email),
                clean_optional(input.phone),
            ],
        ).map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;

        let identifier = next_identifier(&transaction).map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
        transaction.execute(
            r#"INSERT INTO "Seznam" (
                "Identifikátor", "PojištěníOd", "PojištěníDo", "RočPojistné",
                "PojistnáČástka", "Kategorie", "Ztráta", "KódOC", "EvČíslo",
                "Titul", "Příjmení", "Jméno", "RodnéČíslo", "Město", "Adresa",
                "PSČ", "Stát", "Poznámka", "OdbPříslušnost", "ZO", "SkutÚhrada",
                "e-mail", "Telefon", "Tisk"
            )
            SELECT
                ?1, "PojištěníOd", "PojištěníDo", "RočPojistné", "PojistnáČástka",
                "Kategorie", "Ztráta", "KódOC", "EvČíslo", "Titul", "Příjmení",
                "Jméno", "RodnéČíslo", "Město", "Adresa", "PSČ", "Stát",
                "Poznámka", "OdbPříslušnost", "ZO", "SkutÚhrada", "E-mail", "Telefon", 0
            FROM "Editace""#,
            [identifier],
        ).map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
        transaction.execute(r#"DELETE FROM "Editace""#, [])
            .map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
        transaction.execute(
            r#"INSERT INTO "AuditLog"
               ("DatumČas", "Uživatel", "Operace", "IdentifikátorPojištěnce", "Výsledek")
               VALUES (datetime('now'), ?1, 'INSERT', ?2, 'OK')"#,
            params![user, identifier.to_string()],
        ).map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
        transaction.commit().map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
        Ok(SaveResult {
            identifier,
            registration_number,
        })
    })();

    result.map_err(|message| {
        record_error(path, user, None);
        message
    })
}

fn system_username() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "Správce".into())
}

fn auth_initialized(connection: &Connection) -> Result<bool, String> {
    connection
        .query_row(
            r#"SELECT EXISTS(SELECT 1 FROM "AppUsers" WHERE "Active"=1)"#,
            [],
            |row| row.get(0),
        )
        .map_err(|_| "Přihlášení se nepodařilo připravit.".to_string())
}

fn initialize_admin_at(path: &Path, password: &str) -> Result<LoginResult, String> {
    if password.chars().count() < 12 {
        return Err("Heslo musí mít alespoň 12 znaků.".into());
    }
    let mut connection = open_write(path)?;
    #[cfg(test)]
    connection.execute_batch(r#"CREATE TABLE IF NOT EXISTS "AppUsers" (
        "Id" INTEGER PRIMARY KEY AUTOINCREMENT, "Username" TEXT NOT NULL UNIQUE,
        "PasswordHash" TEXT NOT NULL, "Role" TEXT NOT NULL,
        "CreatedAt" TEXT NOT NULL DEFAULT (datetime('now')), "Active" INTEGER NOT NULL DEFAULT 1
    );"#).unwrap();
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| "Účet správce se nepodařilo vytvořit.".to_string())?;
    let exists: bool = transaction
        .query_row(r#"SELECT EXISTS(SELECT 1 FROM "AppUsers")"#, [], |row| {
            row.get(0)
        })
        .map_err(|_| "Účet správce se nepodařilo vytvořit.".to_string())?;
    if exists {
        return Err("Účet správce již byl vytvořen.".into());
    }
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| "Účet správce se nepodařilo vytvořit.".to_string())?
        .to_string();
    let user = system_username();
    transaction
        .execute(
            r#"INSERT INTO "AppUsers" ("Username", "PasswordHash", "Role") VALUES (?1, ?2, 'Správce')"#,
            params![user, password_hash],
        )
        .map_err(|_| "Účet správce se nepodařilo vytvořit.".to_string())?;
    transaction
        .commit()
        .map_err(|_| "Účet správce se nepodařilo vytvořit.".to_string())?;
    Ok(LoginResult {
        user,
        role: "Správce".into(),
    })
}

fn verify_login_at(path: &Path, password: &str) -> Result<LoginResult, String> {
    let connection = open_write(path)?;
    if !auth_initialized(&connection)? {
        return Err("Nejprve vytvořte účet správce.".into());
    }
    let (user, password_hash, role): (String, String, String) = connection
        .query_row(
            r#"SELECT "Username", "PasswordHash", "Role" FROM "AppUsers" WHERE "Active"=1 ORDER BY "Id" LIMIT 1"#,
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| "Neplatné heslo - přístup není povolen!".to_string())?;
    let parsed_hash = PasswordHash::new(&password_hash)
        .map_err(|_| "Neplatné heslo - přístup není povolen!".to_string())?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| "Neplatné heslo - přístup není povolen!".to_string())?;
    Ok(LoginResult { user, role })
}

fn establish_session(
    state: &State<'_, AppState>,
    login: LoginResult,
) -> Result<LoginResult, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "Neplatné heslo - přístup není povolen!".to_string())?;
    session.authenticated = true;
    session.user = login.user.clone();
    session.role = login.role.clone();
    Ok(login)
}

#[tauri::command]
fn get_startup_status(state: State<'_, AppState>) -> Result<StartupStatus, String> {
    state.startup.lock().map(|status| status.clone()).map_err(|_| "Stav databáze není dostupný.".into())
}

#[tauri::command]
fn get_auth_status(app: AppHandle, state: State<'_, AppState>) -> Result<AuthStatus, String> {
    ensure_startup_ready(&state)?;
    let path = working_database_path(&app)?;
    let connection = open_write(&path)?;
    Ok(AuthStatus {
        initialized: auth_initialized(&connection)?,
    })
}

#[tauri::command]
fn initialize_admin(
    app: AppHandle,
    state: State<'_, AppState>,
    password: String,
) -> Result<LoginResult, String> {
    ensure_startup_ready(&state)?;
    let path = working_database_path(&app)?;
    establish_session(&state, initialize_admin_at(&path, &password)?)
}

#[tauri::command]
fn login(
    app: AppHandle,
    password: String,
    state: State<'_, AppState>,
) -> Result<LoginResult, String> {
    ensure_startup_ready(&state)?;
    let path = working_database_path(&app)?;
    establish_session(&state, verify_login_at(&path, &password)?)
}

#[tauri::command]
fn logout(state: State<'_, AppState>) -> Result<(), String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "Odhlášení se nepodařilo.".to_string())?;
    *session = Session::default();
    Ok(())
}

#[tauri::command]
fn calculate_tariff(
    app: AppHandle,
    state: State<'_, AppState>,
    category: String,
    loss: bool,
    annual_amount: i64,
    insurance_from: Option<String>,
    insurance_to: Option<String>,
) -> Result<tariffs::TariffResult, String> {
    authenticated_user(&state)?;
    let start = parse_date(&insurance_from)?;
    let end = parse_date(&insurance_to)?;
    let tariff_date = start.ok_or_else(|| "Zadejte datum začátku pojištění.".to_string())?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    tariffs::calculate(
        &connection,
        &category,
        loss,
        annual_amount,
        tariff_date,
        access_month_count(start, end),
    )
    .map_err(|_| "Sazbu pojistného se nepodařilo načíst.".to_string())?
    .ok_or_else(|| "Pro zadané údaje není platná sazba pojistného.".to_string())
}

#[tauri::command]
fn get_form_options(
    app: AppHandle,
    state: State<'_, AppState>,
    affiliation: String,
    year: i32,
) -> Result<FormOptions, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    let mut statement = connection
        .prepare(
            r#"SELECT DISTINCT "ZO" FROM "Seznam"
               WHERE "OdbPříslušnost" = ?1 AND NULLIF(TRIM("ZO"), '') IS NOT NULL
               ORDER BY "ZO""#,
        )
        .map_err(|_| "Údaje formuláře se nepodařilo načíst.".to_string())?;
    let organizations = statement
        .query_map([affiliation], |row| row.get(0))
        .map_err(|_| "Údaje formuláře se nepodařilo načíst.".to_string())?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|_| "Údaje formuláře se nepodařilo načíst.".to_string())?;
    let (last_registration_number, last_client) = last_registration(&connection, year)
        .map_err(|_| "Údaje formuláře se nepodařilo načíst.".to_string())?;
    Ok(FormOptions {
        organizations,
        last_registration_number,
        last_client,
        annual_amounts: tariffs::insured_amounts(&connection)
            .map_err(|_| "Údaje formuláře se nepodařilo načíst.".to_string())?,
    })
}

#[tauri::command]
fn get_registration_number_options(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<RegistrationNumberOptions, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    registration_number_options(&open_read_only(&path)?)
        .map_err(|_| "Evidenční čísla se nepodařilo načíst.".to_string())
}

#[tauri::command]
fn save_insured(
    app: AppHandle,
    state: State<'_, AppState>,
    insured: NewInsured,
) -> Result<SaveResult, String> {
    let user = authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    save_to_database(&path, &user, insured)
}

fn application_tariffs(connection: &Connection, date: NaiveDate) -> Result<Vec<(String, bool, i64, i64)>, String> {
    let mut rows = Vec::new();
    let amounts = tariffs::application_amounts(connection, date)
        .map_err(|_| "Aktuální limity pojištění se nepodařilo načíst.".to_string())?;
    if amounts.is_empty() {
        return Err("Pro elektronickou přihlášku nejsou dostupné kompletní aktuální sazby.".into());
    }
    for category in ["A", "B", "C"] {
        for loss in [false, true] {
            for amount in &amounts {
                let rate = tariffs::calculate(connection, category, loss, *amount, date, 12)
                    .map_err(|_| "Aktuální sazby se nepodařilo načíst.".to_string())?
                    .ok_or_else(|| format!("Chybí aktuální sazba pro kategorii {category}, limit {amount}."))?;
                rows.push((category.to_string(), loss, *amount, rate.premium));
            }
        }
    }
    Ok(rows)
}

#[tauri::command]
fn get_application_options(
    app: AppHandle,
    state: State<'_, AppState>,
    affiliation: String,
) -> Result<applications::ApplicationOptions, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    let application_start = applications::next_month_start(Local::now().date_naive())
        .ok_or_else(|| "Datum počátku pojištění se nepodařilo vypočítat.".to_string())?;
    Ok(applications::ApplicationOptions {
        organizations: applications::organizations(&connection, &affiliation)
            .map_err(|_| "Organizace se nepodařilo načíst.".to_string())?,
        annual_amounts: tariffs::application_amounts(&connection, application_start)
            .map_err(|_| "Limity pojištění se nepodařilo načíst.".to_string())?,
    })
}

#[tauri::command]
fn create_blank_application_for_printing(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    let connection = open_write(&path)?;
    let today = Local::now().date_naive();
    let rates = application_tariffs(&connection, today)?;
    let destination = std::env::temp_dir().join(format!(
        "federace-prazdna-prihlaska-{}-{}-{}.pdf",
        today.format("%Y-%m-%d"),
        std::process::id(),
        Local::now().timestamp_millis()
    ));
    let payment_settings=payments::load_settings(&connection).map_err(|_|"Údaje Federace se nepodařilo načíst.".to_string())?;
    let receipt_settings=receipts::load_settings(&connection)?;
    let data = applications::PdfData { federation_name:if payment_settings.recipient_name.trim().is_empty(){receipt_settings.policyholder}else{payment_settings.recipient_name}, account:format!("{}/{}",payment_settings.account_number,payment_settings.bank_code).trim_matches('/').to_string(), contract_number:receipt_settings.contract_number, registration_number:String::new(), first_name:String::new(), last_name:String::new(), personal_id:String::new(), address:String::new(), city:String::new(), postal_code:String::new(), email:String::new(), category:String::new(), loss:false, annual_amount:0, premium:0, application_date:today.format("%Y-%m-%d").to_string(), blank:true };
    applications::render_pdf(&data, &rates, &destination)?;
    Ok(destination.to_string_lossy().into_owned())
}

#[tauri::command]
fn complete_application(
    app: AppHandle,
    state: State<'_, AppState>,
    application: applications::ApplicationInput,
) -> Result<applications::ApplicationResult, String> {
    let user = authenticated_user(&state)?;
    let database = working_database_path(&app)?;
    complete_application_at(&database, &user, application, Local::now().date_naive())
}

fn complete_application_at(
    database: &Path,
    user: &str,
    application: applications::ApplicationInput,
    application_date: NaiveDate,
) -> Result<applications::ApplicationResult, String> {
    applications::validate(&application)?;
    let mut connection = open_write(&database)?;
    ensure_backup(&connection, &database)?;
    let insurance_from = applications::next_month_start(application_date).ok_or_else(||"Datum počátku pojištění se nepodařilo vypočítat.".to_string())?;
    let year=insurance_from.year();
    let insurance_to = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();
    let months = access_month_count(Some(insurance_from), Some(insurance_to));
    let tariff = tariffs::calculate(&connection, &application.category, application.loss, application.annual_amount, insurance_from, months)
        .map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?.ok_or_else(||"Pro zadané údaje není platná aktuální sazba pojistného.".to_string())?;
    let rates = application_tariffs(&connection, insurance_from)?;
    let payment_settings=payments::load_settings(&connection).map_err(|_|"Údaje Federace se nepodařilo načíst.".to_string())?;
    let receipt_settings=receipts::load_settings(&connection)?;
    let mut pdf_data = applications::PdfData { federation_name:if payment_settings.recipient_name.trim().is_empty(){receipt_settings.policyholder}else{payment_settings.recipient_name}, account:format!("{}/{}",payment_settings.account_number,payment_settings.bank_code).trim_matches('/').to_string(), contract_number:receipt_settings.contract_number, registration_number:String::new(), first_name:application.first_name.trim().into(), last_name:application.last_name.trim().into(), personal_id:application.personal_id.trim().into(), address:application.address.trim().into(), city:application.city.trim().into(), postal_code:application.postal_code.trim().into(), email:application.email.clone().unwrap_or_default().trim().into(), category:application.category.clone(), loss:application.loss, annual_amount:application.annual_amount, premium:tariff.premium, application_date:application_date.format("%Y-%m-%d").to_string(), blank:false };
    let temp = std::env::temp_dir().join(format!("prihlaska-{}-{}-{}.pdf", application_date.format("%Y%m%d"), std::process::id(), Local::now().timestamp_millis()));

    let outcome = (|| -> Result<(i64,i64,i64),String> {
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
        let registration_number = allocate_registration_number(&transaction, application.registration_number)?;
        let identifier = next_identifier(&transaction).map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
        pdf_data.registration_number=registration_number.to_string();
        let pdf=applications::render_pdf(&pdf_data,&rates,&temp)?;
        transaction.execute(r#"INSERT INTO "Seznam"(
          "Identifikátor","PojištěníOd","PojištěníDo","RočPojistné","PojistnáČástka","Kategorie","Ztráta","KódOC","EvČíslo",
          "Titul","Příjmení","Jméno","RodnéČíslo","Město","Adresa","PSČ","Stát","Poznámka","OdbPříslušnost","ZO","SkutÚhrada","e-mail","Telefon","Tisk")
          VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,NULL,?10,?11,?12,?13,?14,?15,'Česká republika',NULL,?16,?17,0,NULLIF(?18,''),NULL,0)"#,
          params![identifier,sqlite_date(Some(insurance_from)),sqlite_date(Some(insurance_to)),application.annual_amount,tariff.insured_amount.round() as i64,application.category,if application.loss{-1}else{0},application.code.trim(),registration_number,application.last_name.trim(),application.first_name.trim(),application.personal_id.trim(),application.city.trim(),application.address.trim(),application.postal_code.trim(),application.affiliation,application.organization.trim(),application.email.clone().unwrap_or_default().trim()])
          .map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
        let row_id=transaction.last_insert_rowid();
        let application_id=applications::insert_snapshot(&transaction,row_id,identifier,&pdf_data,&insurance_from.format("%Y-%m-%d").to_string(),&pdf)
            .map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
        transaction.execute(r#"INSERT INTO "AuditLog"("DatumČas","Uživatel","Operace","IdentifikátorPojištěnce","Výsledek") VALUES(datetime('now'),?1,'PŘIHLÁŠKA + INSERT',?2,'OK')"#,params![user,identifier.to_string()])
            .map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
        transaction.commit().map_err(|_| FRIENDLY_DATABASE_ERROR.to_string())?;
        Ok((application_id,identifier,registration_number))
    })();
    let outcome=match outcome { Ok(value)=>value, Err(message)=>{let _=fs::remove_file(&temp);record_error(database,user,None);return Err(message);} };
    Ok(applications::ApplicationResult { application_id:outcome.0, identifier:outcome.1, registration_number:outcome.2, application_date:application_date.format("%Y-%m-%d").to_string(), insurance_from:insurance_from.format("%Y-%m-%d").to_string(), premium:tariff.premium, pdf_path:temp.to_string_lossy().into_owned() })
}

#[tauri::command]
fn export_application_snapshot(app:AppHandle,state:State<'_,AppState>,id:i64)->Result<Option<String>,String>{
    authenticated_user(&state)?; let path=working_database_path(&app)?; let connection=open_read_only(&path)?; let (name,pdf)=applications::snapshot(&connection,id)?;
    let Some(destination)=rfd::FileDialog::new().set_file_name(&name).add_filter("Dokument PDF", &["pdf"]).save_file() else{return Ok(None)};
    fs::write(&destination,pdf).map_err(|_|"Přihlášku se nepodařilo uložit.".to_string())?; Ok(Some(destination.to_string_lossy().into_owned()))
}

#[tauri::command]
fn list_members(
    app: AppHandle,
    state: State<'_, AppState>,
    search: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
    filters: Option<MemberFilters>,
) -> Result<MemberPage, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let active_year = ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    archive_member_page(
        &connection,
        active_year,
        search,
        page.unwrap_or(1),
        page_size.unwrap_or(50),
        filters.unwrap_or_default(),
    )
    .map_err(|_| "Seznam členů se nepodařilo načíst.".to_string())
}

#[tauri::command]
fn get_member(
    app: AppHandle,
    state: State<'_, AppState>,
    row_id: i64,
) -> Result<MemberRow, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let connection = open_read_only(&path)?;
    connection
        .query_row(
            &format!("{MEMBER_SELECT} WHERE rowid = ?1"),
            [row_id],
            map_member,
        )
        .map_err(|_| "Detail člena se nepodařilo zobrazit.".to_string())
}

#[tauri::command]
fn get_current_member(
    app: AppHandle,
    state: State<'_, AppState>,
    row_id: i64,
) -> Result<MemberRow, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let active_year = ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    current_member_record(&connection, row_id, active_year)
        .map_err(|_| "Detail člena se nepodařilo zobrazit.".to_string())
}

#[tauri::command]
fn get_member_history(
    app: AppHandle,
    state: State<'_, AppState>,
    row_id: i64,
) -> Result<Vec<MemberRow>, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    member_history_records(&connection, row_id)
        .map_err(|_| "Historii pojištění se nepodařilo načíst.".to_string())
}

#[tauri::command]
fn update_current_member(
    app: AppHandle,
    state: State<'_, AppState>,
    member: MemberUpdate,
) -> Result<MemberRow, String> {
    let user = require_admin(&state)?;
    let path = working_database_path(&app)?;
    let active_year = ensure_current_insurance_year(&path)?;
    let row_id = member.row_id;
    update_current_member_record(&path, &user, active_year, member)?;
    let connection = open_read_only(&path)?;
    current_member_record(&connection, row_id, active_year)
        .map_err(|_| "Detail člena se nepodařilo načíst.".to_string())
}

#[tauri::command]
fn deactivate_current_member(
    app: AppHandle,
    state: State<'_, AppState>,
    row_id: i64,
    reason: String,
) -> Result<(), String> {
    let user = require_admin(&state)?;
    let reason = reason.trim();
    if reason.len() < 5 {
        return Err("Uveďte důvod storna alespoň pěti znaky.".into());
    }
    let path = working_database_path(&app)?;
    let active_year = ensure_current_insurance_year(&path)?;
    deactivate_current_member_at(&path, &user, active_year, row_id, reason)
}

fn deactivate_current_member_at(
    path: &Path,
    user: &str,
    active_year: i32,
    row_id: i64,
    reason: &str,
) -> Result<(), String> {
    let mut connection = open_write(&path)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| "Storno se nepodařilo zahájit.".to_string())?;
    let identifier: String=transaction.query_row(r#"SELECT COALESCE(CAST("Identifikátor" AS TEXT),CAST(rowid AS TEXT)) FROM "Seznam" WHERE rowid=?1 AND pojisteni_rok("PojištěníOd")=?2 AND NULLIF(TRIM("Ukončení"),'') IS NULL"#,params![row_id,active_year],|row|row.get(0)).map_err(|_|"Aktuální aktivní záznam nebyl nalezen.".to_string())?;
    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    transaction.execute(r#"UPDATE "Seznam" SET "Ukončení"=?1,"Poznámka"=TRIM(COALESCE("Poznámka",'') || CASE WHEN NULLIF(TRIM(COALESCE("Poznámka",'')),'') IS NULL THEN '' ELSE char(10) END || 'STORNO: ' || ?2) WHERE rowid=?3"#,params![today,reason,row_id]).map_err(|_|"Záznam se nepodařilo stornovat.".to_string())?;
    transaction.execute(r#"INSERT INTO "AuditLog"("DatumČas","Uživatel","IdentifikátorPojištěnce","Operace","Výsledek") VALUES(CURRENT_TIMESTAMP,?1,?2,'DEACTIVATE','OK')"#,params![user,identifier]).map_err(|_|"Storno se nepodařilo zaznamenat.".to_string())?;
    transaction
        .commit()
        .map_err(|_| "Storno se nepodařilo uložit.".to_string())
}

#[tauri::command]
fn list_archive_years(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<ArchiveYear>, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    archive_years(&connection).map_err(|_| "Archiv se nepodařilo načíst.".to_string())
}

#[tauri::command]
fn list_archive_members(
    app: AppHandle,
    state: State<'_, AppState>,
    year: i32,
    search: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
    filters: Option<MemberFilters>,
) -> Result<MemberPage, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    archive_member_page(
        &connection,
        year,
        search,
        page.unwrap_or(1),
        page_size.unwrap_or(50),
        filters.unwrap_or_default(),
    )
    .map_err(|_| "Archiv se nepodařilo načíst.".to_string())
}

#[tauri::command]
fn list_legacy_table(
    app: AppHandle,
    state: State<'_, AppState>,
    source: String,
    search: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<LegacyTablePage, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let connection = open_read_only(&path)?;
    legacy_table_page(
        &connection,
        &source,
        search,
        page.unwrap_or(1),
        page_size.unwrap_or(100),
    )
}

#[tauri::command]
fn get_dashboard(app: AppHandle, state: State<'_, AppState>) -> Result<DashboardInfo, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let active_insurance_year = ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    let member_count = connection
        .query_row(
            r#"SELECT COUNT(*) FROM "Seznam"
               WHERE pojisteni_rok("PojištěníOd") = ?1"#,
            [active_insurance_year],
            |row| row.get(0),
        )
        .map_err(|_| "Přehled se nepodařilo načíst.".to_string())?;
    let last_registration_number = connection
        .query_row(
            r#"SELECT COALESCE(MAX(CAST("EvČíslo" AS INTEGER)), 0) FROM "Seznam""#,
            [],
            |row| row.get(0),
        )
        .map_err(|_| "Přehled se nepodařilo načíst.".to_string())?;
    let (overdue_count, overdue_amount, oldest_due_date) =
        overdue_summary(&connection, active_insurance_year)
            .map_err(|_| "Přehled se nepodařilo načíst.".to_string())?;
    let modified = fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .map_err(|_| "Přehled se nepodařilo načíst.".to_string())?;
    let database_date = DateTime::<Local>::from(modified)
        .format("%d.%m.%Y")
        .to_string();
    Ok(DashboardInfo {
        member_count,
        last_registration_number,
        database_date,
        program_version: env!("CARGO_PKG_VERSION"),
        database_schema_version: migrations::DB_SCHEMA_VERSION,
        active_insurance_year,
        commit_sha: option_env!("BUILD_COMMIT").unwrap_or("lokální sestavení"),
        build_date: option_env!("BUILD_DATE").unwrap_or("neuvedeno"),
        git_tag: option_env!("BUILD_TAG").unwrap_or("neuvedeno"),
        overdue_count,
        overdue_amount,
        oldest_due_date,
    })
}

#[tauri::command]
fn get_insurer_overview(
    app: AppHandle,
    state: State<'_, AppState>,
    selection: String,
) -> Result<InsurerOverview, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let insurance_year = ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    let condition = match selection.as_str() {
        "new" => {
            r#"pojisteni_rok(\"PojištěníOd\") = ?1 AND NULLIF(TRIM(\"Ukončení\"), '') IS NULL AND COALESCE(\"SkutÚhrada\", 0) > 0"#
        }
        "terminated" => r#"pojisteni_rok(\"Ukončení\") = ?1 AND COALESCE(\"SkutÚhrada\", 0) > 0"#,
        "underpaid" => {
            r#"pojisteni_rok(\"PojištěníOd\") = ?1 AND NULLIF(TRIM(\"Ukončení\"), '') IS NULL AND COALESCE(\"SkutÚhrada\", 0) < COALESCE(\"PojistnáČástka\", 0)"#
        }
        "oc1" => {
            r#"pojisteni_rok(\"PojištěníOd\") = ?1 AND CAST(\"KódOC\" AS TEXT) = '1' AND NULLIF(TRIM(\"Ukončení\"), '') IS NULL AND COALESCE(\"SkutÚhrada\", 0) > 0"#
        }
        "oc2" => {
            r#"pojisteni_rok(\"PojištěníOd\") = ?1 AND CAST(\"KódOC\" AS TEXT) = '2' AND NULLIF(TRIM(\"Ukončení\"), '') IS NULL AND COALESCE(\"SkutÚhrada\", 0) > 0"#
        }
        _ => return Err("Zvolený přehled není podporován.".to_string()),
    };
    let sql = format!(
        "{MEMBER_SELECT} WHERE {condition} ORDER BY CAST(\"EvČíslo\" AS INTEGER), \"Příjmení\""
    );
    let mut statement = connection
        .prepare(&sql)
        .map_err(|_| "Přehled pro pojišťovnu se nepodařilo načíst.".to_string())?;
    let members = statement
        .query_map([insurance_year], map_member)
        .map_err(|_| "Přehled pro pojišťovnu se nepodařilo načíst.".to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Přehled pro pojišťovnu se nepodařilo načíst.".to_string())?;
    let total_premium = members.iter().map(|member| number(&member.premium)).sum();
    let total_paid = members
        .iter()
        .map(|member| number(&member.actual_payment))
        .sum();
    Ok(InsurerOverview {
        insurance_year,
        members,
        total_premium,
        total_paid,
    })
}

#[tauri::command]
fn list_tariff_rates(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<tariffs::TariffRate>, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    tariffs::list(&connection).map_err(|_| "Sazby pojistného se nepodařilo načíst.".to_string())
}

#[tauri::command]
fn save_tariff_rate(
    app: AppHandle,
    state: State<'_, AppState>,
    rate: tariffs::TariffRateInput,
) -> Result<i64, String> {
    require_admin(&state)?;
    let path = working_database_path(&app)?;
    let connection = open_write(&path)?;
    tariffs::save(&connection, rate)
}

fn number(value: &Option<String>) -> i64 {
    value
        .as_deref()
        .unwrap_or_default()
        .trim()
        .replace(',', ".")
        .parse::<f64>()
        .unwrap_or(0.0)
        .round() as i64
}

fn payment_order_draft(
    connection: &Connection,
    row_id: i64,
    active_year: i32,
) -> Result<payments::PaymentOrderDraft, String> {
    let member = current_member_record(connection, row_id, active_year)
        .map_err(|_| "Aktuální pojištění člena se nepodařilo načíst.".to_string())?;
    let settings = payments::load_settings(connection)
        .map_err(|_| "Platební údaje se nepodařilo načíst.".to_string())?;
    let registration_number = member.registration_number.clone().unwrap_or_default();
    let variable_symbol =
        payments::variable_symbol(member.personal_id.as_deref().unwrap_or_default())?;
    let annual_premium = number(&member.premium);
    let actual_payment = number(&member.actual_payment);
    let mut validation_errors = Vec::new();
    if member
        .address
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
        || member
            .postal_code
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
        || member.city.as_deref().unwrap_or_default().trim().is_empty()
    {
        validation_errors.push("U člena chybí úplná poštovní adresa.".to_string());
    }
    if annual_premium <= 0 {
        validation_errors.push("Roční pojistné musí být vyšší než 0 Kč.".to_string());
    }
    if !payments::settings_complete(&settings) {
        validation_errors.push("V Nastavení chybí název příjemce nebo bankovní účet.".to_string());
    }
    let account = if settings.account_number.trim().is_empty() {
        String::new()
    } else {
        format!(
            "{}/{}",
            settings.account_number.trim(),
            settings.bank_code.trim()
        )
    };
    Ok(payments::PaymentOrderDraft {
        row_id,
        payer_name: member.insured,
        address: member.address.unwrap_or_default(),
        city: member.city.unwrap_or_default(),
        postal_code: member.postal_code.unwrap_or_default(),
        registration_number: registration_number.clone(),
        insurance_year: active_year,
        insured_amount: number(&member.annual_premium),
        annual_premium,
        actual_payment,
        amount_due: (annual_premium - actual_payment).max(0),
        organization: member.organization.unwrap_or_default(),
        variable_symbol,
        recipient_name: settings.recipient_name.clone(),
        account,
        iban: settings.iban.clone(),
        bic: settings.bic.clone(),
        constant_symbol: settings.constant_symbol.clone(),
        issue_date: Local::now().date_naive().format("%Y-%m-%d").to_string(),
        due_date: payments::due_date(settings.default_due_days)
            .format("%Y-%m-%d")
            .to_string(),
        message: payments::render_message(
            &settings.message_template,
            active_year,
            &registration_number,
        ),
        settings_complete: payments::settings_complete(&settings),
        validation_errors,
    })
}

#[tauri::command]
fn get_payment_settings(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<payments::PaymentSettings, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    payments::load_settings(&connection)
        .map_err(|_| "Platební údaje se nepodařilo načíst.".to_string())
}

#[tauri::command]
fn save_payment_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: payments::PaymentSettings,
) -> Result<(), String> {
    require_admin(&state)?;
    let path = working_database_path(&app)?;
    let connection = open_write(&path)?;
    payments::save_settings(&connection, &settings)
}

#[tauri::command]
fn prepare_payment_order(
    app: AppHandle,
    state: State<'_, AppState>,
    row_id: i64,
) -> Result<payments::PaymentOrderDraft, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let active_year = ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    payment_order_draft(&connection, row_id, active_year)
}

#[tauri::command]
fn list_member_payments(
    app: AppHandle,
    state: State<'_, AppState>,
    row_id: i64,
) -> Result<Vec<member_payments::MemberPayment>, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let active_year = ensure_current_insurance_year(&path)?;
    let connection = open_write(&path)?;
    let member = current_member_record(&connection, row_id, active_year)
        .map_err(|_| "Platby člena se nepodařilo načíst.".to_string())?;
    let identifier = member.identifier.as_deref().unwrap_or_default();
    let variable_symbol =
        payments::variable_symbol(member.personal_id.as_deref().unwrap_or_default())?;
    member_payments::bootstrap_legacy_payment(
        &connection,
        row_id,
        identifier,
        active_year,
        &variable_symbol,
        number(&member.actual_payment),
    )
    .map_err(|_| "Platby člena se nepodařilo načíst.".to_string())?;
    member_payments::list(&connection, row_id)
        .map_err(|_| "Platby člena se nepodařilo načíst.".to_string())
}

#[tauri::command]
fn save_member_payment(
    app: AppHandle,
    state: State<'_, AppState>,
    payment: member_payments::PaymentInput,
) -> Result<i64, String> {
    let user = authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let active_year = ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    let member = current_member_record(&connection, payment.insurance_row_id, active_year)
        .map_err(|_| "Aktuální pojištění člena se nepodařilo načíst.".to_string())?;
    let identifier = member.identifier.as_deref().unwrap_or_default().to_string();
    let variable_symbol =
        payments::variable_symbol(member.personal_id.as_deref().unwrap_or_default())?;
    drop(connection);
    let row_id = payment.insurance_row_id;
    let payment_id = member_payments::save(
        &path,
        &user,
        &identifier,
        active_year,
        &variable_symbol,
        payment,
    )?;
    let _ = receipts::create_if_eligible(&path, &user, row_id, active_year, true);
    Ok(payment_id)
}

#[tauri::command]
fn list_payment_organizations(
    app: AppHandle,
    state: State<'_, AppState>,
    year: i32,
) -> Result<Vec<organization_payments::OrganizationOption>, String> {
    authenticated_user(&state)?;
    let c = open_write(&working_database_path(&app)?)?;
    organization_payments::organizations(&c, year)
        .map_err(|_| "Organizace se nepodařilo načíst.".into())
}
#[tauri::command]
fn list_organization_payment_members(
    app: AppHandle,
    state: State<'_, AppState>,
    organization: String,
    year: i32,
) -> Result<Vec<organization_payments::OrganizationMember>, String> {
    authenticated_user(&state)?;
    organization_payments::members(
        &open_read_only(&working_database_path(&app)?)?,
        &organization,
        year,
    )
    .map_err(|_| "Členy organizace se nepodařilo načíst.".into())
}
#[tauri::command]
fn save_organization_payment(
    app: AppHandle,
    state: State<'_, AppState>,
    payment: organization_payments::OrganizationPaymentInput,
) -> Result<i64, String> {
    authenticated_user(&state)?;
    organization_payments::save(&working_database_path(&app)?, payment)
}
#[tauri::command]
fn list_organization_payments(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<organization_payments::OrganizationPayment>, String> {
    authenticated_user(&state)?;
    let c = open_write(&working_database_path(&app)?)?;
    organization_payments::list(&c).map_err(|_| "Organizační platby se nepodařilo načíst.".into())
}
#[tauri::command]
fn get_organization_payment(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
) -> Result<organization_payments::OrganizationPaymentDetail, String> {
    authenticated_user(&state)?;
    organization_payments::detail(&open_read_only(&working_database_path(&app)?)?, id)
        .map_err(|_| "Detail organizační platby se nepodařilo načíst.".into())
}

#[tauri::command]
fn get_email_settings(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<email_service::EmailSettings, String> {
    require_admin(&state)?;
    let path = working_database_path(&app)?;
    email_service::load(&open_write(&path)?)
}

#[tauri::command]
fn save_email_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: email_service::SaveEmailSettings,
) -> Result<(), String> {
    require_admin(&state)?;
    let path = working_database_path(&app)?;
    email_service::save(&open_write(&path)?, settings)
}

#[tauri::command]
fn get_receipt_settings(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<receipts::ReceiptSettings, String> {
    require_admin(&state)?;
    let path = working_database_path(&app)?;
    receipts::load_settings(&open_write(&path)?)
}

#[tauri::command]
fn save_receipt_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: receipts::ReceiptSettings,
) -> Result<(), String> {
    require_admin(&state)?;
    let path = working_database_path(&app)?;
    receipts::save_settings(&open_write(&path)?, &settings)
}

#[tauri::command]
fn list_receipts(
    app: AppHandle,
    state: State<'_, AppState>,
    member_row_id: Option<i64>,
    search: Option<String>,
) -> Result<Vec<receipts::Receipt>, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    receipts::list(
        &open_write(&path)?,
        member_row_id,
        search.as_deref().unwrap_or_default(),
    )
}

#[tauri::command]
fn create_receipt(
    app: AppHandle,
    state: State<'_, AppState>,
    row_id: i64,
) -> Result<Option<i64>, String> {
    let user = authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let year = ensure_current_insurance_year(&path)?;
    receipts::create_if_eligible(&path, &user, row_id, year, false)
}

#[tauri::command]
fn list_certificate_organizations(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let year = ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    let mut statement = connection
        .prepare(
            r#"SELECT DISTINCT TRIM("ZO") FROM "Seznam"
               WHERE pojisteni_rok("PojištěníOd")=?1
                 AND NULLIF(TRIM("Ukončení"),'') IS NULL
                 AND NULLIF(TRIM("ZO"),'') IS NOT NULL
               ORDER BY TRIM("ZO") COLLATE NOCASE"#,
        )
        .map_err(|_| "Organizace pro potvrzení se nepodařilo načíst.".to_string())?;
    statement
        .query_map([year], |row| row.get(0))
        .and_then(|rows| rows.collect::<rusqlite::Result<Vec<String>>>())
        .map_err(|_| "Organizace pro potvrzení se nepodařilo načíst.".to_string())
}

fn certificate_batch_row_ids(
    connection: &Connection,
    year: i32,
    paid_from: &str,
    organization_code: &str,
    organization: &str,
) -> Result<Vec<i64>, String> {
    let mut statement = connection
        .prepare(
            r#"SELECT member.rowid
               FROM "Seznam" member
               WHERE pojisteni_rok(member."PojištěníOd") = ?1
                 AND NULLIF(TRIM(member."Ukončení"), '') IS NULL
                 AND COALESCE((SELECT SUM(payment."Castka") FROM "PlatbyClenu" payment
                               WHERE payment."PojistnyZaznamRowId" = member.rowid),
                              COALESCE(member."SkutÚhrada", 0)) >= COALESCE(member."PojistnáČástka", 0)
                 AND (?2 = '' OR EXISTS (SELECT 1 FROM "PlatbyClenu" payment
                                          WHERE payment."PojistnyZaznamRowId" = member.rowid
                                            AND date(payment."DatumPrijeti") >= date(?2)))
                 AND (?3 = '' OR CAST(member."KódOC" AS TEXT) = ?3)
                 AND (?4 = '' OR member."ZO" = ?4)
               ORDER BY CAST(member."EvČíslo" AS INTEGER), member."Příjmení", member."Jméno""#,
        )
        .map_err(|_| "Hromadné potvrzení se nepodařilo připravit.".to_string())?;
    statement
        .query_map(
            params![year, paid_from, organization_code, organization],
            |row| row.get::<_, i64>(0),
        )
        .and_then(|rows| rows.collect::<rusqlite::Result<Vec<_>>>())
        .map_err(|_| "Hromadné potvrzení se nepodařilo připravit.".to_string())
}

#[tauri::command]
fn create_certificate_batch(
    app: AppHandle,
    state: State<'_, AppState>,
    filter: BatchCertificateFilter,
) -> Result<BatchCertificateResult, String> {
    let user = require_admin(&state)?;
    let path = working_database_path(&app)?;
    let year = ensure_current_insurance_year(&path)?;
    let connection = open_write(&path)?;
    let paid_from = filter.paid_from.unwrap_or_default();
    let organization_code = filter.organization_code.unwrap_or_default();
    let organization = filter.organization.unwrap_or_default();
    let row_ids = certificate_batch_row_ids(
        &connection,
        year,
        &paid_from,
        &organization_code,
        &organization,
    )?;
    drop(connection);

    let mut result = BatchCertificateResult {
        selected: row_ids.len(),
        created: 0,
        existing: 0,
        skipped: 0,
        errors: Vec::new(),
        receipt_ids: Vec::new(),
    };
    for row_id in row_ids {
        let before = open_write(&path)?.query_row(
            r#"SELECT COUNT(*) FROM "DokladyOUhrade" WHERE "PojistnyZaznamRowId"=?1 AND "PojistnyRok"=?2"#,
            params![row_id, year],
            |row| row.get::<_, i64>(0),
        ).unwrap_or(0);
        match receipts::create_if_eligible(&path, &user, row_id, year, false) {
            Ok(Some(id)) => {
                if before > 0 {
                    result.existing += 1;
                } else {
                    result.created += 1;
                }
                result.receipt_ids.push(id);
            }
            Ok(None) => result.skipped += 1,
            Err(error) => result.errors.push(format!("Záznam {row_id}: {error}")),
        }
    }
    Ok(result)
}

#[tauri::command]
fn export_certificate_batch(
    app: AppHandle,
    state: State<'_, AppState>,
    receipt_ids: Vec<i64>,
) -> Result<Option<String>, String> {
    let user = authenticated_user(&state)?;
    if receipt_ids.is_empty() {
        return Err("Nejprve vytvořte nebo vyberte dávku potvrzení.".into());
    }
    let unique_ids = receipt_ids.iter().copied().collect::<BTreeSet<_>>();
    if unique_ids.len() != receipt_ids.len() {
        return Err("Hromadné potvrzení obsahuje duplicitní doklady.".into());
    }
    let path = working_database_path(&app)?;
    let connection = open_write(&path)?;
    let mut documents = Vec::with_capacity(receipt_ids.len());
    let mut years = BTreeSet::new();
    for id in &receipt_ids {
        let (_, bytes) = receipts::pdf(&connection, *id)?;
        let year = connection
            .query_row(
                r#"SELECT "PojistnyRok" FROM "DokladyOUhrade" WHERE "Id"=?1"#,
                [id],
                |row| row.get::<_, i32>(0),
            )
            .map_err(|_| format!("Doklad {id} se nepodařilo načíst."))?;
        years.insert(year);
        documents.push(bytes);
    }
    if years.len() != 1 || documents.len() != receipt_ids.len() {
        return Err("Hromadné potvrzení musí obsahovat všechny doklady jednoho pojistného roku.".into());
    }
    let year = years.iter().next().copied().unwrap_or_default();
    let merged = receipts::merge_pdf_documents(documents)?;
    let Some(destination) = rfd::FileDialog::new()
        .set_title("Uložit hromadné potvrzení")
        .add_filter("PDF", &["pdf"])
        .set_file_name(&format!("hromadne-potvrzeni-{year}.pdf"))
        .save_file()
    else {
        return Ok(None);
    };
    fs::write(&destination, merged)
        .map_err(|_| "Hromadné potvrzení se nepodařilo uložit.".to_string())?;
    for id in receipt_ids {
        connection.execute(r#"INSERT INTO "AuditDokladu"("Uzivatel","IdDokladu","IdentifikatorClena","Operace","Vysledek") SELECT ?1,"Id","IdentifikatorClena",'DÁVKOVÝ EXPORT PDF','OK' FROM "DokladyOUhrade" WHERE "Id"=?2"#,params![user,id]).ok();
    }
    Ok(Some(destination.to_string_lossy().into_owned()))
}

#[tauri::command]
fn get_payment_document_basis(
    app: AppHandle,
    state: State<'_, AppState>,
    row_id: i64,
) -> Result<receipts::PaymentDocumentBasis, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let year = ensure_current_insurance_year(&path)?;
    let connection = open_write(&path)?;
    let result = receipts::load_basis(&connection, row_id, year);
    if let Err(error) = &result {
        eprintln!("payment_document_basis_load_failed command=get_payment_document_basis member_rowid={row_id} year={year} category=basis_validation error={error}");
    }
    result
}

#[tauri::command]
fn export_receipt_pdf(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<String>, String> {
    let user = authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let connection = open_write(&path)?;
    let (name, bytes) = receipts::pdf(&connection, id)?;
    let selected = rfd::FileDialog::new()
        .set_title("Uložit doklad o zaplacení")
        .set_file_name(&name)
        .add_filter("Dokument PDF", &["pdf"])
        .save_file();
    let Some(destination) = selected else {
        return Ok(None);
    };
    fs::write(&destination, bytes).map_err(|_| "Doklad se nepodařilo uložit.".to_string())?;
    connection.execute(r#"INSERT INTO "AuditDokladu"("Uzivatel","IdDokladu","IdentifikatorClena","Operace","Vysledek") SELECT ?1,"Id","IdentifikatorClena",'EXPORT PDF','OK' FROM "DokladyOUhrade" WHERE "Id"=?2"#,params![user,id]).ok();
    Ok(Some(destination.to_string_lossy().into_owned()))
}

#[tauri::command]
fn open_receipt_pdf(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
    print: bool,
) -> Result<(), String> {
    let user = authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let connection = open_write(&path)?;
    let (name, bytes) = receipts::pdf(&connection, id)?;
    let destination = std::env::temp_dir().join(name);
    fs::write(&destination, bytes).map_err(|_| "Doklad se nepodařilo otevřít.".to_string())?;
    let mut command = Command::new("powershell.exe");
    let verb = if print { "Print" } else { "Open" };
    command
        .args(["-NoProfile", "-Command", "Start-Process", "-LiteralPath"])
        .arg(&destination)
        .args(["-Verb", verb, "-WindowStyle", "Hidden"]);
    command
        .spawn()
        .map_err(|_| "Doklad se nepodařilo otevřít.".to_string())?;
    connection.execute(r#"INSERT INTO "AuditDokladu"("Uzivatel","IdDokladu","IdentifikatorClena","Operace","Vysledek") SELECT ?1,"Id","IdentifikatorClena",?2,'OK' FROM "DokladyOUhrade" WHERE "Id"=?3"#,params![user,if print{"TISK"}else{"NÁHLED"},id]).ok();
    Ok(())
}

#[tauri::command]
fn send_receipt_email(app: AppHandle, state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let user = authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    receipts::send(&path, &user, id)
}

#[tauri::command]
fn delete_member_payment(
    app: AppHandle,
    state: State<'_, AppState>,
    row_id: i64,
    payment_id: i64,
) -> Result<(), String> {
    let user = authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let active_year = ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    let member = current_member_record(&connection, row_id, active_year)
        .map_err(|_| "Aktuální pojištění člena se nepodařilo načíst.".to_string())?;
    let identifier = member.identifier.as_deref().unwrap_or_default().to_string();
    drop(connection);
    member_payments::delete(&path, &user, &identifier, active_year, row_id, payment_id)
}

#[tauri::command]
fn generate_payment_order_pdf(
    app: AppHandle,
    state: State<'_, AppState>,
    order: payments::PaymentOrderPdfInput,
) -> Result<Option<String>, String> {
    let user = authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let active_year = ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    let draft = payment_order_draft(&connection, order.row_id, active_year)?;
    if !draft.validation_errors.is_empty() {
        return Err(draft.validation_errors.join(" "));
    }
    let member = current_member_record(&connection, order.row_id, active_year)
        .map_err(|_| "Aktuální pojištění člena se nepodařilo načíst.".to_string())?;
    let filename = payments::safe_filename(
        active_year,
        &draft.registration_number,
        member.last_name.as_deref().unwrap_or_default(),
        member.first_name.as_deref().unwrap_or_default(),
    );
    let Some(destination) = rfd::FileDialog::new()
        .set_title("Uložit příkaz k úhradě")
        .set_file_name(&filename)
        .add_filter("Dokument PDF", &["pdf"])
        .save_file()
    else {
        return Ok(None);
    };
    payments::create_pdf(
        &destination,
        &draft,
        draft.amount_due,
        &draft.due_date,
        &draft.message,
    )?;
    let member_identifier = member
        .identifier
        .unwrap_or_else(|| order.row_id.to_string());
    drop(connection);
    let audit_connection = open_write(&path)?;
    payments::record_order(
        &audit_connection,
        &member_identifier,
        order.row_id,
        active_year,
        &draft.due_date,
        draft.amount_due,
        &draft.issue_date,
    )?;
    payments::record_audit(
        &audit_connection,
        &user,
        &member_identifier,
        active_year,
        "PDF",
    )?;
    Ok(Some(destination.to_string_lossy().to_string()))
}

#[tauri::command]
fn audit_payment_order_print(
    app: AppHandle,
    state: State<'_, AppState>,
    row_id: i64,
) -> Result<(), String> {
    let user = authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let active_year = ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    let draft = payment_order_draft(&connection, row_id, active_year)?;
    if !draft.validation_errors.is_empty() {
        return Err(draft.validation_errors.join(" "));
    }
    let member = current_member_record(&connection, row_id, active_year)
        .map_err(|_| "Aktuální pojištění člena se nepodařilo načíst.".to_string())?;
    let identifier = member.identifier.unwrap_or_else(|| row_id.to_string());
    drop(connection);
    let audit_connection = open_write(&path)?;
    payments::record_order(
        &audit_connection,
        &identifier,
        row_id,
        active_year,
        &draft.due_date,
        draft.amount_due,
        &draft.issue_date,
    )?;
    payments::record_audit(&audit_connection, &user, &identifier, active_year, "PRINT")
}

#[tauri::command]
fn open_generated_pdf(
    state: State<'_, AppState>,
    path: String,
    folder: bool,
) -> Result<(), String> {
    authenticated_user(&state)?;
    let path = PathBuf::from(path);
    if !path.is_file()
        || path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_lowercase()
            != "pdf"
    {
        return Err("Vygenerovaný dokument se nepodařilo otevřít.".into());
    }
    let mut command = Command::new("explorer.exe");
    if folder {
        command.arg("/select,").arg(&path);
    } else {
        command.arg(&path);
    }
    command
        .spawn()
        .map_err(|_| "Vygenerovaný dokument se nepodařilo otevřít.".to_string())?;
    Ok(())
}

#[tauri::command]
fn print_generated_pdf(
    state: State<'_, AppState>,
    path: String,
    delete_after_print: bool,
) -> Result<(), String> {
    authenticated_user(&state)?;
    let path = PathBuf::from(path);
    if !path.is_file()
        || path.extension().and_then(|value| value.to_str()).unwrap_or_default().to_lowercase() != "pdf"
    {
        return Err("Vygenerovaný dokument se nepodařilo vytisknout.".into());
    }
    Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", "Start-Process", "-LiteralPath"])
        .arg(&path)
        .args(["-Verb", "Print", "-WindowStyle", "Hidden"])
        .spawn()
        .map_err(|_| "Tiskový dialog se nepodařilo otevřít.".to_string())?;
    if delete_after_print
        && path.parent() == Some(std::env::temp_dir().as_path())
        && path.file_name().and_then(|value| value.to_str()).is_some_and(|name| name.starts_with("federace-prazdna-prihlaska-"))
    {
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(15 * 60));
            let _ = fs::remove_file(path);
        });
    }
    Ok(())
}

#[tauri::command]
fn preview_operational_report(
    app: AppHandle,
    state: State<'_, AppState>,
    filter: reports::ReportFilter,
) -> Result<reports::ReportPreview, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    reports::preview(&open_write(&path)?, &filter)
}

#[tauri::command]
fn list_legacy_report_history(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<reports::LegacyReportHistory>, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    reports::legacy_history(&open_read_only(&path)?)
        .map_err(|_| "Historii sestav se nepodařilo načíst.".to_string())
}

#[tauri::command]
fn export_operational_report(
    app: AppHandle,
    state: State<'_, AppState>,
    filter: reports::ReportFilter,
    format: String,
) -> Result<Option<String>, String> {
    let user = authenticated_user(&state)?;
    if format != "csv" && format != "pdf" {
        return Err("Nepodporovaný formát sestavy.".into());
    }
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    let connection = open_write(&path)?;
    let report = reports::preview(&connection, &filter)?;
    let safe_title =
        deunicode::deunicode(&report.title).replace(|c: char| !c.is_ascii_alphanumeric(), "_");
    let filename = format!(
        "{}_{}.{}",
        safe_title.trim_matches('_'),
        filter.year,
        format
    );
    let Some(destination) = rfd::FileDialog::new()
        .set_title("Uložit sestavu")
        .set_file_name(&filename)
        .add_filter(
            if format == "pdf" {
                "Dokument PDF"
            } else {
                "Tabulka CSV"
            },
            &[format.as_str()],
        )
        .save_file()
    else {
        return Ok(None);
    };
    if format == "pdf" {
        reports::pdf(&report, &destination)?;
    } else {
        fs::write(&destination, reports::csv(&report))
            .map_err(|_| "CSV se nepodařilo uložit.".to_string())?;
    }
    reports::audit(&connection, &user, &filter, &format, report.total_rows);
    Ok(Some(destination.to_string_lossy().into_owned()))
}

#[tauri::command]
fn list_invoices(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<financial_documents::Invoice>, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    financial_documents::list_invoices(&open_write(&path)?)
}

#[tauri::command]
fn create_invoice(
    app: AppHandle,
    state: State<'_, AppState>,
    invoice: financial_documents::InvoiceInput,
) -> Result<i64, String> {
    let user = require_admin(&state)?;
    let path = working_database_path(&app)?;
    financial_documents::create_invoice(&mut open_write(&path)?, &user, invoice)
}

#[tauri::command]
fn export_payment_batch(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let user = require_admin(&state)?;
    let path = working_database_path(&app)?;
    let mut connection = open_write(&path)?;
    let batch = financial_documents::prepare_batch(&mut connection, &user)?;
    let Some(destination) = rfd::FileDialog::new()
        .set_title("Uložit dávkový příkaz k úhradě")
        .set_file_name(&format!("platebni-davka-{}.csv", batch.batch_id))
        .add_filter("Bankovní dávka CSV", &["csv"])
        .save_file()
    else {
        return Ok(None);
    };
    if let Err(error) = fs::write(&destination, &batch.bytes) {
        return Err(format!("Dávku se nepodařilo uložit: {error}"));
    }
    financial_documents::finish_batch(&mut connection, &user, batch.batch_id, &destination)?;
    Ok(Some(destination.to_string_lossy().into_owned()))
}

#[tauri::command]
fn export_member_document(
    app: AppHandle,
    state: State<'_, AppState>,
    row_id: i64,
    kind: String,
) -> Result<Option<String>, String> {
    let user = authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let year = ensure_current_insurance_year(&path)?;
    let connection = open_write(&path)?;
    let data = financial_documents::member_data(&connection, row_id, year)?;
    let label = match kind.as_str() {
        "application" => "prihlaska",
        "voucher" => "poukazka",
        "envelope" => "obalka",
        "label" => "stitek",
        _ => return Err("Neznámý typ dokumentu.".into()),
    };
    let Some(destination) = rfd::FileDialog::new()
        .set_title("Uložit dokument")
        .set_file_name(&format!(
            "{}_{}_{}.pdf",
            label, data.year, data.registration
        ))
        .add_filter("Dokument PDF", &["pdf"])
        .save_file()
    else {
        return Ok(None);
    };
    financial_documents::member_pdf(&data, &kind, &destination)?;
    let bytes = fs::read(&destination)
        .map_err(|_| "Vytvořený dokument se nepodařilo ověřit.".to_string())?;
    financial_documents::record_member_document(
        &mut open_write(&path)?,
        &user,
        &data,
        &kind,
        &bytes,
    )?;
    Ok(Some(destination.to_string_lossy().into_owned()))
}

#[tauri::command]
fn get_system_diagnostics(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SystemDiagnostics, String> {
    require_admin(&state)?;
    let path = working_database_path(&app)?;
    let year = ensure_current_insurance_year(&path)?;
    let connection = open_write(&path)?;
    let count = |table: &str| {
        connection
            .query_row(&format!("SELECT COUNT(*) FROM \"{table}\""), [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap_or(0)
    };
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap_or_else(|_| "error".into());
    let backups = path
        .parent()
        .map(|p| p.join("backups"))
        .and_then(|p| fs::read_dir(p).ok())
        .map(|x| x.filter_map(Result::ok).count() as i64)
        .unwrap_or(0);
    let claims = connection
        .query_row(
            r#"SELECT
              (SELECT COUNT(*) FROM "PojistneUdalosti") +
              (SELECT COUNT(*) FROM "MigracePojistnychUdalostiAccess" WHERE "StavVazby"<>'VYRESENA')"#,
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(SystemDiagnostics {
        database_path: path.to_string_lossy().into_owned(),
        integrity,
        application_version: env!("CARGO_PKG_VERSION").into(),
        database_schema_version: database_backup::current_schema(&path)?,
        active_year: year,
        members: count("Seznam"),
        claims,
        receipts: count("DokladyOUhrade"),
        invoices: count("VydaneFaktury"),
        backups,
    })
}

#[tauri::command]
fn choose_member_import(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<member_import::ImportPreview>, String> {
    require_admin(&state)?;
    let Some(source) = rfd::FileDialog::new()
        .set_title("Vybrat CSV pro import členů")
        .add_filter("CSV UTF-8", &["csv"])
        .pick_file()
    else {
        return Ok(None);
    };
    let path = working_database_path(&app)?;
    Ok(Some(member_import::preview(
        &open_read_only(&path)?,
        &source,
    )?))
}

#[tauri::command]
fn execute_member_import(
    app: AppHandle,
    state: State<'_, AppState>,
    source: String,
) -> Result<member_import::ImportResult, String> {
    let user = require_admin(&state)?;
    let source = PathBuf::from(source);
    if !source.is_file()
        || source
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_lowercase()
            != "csv"
    {
        return Err("Importní soubor není platný.".into());
    }
    let path = working_database_path(&app)?;
    let mut connection = open_write(&path)?;
    ensure_backup(&connection, &path)?;
    member_import::execute(&mut connection, &source, &user)
}

#[tauri::command]
fn list_member_claims(
    app: AppHandle,
    state: State<'_, AppState>,
    row_id: i64,
) -> Result<Vec<claims::Claim>, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    let identifier = connection
        .query_row(
            r#"SELECT "Identifikátor" FROM "Seznam" WHERE rowid=?1"#,
            [row_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|_| "Pojistné události člena se nepodařilo načíst.".to_string())?;
    claims::list_for_member(&connection, identifier)
        .map_err(|_| "Pojistné události člena se nepodařilo načíst.".to_string())
}

#[tauri::command]
fn list_claims(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<claims::ClaimOverview>, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    claims::list_all(&connection).map_err(|_| "Pojistné události se nepodařilo načíst.".to_string())
}

#[tauri::command]
fn export_claim_document(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<String>, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    let Some(destination) = rfd::FileDialog::new()
        .set_title("Uložit hlášení pojistné události")
        .set_file_name(&format!("pojistna-udalost-{id}.pdf"))
        .add_filter("Dokument PDF", &["pdf"])
        .save_file()
    else {
        return Ok(None);
    };
    claims::export_pdf(&connection, id, &destination)?;
    Ok(Some(destination.to_string_lossy().into_owned()))
}

#[tauri::command]
fn get_member_audit_history(
    app: AppHandle,
    state: State<'_, AppState>,
    row_id: i64,
) -> Result<Vec<AuditEntry>, String> {
    authenticated_user(&state)?;
    let path = working_database_path(&app)?;
    let active_year = ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    let member = current_member_record(&connection, row_id, active_year)
        .map_err(|_| "Historii člena se nepodařilo načíst.".to_string())?;
    let identifier = member.identifier.unwrap_or_default();
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='AuditLog')",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "Historii člena se nepodařilo načíst.".to_string())?;
    let mut history = Vec::new();
    if exists {
        let mut statement = connection
            .prepare(
                r#"SELECT "DatumČas", "Uživatel", "Operace", "Výsledek"
               FROM "AuditLog"
               WHERE "IdentifikátorPojištěnce" = ?1
               ORDER BY "DatumČas" DESC, "Id" DESC"#,
            )
            .map_err(|_| "Historii člena se nepodařilo načíst.".to_string())?;
        history = statement
            .query_map([&identifier], |row| {
                Ok(AuditEntry {
                    occurred_at: row.get(0)?,
                    user: row.get(1)?,
                    operation: row.get(2)?,
                    result: row.get(3)?,
                })
            })
            .map_err(|_| "Historii člena se nepodařilo načíst.".to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Historii člena se nepodařilo načíst.".to_string())?;
    }
    let payment_audit_exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='AuditPlateb')",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "Historii člena se nepodařilo načíst.".to_string())?;
    if payment_audit_exists {
        let mut statement = connection.prepare(
            r#"SELECT "DatumCas", "Uzivatel", "Operace" || ':' || "Castka" || ':' || "PojistnyRok", 'OK'
               FROM "AuditPlateb" WHERE "IdentifikatorClena"=?1 ORDER BY "DatumCas" DESC, "Id" DESC"#,
        ).map_err(|_| "Historii člena se nepodařilo načíst.".to_string())?;
        let entries = statement
            .query_map([&identifier], |row| {
                Ok(AuditEntry {
                    occurred_at: row.get(0)?,
                    user: row.get(1)?,
                    operation: row.get(2)?,
                    result: row.get(3)?,
                })
            })
            .map_err(|_| "Historii člena se nepodařilo načíst.".to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Historii člena se nepodařilo načíst.".to_string())?;
        history.extend(entries);
    }
    history.sort_by(|left, right| right.occurred_at.cmp(&left.occurred_at));
    Ok(history)
}

#[tauri::command]
fn create_claim(
    app: AppHandle,
    state: State<'_, AppState>,
    claim: claims::NewClaim,
) -> Result<i64, String> {
    let user = require_admin(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    let occurred_on = claim.occurred_on.as_deref().unwrap_or_default();
    let (member, claim_year) =
        claim_member_record(&connection, claim.insurance_row_id, occurred_on)?;
    let identifier = member
        .identifier
        .as_deref()
        .unwrap_or_default()
        .parse::<i64>()
        .map_err(|_| "Vybraný člen nemá platný interní identifikátor.".to_string())?;
    drop(connection);
    let mut claim = claim;
    claim.insurance_row_id = member.row_id;
    claims::create(&path, identifier, claim_year, claim, &user)
}

#[tauri::command]
fn update_claim(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
    claim: claims::NewClaim,
) -> Result<(), String> {
    let user = require_admin(&state)?;
    let path = working_database_path(&app)?;
    ensure_current_insurance_year(&path)?;
    let connection = open_read_only(&path)?;
    let occurred_on = claim.occurred_on.as_deref().unwrap_or_default();
    let (member, claim_year) =
        claim_member_record(&connection, claim.insurance_row_id, occurred_on)?;
    let identifier = member
        .identifier
        .as_deref()
        .unwrap_or_default()
        .parse::<i64>()
        .map_err(|_| "Vybraný člen nemá platný interní identifikátor.".to_string())?;
    drop(connection);
    let mut claim = claim;
    claim.insurance_row_id = member.row_id;
    claims::update(&path, id, identifier, claim_year, claim, &user)
}

#[tauri::command]
fn quit_application(app: AppHandle) {
    app.exit(0);
}

fn managed_backup_directory(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|_| "Správu záloh se nepodařilo otevřít.".to_string())?
        .join("backups");
    fs::create_dir_all(&directory)
        .map_err(|_| "Správu záloh se nepodařilo otevřít.".to_string())?;
    Ok(directory)
}

#[tauri::command]
fn create_database_backup(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<database_backup::BackupInfo>, String> {
    authenticated_user(&state)?;
    let _maintenance = state
        .database_maintenance
        .lock()
        .map_err(|_| "Zálohu se nepodařilo vytvořit.".to_string())?;
    let file_name = database_backup::default_file_name("Federace_Backup");
    let Some(destination) = rfd::FileDialog::new()
        .set_title("Vytvořit zálohu databáze")
        .set_file_name(&file_name)
        .add_filter("Záloha Federace", &["fvcbackup"])
        .save_file()
    else {
        return Ok(None);
    };
    let database_path = working_database_path(&app)?;
    let backup = database_backup::create(&database_path, &destination, false)?;
    let directory = managed_backup_directory(&app)?;
    database_backup::remember(
        &directory,
        &backup,
        database_backup::current_schema(&database_path)?,
    )?;
    Ok(Some(backup))
}

#[tauri::command]
fn select_database_backup(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<database_backup::BackupInfo>, String> {
    authenticated_user(&state)?;
    let Some(path) = rfd::FileDialog::new()
        .set_title("Obnovit databázi ze zálohy")
        .add_filter("Záloha Federace", &["fvcbackup"])
        .pick_file()
    else {
        return Ok(None);
    };
    let database_path = working_database_path(&app)?;
    database_backup::inspect(&path, database_backup::current_schema(&database_path)?).map(Some)
}

#[tauri::command]
fn restore_database_backup(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<database_backup::RestoreResult, String> {
    authenticated_user(&state)?;
    let _maintenance = state
        .database_maintenance
        .lock()
        .map_err(|_| "Obnovu se nepodařilo zahájit.".to_string())?;
    let database_path = working_database_path(&app)?;
    let directory = managed_backup_directory(&app)?;
    let result = database_backup::restore(&database_path, Path::new(&path), &directory)?;
    database_backup::remember(
        &directory,
        &result.emergency_backup,
        database_backup::current_schema(&database_path)?,
    )?;
    Ok(result)
}

#[tauri::command]
fn list_database_backups(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<database_backup::BackupInfo>, String> {
    authenticated_user(&state)?;
    let database_path = working_database_path(&app)?;
    Ok(database_backup::list(
        &managed_backup_directory(&app)?,
        database_backup::current_schema(&database_path)?,
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState::default())
        .setup(|app| {
            let state = app.state::<AppState>();
            let status = match working_database_path_with_state(app.handle()) {
                Ok((path, newly_created)) => {
                    let backup_directory = path.parent().unwrap_or_else(|| Path::new(".")).join("backups").join("migrations");
                    match migrations::migrate(&path, &backup_directory, newly_created) {
                        Ok(outcome) => {
                            if let Some(path) = &outcome.backup_path {
                                println!("Databázová migrace: ověřená záloha {}", path.display());
                            }
                            StartupStatus {
                                ready: true,
                                application_version: env!("CARGO_PKG_VERSION").into(),
                                database_schema_version: Some(outcome.schema_version),
                                supported_schema_version: migrations::DB_SCHEMA_VERSION,
                                database_path: Some(path.display().to_string()),
                                smoke_mode: cfg!(feature = "smoke-test"),
                                failure: None,
                            }
                        },
                        Err(failure) => StartupStatus {
                            ready: false,
                            application_version: env!("CARGO_PKG_VERSION").into(),
                            database_schema_version: Some(failure.source_version),
                            supported_schema_version: migrations::DB_SCHEMA_VERSION,
                            database_path: Some(path.display().to_string()),
                            smoke_mode: cfg!(feature = "smoke-test"),
                            failure: Some(failure),
                        },
                    }
                }
                Err(cause) => StartupStatus {
                    ready: false,
                    application_version: env!("CARGO_PKG_VERSION").into(),
                    database_schema_version: None,
                    supported_schema_version: migrations::DB_SCHEMA_VERSION,
                    database_path: None,
                    smoke_mode: cfg!(feature = "smoke-test"),
                    failure: Some(migrations::MigrationFailure {
                        source_version: -1,
                        target_version: migrations::DB_SCHEMA_VERSION,
                        failed_migration: "příprava pracovní databáze".into(),
                        cause,
                        backup_path: None,
                    }),
                },
            };
            *state.startup.lock().expect("stav databáze") = status;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_startup_status,
            get_auth_status,
            initialize_admin,
            login,
            logout,
            calculate_tariff,
            get_form_options,
            get_registration_number_options,
            save_insured,
            get_application_options,
            create_blank_application_for_printing,
            complete_application,
            export_application_snapshot,
            list_members,
            get_member,
            get_current_member,
            get_member_history,
            update_current_member,
            deactivate_current_member,
            list_archive_years,
            list_archive_members,
            list_legacy_table,
            get_dashboard,
            get_insurer_overview,
            list_tariff_rates,
            save_tariff_rate,
            get_payment_settings,
            save_payment_settings,
            get_email_settings,
            save_email_settings,
            get_receipt_settings,
            save_receipt_settings,
            prepare_payment_order,
            list_member_payments,
            save_member_payment,
            list_payment_organizations,
            list_organization_payment_members,
            save_organization_payment,
            list_organization_payments,
            get_organization_payment,
            delete_member_payment,
            list_receipts,
            create_receipt,
            list_certificate_organizations,
            create_certificate_batch,
            export_certificate_batch,
            get_payment_document_basis,
            export_receipt_pdf,
            open_receipt_pdf,
            send_receipt_email,
            generate_payment_order_pdf,
            audit_payment_order_print,
            open_generated_pdf,
            print_generated_pdf,
            preview_operational_report,
            list_legacy_report_history,
            export_operational_report,
            list_invoices,
            create_invoice,
            export_payment_batch,
            export_member_document,
            get_system_diagnostics,
            choose_member_import,
            execute_member_import,
            list_member_claims,
            list_claims,
            export_claim_document,
            get_member_audit_history,
            create_claim,
            resolve_claim_insurance,
            update_claim,
            create_database_backup,
            select_database_backup,
            restore_database_backup,
            list_database_backups,
            quit_application
        ])
        .run(tauri::generate_context!())
        .expect("aplikaci se nepodařilo spustit");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn production_database_selection_ignores_smoke_override() {
        let standard = PathBuf::from(r"C:\Users\tester\AppData\Roaming\federace\dd.sqlite");
        let selected = resolve_database_path(
            false,
            Some(std::ffi::OsString::from(r"D:\smoke\dd.sqlite")),
            &standard,
            &[],
        )
        .unwrap();
        assert_eq!(selected, standard);
    }

    #[test]
    fn smoke_database_selection_fails_closed_without_explicit_path() {
        let error = resolve_database_path(true, None, Path::new("standard.sqlite"), &[])
            .unwrap_err();
        assert!(error.contains("FEDERACE_DB_PATH"));
    }

    #[test]
    fn smoke_database_selection_rejects_standard_and_appdata_paths() {
        let temp = tempfile::tempdir().unwrap();
        let appdata = temp.path().join("AppData");
        let standard = appdata.join("Federace").join(DATABASE_FILE);
        fs::create_dir_all(standard.parent().unwrap()).unwrap();
        fs::write(&standard, b"test").unwrap();

        assert!(resolve_database_path(
            true,
            Some(standard.clone().into_os_string()),
            &standard,
            std::slice::from_ref(&appdata),
        )
        .is_err());
    }

    #[test]
    fn smoke_database_selection_uses_only_explicit_external_copy() {
        let temp = tempfile::tempdir().unwrap();
        let appdata = temp.path().join("AppData");
        let standard = appdata.join("Federace").join(DATABASE_FILE);
        let smoke = temp.path().join("smoke-data").join("dd-smoke.sqlite");
        fs::create_dir_all(smoke.parent().unwrap()).unwrap();
        fs::write(&smoke, b"test").unwrap();

        let selected = resolve_database_path(
            true,
            Some(smoke.clone().into_os_string()),
            &standard,
            &[appdata],
        )
        .unwrap();
        assert_eq!(selected, fs::canonicalize(smoke).unwrap());
        assert!(!standard.exists());
    }

    fn create_synthetic_database(path: &Path) {
        let connection = Connection::open(path).unwrap();
        register_insurance_year(&connection).unwrap();
        connection
            .execute_batch(
                r#"CREATE TABLE "Seznam" (
                    "Identifikátor" INTEGER NOT NULL,
                    "PojištěníOd" TEXT,
                    "PojištěníDo" TEXT,
                    "RočPojistné" INTEGER,
                    "PojistnáČástka" INTEGER,
                    "PojistNespotř" INTEGER,
                    "Kategorie" TEXT,
                    "Ztráta" INTEGER NOT NULL DEFAULT 0,
                    "KódOC" TEXT,
                    "EvČíslo" INTEGER,
                    "Titul" TEXT,
                    "Příjmení" TEXT,
                    "Jméno" TEXT,
                    "RodnéČíslo" TEXT,
                    "Město" TEXT,
                    "Adresa" TEXT,
                    "PSČ" TEXT,
                    "Stát" TEXT,
                    "Poznámka" TEXT,
                    "OdbPříslušnost" TEXT,
                    "ZO" TEXT,
                    "Ukončení" TEXT,
                    "SkutÚhrada" INTEGER,
                    "Doklad" INTEGER,
                    "e-mail" TEXT,
                    "Tisk" INTEGER NOT NULL DEFAULT 0,
                    "DatumTisku" TEXT
                );
                CREATE TABLE "Editace" (
                    "PojištěníOd" TEXT, "PojištěníDo" TEXT, "RočPojistné" INTEGER,
                    "Kategorie" TEXT, "Ztráta" INTEGER NOT NULL DEFAULT 0,
                    "PojistnáČástka" INTEGER, "KódOC" TEXT, "EvČíslo" INTEGER,
                    "Titul" TEXT, "Příjmení" TEXT, "Jméno" TEXT, "RodnéČíslo" TEXT,
                    "Město" TEXT, "Adresa" TEXT, "PSČ" TEXT, "Stát" TEXT,
                    "Poznámka" TEXT, "OdbPříslušnost" TEXT, "ZO" TEXT,
                    "Ukončení" TEXT, "SkutÚhrada" INTEGER, "E-mail" TEXT
                );
                CREATE TABLE "Kategorie" (
                    "ID" INTEGER NOT NULL, "Kategorie" TEXT, "Ztráta" INTEGER,
                    "Roč_Částka" INTEGER, "Pojistné" INTEGER, "Období" INTEGER
                );"#,
            )
            .unwrap();
        payments::ensure_order_schema(&connection).unwrap();
        connection.execute_batch(
            r#"CREATE INDEX "idx_test_Seznam_Identifikator" ON "Seznam"("Identifikátor");
               CREATE INDEX "idx_test_Seznam_KodOC" ON "Seznam"("KódOC");"#,
        ).unwrap();
        for index in 1..=60_i64 {
            let personal_id = format!("TEST-{index:04}");
            for year in [2024_i64, 2026_i64] {
                let identifier = year * 1_000 + index;
                let prescribed = 495_i64;
                let paid = if index % 2 == 0 { prescribed } else { 0 };
                connection
                    .execute(
                        r#"INSERT INTO "Seznam" (
                            "Identifikátor", "PojištěníOd", "PojištěníDo", "RočPojistné",
                            "PojistnáČástka", "Kategorie", "Ztráta", "KódOC", "EvČíslo",
                            "Titul", "Příjmení", "Jméno", "RodnéČíslo", "Město", "Adresa",
                            "PSČ", "Stát", "Poznámka", "OdbPříslušnost", "ZO", "Ukončení",
                            "SkutÚhrada", "Doklad", "e-mail", "Tisk"
                        ) VALUES (
                            ?1, ?2, ?3, 200000, ?4, 'B', 0, '1', ?5,
                            NULL, ?6, 'Člen', ?7, 'Testovací město', 'Testovací adresa',
                            '00000', 'CZ', NULL, 'FVČ', 'TEST', NULL, ?8, 0, NULL, 0
                        )"#,
                        params![
                            identifier,
                            format!("{year}-01-01 00:00:00"),
                            format!("{year}-12-31 00:00:00"),
                            prescribed,
                            index,
                            format!("Testovací{index:03}"),
                            personal_id,
                            paid,
                        ],
                    )
                    .unwrap();
            }
        }
        drop(connection);
        migrations::migrate(
            path,
            &path.parent().unwrap().join("migration-backups"),
            true,
        )
        .unwrap();
    }

    fn synthetic_database() -> (tempfile::TempDir, PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join(DATABASE_FILE);
        create_synthetic_database(&database);
        (directory, database)
    }

    fn registration_number_fixture() -> (tempfile::TempDir, PathBuf) {
        let (directory, database) = synthetic_database();
        let connection = open_write(&database).unwrap();
        connection
            .execute_batch(
                r#"
                DELETE FROM "Seznam";
                INSERT INTO "Seznam" (
                    "Identifikátor", "PojištěníOd", "PojištěníDo", "RočPojistné",
                    "PojistnáČástka", "Kategorie", "Ztráta", "KódOC", "EvČíslo",
                    "Příjmení", "Jméno", "RodnéČíslo", "OdbPříslušnost", "ZO",
                    "SkutÚhrada", "Doklad", "Tisk"
                ) VALUES
                    (2026001, '2026-01-01', '2026-12-31', 495, 200000, 'B', 0, '1', 1,
                     'První', 'Aktuální', 'CURRENT-0001', 'FVČ', 'TEST', 0, 0, 0),
                    (20261344, '2026-01-01', '2026-12-31', 495, 200000, 'B', 0, '1', 1344,
                     'Poslední', 'Aktuální', 'CURRENT-1344', 'FVČ', 'TEST', 0, 0, 0);
                CREATE TABLE "2009" (
                    "EvČíslo" INTEGER,
                    "Příjmení" TEXT,
                    "Jméno" TEXT,
                    "RodnéČíslo" TEXT,
                    "Poznámka" TEXT
                );
                INSERT INTO "2009" VALUES
                    (943, 'Historický', 'Držitel', 'HISTORY-0943', 'Nesmí být změněno ani připojeno');
                "#,
            )
            .unwrap();
        (directory, database)
    }

    #[test]
    fn first_run_creates_argon2_admin_without_default_password() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("auth.sqlite");
        let created = initialize_admin_at(&database, "bezpecne-heslo-2026").unwrap();
        assert_eq!(created.role, "Správce");
        assert!(verify_login_at(&database, "bezpecne-heslo-2026").is_ok());
        assert!(verify_login_at(&database, "nespravne-heslo").is_err());
        let connection = Connection::open(database).unwrap();
        let stored: String = connection
            .query_row(
                r#"SELECT "PasswordHash" FROM "AppUsers" LIMIT 1"#,
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(stored.starts_with("$argon2"));
        assert!(!stored.contains("bezpecne-heslo-2026"));
    }

    #[test]
    fn first_run_rejects_short_password_and_second_admin() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("auth.sqlite");
        assert!(initialize_admin_at(&database, "kratke").is_err());
        initialize_admin_at(&database, "dostatecne-dlouhe-heslo").unwrap();
        assert!(initialize_admin_at(&database, "jine-dostatecne-heslo").is_err());
    }

    #[test]
    fn access_month_formula_is_inclusive_and_ignores_year() {
        let start = NaiveDate::from_ymd_opt(2026, 3, 10);
        let end = NaiveDate::from_ymd_opt(2026, 5, 20);
        assert_eq!(access_month_count(start, end), 3);
    }

    #[test]
    fn historical_claim_uses_insurance_record_from_occurrence_year() {
        let (_directory, database) = synthetic_database();
        let connection = open_write(&database).unwrap();
        ensure_member_contact_schema(&connection).unwrap();
        let current_row:i64=connection.query_row(r#"SELECT rowid FROM "Seznam" WHERE "RodnéČíslo"='TEST-0001' AND pojisteni_rok("PojištěníOd")=2026"#,[],|row|row.get(0)).unwrap();
        let (historical, year) =
            claim_member_record(&connection, current_row, "2024-08-15").unwrap();
        assert_eq!(year, 2024);
        assert_eq!(
            insurance_year(historical.insurance_from.as_deref().unwrap()),
            Some(2024)
        );
        assert_eq!(historical.premium.as_deref(), Some("495"));
        assert_ne!(historical.row_id, current_row);
        assert!(claim_member_record(&connection, current_row, "2023-08-15")
            .unwrap_err()
            .contains("nejsou dostupné"));
    }

    #[test]
    fn access_mapping_requires_matching_code() {
        let input = NewInsured {
            title: None,
            last_name: None,
            first_name: None,
            personal_id: None,
            organization: None,
            affiliation: "FVČ".into(),
            city: None,
            address: None,
            postal_code: None,
            country: None,
            note: None,
            insurance_from: None,
            insurance_to: None,
            annual_amount: 200_000,
            category: "B".into(),
            loss: false,
            actual_payment: None,
            code: 2,
            registration_year: 2026,
            email: None,
            phone: None,
            registration_number: 51,
        };
        assert_eq!(validate_input(&input).unwrap_err(), "Zkontrolujte KódOC.");
    }

    #[test]
    fn transaction_creates_backup_record_and_audit() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "pojisteni-sprint03-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        let database = directory.join(DATABASE_FILE);
        create_synthetic_database(&database);
        let registration_number=registration_number_options(&open_read_only(&database).unwrap()).unwrap().next_number;

        let input = NewInsured {
            title: Some("Ing.".into()),
            last_name: Some("Testovací".into()),
            first_name: Some("Klient".into()),
            personal_id: None,
            organization: Some("TEST".into()),
            affiliation: "FVČ".into(),
            city: None,
            address: None,
            postal_code: None,
            country: None,
            note: None,
            insurance_from: Some("2026-01-01".into()),
            insurance_to: Some("2026-12-31".into()),
            annual_amount: 200_000,
            category: "B".into(),
            loss: false,
            actual_payment: Some(0),
            code: 1,
            registration_year: 2026,
            email: None,
            phone: None,
            registration_number,
        };

        let result = save_to_database(&database, "test-user", input).unwrap();
        let connection = Connection::open(&database).unwrap();
        let saved: (String, String, String) = connection
            .query_row(
                r#"SELECT "OdbPříslušnost", "KódOC", "ZO"
                   FROM "Seznam" WHERE "Identifikátor" = ?1"#,
                [result.identifier],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(saved, ("FVČ".into(), "1".into(), "TEST".into()));
        assert_eq!(
            connection
                .query_row(r#"SELECT COUNT(*) FROM "Editace""#, [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            connection
                .query_row(
                    r#"SELECT "Výsledek" FROM "AuditLog"
                       WHERE "IdentifikátorPojištěnce" = ?1"#,
                    [result.identifier.to_string()],
                    |row| row.get::<_, String>(0),
                )
                .unwrap(),
            "OK"
        );
        assert!(directory
            .join("backups")
            .join("dd-before-first-write.sqlite")
            .is_file());
        drop(connection);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn member_paging_is_complete_stable_and_read_only() {
        let (_directory, database) = synthetic_database();
        let connection = open_read_only(&database).unwrap();
        let total: i64 = connection
            .query_row(r#"SELECT COUNT(*) FROM "Seznam""#, [], |row| row.get(0))
            .unwrap();
        let first = member_page(&connection, None, 1, 50).unwrap();
        let second = member_page(&connection, None, 2, 50).unwrap();
        assert_eq!(first.total, total);
        assert_eq!(first.members.len(), 50);
        assert_eq!(second.members.len(), 50);
        assert!(first.members.iter().all(|member| second
            .members
            .iter()
            .all(|next| next.row_id != member.row_id)));
    }

    #[test]
    fn member_search_returns_matching_existing_member() {
        let (_directory, database) = synthetic_database();
        let connection = open_read_only(&database).unwrap();
        let page = member_page(&connection, Some("Testovací005".into()), 1, 50).unwrap();
        assert!(page.total > 0);
        assert!(page
            .members
            .iter()
            .any(|member| member.insured.contains("Testovací005")));
    }

    #[test]
    fn insurance_year_parses_verified_database_format_safely() {
        assert_eq!(insurance_year("2026-01-31 00:00:00"), Some(2026));
        assert_eq!(insurance_year("31.01.2026"), Some(2026));
        assert_eq!(insurance_year("neplatné datum"), None);
        assert_eq!(insurance_year(""), None);
    }

    #[test]
    fn archive_year_counts_cover_all_dated_records() {
        let directory = std::env::temp_dir().join(format!(
            "pojisteni-archive-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&directory).unwrap();
        let database = directory.join(DATABASE_FILE);
        create_synthetic_database(&database);
        let mut writable = open_write(&database).unwrap();
        CurrentInsuranceYear::initialize(&mut writable, &database, 2026).unwrap();
        drop(writable);
        let connection = open_read_only(&database).unwrap();
        let years = archive_years(&connection).unwrap();
        assert!(years.windows(2).all(|pair| pair[0].year > pair[1].year));
        let dated: i64 = connection
            .query_row(
                r#"SELECT COUNT(*) FROM "Seznam"
                   INNER JOIN "PojistnaObdobi" period
                     ON period."Rok" = pojisteni_rok("PojištěníOd")
                    AND period."Stav" = 'UZAVRENO'"#,
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            years.iter().map(|year| year.record_count).sum::<i64>(),
            dated
        );
        drop(connection);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn archive_paging_and_search_stay_inside_selected_year() {
        let (_directory, database) = synthetic_database();
        let connection = open_read_only(&database).unwrap();
        let first =
            archive_member_page(&connection, 2024, None, 1, 25, MemberFilters::default()).unwrap();
        let second =
            archive_member_page(&connection, 2024, None, 2, 25, MemberFilters::default()).unwrap();
        assert_eq!(first.members.len(), 25);
        assert!(first
            .members
            .iter()
            .chain(second.members.iter())
            .all(|member| member.insurance_from.as_deref().and_then(insurance_year) == Some(2024)));
        assert!(first.members.iter().all(|member| second
            .members
            .iter()
            .all(|other| other.row_id != member.row_id)));

        let needle = first.members[0].registration_number.clone().unwrap();
        let found = archive_member_page(
            &connection,
            2024,
            Some(needle),
            1,
            50,
            MemberFilters::default(),
        )
        .unwrap();
        assert!(found.total > 0);
        assert!(found.members.iter().all(|member| member
            .insurance_from
            .as_deref()
            .and_then(insurance_year)
            == Some(2024)));
    }

    #[test]
    fn payment_status_filter_compares_paid_and_prescribed_amounts() {
        let (_directory, database) = synthetic_database();
        let connection = open_read_only(&database).unwrap();
        let unpaid = archive_member_page(
            &connection,
            2026,
            None,
            1,
            200,
            MemberFilters {
                payment_status: Some("neuhrazeno".into()),
                ..MemberFilters::default()
            },
        )
        .unwrap();
        assert!(unpaid.total > 0);
        assert!(unpaid.members.iter().all(|member| {
            member.actual_payment.as_deref().unwrap_or("0")
                != member.premium.as_deref().unwrap_or("0")
        }));
        let paid = archive_member_page(
            &connection,
            2026,
            None,
            1,
            200,
            MemberFilters {
                payment_status: Some("uhrazeno".into()),
                ..MemberFilters::default()
            },
        )
        .unwrap();
        assert!(paid.total > 0);
        assert!(paid.members.iter().all(|member| {
            member.actual_payment.as_deref().unwrap_or("0")
                == member.premium.as_deref().unwrap_or("0")
        }));
    }

    #[test]
    fn overdue_filter_and_dashboard_use_unpaid_premium_not_insurance_limit() {
        let (_directory, database) = synthetic_database();
        let connection = Connection::open(&database).unwrap();
        register_insurance_year(&connection).unwrap();
        let (row_id, identifier): (i64, String) = connection
            .query_row(
                r#"SELECT rowid, CAST("Identifikátor" AS TEXT) FROM "Seznam"
               WHERE pojisteni_rok("PojištěníOd")=2026 LIMIT 1"#,
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        connection.execute(
            r#"UPDATE "Seznam" SET "RočPojistné"=320000,"PojistnáČástka"=787,"SkutÚhrada"=197 WHERE rowid=?1"#,
            [row_id],
        ).unwrap();
        payments::record_order(
            &connection,
            &identifier,
            row_id,
            2026,
            "2020-01-01",
            200_000,
            "2019-12-01",
        )
        .unwrap();
        assert_eq!(
            overdue_summary(&connection, 2026).unwrap(),
            (1, 590, Some("2020-01-01".into()))
        );
        let overdue = archive_member_page(
            &connection,
            2026,
            None,
            1,
            200,
            MemberFilters {
                overdue: Some("po_splatnosti".into()),
                ..MemberFilters::default()
            },
        )
        .unwrap();
        assert_eq!(overdue.total, 1);
        connection
            .execute(
                r#"UPDATE "Seznam" SET "SkutÚhrada"="PojistnáČástka" WHERE rowid=?1"#,
                [row_id],
            )
            .unwrap();
        let paid = archive_member_page(
            &connection,
            2026,
            None,
            1,
            200,
            MemberFilters {
                overdue: Some("po_splatnosti".into()),
                ..MemberFilters::default()
            },
        )
        .unwrap();
        assert_eq!(paid.total, 0);
        assert_eq!(overdue_summary(&connection, 2026).unwrap(), (0, 0, None));
    }

    #[test]
    fn yearly_roll_forward_is_transactional_backed_up_and_idempotent() {
        let directory = std::env::temp_dir().join(format!(
            "pojisteni-period-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&directory).unwrap();
        let database = directory.join(DATABASE_FILE);
        create_synthetic_database(&database);
        let mut connection = open_write(&database).unwrap();
        tariffs::ensure_schema(&connection).unwrap();
        assert_eq!(
            CurrentInsuranceYear::initialize(&mut connection, &database, 2026).unwrap(),
            2026
        );
        let eligible: i64 = connection
            .query_row(
                r#"SELECT COUNT(*) FROM "Seznam"
                   WHERE pojisteni_rok("PojištěníOd") = 2026
                     AND NULLIF(TRIM("Ukončení"), '') IS NULL
                     AND date("PojištěníDo") >= date('2026-12-31')"#,
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            CurrentInsuranceYear::initialize(&mut connection, &database, 2027).unwrap(),
            2027
        );
        let created: (i64, i64, i64) = connection
            .query_row(
                r#"SELECT COUNT(*),
                          SUM(CASE WHEN "SkutÚhrada" = 0 THEN 1 ELSE 0 END),
                          SUM(CASE WHEN "Ukončení" IS NULL THEN 1 ELSE 0 END)
                   FROM "Seznam" WHERE pojisteni_rok("PojištěníOd") = 2027"#,
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(created, (eligible, eligible, eligible));
        let tariff_mismatches: i64 = connection
            .query_row(
                r#"SELECT COUNT(*) FROM "Seznam" member
                   WHERE pojisteni_rok(member."PojištěníOd") = 2027
                     AND member."PojistnáČástka" <> (
                         SELECT rate."rocni_pojistne" FROM "sazby_pojistneho" rate
                         WHERE rate."pojistna_castka" = member."RočPojistné"
                           AND rate."kategorie" = member."Kategorie"
                           AND rate."pojisteni_ztraty" =
                               CASE WHEN member."Ztráta" <> 0 THEN 1 ELSE 0 END
                           AND rate."aktivni" = 1
                           AND date(rate."platnost_od") <= date('2027-01-01')
                           AND (rate."platnost_do" IS NULL
                                OR date(rate."platnost_do") >= date('2027-01-01'))
                         ORDER BY date(rate."platnost_od") DESC LIMIT 1
                     )"#,
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tariff_mismatches, 0);
        CurrentInsuranceYear::initialize(&mut connection, &database, 2027).unwrap();
        let after_repeat: i64 = connection
            .query_row(
                r#"SELECT COUNT(*) FROM "Seznam"
                   WHERE pojisteni_rok("PojištěníOd") = 2027"#,
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(after_repeat, eligible);
        assert!(directory
            .join("backups")
            .read_dir()
            .unwrap()
            .any(|entry| entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("period-2027")));
        drop(connection);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn every_operational_report_previews_and_exports_csv() {
        let (_directory, database) = synthetic_database();
        let connection = open_write(&database).unwrap();
        claims::ensure_schema(&connection).unwrap();
        for kind in [
            "insurer",
            "oc",
            "zo",
            "claims",
            "starts",
            "terminations",
            "contacts",
            "duplicates",
            "quality",
            "member",
        ] {
            let filter = reports::ReportFilter {
                kind: kind.into(),
                year: 2026,
                organization_code: None,
                organization: None,
                date_from: None,
                date_to: None,
                search: None,
            };
            let report = reports::preview(&connection, &filter)
                .unwrap_or_else(|error| panic!("report {kind}: {error}"));
            assert!(!report.columns.is_empty(), "report {kind} has no columns");
            let csv = reports::csv(&report);
            assert!(
                csv.starts_with(&[0xEF, 0xBB, 0xBF]),
                "report {kind} has no UTF-8 BOM"
            );
        }
    }

    #[test]
    fn controlled_deactivation_ends_only_current_record_and_audits() {
        let (_directory, database) = synthetic_database();
        let connection = open_read_only(&database).unwrap();
        let row_id: i64 = connection.query_row(r#"SELECT rowid FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=2026 AND NULLIF(TRIM("Ukončení"),'') IS NULL LIMIT 1"#,[],|row|row.get(0)).unwrap();
        drop(connection);
        deactivate_current_member_at(&database, "tester", 2026, row_id, "ukončení na žádost")
            .unwrap();
        let connection = open_read_only(&database).unwrap();
        let (termination, note): (String, String) = connection
            .query_row(
                r#"SELECT "Ukončení",COALESCE("Poznámka",'') FROM "Seznam" WHERE rowid=?1"#,
                [row_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert!(!termination.is_empty());
        assert!(note.contains("STORNO:"));
        let audit:i64=connection.query_row(r#"SELECT COUNT(*) FROM "AuditLog" WHERE "Operace"='DEACTIVATE' AND "Výsledek"='OK'"#,[],|row|row.get(0)).unwrap();
        assert_eq!(audit, 1);
    }

    #[test]
    fn all_member_document_pdfs_are_created_and_voucher_is_linked() {
        let (directory, database) = synthetic_database();
        let mut connection = open_write(&database).unwrap();
        financial_documents::ensure_schema(&connection).unwrap();
        let row_id: i64 = connection
            .query_row(
                r#"SELECT rowid FROM "Seznam" WHERE pojisteni_rok("PojištěníOd")=2026 LIMIT 1"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        let data = financial_documents::member_data(&connection, row_id, 2026).unwrap();
        for kind in ["application", "voucher", "envelope", "label"] {
            let path = directory.path().join(format!("{kind}.pdf"));
            financial_documents::member_pdf(&data, kind, &path).unwrap();
            let bytes = fs::read(&path).unwrap();
            assert!(bytes.starts_with(b"%PDF"));
            financial_documents::record_member_document(
                &mut connection,
                "tester",
                &data,
                kind,
                &bytes,
            )
            .unwrap();
        }
        let documents: i64 = connection
            .query_row(r#"SELECT COUNT(*) FROM "ClenskeDoklady""#, [], |r| r.get(0))
            .unwrap();
        assert_eq!(documents, 5);
    }

    #[test]
    fn member_import_previews_inserts_and_audits() {
        let (directory, database) = synthetic_database();
        let source = directory.path().join("import.csv");
        fs::write(&source,"Jméno;Příjmení;Rodné číslo;Evidenční číslo;OC;ZO;Pojištění od;Pojištění do;Pojistná částka;Kategorie;Pojistné;E-mail\nImport;Test;991231/9999;99991;2;Test ZO;2026-01-01;2026-12-31;320000;B;781;import@example.invalid\n").unwrap();
        let mut connection = open_write(&database).unwrap();
        create_audit_table(&connection).unwrap();
        let preview = member_import::preview(&connection, &source).unwrap();
        assert_eq!((preview.valid, preview.errors.len()), (1, 0));
        let result = member_import::execute(&mut connection, &source, "tester").unwrap();
        assert_eq!((result.inserted, result.skipped_duplicates), (1, 0));
        let duplicate = member_import::preview(&connection, &source).unwrap();
        assert_eq!(duplicate.skipped_duplicates, 1);
        let audit: i64 = connection
            .query_row(
                r#"SELECT COUNT(*) FROM "AuditLog" WHERE "Operace"='IMPORT'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(audit, 1);
    }

    #[test]
    fn member_detail_uses_exact_current_record_and_verified_history_identity() {
        let (_directory, database) = synthetic_database();
        let connection = open_read_only(&database).unwrap();
        let row_id: i64 = connection
            .query_row(
                r#"SELECT current.rowid
                   FROM "Seznam" current
                   WHERE pojisteni_rok(current."PojištěníOd") = 2026
                     AND NULLIF(TRIM(current."RodnéČíslo"), '') IS NOT NULL
                     AND EXISTS (
                         SELECT 1 FROM "Seznam" history
                         WHERE TRIM(history."RodnéČíslo") = TRIM(current."RodnéČíslo")
                           AND history.rowid <> current.rowid
                     )
                   LIMIT 1"#,
                [],
                |row| row.get(0),
            )
            .unwrap();
        let current = current_member_record(&connection, row_id, 2026).unwrap();
        assert_eq!(
            current.insurance_from.as_deref().and_then(insurance_year),
            Some(2026)
        );
        assert!(current.insurance_to.is_some());
        let history = member_history_records(&connection, row_id).unwrap();
        assert!(!history.is_empty());
        assert!(history.iter().all(|item| item.row_id != row_id));
        assert!(history
            .iter()
            .all(|item| item.personal_id == current.personal_id));
        assert!(current_member_record(&connection, row_id, 2025).is_err());
    }

    #[test]
    fn tariff_rates_are_date_bound_non_overlapping_and_preserve_history() {
        let directory = std::env::temp_dir().join(format!(
            "pojisteni-tariff-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&directory).unwrap();
        let database = directory.join(DATABASE_FILE);
        create_synthetic_database(&database);
        let connection = open_write(&database).unwrap();
        let history_before: (i64, i64) = connection
            .query_row(
                r#"SELECT COUNT(*), SUM(COALESCE("PojistnáČástka", 0)) FROM "Seznam""#,
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        tariffs::ensure_schema(&connection).unwrap();
        let original = tariffs::list(&connection)
            .unwrap()
            .into_iter()
            .find(|rate| {
                rate.insured_amount == 200_000
                    && rate.category == "B"
                    && !rate.loss_insurance
                    && rate.annual_premium == 495
            })
            .unwrap();
        tariffs::save(
            &connection,
            tariffs::TariffRateInput {
                id: Some(original.id),
                insured_amount: original.insured_amount,
                category: original.category,
                loss_insurance: original.loss_insurance,
                annual_premium: original.annual_premium,
                valid_from: original.valid_from,
                valid_to: Some("2026-12-31".into()),
                active: true,
                note: original.note,
            },
        )
        .unwrap();
        tariffs::save(
            &connection,
            tariffs::TariffRateInput {
                id: None,
                insured_amount: 200_000,
                category: "B".into(),
                loss_insurance: false,
                annual_premium: 520,
                valid_from: "2027-01-01".into(),
                valid_to: None,
                active: true,
                note: Some("Sazba 2027".into()),
            },
        )
        .unwrap();
        assert_eq!(
            tariffs::calculate(
                &connection,
                "B",
                false,
                200_000,
                NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
                12
            )
            .unwrap()
            .unwrap()
            .premium,
            495
        );
        assert_eq!(
            tariffs::calculate(
                &connection,
                "B",
                false,
                200_000,
                NaiveDate::from_ymd_opt(2027, 6, 1).unwrap(),
                12
            )
            .unwrap()
            .unwrap()
            .premium,
            520
        );
        assert!(tariffs::save(
            &connection,
            tariffs::TariffRateInput {
                id: None,
                insured_amount: 200_000,
                category: "B".into(),
                loss_insurance: false,
                annual_premium: 530,
                valid_from: "2027-06-01".into(),
                valid_to: None,
                active: true,
                note: None,
            },
        )
        .is_err());
        let history_after: (i64, i64) = connection
            .query_row(
                r#"SELECT COUNT(*), SUM(COALESCE("PojistnáČástka", 0)) FROM "Seznam""#,
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(history_after, history_before);
        drop(connection);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn member_edit_updates_only_current_record_and_logs_no_personal_data() {
        let directory = std::env::temp_dir().join(format!(
            "pojisteni-member-edit-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&directory).unwrap();
        let database = directory.join(DATABASE_FILE);
        create_synthetic_database(&database);
        let connection = open_read_only(&database).unwrap();
        let row_id: i64 = connection
            .query_row(
                r#"SELECT current.rowid FROM "Seznam" current
                   WHERE pojisteni_rok(current."PojištěníOd") = 2026
                     AND NULLIF(TRIM(current."RodnéČíslo"), '') IS NOT NULL
                     AND EXISTS (
                         SELECT 1 FROM "Seznam" history
                         WHERE history."RodnéČíslo" = current."RodnéČíslo"
                           AND pojisteni_rok(history."PojištěníOd") < 2026
                     )
                   LIMIT 1"#,
                [],
                |row| row.get(0),
            )
            .unwrap();
        let current = current_member_record(&connection, row_id, 2026).unwrap();
        let history_before: Vec<(i64, Option<String>, Option<String>)> = connection
            .prepare(
                r#"SELECT rowid, "Poznámka", "e-mail" FROM "Seznam"
                   WHERE "RodnéČíslo" = ?1 AND pojisteni_rok("PojištěníOd") < 2026
                   ORDER BY rowid"#,
            )
            .unwrap()
            .query_map([current.personal_id.clone().unwrap()], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        drop(connection);
        let private_email = "citlive@example.invalid";
        update_current_member_record(
            &database,
            "test-user",
            2026,
            MemberUpdate {
                row_id,
                title: current.title,
                last_name: current.last_name,
                first_name: current.first_name,
                personal_id: current.personal_id,
                registration_number: current
                    .registration_number
                    .and_then(|value| value.parse().ok()),
                city: current.city,
                address: current.address,
                postal_code: current.postal_code,
                country: current.country,
                organization: current.organization,
                affiliation: current.affiliation.unwrap(),
                code: current.code.unwrap(),
                email: Some(private_email.into()),
                phone: Some("+420 123 456 789".into()),
                note: Some("Aktuální poznámka".into()),
                actual_payment: current.actual_payment.and_then(|value| value.parse().ok()),
                actual_termination: current.actual_termination,
            },
        )
        .unwrap();
        let connection = open_read_only(&database).unwrap();
        let updated = current_member_record(&connection, row_id, 2026).unwrap();
        assert_eq!(updated.email.as_deref(), Some(private_email));
        assert_eq!(updated.phone.as_deref(), Some("+420 123 456 789"));
        let history_after: Vec<(i64, Option<String>, Option<String>)> = connection
            .prepare(
                r#"SELECT rowid, "Poznámka", "e-mail" FROM "Seznam"
                   WHERE "RodnéČíslo" = ?1 AND pojisteni_rok("PojištěníOd") < 2026
                   ORDER BY rowid"#,
            )
            .unwrap()
            .query_map([updated.personal_id.unwrap()], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(history_after, history_before);
        let leaked: i64 = connection
            .query_row(
                r#"SELECT COUNT(*) FROM "AuditLog"
                   WHERE COALESCE("Uživatel", '') LIKE '%' || ?1 || '%'
                      OR COALESCE("IdentifikátorPojištěnce", '') LIKE '%' || ?1 || '%'"#,
                [private_email],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(leaked, 0);
        drop(connection);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn legacy_tables_are_read_only_paginated_and_allowlisted() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                r#"
                CREATE TABLE "2002" ("EvČíslo" TEXT, "Příjmení" TEXT);
                INSERT INTO "2002" VALUES ('1', 'Novák'), ('2', 'Svoboda');
                CREATE TABLE "Faktura" ("VS" TEXT, "Částka" INTEGER);
                INSERT INTO "Faktura" VALUES ('1001', 500), ('1002', 700);
                "#,
            )
            .unwrap();

        let archive = legacy_table_page(&connection, "2002", Some("Nov".into()), 1, 100).unwrap();
        assert_eq!(archive.total, 1);
        assert_eq!(archive.columns, vec!["EvČíslo", "Příjmení"]);
        assert_eq!(archive.rows[0], vec!["1", "Novák"]);

        let invoices = legacy_table_page(&connection, "Faktura", None, 1, 1).unwrap();
        assert_eq!(invoices.total, 2);
        assert_eq!(invoices.rows.len(), 1);
        assert!(legacy_table_page(&connection, "VydaneFaktury", None, 1, 100).is_err());
    }

    #[test]
    fn electronic_application_accepts_every_current_limit_and_insurance_variant() {
        let (_directory, database) = synthetic_database();
        let limits = [140_000, 200_000, 240_000, 280_000, 320_000, 360_000, 400_000, 500_000, 600_000];
        let variants = [("B", false), ("B", true), ("A", false), ("A", true), ("C", false), ("C", true)];
        {
            let connection = open_write(&database).unwrap();
            tariffs::ensure_schema(&connection).unwrap();
            applications::ensure_schema(&connection).unwrap();
            connection.execute(r#"DELETE FROM "sazby_pojistneho""#, []).unwrap();
            for (variant_index, (category, loss)) in variants.iter().enumerate() {
                for (limit_index, limit) in limits.iter().enumerate() {
                    let premium = 700 + variant_index as i64 * 100 + limit_index as i64 * 10;
                    connection.execute(
                        r#"INSERT INTO "sazby_pojistneho"("pojistna_castka","kategorie","pojisteni_ztraty","rocni_pojistne","platnost_od","aktivni") VALUES(?1,?2,?3,?4,'2020-01-01',1)"#,
                        params![limit, category, i64::from(*loss), premium],
                    ).unwrap();
                }
            }
            assert_eq!(tariffs::application_amounts(&connection, NaiveDate::from_ymd_opt(2026, 9, 1).unwrap()).unwrap(), limits);
            for (variant_index, (category, loss)) in variants.iter().enumerate() {
                for (limit_index, limit) in limits.iter().enumerate() {
                    let tariff = tariffs::calculate(&connection, category, *loss, *limit, NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(), 12).unwrap().unwrap();
                    assert_eq!(tariff.premium, 700 + variant_index as i64 * 100 + limit_index as i64 * 10);
                }
            }
        }

        for (index, limit) in limits.iter().enumerate() {
            let (category, loss) = variants[index % variants.len()];
            let input = applications::ApplicationInput {
                first_name: format!("Test{index}"), last_name: "Přihláška".into(),
                personal_id: format!("{:06}/{:04}", 900101 + index, 1000 + index),
                address: "Testovací 1".into(), city: "Praha".into(), postal_code: "110 00".into(),
                email: Some("test@example.cz".into()), category: category.into(), loss,
                annual_amount: *limit, affiliation: "FVČ".into(), organization: "TEST".into(), code: "1".into(),
                registration_number: registration_number_options(&open_read_only(&database).unwrap()).unwrap().next_number,
            };
            let result = complete_application_at(&database, "test-user", input, NaiveDate::from_ymd_opt(2026, 8, 10).unwrap()).unwrap();
            let connection = open_read_only(&database).unwrap();
            let saved: (i64, String, i64, i64) = connection.query_row(
                r#"SELECT "RočPojistné","Kategorie","Ztráta","PojistnáČástka" FROM "Seznam" WHERE "Identifikátor"=?1"#,
                [result.identifier], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            ).unwrap();
            assert_eq!((saved.0, saved.1.as_str(), saved.2 != 0), (*limit, category, loss));
            assert_eq!(saved.3, (result.premium as f64 / 12.0 * 4.0).ceil() as i64);
            let snapshot: (i64, i64, String, String, Vec<u8>) = connection.query_row(
                r#"SELECT "RocniLimit","Pojistne","DatumPrihlasky","PojisteniOd","Pdf" FROM "Prihlasky" WHERE "Id"=?1"#,
                [result.application_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            ).unwrap();
            assert_eq!((snapshot.0, snapshot.1, snapshot.2.as_str(), snapshot.3.as_str()), (*limit, result.premium, "2026-08-10", "2026-09-01"));
            assert!(snapshot.4.starts_with(b"%PDF"));
            assert!(Path::new(&result.pdf_path).is_file());
            let _ = fs::remove_file(&result.pdf_path);
        }

        let missing = applications::ApplicationInput {
            first_name: "Bez".into(), last_name: "Limitu".into(), personal_id: "900101/9999".into(),
            address: "Testovací 1".into(), city: "Praha".into(), postal_code: "110 00".into(), email: None,
            category: "B".into(), loss: false, annual_amount: 0, affiliation: "FVČ".into(), organization: "TEST".into(), code: "1".into(),
            registration_number: 1,
        };
        assert_eq!(applications::validate(&missing).unwrap_err(), "Vyberte právě jeden roční limit.");
        let invalid = applications::ApplicationInput { annual_amount: 500_001, ..missing };
        assert_eq!(complete_application_at(&database, "test-user", invalid, NaiveDate::from_ymd_opt(2026, 8, 10).unwrap()).unwrap_err(), "Pro zadané údaje není platná aktuální sazba pojistného.");
    }

    #[test]
    fn registration_number_allocator_uses_next_and_current_seznam_gaps_safely() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(r#"CREATE TABLE "Seznam"("EvČíslo" INTEGER);INSERT INTO "Seznam" VALUES(1),(2),(4),(5),(8),(0);CREATE TABLE "2009"("EvČíslo" INTEGER);INSERT INTO "2009" VALUES(3),(6),(7);"#).unwrap();
        assert_eq!(registration_number_options(&connection).unwrap(), RegistrationNumberOptions { next_number:9, free_numbers:vec![3,6,7] });
        {
            let transaction=connection.transaction_with_behavior(TransactionBehavior::Immediate).unwrap();
            assert_eq!(allocate_registration_number(&transaction,9).unwrap(),9);
            assert_eq!(allocate_registration_number(&transaction,3).unwrap(),3);
            transaction.execute(r#"INSERT INTO "Seznam" VALUES(3)"#,[]).unwrap();
            assert_eq!(allocate_registration_number(&transaction,3).unwrap_err(),"Evidenční číslo 3 již bylo použito. Vyberte jiné volné číslo.");
            transaction.rollback().unwrap();
        }
        connection.execute(r#"DELETE FROM "Seznam""#,[]).unwrap();
        assert_eq!(registration_number_options(&connection).unwrap(), RegistrationNumberOptions { next_number:1, free_numbers:vec![] });
        for number in 1..=100 { connection.execute(r#"INSERT INTO "Seznam" VALUES(?1)"#,[number]).unwrap(); }
        assert_eq!(registration_number_options(&connection).unwrap(), RegistrationNumberOptions { next_number:101, free_numbers:vec![] });
    }

    #[test]
    fn certificate_batch_keeps_every_matching_organization_member() {
        let connection = Connection::open_in_memory().unwrap();
        register_insurance_year(&connection).unwrap();
        connection.execute_batch(r#"
            CREATE TABLE "Seznam"(
                "Jméno" TEXT,"Příjmení" TEXT,"EvČíslo" INTEGER,"KódOC" TEXT,"ZO" TEXT,
                "PojištěníOd" TEXT,"Ukončení" TEXT,"SkutÚhrada" INTEGER,"PojistnáČástka" INTEGER
            );
            CREATE TABLE "PlatbyClenu"("PojistnyZaznamRowId" INTEGER,"Castka" INTEGER,"DatumPrijeti" TEXT);
            INSERT INTO "Seznam" VALUES
                ('Anna','První',10,'1','TEST ZO','2026-01-01',NULL,700,700),
                ('Boris','Druhý',11,'1','TEST ZO','2026-01-01',NULL,700,700),
                ('Cyril','Neuhrazený',12,'1','TEST ZO','2026-01-01',NULL,200,700),
                ('Dana','Jiná',13,'1','JINÁ ZO','2026-01-01',NULL,700,700);
            INSERT INTO "PlatbyClenu" VALUES(1,700,'2026-02-01'),(2,700,'2026-03-01'),(4,700,'2026-03-01');
        "#).unwrap();
        let selected = certificate_batch_row_ids(&connection, 2026, "", "1", "TEST ZO").unwrap();
        assert_eq!(selected, vec![1, 2]);
        let after_february = certificate_batch_row_ids(&connection, 2026, "2026-02-15", "1", "TEST ZO").unwrap();
        assert_eq!(after_february, vec![2]);
    }

    #[test]
    fn representative_copy_assigns_943_and_1345_without_touching_history() {
        let (_directory, database) = registration_number_fixture();
        {
            let connection=open_write(&database).unwrap();
            ensure_member_contact_schema(&connection).unwrap();
            applications::ensure_schema(&connection).unwrap();
            tariffs::ensure_schema(&connection).unwrap();
            payments::ensure_schema(&connection).unwrap();
            receipts::ensure_schema(&connection).unwrap();
        }
        let before=open_read_only(&database).unwrap();
        let options=registration_number_options(&before).unwrap();
        assert_eq!(options.next_number,1345);
        assert!(options.free_numbers.contains(&943));
        let historical_943: (String, String, String, String) = before
            .query_row(
                r#"SELECT "Příjmení","Jméno","RodnéČíslo","Poznámka" FROM "2009" WHERE CAST("EvČíslo" AS INTEGER)=943"#,
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        drop(before);

        let application=applications::ApplicationInput{
            first_name:"Nový".into(),last_name:"Člen943".into(),personal_id:"991231/9430".into(),address:"Testovací 1".into(),city:"Praha".into(),postal_code:"110 00".into(),email:None,category:"B".into(),loss:false,annual_amount:200_000,affiliation:"FVČ".into(),organization:"TEST".into(),code:"1".into(),registration_number:943,
        };
        let result=complete_application_at(&database,"test-user",application,NaiveDate::from_ymd_opt(2026,8,10).unwrap()).unwrap();
        assert_eq!(result.registration_number,943);
        let connection=open_read_only(&database).unwrap();
        let new_row:i64=connection.query_row(r#"SELECT rowid FROM "Seznam" WHERE "Identifikátor"=?1 AND CAST("EvČíslo" AS INTEGER)=943"#,[result.identifier],|row|row.get(0)).unwrap();
        assert_eq!(
            connection
                .query_row(
                    r#"SELECT "Příjmení","Jméno","RodnéČíslo","Poznámka" FROM "2009" WHERE CAST("EvČíslo" AS INTEGER)=943"#,
                    [],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?)),
                )
                .unwrap(),
            historical_943
        );
        assert_eq!(connection.query_row(r#"SELECT COUNT(*) FROM "PojistneUdalosti" WHERE "PojistnyZaznamRowId"=?1"#,[new_row],|row|row.get::<_,i64>(0)).unwrap(),0);
        assert_eq!(
            connection
                .query_row(
                    r#"SELECT COUNT(*) FROM "Seznam" WHERE CAST("EvČíslo" AS INTEGER)=943"#,
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap(),
            1
        );
        assert_ne!(
            connection
                .query_row(
                    r#"SELECT "RodnéČíslo" FROM "Seznam" WHERE rowid=?1"#,
                    [new_row],
                    |row| row.get::<_, String>(0),
                )
                .unwrap(),
            historical_943.2
        );
        assert!(connection.query_row(r#"SELECT "Pdf" FROM "Prihlasky" WHERE "Id"=?1"#,[result.application_id],|row|row.get::<_,Vec<u8>>(0)).unwrap().starts_with(b"%PDF"));
        drop(connection);
        let duplicate=applications::ApplicationInput{first_name:"Druhý".into(),last_name:"Člen943".into(),personal_id:"991231/9431".into(),address:"Testovací 2".into(),city:"Praha".into(),postal_code:"110 00".into(),email:None,category:"B".into(),loss:false,annual_amount:200_000,affiliation:"FVČ".into(),organization:"TEST".into(),code:"1".into(),registration_number:943};
        assert_eq!(complete_application_at(&database,"test-user",duplicate,NaiveDate::from_ymd_opt(2026,8,10).unwrap()).unwrap_err(),"Evidenční číslo 943 již bylo použito. Vyberte jiné volné číslo.");

        let manual=NewInsured{title:None,last_name:Some("Člen1345".into()),first_name:Some("Nový".into()),personal_id:Some("991231/1345".into()),organization:Some("TEST".into()),affiliation:"FVČ".into(),city:Some("Praha".into()),address:Some("Testovací 3".into()),postal_code:Some("110 00".into()),country:Some("Česká republika".into()),note:None,insurance_from:Some("2026-09-01".into()),insurance_to:Some("2026-12-31".into()),annual_amount:200_000,category:"B".into(),loss:false,actual_payment:Some(0),code:1,registration_year:2026,email:None,phone:None,registration_number:1345};
        assert_eq!(save_to_database(&database,"test-user",manual).unwrap().registration_number,1345);
        let _=fs::remove_file(result.pdf_path);
    }
}
