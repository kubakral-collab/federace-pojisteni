# Sprint FED-PARITY-CLAIMS-MIGRATION-01 — výsledek

Dokončeno 2026-08-10. Priorita P0, milestone M1 — Stabilizace Core.

## Výsledek migrace

- Zdrojová tabulka `Poj_udalost`: 112 řádků, 112 unikátních původních ID v rozsahu 1–115.
- Bezeztrátový archiv `MigracePojistnychUdalostiAccess`: 112 řádků se všemi 16 původními poli a stavem vazby.
- Jednoznačně napojené události v pracovní agendě `PojistneUdalosti`: 98.
- Fronta historických vazeb bez automatického odhadu: 14.
  - 2 záznamy bez data vzniku (`CHYBI_DATUM`),
  - 11 záznamů bez pojistného záznamu pro přesný rok (`CHYBI_ROK`),
  - 1 záznam se dvěma kandidáty ve stejném roce (`NEJEDNOZNACNA`).

Nevyřešená původní ID: 28, 33, 34, 51, 52, 53, 54, 59, 61, 64, 65, 75, 76 a 95. Jde pouze o technická ID událostí, nikoli osobní údaje.

## Chování aplikace

- Centrální agenda zobrazuje všech 112 původních událostí.
- Jednoznačně napojených 98 událostí funguje jako běžné události včetně detailu člena a PDF.
- Zbývajících 14 je viditelných jako historické záznamy s upozorněním „historická vazba k dořešení“.
- U nevyřešených vazeb jsou úprava, PDF a přechod na konkrétní pojistný záznam zablokovány, aby aplikace nepoužila nesprávný rok.
- Diagnostika počítá vyřešené i nevyřešené události.

## Bezpečnost a idempotence

Migrace používá původní ID a `INSERT OR IGNORE`. Opakované spuštění nevytváří duplicity. Zdrojová tabulka `Poj_udalost` zůstává nedotčena. Ověření proběhlo na izolované kopii `dd.sqlite`; hash zdrojového souboru před a po testu zůstal `F8F90FDEEDEDFED1159D4AA2F24B52AA5B96B94A9726201DD57CDD2F63B93D2D`.

Porovnání všech 16 původních polí mezi zdrojem a archivem potvrdilo sémantickou shodu všech 112 řádků. Rozdíly SQLite typové afinity (například celé číslo oproti numericky ekvivalentnímu desetinnému typu) nejsou změnou hodnoty.

## Testy

- Migrace syntetických stavů: PASS.
- Dvojí spuštění migrace bez duplicit: PASS.
- Reálná izolovaná kopie: 112 archivováno / 98 napojeno / 14 ve frontě / 112 viditelných unikátních ID: PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib`: 64 testů PASS.
- `npm.cmd run build`: PASS.

## Další produktové rozhodnutí

Těchto 14 vazeb nesmí být opraveno odhadem. Jejich případné ruční přiřazení vyžaduje samostatně schválený postup a znalost správného historického pojistného záznamu.
