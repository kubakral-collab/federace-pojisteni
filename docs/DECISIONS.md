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

Důvodem rozhodnutí je kontinuita provozu, jednoduchost, ochrana dat a řízený dlouhodobý vývoj.
