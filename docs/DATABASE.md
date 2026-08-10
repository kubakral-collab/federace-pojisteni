# Databáze

Původní Access je historický zdroj logiky, zdrojová `dd.sqlite` migrační vstup a pracovní SQLite jediný zapisovaný soubor. Oba zdroje jsou read-only.

Ověřený model v Rust kódu: členové v `Seznam`; roky/archiv v `PojistnaObdobi`; sazby v `sazby_pojistneho`; platby v `PlatbyClenu`/`AuditPlateb`; příkazy a splatnost v `PlatebniNastaveni`/`PrikazyKUhrade`; doklady v `DokladyOUhrade`/`NastaveniDokladu`/`AuditDokladu`; události v `PojistneUdalosti`; správce v `AppUsers`; vybraný audit v `AuditLog`. Splatnost je navázána na příkaz a pojistný rok. Schéma vzniká idempotentními kontrolami v `src-tauri/src/*.rs`.

Záloha/obnova používá SQLite Backup API, `.fvcbackup`, SHA-256, kontrolu integrity a nouzovou zálohu; viz [database-backups.md](database-backups.md).

Otevřeno: SQLCipher migrace/recovery, klíče šifrovaných záloh, sjednocení auditu, přesná pravidla ročního převodu a bankovního párování.

## Organizační platby

`OrganizacniPlatby` uchovává jednu skutečně přijatou platbu organizace, očekávanou částku a nepřiřazený přeplatek. `RozpisOrganizacniPlatby` váže hlavní platbu na řádky členů a připsané částky. Každá kladná položka rozpisu současně vzniká v `PlatbyClenu` s nullable odkazem `OrganizacniPlatbaId`; staré a individuální platby mají odkaz `NULL`. Vložení hlavní platby, rozpisu, členských plateb a přepočtu `SkutÚhrada` probíhá v jedné transakci.

## Telefon a historické události

Pracovní tabulky `Seznam` a `Editace` dostávají idempotentní migrací nullable textový sloupec `Telefon`; zdrojová `dd.sqlite` se nemění. Telefon se kopíruje při ročním převodu. Pojistná událost ukládá `PojistnyZaznamRowId` a `PojistnyRok` z historického záznamu dohledaného podle rodného čísla a roku data `VznikPU`. Chybějící historický rok je chyba a nikdy se nenahrazuje aktuálním záznamem.
