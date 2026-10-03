use crate::{
    applications, claims, database_backup, email_service, financial_documents,
    member_payments, organization_payments, payments, receipts, reports, tariffs,
};
use rusqlite::{Connection, Transaction};
use serde::Serialize;
use std::{fs, path::{Path, PathBuf}};

pub const DB_SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationFailure {
    pub source_version: i64,
    pub target_version: i64,
    pub failed_migration: String,
    pub cause: String,
    pub backup_path: Option<String>,
}

impl std::fmt::Display for MigrationFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Migrace databáze {} → {} selhala ({}): {}{}",
            self.source_version,
            self.target_version,
            self.failed_migration,
            self.cause,
            self.backup_path
                .as_ref()
                .map(|path| format!(" Záloha: {path}"))
                .unwrap_or_default()
        )
    }
}

pub struct MigrationOutcome {
    pub schema_version: i64,
    pub backup_path: Option<PathBuf>,
}

struct Migration {
    source: i64,
    target: i64,
    name: &'static str,
    apply: fn(&Transaction<'_>) -> rusqlite::Result<()>,
}

const REQUIRED_V0: &[(&str, &[&str])] = &[
    ("Seznam", &["Identifikátor", "PojištěníOd", "EvČíslo", "Příjmení", "Jméno"]),
    ("Editace", &["PojištěníOd", "EvČíslo", "Příjmení", "Jméno"]),
    ("Kategorie", &["Kategorie", "Ztráta", "Pojistné"]),
];

const REQUIRED_V1: &[(&str, &[&str])] = &[
    ("AppUsers", &["Id", "Username", "PasswordHash", "Role", "Active"]),
    ("AuditLog", &["Id", "DatumČas", "Uživatel", "Operace", "Výsledek"]),
    ("PojistnaObdobi", &["Rok", "Stav", "Vytvoreno"]),
    ("sazby_pojistneho", &["id", "pojistna_castka", "kategorie"]),
    ("PrikazyKUhrade", &["Id", "PojistnyZaznamRowId", "PojistnyRok"]),
    ("PlatbyClenu", &["Id", "PojistnyZaznamRowId", "PojistnyRok", "OrganizacniPlatbaId"]),
    ("OrganizacniPlatby", &["Id", "Organizace", "PojistnyRok"]),
    ("PojistneUdalosti", &["ID", "IdentifikatorClena", "PojistnyRok"]),
    ("DokladyOUhrade", &["Id", "IdentifikatorClena", "PojistnyRok"]),
    ("EmailNastaveni", &["Id", "Server", "Port"]),
    ("Prihlasky", &["Id", "IdentifikatorClena"]),
    ("VydaneFaktury", &["Id", "Cislo", "Castka"]),
    ("AuditSestav", &["Id", "Druh", "PojistnyRok"]),
];

fn has_table(connection: &Connection, table: &str) -> rusqlite::Result<bool> {
    connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
        [table],
        |row| row.get(0),
    )
}

fn columns(connection: &Connection, table: &str) -> rusqlite::Result<Vec<String>> {
    let escaped = table.replace('"', "\"\"");
    connection
        .prepare(&format!("PRAGMA table_info(\"{escaped}\")"))?
        .query_map([], |row| row.get(1))?
        .collect()
}

fn validate_tables(connection: &Connection, required: &[(&str, &[&str])]) -> Result<(), String> {
    for (table, expected_columns) in required {
        if !has_table(connection, table).map_err(|error| error.to_string())? {
            return Err(format!("chybí tabulka {table}"));
        }
        let actual = columns(connection, table).map_err(|error| error.to_string())?;
        for column in *expected_columns {
            if !actual.iter().any(|actual| actual == column) {
                return Err(format!("v tabulce {table} chybí sloupec {column}"));
            }
        }
    }
    Ok(())
}

fn validate_integrity(connection: &Connection) -> Result<(), String> {
    let result: String = connection
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if result == "ok" { Ok(()) } else { Err(format!("kontrola integrity: {result}")) }
}

fn has_index_on(connection: &Connection, table: &str, column: &str) -> Result<bool, String> {
    let escaped = table.replace('"', "\"\"");
    let indexes = connection
        .prepare(&format!("PRAGMA index_list(\"{escaped}\")"))
        .and_then(|mut statement| {
            statement.query_map([], |row| row.get::<_, String>(1))?.collect::<rusqlite::Result<Vec<_>>>()
        })
        .map_err(|error| error.to_string())?;
    for index in indexes {
        let escaped_index = index.replace('"', "\"\"");
        let matches = connection
            .prepare(&format!("PRAGMA index_info(\"{escaped_index}\")"))
            .and_then(|mut statement| {
                statement.query_map([], |row| row.get::<_, String>(2))?.collect::<rusqlite::Result<Vec<_>>>()
            })
            .map_err(|error| error.to_string())?
            .iter()
            .any(|name| name == column);
        if matches { return Ok(true); }
    }
    Ok(false)
}

fn validate_v0(connection: &Connection) -> Result<(), String> {
    validate_integrity(connection)?;
    validate_tables(connection, REQUIRED_V0)?;
    for column in ["Identifikátor", "KódOC"] {
        if !has_index_on(connection, "Seznam", column)? {
            return Err(format!("chybí index tabulky Seznam pro sloupec {column}"));
        }
    }

    // Tabulky přidávané verzí 0.24 mohly vzniknout až po otevření konkrétního
    // modulu. Pokud už existují, nesmějí mít neznámý/neúplný tvar.
    for (table, expected_columns) in REQUIRED_V1 {
        if has_table(connection, table).map_err(|error| error.to_string())? {
            validate_tables(connection, &[(*table, *expected_columns)])?;
        }
    }
    Ok(())
}

fn validate_v1(connection: &Connection) -> Result<(), String> {
    validate_integrity(connection)?;
    validate_tables(connection, REQUIRED_V0)?;
    validate_tables(connection, REQUIRED_V1)?;
    for index in [
        "idx_sazby_vyhledani",
        "IX_PrikazyKUhrade_RokSplatnost",
        "IX_PlatbyClenu_ClenRok",
        "IX_PojistneUdalosti_Clen",
        "IX_DokladyOUhrade_Clen",
    ] {
        let exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='index' AND name=?1)",
                [index],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if !exists { return Err(format!("chybí index {index}")); }
    }
    for trigger in ["trg_prihlasky_immutable_update", "trg_prihlasky_immutable_delete"] {
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='trigger' AND name=?1)",
            [trigger], |row| row.get(0),
        ).map_err(|error| error.to_string())?;
        if !exists { return Err(format!("chybí trigger {trigger}")); }
    }
    Ok(())
}

fn ensure_column(connection: &Connection, table: &str, column: &str) -> rusqlite::Result<()> {
    if !columns(connection, table)?.iter().any(|name| name == column) {
        connection.execute(
            &format!("ALTER TABLE \"{}\" ADD COLUMN \"{}\" TEXT", table.replace('"', "\"\""), column.replace('"', "\"\"")),
            [],
        )?;
    }
    Ok(())
}

fn baseline_v1(transaction: &Transaction<'_>) -> rusqlite::Result<()> {
    ensure_column(transaction, "Seznam", "Telefon")?;
    ensure_column(transaction, "Editace", "Telefon")?;
    transaction.execute_batch(
        r#"CREATE TABLE IF NOT EXISTS "PojistnaObdobi" (
               "Rok" INTEGER PRIMARY KEY,
               "Stav" TEXT NOT NULL CHECK ("Stav" IN ('AKTIVNI', 'UZAVRENO')),
               "Vytvoreno" TEXT NOT NULL DEFAULT (datetime('now'))
           );
           CREATE TABLE IF NOT EXISTS "NeprevadetCleny" (
               "RodneCislo" TEXT PRIMARY KEY,
               "Duvod" TEXT
           );
           CREATE TABLE IF NOT EXISTS "AppUsers" (
               "Id" INTEGER PRIMARY KEY AUTOINCREMENT,
               "Username" TEXT NOT NULL UNIQUE,
               "PasswordHash" TEXT NOT NULL,
               "Role" TEXT NOT NULL,
               "CreatedAt" TEXT NOT NULL DEFAULT (datetime('now')),
               "Active" INTEGER NOT NULL DEFAULT 1
           );
           CREATE TABLE IF NOT EXISTS "AuditLog" (
               "Id" INTEGER PRIMARY KEY AUTOINCREMENT,
               "DatumČas" TEXT NOT NULL,
               "Uživatel" TEXT NOT NULL,
               "Operace" TEXT NOT NULL,
               "IdentifikátorPojištěnce" TEXT,
               "Výsledek" TEXT NOT NULL
           );"#,
    )?;
    tariffs::ensure_schema(transaction)?;
    payments::ensure_schema(transaction)?;
    payments::ensure_order_schema(transaction)?;
    member_payments::ensure_schema(transaction)?;
    organization_payments::ensure_schema(transaction)?;
    claims::ensure_schema(transaction)?;
    email_service::ensure_schema(transaction)?;
    receipts::ensure_schema(transaction)?;
    applications::ensure_schema(transaction)?;
    financial_documents::ensure_schema(transaction)?;
    reports::ensure_schema(transaction)?;
    Ok(())
}

fn migrations() -> Vec<Migration> {
    vec![Migration { source: 0, target: 1, name: "baseline-v1", apply: baseline_v1 }]
}

fn unique_backup_path(directory: &Path, source: i64, target: i64) -> PathBuf {
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let base = format!("PreMigration_{source}-to-{target}_{stamp}");
    let mut path = directory.join(format!("{base}.fvcbackup"));
    let mut suffix = 2;
    while path.exists() {
        path = directory.join(format!("{base}_{suffix}.fvcbackup"));
        suffix += 1;
    }
    path
}

fn create_verified_backup(database_path: &Path, directory: &Path, version: i64) -> Result<PathBuf, String> {
    fs::create_dir_all(directory).map_err(|error| format!("nelze vytvořit adresář záloh: {error}"))?;
    let path = unique_backup_path(directory, version, version + 1);
    database_backup::create(database_path, &path, false)
        .map_err(|error| format!("vytvoření zálohy: {error}"))?;
    database_backup::inspect(&path, version)
        .map_err(|error| format!("ověření zálohy: {error}"))?;
    Ok(path)
}

fn apply_chain(
    connection: &mut Connection,
    registry: &[Migration],
    target: i64,
) -> Result<(), (i64, i64, String, String)> {
    let mut current: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| (0, target, "čtení verze".into(), error.to_string()))?;
    while current < target {
        let migration = registry.iter().find(|migration| migration.source == current)
            .ok_or_else(|| (current, target, "výběr migrace".into(), "chybí navazující migrační krok".into()))?;
        let transaction = connection.transaction()
            .map_err(|error| (current, migration.target, migration.name.into(), error.to_string()))?;
        (migration.apply)(&transaction)
            .map_err(|error| (current, migration.target, migration.name.into(), error.to_string()))?;
        if migration.target == DB_SCHEMA_VERSION {
            validate_v1(&transaction)
                .map_err(|error| (current, migration.target, migration.name.into(), error))?;
        }
        transaction.pragma_update(None, "user_version", migration.target)
            .map_err(|error| (current, migration.target, migration.name.into(), error.to_string()))?;
        transaction.commit()
            .map_err(|error| (current, migration.target, migration.name.into(), error.to_string()))?;
        current = migration.target;
    }
    Ok(())
}

pub fn migrate(
    database_path: &Path,
    backup_directory: &Path,
    newly_created: bool,
) -> Result<MigrationOutcome, MigrationFailure> {
    let connection = Connection::open(database_path).map_err(|error| MigrationFailure {
        source_version: -1,
        target_version: DB_SCHEMA_VERSION,
        failed_migration: "otevření databáze".into(),
        cause: error.to_string(),
        backup_path: None,
    })?;
    connection.busy_timeout(std::time::Duration::from_secs(10)).ok();
    let source: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| MigrationFailure { source_version: -1, target_version: DB_SCHEMA_VERSION, failed_migration: "čtení verze".into(), cause: error.to_string(), backup_path: None })?;
    if source > DB_SCHEMA_VERSION {
        return Err(MigrationFailure { source_version: source, target_version: DB_SCHEMA_VERSION, failed_migration: "kontrola kompatibility".into(), cause: "databáze pochází z novější verze aplikace".into(), backup_path: None });
    }
    let validation = match source { 0 => validate_v0(&connection), 1 => validate_v1(&connection), _ => Err("nepodporovaná verze databáze".into()) };
    validation.map_err(|cause| MigrationFailure { source_version: source, target_version: DB_SCHEMA_VERSION, failed_migration: "strukturální validace".into(), cause, backup_path: None })?;
    if source == DB_SCHEMA_VERSION {
        return Ok(MigrationOutcome { schema_version: source, backup_path: None });
    }

    drop(connection);
    let backup = if newly_created { None } else {
        Some(create_verified_backup(database_path, backup_directory, source).map_err(|cause| MigrationFailure {
            source_version: source, target_version: DB_SCHEMA_VERSION, failed_migration: "předmigrační záloha".into(), cause, backup_path: None,
        })?)
    };
    let mut connection = Connection::open(database_path).map_err(|error| MigrationFailure {
        source_version: source, target_version: DB_SCHEMA_VERSION, failed_migration: "otevření databáze".into(), cause: error.to_string(), backup_path: backup.as_ref().map(|path| path.to_string_lossy().into_owned()),
    })?;
    apply_chain(&mut connection, &migrations(), DB_SCHEMA_VERSION).map_err(|(from, to, name, cause)| MigrationFailure {
        source_version: from, target_version: to, failed_migration: name, cause, backup_path: backup.as_ref().map(|path| path.to_string_lossy().into_owned()),
    })?;
    Ok(MigrationOutcome { schema_version: DB_SCHEMA_VERSION, backup_path: backup })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy(path: &Path) {
        Connection::open(path).unwrap().execute_batch(r#"
            CREATE TABLE Seznam ("Identifikátor" INTEGER, "PojištěníOd" TEXT, "EvČíslo" INTEGER, "Příjmení" TEXT, "Jméno" TEXT, "KódOC" TEXT);
            CREATE TABLE Editace ("PojištěníOd" TEXT, "EvČíslo" INTEGER, "Příjmení" TEXT, "Jméno" TEXT);
            CREATE TABLE Kategorie ("Kategorie" TEXT, "Ztráta" INTEGER, "Pojistné" INTEGER);
            CREATE INDEX idx_Seznam_Identifikator ON Seznam("Identifikátor");
            CREATE INDEX idx_Seznam_KodOC ON Seznam("KódOC");
            INSERT INTO Seznam VALUES (42, '2026-01-01', 7, 'Novák', 'Jan', '1');
        "#).unwrap();
    }

    fn version(path: &Path) -> i64 {
        Connection::open(path).unwrap().pragma_query_value(None, "user_version", |row| row.get(0)).unwrap()
    }

    #[test]
    fn a_fresh_database_is_created_as_v1() {
        let dir = tempfile::tempdir().unwrap(); let db = dir.path().join("fresh.sqlite"); legacy(&db);
        let result = migrate(&db, &dir.path().join("backups"), true).unwrap();
        assert_eq!(result.schema_version, 1); assert!(result.backup_path.is_none()); assert_eq!(version(&db), 1);
    }

    #[test]
    fn b_compatible_v0_is_backed_up_and_preserves_data() {
        let dir = tempfile::tempdir().unwrap(); let db = dir.path().join("old.sqlite"); legacy(&db);
        let result = migrate(&db, &dir.path().join("backups"), false).unwrap();
        assert!(result.backup_path.unwrap().is_file());
        let value: i64 = Connection::open(db).unwrap().query_row("SELECT \"Identifikátor\" FROM Seznam", [], |row| row.get(0)).unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn c_incompatible_v0_is_rejected_without_backup_or_changes() {
        let dir = tempfile::tempdir().unwrap(); let db = dir.path().join("bad.sqlite");
        Connection::open(&db).unwrap().execute_batch("CREATE TABLE Seznam (x TEXT);").unwrap();
        assert!(migrate(&db, &dir.path().join("backups"), false).is_err()); assert_eq!(version(&db), 0);
        assert!(!dir.path().join("backups").exists());
    }

    #[test]
    fn d_failed_migration_rolls_back_and_keeps_version() {
        fn fail(tx: &Transaction<'_>) -> rusqlite::Result<()> { tx.execute_batch("CREATE TABLE transient(x); SELECT * FROM missing;") }
        let dir = tempfile::tempdir().unwrap(); let db = dir.path().join("failure.sqlite"); legacy(&db);
        let backup = create_verified_backup(&db, &dir.path().join("backups"), 0).unwrap();
        let mut connection = Connection::open(&db).unwrap();
        let registry = [Migration { source: 0, target: 1, name: "fail", apply: fail }];
        assert!(apply_chain(&mut connection, &registry, 1).is_err());
        let exists: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='transient')", [], |row| row.get(0)).unwrap();
        assert!(!exists); assert_eq!(connection.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0)).unwrap(), 0); assert!(backup.is_file());
    }

    #[test]
    fn e_backup_failure_blocks_migration() {
        let dir = tempfile::tempdir().unwrap(); let db = dir.path().join("old.sqlite"); legacy(&db);
        let blocker = dir.path().join("not-a-directory"); fs::write(&blocker, b"x").unwrap();
        assert!(migrate(&db, &blocker, false).is_err()); assert_eq!(version(&db), 0);
    }

    #[test]
    fn f_newer_database_is_refused() {
        let dir = tempfile::tempdir().unwrap(); let db = dir.path().join("future.sqlite"); legacy(&db);
        Connection::open(&db).unwrap().pragma_update(None, "user_version", 99).unwrap();
        assert!(migrate(&db, &dir.path().join("backups"), false).is_err()); assert_eq!(version(&db), 99);
    }

    #[test]
    fn g_repeated_start_is_idempotent() {
        let dir = tempfile::tempdir().unwrap(); let db = dir.path().join("repeat.sqlite"); legacy(&db);
        migrate(&db, &dir.path().join("backups"), true).unwrap();
        migrate(&db, &dir.path().join("backups"), false).unwrap(); assert_eq!(version(&db), 1);
    }

    #[test]
    fn h_verified_backup_is_readable_and_has_checksum() {
        let dir = tempfile::tempdir().unwrap(); let db = dir.path().join("old.sqlite"); legacy(&db);
        let result = migrate(&db, &dir.path().join("backups"), false).unwrap();
        let info = database_backup::inspect(&result.backup_path.unwrap(), 0).unwrap(); assert_eq!(info.checksum.len(), 64);
    }

    #[test]
    fn i_chain_runs_in_order_from_one_to_three() {
        fn two(tx: &Transaction<'_>) -> rusqlite::Result<()> { tx.execute_batch("CREATE TABLE two(x);") }
        fn three(tx: &Transaction<'_>) -> rusqlite::Result<()> { tx.execute_batch("CREATE TABLE three(x);") }
        let mut connection = Connection::open_in_memory().unwrap(); connection.pragma_update(None, "user_version", 1).unwrap();
        let registry = [Migration { source: 1, target: 2, name: "1-2", apply: two }, Migration { source: 2, target: 3, name: "2-3", apply: three }];
        apply_chain(&mut connection, &registry, 3).unwrap();
        assert_eq!(connection.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0)).unwrap(), 3);
    }

    #[test]
    fn j_existing_v1_is_not_backed_up_or_modified() {
        let dir = tempfile::tempdir().unwrap(); let db = dir.path().join("v1.sqlite"); legacy(&db);
        migrate(&db, &dir.path().join("backups"), true).unwrap();
        let before = fs::metadata(&db).unwrap().len(); let outcome = migrate(&db, &dir.path().join("backups"), false).unwrap();
        assert!(outcome.backup_path.is_none()); assert_eq!(fs::metadata(&db).unwrap().len(), before);
    }

    #[test]
    fn packaged_database_resource_is_compatible() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("dd.sqlite");
        if !source.is_file() { return; }
        let dir = tempfile::tempdir().unwrap(); let db = dir.path().join("dd.sqlite");
        fs::copy(source, &db).unwrap();
        migrate(&db, &dir.path().join("backups"), true).unwrap();
        assert_eq!(version(&db), DB_SCHEMA_VERSION);
    }
}
