# Sprint FED-PARITY-WORKFLOWS-01

Datum dokončení: 2026-08-10

Priorita: P1

Milestone: M1 — Stabilizace Core

## Výsledek

Side-by-side audit provozních workflow byl dokončen nad produkčním Accessem a pracovním `dd.sqlite`. Zdrojové databáze byly otevřeny pouze pro čtení a jejich SHA-256 zůstal nezměněn.

- Access: `DF176160A48085BDA225FCAB3D8A9194371E2EBEE43C9EB27CA497D696E0DEDC`, 14 408 řádků `Seznam`.
- SQLite: `F8F90FDEEDEDFED1159D4AA2F24B52AA5B96B94A9726201DD57CDD2F63B93D2D`, 14 416 řádků `Seznam`.
- Dřívější řádkové porovnání zůstává platné: 14 370 přesných shod, 36 změněných řádků, 2 pouze v Accessu a 10 pouze v SQLite.
- Roční agregace se shodují s výjimkou let 2018, 2023, 2025 a 2026. Rozdíly přesně odpovídají rozdílným zdrojovým řádkům; nejde o nově zjištěnou chybu výpočtu Federace.
- Ve zdroji jsou také dva zjevně neplatné roky `200` a `218`. Audit je pouze hlásí a data neopravuje.

## Posouzení workflow

| Oblast | Závěr |
|---|---|
| Pojištěnci | Datový význam aktivního `Seznam` je zachován. Rozdílné zdrojové snímky jsou již zdokumentované a bez rozhodnutí Product Ownera se neslučují. |
| Tarify a pojistné | Federace používá verzované sazby podle data a poměrnou část zaokrouhluje nahoru až po dokončení výpočtu. Historicky uložené pojistné se nepřepočítává současnou sazbou. Access měl sazby ročního převodu pevně ve VBA; jejich doslovné převzetí by poškodilo novější historická data. |
| Platby | Původní agregát `SkutÚhrada` zůstává kompatibilní. Nová kniha individuálních a organizačních plateb je auditovatelné rozšíření; původní Access neobsahoval rovnocennou historii jednotlivých plateb. |
| Příkazy k úhradě | Federace poskytuje dohledatelný příkaz, PDF a dávku. Jde o bezpečnou moderní náhradu. Historické faktury/poukázky jsou mimo tento sprint rozhodnutím D-021. |
| Roční převod | Access kopíroval řádky přes pracovní tabulku a měl sazby natvrdo. Federace převádí pouze způsobilé aktivní záznamy, respektuje výjimky, vyžaduje platný tarif, vytváří zálohu, používá transakci a je idempotentní. Jde o vědomou bezpečnou náhradu, nikoli o doslovné kopírování rizikového postupu. |

## Rozhodnutí a omezení

Audit neprokázal jednoznačnou programovou odchylku, kterou by bylo bezpečné v tomto sprintu automaticky opravit. Nebyla proto změněna aplikační logika ani databázové schéma. Zejména nebyly přepisovány historické částky, slučovány rozdílné zdroje ani obnovovány odložené faktury.

Pro opakovatelnou kontrolu vznikl read-only nástroj `scripts/audit-workflow-parity.py`. Vypisuje pouze agregace a hashe, nikoli osobní údaje.

## Ověření

- audit produkčního Accessu proti `dd.sqlite`: PASS;
- kontrola syntaxe auditního skriptu: PASS;
- Rust testy: PASS, 66/66;
- frontend build: PASS (`npm.cmd run build`, TypeScript + Vite).
