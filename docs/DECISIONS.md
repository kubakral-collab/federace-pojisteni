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

Důvodem rozhodnutí je kontinuita provozu, jednoduchost, ochrana dat a řízený dlouhodobý vývoj.
