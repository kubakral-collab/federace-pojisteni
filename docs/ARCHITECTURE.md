# Architektura

Tauri 2 desktop pro Windows; React/TypeScript/Vite frontend; Rust backend s Tauri commands; pracovní SQLite přes `rusqlite`. GitHub Actions vytváří GitHub Releases, EXE/MSI a podepsané artefakty Tauri Updateru. Lokální `.fvcbackup` používá SQLite Backup API, manifest a checksum.

Frontend řídí UI, Rust validaci, transakce, PDF, SMTP a soubory. SMTP EmailService a Credential Manager jsou implementované, před RC vyžadují provozní end-to-end test.

Původní Access a zdrojová `dd.sqlite` jsou read-only. Aplikace zapisuje pouze do pracovní kopie v aplikačním datovém adresáři. Viz [DATABASE.md](DATABASE.md), [SECURITY.md](SECURITY.md), [release.md](release.md) a [TESTING.md](TESTING.md).
