# Sprint FED-PARITY-DOCUMENTS-01 — výsledek

Dokončeno 2026-08-10. Priorita P1, milestone M1 — Stabilizace Core.

## Rozsah

Aktivní dokumenty byly porovnány s exportovanými definicemi Access sestav. Pro `PojUdálost` a `PojUdálost_sest` byly navíc vytvořeny referenční PDF přímo z izolované kopie produkčního Accessu a vizuálně porovnány.

- Doklad/potvrzení: zachován existující třístránkový Access layout a jeho regresní test.
- Přihláška: doplněna původní struktura formuláře, identifikační údaje, kategorie, limit, pojistné období, pojistné, prohlášení, datum a podpis.
- Hlášení pojistné události: přidán samostatný PDF výstup dostupný z centrální agendy i detailu člena.
- HVP: export používá souhrnný řádek události a samostatný víceřádkový popis podle Access sestavy.
- OC/ZO a přehled pro pojišťovnu: datasety a pořadí sloupců byly dorovnány k aktivním Access sestavám; PDF má opakované záhlaví, stránkování a celkový počet.

Podle D-017 nejsou součástí sprintu přesné rozměry a rozmístění poštovní poukázky, obálky ani adresního štítku. Jejich existující základní výstupy nebyly změněny.

## Databáze

Nebyla provedena databázová migrace ani změna produkčních dat. Výstupy čtou existující sloupce a individuální hlášení používá již existující vazbu události na konkrétní pojistný záznam.

## Ověření

- `cargo test --manifest-path src-tauri/Cargo.toml --lib`: 62 testů PASS.
- `npm.cmd run build`: PASS.
- Přidány regresní testy pro Přihlášku, individuální hlášení, HVP a tabulkovou OC sestavu.
- Všechna čtyři kontrolní PDF byla vyrenderována přes Poppler a vizuálně zkontrolována; nebylo zjištěno překrývání ani oříznutí textu.

## Omezení

Výstupy zachovávají obsahovou a provozní strukturu Accessu, nejsou však binárně ani pixelově totožné s Access PDF. Používají současný PDF renderer a systémový Arial. Historických 112 pojistných událostí se tento sprint nedotýká; jejich zpřístupnění je samostatný navazující sprint.
