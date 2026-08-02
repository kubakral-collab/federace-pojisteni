# Databáze

Původní Access je historický zdroj logiky, zdrojová `dd.sqlite` migrační vstup a pracovní SQLite jediný zapisovaný soubor. Oba zdroje jsou read-only.

Ověřený model v Rust kódu: členové v `Seznam`; roky/archiv v `PojistnaObdobi`; sazby v `sazby_pojistneho`; platby v `PlatbyClenu`/`AuditPlateb`; příkazy a splatnost v `PlatebniNastaveni`/`PrikazyKUhrade`; doklady v `DokladyOUhrade`/`NastaveniDokladu`/`AuditDokladu`; události v `PojistneUdalosti`; správce v `AppUsers`; vybraný audit v `AuditLog`. Splatnost je navázána na příkaz a pojistný rok. Schéma vzniká idempotentními kontrolami v `src-tauri/src/*.rs`.

Záloha/obnova používá SQLite Backup API, `.fvcbackup`, SHA-256, kontrolu integrity a nouzovou zálohu; viz [database-backups.md](database-backups.md).

Otevřeno: SQLCipher migrace/recovery, klíče šifrovaných záloh, sjednocení auditu, přesná pravidla ročního převodu a bankovního párování.
