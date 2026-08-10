# Testovací strategie

Povinné vrstvy: Rust testy; `npm run build`; `npm run verify:version`; Tauri release build; smoke a regrese; updater end-to-end včetně cizího/poškozeného/chybějícího podpisu; záloha/obnova; migrace a idempotence; PDF vizuální porovnání a tisk A4; platební workflow; prázdné/NULL hodnoty; návrat se zachováním filtrů; checksum Accessu a zdrojové `dd.sqlite` před/po testu.

Regrese událostí musí obsahovat založení v aktuálním roce s datem vzniku v historickém roce a ověřit historický `PojistnyZaznamRowId`, pojistné a odmítnutí neexistujícího roku. Regrese kontaktů ověřuje textový telefon po uložení a novém otevření databáze. Tarifní test ověřuje `ceil` až nad výsledkem `roční pojistné / 12 × počet měsíců`.

## Release Candidate checklist
- [ ] Scope odpovídá schválené roadmapě a verze jsou shodné.
- [ ] Rust testy, frontend a release build prošly.
- [ ] Migrace, smoke, regrese, platby, záloha/obnova prošly.
- [ ] PDF/A4, SMTP/Credential Manager a updater byly ověřeny end-to-end podle scope.
- [ ] Zdrojová data mají nezměněné checksumy a není otevřený P0/P1 nález.
- [ ] Dokumentace odpovídá skutečnosti a Product Owner schválil produkční release.
