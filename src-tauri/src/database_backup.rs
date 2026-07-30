use chrono::{Local, Utc};
use rusqlite::{Connection, MAIN_DB};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tempfile::TempDir;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const FORMAT_VERSION: u32 = 1;
const MANIFEST_FILE: &str = "manifest.json";
const DATABASE_FILE: &str = "database.sqlite";
const CHECKSUM_FILE: &str = "checksum.sha256";
const MAX_DATABASE_SIZE: u64 = 4 * 1024 * 1024 * 1024;

const CREATE_ERROR: &str = "Zálohu se nepodařilo vytvořit. Databáze nebyla změněna.";
const INVALID_ERROR: &str = "Vybraný soubor není platná nebo neporušená záloha Federace.";
const COMPATIBILITY_ERROR: &str = "Záloha není kompatibilní s touto verzí databáze.";
const RESTORE_ERROR: &str = "Obnovu se nepodařilo dokončit. Původní databáze byla zachována.";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupManifest {
    pub format_version: u32,
    pub application_version: String,
    pub schema_version: i64,
    pub created_at: String,
    pub member_count: i64,
    pub database_size: u64,
    pub database_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub path: String,
    pub file_name: String,
    pub application_version: String,
    pub schema_version: i64,
    pub created_at: String,
    pub member_count: i64,
    pub database_size: u64,
    pub checksum: String,
    pub emergency: bool,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub emergency_backup: BackupInfo,
}

struct ValidatedBackup {
    manifest: BackupManifest,
    database_path: PathBuf,
    _temporary_directory: TempDir,
}

fn sha256(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|_| INVALID_ERROR.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| INVALID_ERROR.to_string())?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn schema_version(connection: &Connection) -> Result<i64, String> {
    connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| INVALID_ERROR.to_string())
}

fn member_count(connection: &Connection) -> Result<i64, String> {
    let has_identifier: bool = connection
        .query_row(
            r#"SELECT EXISTS(SELECT 1 FROM pragma_table_info('Seznam') WHERE name='Identifikátor')"#,
            [],
            |row| row.get(0),
        )
        .map_err(|_| INVALID_ERROR.to_string())?;
    let query = if has_identifier {
        r#"SELECT COUNT(DISTINCT NULLIF(TRIM(CAST("Identifikátor" AS TEXT)), '')) FROM "Seznam""#
    } else {
        r#"SELECT COUNT(*) FROM "Seznam""#
    };
    connection
        .query_row(query, [], |row| row.get(0))
        .map_err(|_| INVALID_ERROR.to_string())
}

fn verify_sqlite(path: &Path) -> Result<(i64, i64), String> {
    let connection = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| INVALID_ERROR.to_string())?;
    let integrity: String = connection
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(|_| INVALID_ERROR.to_string())?;
    if integrity != "ok" {
        return Err(INVALID_ERROR.to_string());
    }
    Ok((schema_version(&connection)?, member_count(&connection)?))
}

fn create_snapshot(database_path: &Path, snapshot_path: &Path) -> Result<(), String> {
    let connection = Connection::open(database_path).map_err(|_| CREATE_ERROR.to_string())?;
    connection
        .busy_timeout(std::time::Duration::from_secs(10))
        .map_err(|_| CREATE_ERROR.to_string())?;
    connection
        .backup(MAIN_DB, snapshot_path, None)
        .map_err(|_| CREATE_ERROR.to_string())?;
    verify_sqlite(snapshot_path).map_err(|_| CREATE_ERROR.to_string())?;
    Ok(())
}

fn manifest_for_snapshot(snapshot_path: &Path) -> Result<BackupManifest, String> {
    let (schema_version, member_count) = verify_sqlite(snapshot_path)?;
    let database_size = fs::metadata(snapshot_path)
        .map_err(|_| CREATE_ERROR.to_string())?
        .len();
    Ok(BackupManifest {
        format_version: FORMAT_VERSION,
        application_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version,
        created_at: Utc::now().to_rfc3339(),
        member_count,
        database_size,
        database_sha256: sha256(snapshot_path).map_err(|_| CREATE_ERROR.to_string())?,
    })
}

fn write_package(snapshot_path: &Path, destination: &Path) -> Result<BackupManifest, String> {
    let manifest = manifest_for_snapshot(snapshot_path)?;
    let parent = destination
        .parent()
        .ok_or_else(|| CREATE_ERROR.to_string())?;
    fs::create_dir_all(parent).map_err(|_| CREATE_ERROR.to_string())?;
    let temporary_package = parent.join(format!(
        ".{}.tmp",
        destination
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    ));
    let result = (|| -> Result<(), String> {
        let file = File::create(&temporary_package).map_err(|_| CREATE_ERROR.to_string())?;
        let mut archive = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        archive
            .start_file(MANIFEST_FILE, options)
            .map_err(|_| CREATE_ERROR.to_string())?;
        archive
            .write_all(&serde_json::to_vec_pretty(&manifest).map_err(|_| CREATE_ERROR.to_string())?)
            .map_err(|_| CREATE_ERROR.to_string())?;
        archive
            .start_file(DATABASE_FILE, options)
            .map_err(|_| CREATE_ERROR.to_string())?;
        let mut database = File::open(snapshot_path).map_err(|_| CREATE_ERROR.to_string())?;
        std::io::copy(&mut database, &mut archive).map_err(|_| CREATE_ERROR.to_string())?;
        archive
            .start_file(CHECKSUM_FILE, options)
            .map_err(|_| CREATE_ERROR.to_string())?;
        archive
            .write_all(format!("{}  {}\n", manifest.database_sha256, DATABASE_FILE).as_bytes())
            .map_err(|_| CREATE_ERROR.to_string())?;
        archive.finish().map_err(|_| CREATE_ERROR.to_string())?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary_package);
        return Err(CREATE_ERROR.to_string());
    }
    if destination.exists() {
        fs::remove_file(destination).map_err(|_| CREATE_ERROR.to_string())?;
    }
    fs::rename(&temporary_package, destination).map_err(|_| CREATE_ERROR.to_string())?;
    validate_package(destination, None).map_err(|_| CREATE_ERROR.to_string())?;
    Ok(manifest)
}

fn validate_package(path: &Path, expected_schema: Option<i64>) -> Result<ValidatedBackup, String> {
    if path.extension().and_then(|value| value.to_str()) != Some("fvcbackup") {
        return Err(INVALID_ERROR.to_string());
    }
    let file = File::open(path).map_err(|_| INVALID_ERROR.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|_| INVALID_ERROR.to_string())?;
    let names = (0..archive.len())
        .filter_map(|index| {
            archive
                .by_index(index)
                .ok()
                .map(|entry| entry.name().to_string())
        })
        .collect::<BTreeSet<_>>();
    let expected = [MANIFEST_FILE, DATABASE_FILE, CHECKSUM_FILE]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    if names != expected {
        return Err(INVALID_ERROR.to_string());
    }
    let manifest: BackupManifest = {
        let mut entry = archive
            .by_name(MANIFEST_FILE)
            .map_err(|_| INVALID_ERROR.to_string())?;
        if entry.size() > 64 * 1024 {
            return Err(INVALID_ERROR.to_string());
        }
        serde_json::from_reader(&mut entry).map_err(|_| INVALID_ERROR.to_string())?
    };
    if manifest.format_version != FORMAT_VERSION || manifest.database_size > MAX_DATABASE_SIZE {
        return Err(INVALID_ERROR.to_string());
    }
    if expected_schema.is_some_and(|version| version != manifest.schema_version) {
        return Err(COMPATIBILITY_ERROR.to_string());
    }
    let checksum_text = {
        let mut entry = archive
            .by_name(CHECKSUM_FILE)
            .map_err(|_| INVALID_ERROR.to_string())?;
        let mut text = String::new();
        entry
            .read_to_string(&mut text)
            .map_err(|_| INVALID_ERROR.to_string())?;
        text
    };
    if checksum_text.trim() != format!("{}  {}", manifest.database_sha256, DATABASE_FILE) {
        return Err(INVALID_ERROR.to_string());
    }
    let temporary_directory = tempfile::tempdir().map_err(|_| INVALID_ERROR.to_string())?;
    let database_path = temporary_directory.path().join(DATABASE_FILE);
    {
        let mut entry = archive
            .by_name(DATABASE_FILE)
            .map_err(|_| INVALID_ERROR.to_string())?;
        if entry.size() != manifest.database_size || entry.size() > MAX_DATABASE_SIZE {
            return Err(INVALID_ERROR.to_string());
        }
        let mut output = File::create(&database_path).map_err(|_| INVALID_ERROR.to_string())?;
        std::io::copy(&mut entry, &mut output).map_err(|_| INVALID_ERROR.to_string())?;
        output.sync_all().map_err(|_| INVALID_ERROR.to_string())?;
    }
    if sha256(&database_path)? != manifest.database_sha256 {
        return Err(INVALID_ERROR.to_string());
    }
    let (actual_schema, actual_members) = verify_sqlite(&database_path)?;
    if actual_schema != manifest.schema_version || actual_members != manifest.member_count {
        return Err(INVALID_ERROR.to_string());
    }
    Ok(ValidatedBackup {
        manifest,
        database_path,
        _temporary_directory: temporary_directory,
    })
}

fn info(path: &Path, manifest: &BackupManifest, emergency: bool) -> BackupInfo {
    BackupInfo {
        path: path.to_string_lossy().to_string(),
        file_name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        application_version: manifest.application_version.clone(),
        schema_version: manifest.schema_version,
        created_at: manifest.created_at.clone(),
        member_count: manifest.member_count,
        database_size: manifest.database_size,
        checksum: manifest.database_sha256.clone(),
        emergency,
        provider: "Místní soubor".to_string(),
    }
}

pub fn default_file_name(prefix: &str) -> String {
    format!(
        "{prefix}_{}.fvcbackup",
        Local::now().format("%Y-%m-%d_%H-%M")
    )
}

pub fn create(
    database_path: &Path,
    destination: &Path,
    emergency: bool,
) -> Result<BackupInfo, String> {
    let temporary_directory = tempfile::tempdir().map_err(|_| CREATE_ERROR.to_string())?;
    let snapshot_path = temporary_directory.path().join(DATABASE_FILE);
    create_snapshot(database_path, &snapshot_path)?;
    let manifest = write_package(&snapshot_path, destination)?;
    println!("Zálohování databáze: vytvoření zálohy dokončeno.");
    Ok(info(destination, &manifest, emergency))
}

pub fn inspect(path: &Path, expected_schema: i64) -> Result<BackupInfo, String> {
    let validated = validate_package(path, Some(expected_schema))?;
    Ok(info(
        path,
        &validated.manifest,
        path.file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with("Emergency_Backup_")),
    ))
}

pub fn current_schema(database_path: &Path) -> Result<i64, String> {
    let connection = Connection::open(database_path).map_err(|_| INVALID_ERROR.to_string())?;
    schema_version(&connection)
}

pub fn restore(
    database_path: &Path,
    backup_path: &Path,
    emergency_directory: &Path,
) -> Result<RestoreResult, String> {
    let current_schema = current_schema(database_path)?;
    let selected = validate_package(backup_path, Some(current_schema))?;
    fs::create_dir_all(emergency_directory).map_err(|_| RESTORE_ERROR.to_string())?;
    let mut emergency_path = emergency_directory.join(default_file_name("Emergency_Backup"));
    let mut suffix = 2;
    while emergency_path.exists() {
        emergency_path = emergency_directory.join(
            default_file_name("Emergency_Backup")
                .replace(".fvcbackup", &format!("_{suffix}.fvcbackup")),
        );
        suffix += 1;
    }
    let emergency =
        create(database_path, &emergency_path, true).map_err(|_| RESTORE_ERROR.to_string())?;
    println!("Zálohování databáze: nouzová záloha vytvořena.");
    let emergency_validated = validate_package(&emergency_path, Some(current_schema))?;
    let mut destination = Connection::open(database_path).map_err(|_| RESTORE_ERROR.to_string())?;
    destination
        .busy_timeout(std::time::Duration::from_secs(10))
        .map_err(|_| RESTORE_ERROR.to_string())?;
    let restore_result = destination
        .restore(
            MAIN_DB,
            &selected.database_path,
            None::<fn(rusqlite::backup::Progress)>,
        )
        .and_then(|_| {
            let integrity: String =
                destination.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
            if integrity == "ok" {
                Ok(())
            } else {
                Err(rusqlite::Error::InvalidQuery)
            }
        });
    if restore_result.is_err() {
        let rollback = destination.restore(
            MAIN_DB,
            &emergency_validated.database_path,
            None::<fn(rusqlite::backup::Progress)>,
        );
        println!(
            "Zálohování databáze: obnova selhala, nouzový návrat {}.",
            if rollback.is_ok() {
                "dokončen"
            } else {
                "selhal"
            }
        );
        return Err(RESTORE_ERROR.to_string());
    }
    println!("Zálohování databáze: obnova dokončena.");
    Ok(RestoreResult {
        emergency_backup: emergency,
    })
}

pub fn list(directory: &Path, current_schema: i64) -> Vec<BackupInfo> {
    let registry_path = directory.join("backups.json");
    let mut backups = fs::read(&registry_path)
        .ok()
        .and_then(|content| serde_json::from_slice::<Vec<BackupInfo>>(&content).ok())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| inspect(Path::new(&entry.path), current_schema).ok())
        .collect::<Vec<_>>();
    backups.extend(
        fs::read_dir(directory)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.path().extension().and_then(|value| value.to_str()) == Some("fvcbackup")
            })
            .filter_map(|entry| inspect(&entry.path(), current_schema).ok()),
    );
    backups.sort_by(|left, right| left.path.cmp(&right.path));
    backups.dedup_by(|left, right| left.path == right.path);
    backups.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    backups
}

pub fn remember(directory: &Path, backup: &BackupInfo, current_schema: i64) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|_| CREATE_ERROR.to_string())?;
    let mut backups = list(directory, current_schema);
    backups.retain(|entry| entry.path != backup.path);
    backups.push(backup.clone());
    let temporary = directory.join("backups.json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(&backups).map_err(|_| CREATE_ERROR.to_string())?,
    )
    .map_err(|_| CREATE_ERROR.to_string())?;
    fs::rename(&temporary, directory.join("backups.json")).map_err(|_| CREATE_ERROR.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_database(path: &Path, value: &str) {
        let connection = Connection::open(path).unwrap();
        connection
            .execute_batch("PRAGMA user_version=7; CREATE TABLE Seznam (Jmeno TEXT);")
            .unwrap();
        connection
            .execute("INSERT INTO Seznam VALUES (?1)", [value])
            .unwrap();
    }

    #[test]
    fn package_contains_verified_snapshot() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("source.sqlite");
        let package = directory.path().join("backup.fvcbackup");
        sample_database(&database, "test");
        let created = create(&database, &package, false).unwrap();
        let inspected = inspect(&package, 7).unwrap();
        assert_eq!(created.checksum, inspected.checksum);
        assert_eq!(inspected.member_count, 1);
    }

    #[test]
    fn tampered_database_is_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("source.sqlite");
        let package = directory.path().join("backup.fvcbackup");
        sample_database(&database, "test");
        create(&database, &package, false).unwrap();
        let bytes = fs::read(&package).unwrap();
        let mut damaged = bytes;
        let index = damaged.len() / 2;
        damaged[index] ^= 1;
        fs::write(&package, damaged).unwrap();
        assert!(inspect(&package, 7).is_err());
    }

    #[test]
    fn restore_replaces_database_and_creates_emergency_backup() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("live.sqlite");
        let source = directory.path().join("source.sqlite");
        let package = directory.path().join("backup.fvcbackup");
        sample_database(&database, "old");
        sample_database(&source, "new");
        create(&source, &package, false).unwrap();
        let result = restore(&database, &package, &directory.path().join("managed")).unwrap();
        assert!(Path::new(&result.emergency_backup.path).is_file());
        let connection = Connection::open(&database).unwrap();
        let value: String = connection
            .query_row("SELECT Jmeno FROM Seznam", [], |row| row.get(0))
            .unwrap();
        assert_eq!(value, "new");
    }

    #[test]
    fn locked_database_stays_unchanged_when_restore_fails() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("live.sqlite");
        let source = directory.path().join("source.sqlite");
        let package = directory.path().join("backup.fvcbackup");
        sample_database(&database, "old");
        sample_database(&source, "new");
        create(&source, &package, false).unwrap();
        let lock = Connection::open(&database).unwrap();
        lock.execute_batch("BEGIN IMMEDIATE; UPDATE Seznam SET Jmeno='locked';")
            .unwrap();
        assert!(restore(&database, &package, &directory.path().join("managed")).is_err());
        lock.execute_batch("ROLLBACK;").unwrap();
        let connection = Connection::open(&database).unwrap();
        let value: String = connection
            .query_row("SELECT Jmeno FROM Seznam", [], |row| row.get(0))
            .unwrap();
        assert_eq!(value, "old");
    }
}
