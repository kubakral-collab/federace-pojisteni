# Backlog

| ID | Název | Priorita | Milestone | Stav | Závislosti | Důvod |
|---|---|---|---|---|---|---|
| PARITY-000 | Rozdílový audit produkčního Accessu a `dd.sqlite` | P0 | M1 — Stabilizace Core | `FED-DATA-AUTHORITY-01` dokončen a akceptován; autoritativní je produkční Access | případná synchronizace vyžaduje samostatný bezpečný migrační sprint | Zjištěno 14 370 přesně shodných řádků, 36 změněných, 2 pouze v Accessu a 10 pouze v SQLite; žádná synchronizace nebyla provedena. |
| PARITY-001 | Zpřístupnění 112 původních pojistných událostí | P0 | M1 — Stabilizace Core | Dokončeno, otestováno a akceptováno Product Ownerem | 98 přesných vazeb; 14 záměrně read-only; idempotence ověřena | Všech 112 je viditelných, původní ID a všechna zdrojová pole jsou zachována. |
| PARITY-002 | Archiv ročních tabulek 2002–2010 | P0 | M1 — Stabilizace Core | Návrh sprintu `FED-PARITY-ARCHIVE-LEGACY-01`; odloženo D-020, bez nového schválení neimplementovat | sjednocený read-only model; kontrolní počty 4 386 řádků | Současný Archiv čte pouze `Seznam` a starší roky nejsou uživatelsky dostupné. |
| PARITY-003 | Původní faktury a význam finančního workflow | P0 | M1 — Stabilizace Core | Návrh sprintu `FED-PARITY-INVOICES-LEGACY-01`; odloženo D-021, bez nového schválení neimplementovat | mapování `Faktura` ↔ nové finanční tabulky; zákaz omylem exportovat historii | Nová kniha nezobrazuje 4 652 původních řádků a její obchodní význam není proti Accessu potvrzen. |
| PARITY-004 | Obsahová a provozní shoda aktivních dokumentů a Access sestav | P1 | M1 — Stabilizace Core | Dokončeno, otestováno a akceptováno Product Ownerem | regresní PDF fixture; Access definice; referenční PDF událostí | Doklad, přihláška, událost, OC, ZO a HVP dorovnány. Pixelová identita není cílem; D-017 vyřazuje poukázku, obálku a štítek. |
| PARITY-005 | Historie 141 vystavených sestav | P1 | M1 — Stabilizace Core | Dokončeno, otestováno a akceptováno Product Ownerem | read-only tabulky `Sestavy`, `SestavyHVP` | Všech 141 záznamů je dostupných odděleně od nového auditu exportů. |
| PARITY-006 | Side-by-side ověření ročního převodu a tarifů | P1 | M1 — Stabilizace Core | Dokončeno v `FED-PARITY-WORKFLOWS-01`; bezpečná náhrada zdokumentována | read-only agregátní audit; automatické regresní testy | Rozdíly let 2018, 2023, 2025 a 2026 odpovídají rozdílným zdrojovým snímkům; aplikační chyba nebyla prokázána. |
| PARITY-007 | Klasifikace `Seznam_`, `Odklad` a `Břeclav` | P2 | M1 — Stabilizace Core | Návrh sprintu `FED-PARITY-AUXILIARY-DATA-01`; čeká na schválení | porovnání identit bez zápisu | Pomocná data nesmějí být ignorována ani automaticky sloučena. |
| PARITY-008 | Rozhodnutí o externí členské základně a e-mailovém Excelu | P2 | M1 — Stabilizace Core | Návrh sprintu `FED-PARITY-EXTERNAL-SOURCES-01`; čeká na schválení | dostupnost externích zdrojů a potvrzení provozní potřeby | Původní Access používal dvě připojené tabulky, které nebyly součástí lokální migrace. |
| UDALOSTI-001 | Historické pojištění podle data vzniku události | P1 | M1 — Stabilizace Core | Implementováno; smoke test vypsán jako `FED-SMOKE-CLAIM-HISTORY-01` | historie `Seznam`; datum vzniku | Zabránit použití současných pojistných údajů u historické události. |
| CONTACT-CALC-001 | Telefon člena a zaokrouhlení poměrného pojistného nahoru | P1 | M1 — Stabilizace Core | Implementováno; smoke test vypsán jako `FED-SMOKE-CONTACT-CALC-01` | bezpečná migrace Telefon; tarifní výpočet | Doplnit provozní kontakt a odstranit podhodnocení desetinného výsledku. |
| PLATBY-002 | Individuální a organizační platby s rozúčtováním | P1 | M2 — Provozní automatizace | Implementováno; `FED-SMOKE-PAYMENTS-01` aktivní, 16 automatických testů prošlo, čeká uživatel | současná evidence členů a plateb; bezpečná migrace | Evidovat jednu přijatou organizační platbu a dohledatelný rozpis bez svévolného dělení nedoplatku či přeplatku. |
| PLATBY-003 | Oprava částek při hromadném připisování plateb | P1 | M1 — Stabilizace Core | Dokončeno, otestováno a akceptováno Product Ownerem | existující období, uložené pojistné a platby členů | Odděleno bez migrace; doplatek i backendová validace používají skutečné pojistné a úhradu stejného roku. |
| DOKLADY-001 | Oprava načítání podkladů a doklad z existující platby | P1 | M1 — Stabilizace Core | Dokončeno, otestováno a akceptováno Product Ownerem | existující platby, evidence dokladů, vazba na pojistný rok | Opraven číselný historický identifikátor; doklad je propojen s existující platbou, zachovává úhradu a nevytváří duplicitu. |
| PLATBY-001 | Individuální a organizační platby | P2 | M2 — Provozní automatizace | Nahrazeno dokončenou položkou PLATBY-002; samostatně neimplementovat | PLATBY-002 | Původní návrh byl zpřesněn a realizován včetně řízeného rozúčtování; zbývá uživatelský smoke test. |
| UX-001 | Pořadí položek „Nový pojištěnec“ a „Seznam pojištěnců“ | P2 | M1 — Stabilizace Core | Sprint `FEDERACE-MENU-01` schválen, čeká na aktivaci | dokončení nebo uzavření S1; existující konfigurace levého menu | Zjednodušit přirozený začátek práce bez změny funkčnosti navigace. |
| UX-002 | Uživatelské nastavení velikosti písma | P2 | M1 — Stabilizace Core | Návrh sprintu `FED-UX-FONT-01`, čeká na schválení Product Ownerem | audit typografie a responzivity všech hlavních obrazovek; trvalé uživatelské nastavení | Zpřístupnit celou aplikaci uživatelům vyžadujícím větší text bez nekontrolovaného zoomu a rozbití UI. |
| ZMĚNA-003 | Automatické každoroční příkazy a odeslání | P2 | M2 | Sprint `FED-AUTOMATION-ANNUAL-ORDERS-01` vypsán, čeká na aktivaci | rok, sazby, SMTP | Omezit ruční práci. |
| ZMĚNA-004 | Bankovní import a párování | P2 | M2 | Sprint `FED-BANK-IMPORT-01` vypsán, čeká na aktivaci | platby, pravidla párování | Odstranit přepis plateb. |
| BEZPEČNOST-002 | SQLCipher | P2 | M3 | Sprint `FED-SECURITY-SQLCIPHER-01` vypsán, čeká na aktivaci | migrace, recovery, audit | Šifrování osobních dat. |
| BEZPEČNOST-004 | Úplný audit citlivých operací | P2 | M3 | Částečně implementováno; návrh sprintu `FED-SECURITY-AUDIT-01` | jednotný audit | Dohledatelnost změn. |

## UX-002 — Uživatelské nastavení velikosti písma

- **Navržené označení sprintu:** `FED-UX-FONT-01`
- **Volby:** 100 % (Výchozí), 110 % (Větší), 125 % (Velké), 150 % (Velmi velké).
- **Scope:** systémové škálování typografie a souvisejících rozměrů komponent v celé aplikaci; levé menu, seznamy, tabulky, detaily, formuláře, Platby, Pojistné události, tlačítka, dialogy, nadpisy a běžné texty.
- **Uložení:** trvalá uživatelská volba zachovaná po restartu aplikace.
- **Náhled:** okamžitě aktualizovaný ukázkový text přímo v Nastavení.
- **Akceptace:** všechny čtyři velikosti zůstávají použitelné na Hlavním panelu, v Seznamu pojištěnců, Detailu, Platbách, Pojistných událostech a Nastavení; bez překryvů, oříznutí, přetékání nebo rozbití tabulek a dialogů; volba přetrvá restart.
- **Mimo scope:** změna barevnosti, pořadí či rozložení menu a jiné designové změny nad rámec podpory větší typografie.
- **Produktové rozhodnutí před implementací:** schválit prioritu, Milestone a pořadí sprintu.

## Dokončená historie
BUG-001 platební filtr; FUNKCE-001 bezpečný roční převod; REPORT-001 sestavy OC/ZO/HVP; ZMĚNA-001 live search; ZMĚNA-002 akce v detailu; ZMĚNA-005 updater; BEZPEČNOST-001 ruční backup/recovery; BEZPEČNOST-003 přihlášení (zamykání zbývá); pojistné události a přehled pro pojišťovnu jsou implementované. Podrobné důkazy jsou v `ACCESS_PARITY_COMPLETION.md`.
