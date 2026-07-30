# Federace Pojištění

Windows aplikace Tauri 2 + React + Rust pro správu pojištění členů Federace.

## Vývoj

- Instalace závislostí: `npm.cmd install`
- Web build: `npm.cmd run build`
- Tauri vývoj: `npm.cmd run tauri dev`
- Windows instalátor: `npm.cmd run tauri build -- --bundles nsis`

Produkční databáze, Access soubory a forenzní exporty nejsou součástí repozitáře. CI před sestavením vytvoří pouze prázdný SQLite zdroj bez osobních údajů. Existující instalace pracuje s vlastní databází v aplikačním datovém adresáři Windows.

Při prvním spuštění se vytváří účet správce. Aplikace neobsahuje výchozí heslo ani pevně vložený hash; heslo se ukládá pouze jako Argon2id hash v pracovní databázi.
