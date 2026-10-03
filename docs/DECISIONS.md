# Schválená rozhodnutí

Konsolidováno 2026-08-02. Rozhodnutí není oficiální, dokud zde není zapsáno.

- D-001: zachovat funkce a pochopit logiku Accessu, ne slepě kopírovat obrazovky.
- D-002: hlavním uživatelem je jeden správce; UX musí být jednoduché i pro nezkušeného uživatele.
- D-003: akce osoby patří do detailu člena; globální přehled může být současně v levém menu.
- D-004: doklad vizuálně odpovídá Accessu a současně potvrzuje pojištění.
- D-005: variabilní symbol je rodné číslo bez lomítka.
- D-006: aktualizace používají GitHub Releases a Tauri Updater.
- D-007: release se nevydává po každém sprintu.
- D-008: platí Roadmap First; sprinty mimo roadmapu nejsou povoleny.
- D-009: stabilita a ochrana dat mají přednost před funkcemi.
- D-010: Codex fyzicky udržuje dokumentaci v repozitáři.
- D-011 (2026-08-02): Product Owner schválil `BUG-001` s prioritou P1 pro `M1 — Stabilizace Core` a sprint `S1 — Opravy funkčnosti`; scope je omezen na správné výsledky filtrů „Uhrazeno“ a „Neuhrazeno“.
- D-012 (2026-08-10): Product Owner schválil `UX-001` s prioritou P2 pro `M1 — Stabilizace Core`. Požadavek pouze přesouvá „Nový pojištěnec“ bezprostředně nad „Seznam pojištěnců“; implementace čeká na zařazení po aktuálním sprintu.
- D-013 (2026-08-10): Product Owner schválil `PLATBY-001` s prioritou P2 pro `M2 — Provozní automatizace`. Platby mají jednoznačně rozlišovat typ „Jednotlivec“ a „Organizace“ a být navázány na člena, respektive organizaci. Automatické rozúčtování organizační platby není schváleno a vyžaduje samostatné rozhodnutí.
- D-014 (2026-08-10): Product Owner schválil a aktivoval `FEDERACE-PLATBY-02` jako aktuální P1 před dosavadním pořadím. Schválená pravidla rozúčtování: přesná částka se rozdělí podle očekávaného pojistného; nedoplatek pouze ručně se shodným součtem; přeplatek se členům nepřipíše a zůstane nepřiřazený na hlavní platbě.
- D-015 (2026-08-10): Product Owner schválil `FED-UDALOSTI-HISTORY-01` jako aktuální P1. Autoritativním rokem události je rok data vzniku; chybějící historický záznam musí vyvolat srozumitelnou chybu bez fallbacku.
- D-016 (2026-08-10): Product Owner schválil navazující `FED-CONTACT-CALC-01` jako P1. Telefon bude textový údaj v pracovním schématu a poměrné pojistné se zaokrouhlí nahoru až po dokončení celého výpočtu.
- D-017 (2026-08-10): Product Owner rozhodl, že přesná tisková shoda poštovní poukázky, obálky a adresního štítku s Accessem není požadována. Existující základní výstupy se nemažou, ale jejich rozměry, rozmístění a vizuální dorovnání nejsou součástí cílové parity ani sprintu dokumentů.
- D-018 (2026-08-10): Product Owner schválil `FED-PARITY-DOCUMENTS-01` jako sprint 2 a přesunul jej před migraci historických událostí. Scope tvoří doklad, přihláška, hlášení pojistné události a aktivní sestavy OC/ZO/HVP; poštovní poukázka, obálka a adresní štítek zůstávají podle D-017 mimo scope.
- D-019 (2026-08-10): Product Owner po dokončení sprintu 2 schválil jako sprint 3 `FED-PARITY-CLAIMS-MIGRATION-01`. Migrace 112 původních událostí musí být idempotentní, zachovat původní ID a nesmí automaticky odhadovat nejednoznačné historické vazby.
- D-020 (2026-08-10): Product Owner rozhodl přeskočit `FED-PARITY-ARCHIVE-LEGACY-01` (sprint 4). Zpřístupnění 4 386 archivních záznamů z let 2002–2010 se vrací do backlogu, není zrušeno a nesmí být implementováno bez nového výslovného schválení.
- D-021 (2026-08-10): Product Owner rozhodl přeskočit `FED-PARITY-INVOICES-LEGACY-01` (sprint 5). Zpřístupnění 4 652 původních faktur a poukázek se vrací do backlogu, není zrušeno a nesmí být implementováno bez nového výslovného schválení.
- D-022 (2026-08-10): Product Owner schválil `FED-PARITY-REPORT-HISTORY-01` jako aktuální sprint 6 s prioritou P1. Historie 141 záznamů z tabulek `Sestavy` a `SestavyHVP` bude zpřístupněna pouze pro čtení a bez změny původních dat.
- D-023 (2026-08-10): Product Owner schválil `FED-PARITY-WORKFLOWS-01` jako aktuální sprint 7 s prioritou P1. Side-by-side audit pojištěnců, tarifů, plateb, příkazů a ročního převodu musí proběhnout nad izolovanými kopiemi; implementovat lze pouze jednoznačně prokázané odchylky.
- D-024 (2026-08-10): Sprint 7 neprokázal novou aplikační chybu. Rozdíly ročních agregací odpovídají již známým rozdílům zdrojových snímků. Roční převod Federace se eviduje jako bezpečná náhrada Access VBA: zachovává obchodní výsledek, ale přidává zálohu, transakci, idempotenci, výjimky a verzované tarify. Historická data ani odložené finanční workflow se bez dalšího schválení nemění.
- D-025 (2026-08-12): Product Owner akceptoval dokončený sprint 7 a výslovně schválil přípravu i publikaci ostrého release `v0.23.0`. Release obsahuje dokončené parity sprinty od produkčního tagu `v0.22.0`; další produktový sprint se tím automaticky neschvaluje.
- D-026 (2026-08-27): Product Owner schválil `PLATBY-003` jako aktuální sprint s prioritou P1 v M1. Oprava smí měnit pouze zdroj, výpočet, validaci a zobrazení částek hromadného připisování plateb; bez migrace dat a redesignu modulu.
- D-027 (2026-08-27): `PLATBY-003` zachovává původní Access význam sloupců: `RočPojistné` je pojistná částka/limit a `PojistnáČástka` je skutečné pojistné. Historická data se nepřepočítávají ani nemigrují; oprava mění pouze jejich správné použití v organizačních platbách.
- D-028 (2026-08-27): Product Owner akceptoval `PLATBY-003` a schválil `DOKLADY-001` jako aktuální P1 sprint v M1. Vyhledávání členů zůstává beze změny; opravuje se pouze načtení podkladů a bezpečné vystavení dokladu k existující platbě.
- D-029 (2026-08-27): `DOKLADY-001` respektuje existující pravidlo jednoho dokladu člena za pojistný rok. Původní souhrnná úhrada bez detailního řádku se při vystavení dokladu propojí přes již zavedený idempotentní technický platební záznam; nevzniká nová finanční hodnota a úhrada člena se nemění.
- D-030 (2026-08-27): Product Owner akceptoval `DOKLADY-001` a schválil ostrý release `v0.24.0`. Release balí pouze dokončené sprinty `PLATBY-003` a `DOKLADY-001`; neschvaluje žádný další produktový sprint.
- D-031 (2026-09-01): Product Owner pokynem zahájit postupně sprinty aktivoval jako první `FED-DATA-AUTHORITY-01`. Sprint je pouze read-only: ověří existující rozdílový audit a připraví rozhodnutí o autoritativním snímku; nesmí provést synchronizaci ani jinou změnu dat. Volba autoritativní varianty zůstává samostatným rozhodnutím Product Ownera.
- D-032 (2026-09-01): Product Owner potvrdil jako autoritativní datový snímek produkční Access `Pojištění.accdb` s SHA-256 `DF176160A48085BDA225FCAB3D8A9194371E2EBEE43C9EB27CA497D696E0DEDC`. Rozhodnutí uzavírá `FED-DATA-AUTHORITY-01`, ale neschvaluje synchronizaci 36 změněných, 2 Access-only ani odstranění 10 SQLite-only řádků; jakákoli synchronizace vyžaduje nový samostatný vratný sprint. Podle schváleného pořadí se aktivuje pouze ověřovací sprint `FED-SMOKE-PAYMENTS-01`.

Důvodem rozhodnutí je kontinuita provozu, jednoduchost, ochrana dat a řízený dlouhodobý vývoj.
