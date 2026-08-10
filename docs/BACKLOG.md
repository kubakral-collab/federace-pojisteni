# Backlog

| ID | Název | Priorita | Milestone | Stav | Závislosti | Důvod |
|---|---|---|---|---|---|---|
| PARITY-000 | Rozdílový audit produkčního Accessu a `dd.sqlite` | P0 | M1 — Stabilizace Core | Audit odhalil rozpor, čeká na schválení read-only sprintu | potvrzení autoritativního zdroje Product Ownerem | Produkční Access má 14 408 členových řádků, `dd.sqlite` 14 416; před migracemi je nutné přesně určit osm rozdílů. |
| PARITY-001 | Zpřístupnění 112 původních pojistných událostí | P0 | M1 — Stabilizace Core | Auditováno, čeká na schválení opravného sprintu | idempotentní migrace; ruční fronta nejasných ročních vazeb | Nová agenda nyní čte jinou tabulku a původní události nezobrazuje. |
| PARITY-002 | Archiv ročních tabulek 2002–2010 | P0 | M1 — Stabilizace Core | Auditováno, čeká na schválení opravného sprintu | sjednocený read-only model; kontrolní počty 4 386 řádků | Současný Archiv čte pouze `Seznam` a starší roky nejsou uživatelsky dostupné. |
| PARITY-003 | Původní faktury a význam finančního workflow | P0 | M1 — Stabilizace Core | Auditováno, čeká na schválení opravného sprintu | mapování `Faktura` ↔ nové finanční tabulky; zákaz omylem exportovat historii | Nová kniha nezobrazuje 4 652 původních řádků a její obchodní význam není proti Accessu potvrzen. |
| PARITY-004 | Přesná shoda dokumentů a 31 Access sestav | P1 | M1 — Stabilizace Core | Auditováno, vyžaduje referenční scénáře a snapshoty | anonymizované fixture; referenční PDF; tisková kontrola | Obecné PDF/CSV není důkazem stejného obsahu, součtů a rozložení. |
| PARITY-005 | Historie 141 vystavených sestav | P1 | M1 — Stabilizace Core | Auditováno, čeká na návrh read-only zpřístupnění | tabulky `Sestavy`, `SestavyHVP` | Nový audit sestav eviduje jen nové exporty. |
| PARITY-006 | Side-by-side ověření ročního převodu a tarifů | P1 | M1 — Stabilizace Core | Automatické testy existují, provozní shoda neověřena | izolovaná kopie stejného roku v Accessu a Federaci | Potvrdit kontrolní počty, sazby, ukončení, období a úhrady na reálné struktuře. |
| PARITY-007 | Klasifikace `Seznam_`, `Odklad` a `Břeclav` | P2 | M1 — Stabilizace Core | Čeká na report duplicit a rozhodnutí Product Ownera | porovnání identit bez zápisu | Pomocná data nesmějí být ignorována ani automaticky sloučena. |
| PARITY-008 | Rozhodnutí o externí členské základně a e-mailovém Excelu | P2 | M1 — Stabilizace Core | Čeká na Product Ownera | dostupnost externích zdrojů a potvrzení provozní potřeby | Původní Access používal dvě připojené tabulky, které nebyly součástí lokální migrace. |
| UDALOSTI-001 | Historické pojištění podle data vzniku události | P1 | M1 — Stabilizace Core | Implementováno, čeká na uživatelský smoke test | historie `Seznam`; datum vzniku | Zabránit použití současných pojistných údajů u historické události. |
| CONTACT-CALC-001 | Telefon člena a zaokrouhlení poměrného pojistného nahoru | P1 | M1 — Stabilizace Core | Implementováno, čeká na uživatelský smoke test | bezpečná migrace Telefon; tarifní výpočet | Doplnit provozní kontakt a odstranit podhodnocení desetinného výsledku. |
| PLATBY-002 | Individuální a organizační platby s rozúčtováním | P1 | M2 — Provozní automatizace | Implementováno, čeká na uživatelský smoke test | současná evidence členů a plateb; bezpečná migrace | Evidovat jednu přijatou organizační platbu a dohledatelný rozpis bez svévolného dělení nedoplatku či přeplatku. |
| PLATBY-001 | Individuální a organizační platby | P2 | M2 — Provozní automatizace | Schváleno, čeká na zařazení do pořadí sprintů | audit současného modelu plateb; identifikace organizace; bezpečná migrace | Evidovat platby za člena i organizaci přímo z modulu Platby bez vymyšleného rozúčtování. |
| UX-001 | Pořadí položek „Nový pojištěnec“ a „Seznam pojištěnců“ | P2 | M1 — Stabilizace Core | Schváleno, čeká na zařazení po aktuálním sprintu | dokončení nebo uzavření S1; existující konfigurace levého menu | Zjednodušit přirozený začátek práce bez změny funkčnosti navigace. |
| UX-002 | Uživatelské nastavení velikosti písma | P2 | M1 — Stabilizace Core | Návrh v Backlogu, čeká na schválení Product Ownerem | audit typografie a responzivity všech hlavních obrazovek; trvalé uživatelské nastavení | Zpřístupnit celou aplikaci uživatelům vyžadujícím větší text bez nekontrolovaného zoomu a rozbití UI. |
| ZMĚNA-003 | Automatické každoroční příkazy a odeslání | P2 | M2 | Schváleno, bez sprintu | rok, sazby, SMTP | Omezit ruční práci. |
| ZMĚNA-004 | Bankovní import a párování | P2 | M2 | Schváleno, bez sprintu | platby, pravidla párování | Odstranit přepis plateb. |
| BEZPEČNOST-002 | SQLCipher | P2 | M3 | Schváleno, bez sprintu | migrace, recovery | Šifrování osobních dat. |
| BEZPEČNOST-004 | Úplný audit citlivých operací | P2 | M3 | Částečně implementováno | jednotný audit | Dohledatelnost změn. |

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
