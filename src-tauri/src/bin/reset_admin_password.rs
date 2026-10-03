use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use chrono::Local;
use rand_core::OsRng;
use rusqlite::{params, Connection, TransactionBehavior};
use std::{env, fs, io, path::PathBuf};

fn main() -> Result<(), String> {
    let database = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| "Použití: reset_admin_password <cesta-k-dd.sqlite>".to_string())?;
    if !database.is_file() {
        return Err("Databázový soubor nebyl nalezen.".to_string());
    }

    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .map_err(|_| "Nové heslo se nepodařilo načíst.".to_string())?;
    let password = password.trim_end_matches(['\r', '\n']);
    if password.chars().count() < 12 {
        return Err("Heslo musí mít alespoň 12 znaků.".to_string());
    }

    let backup = database.with_file_name(format!(
        "dd-before-password-reset-{}.sqlite",
        Local::now().format("%Y%m%d-%H%M%S")
    ));
    fs::copy(&database, &backup)
        .map_err(|_| "Bezpečnostní kopii databáze se nepodařilo vytvořit.".to_string())?;

    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .map_err(|_| "Nové heslo se nepodařilo zahashovat.".to_string())?
        .to_string();
    let mut connection = Connection::open(&database)
        .map_err(|_| "Databázi se nepodařilo otevřít.".to_string())?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| "Databáze je používána; nejprve ukončete aplikaci.".to_string())?;
    let changed = transaction
        .execute(
            r#"UPDATE "AppUsers" SET "PasswordHash"=?1 WHERE "Active"=1"#,
            params![hash],
        )
        .map_err(|_| "Heslo se nepodařilo změnit.".to_string())?;
    if changed != 1 {
        return Err("Nebyl nalezen právě jeden aktivní účet správce.".to_string());
    }
    transaction
        .commit()
        .map_err(|_| "Změnu hesla se nepodařilo uložit.".to_string())?;

    println!("Heslo bylo změněno. Záloha: {}", backup.display());
    Ok(())
}
