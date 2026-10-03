# Roadmapa

Aktualizováno 2026-09-01. Jediný zdroj pravdy pro plán vývoje. Aktuální ověřený stav migrace Accessu je v [ACCESS_PARITY_AUDIT_AND_PLAN.md](ACCESS_PARITY_AUDIT_AND_PLAN.md).

## Vize
Jednoduchá, bezpečná a aktualizovatelná náhrada Accessu pro jednoho správce, zachovávající provozní logiku a omezující ruční práci.

## Produktová pravidla
- **PR-001 Roadmap First:** žádný sprint mimo roadmapu; nový nápad nejprve do Backlogu/Parking Lotu; sprint až po schválení Product Ownerem.
- **PR-002 Release Management:** sprint ani commit není release; více sprintů tvoří release; Development → Stabilizace → RC → Produkční release. Okamžitý release jen pro P0, bezpečnost, pád, ztrátu dat nebo nefunkční updater.
- **PR-003 Stabilita před funkcemi:** stabilita a ochrana dat mají vždy přednost.
- **PR-004 Dokumentace jako zdroj pravdy:** rozhodnutí platí až zápisem; Lead Developer bez schválení nemění roadmapu, priority ani release plán.

## Role a priority
Product Owner schvaluje roadmapu, sprinty a release. CTO řídí architekturu, priority a release management. Lead Developer implementuje pouze schválené sprinty a udržuje dokumentaci.

P0 kritická chyba; P1 vysoká priorita; P2 plánovaná funkce; P3 budoucí rozvoj.

## Ověřený stav

Ověřeno 2026-08-31 proti produkční verzi `v0.24.0`, aktuálnímu kódu, frontend buildu a 70 backendovým testům.

**Implementováno a automaticky ověřeno:** hlavní evidence pojištěnců; tarify; individuální a organizační platby; nové faktury a platební dávky; doklady, přihláška a provozní sestavy; nové i všech 112 původních pojistných událostí; historie 141 sestav; roční převod; import členů; přihlášení; audit vybraných operací; zálohy; CI/CD a updater.

**Není dokončeno:** uživatelské zpřístupnění 4 386 archivních záznamů 2002–2010 a 4 652 původních faktur/poukázek. Obě oblasti byly odloženy rozhodnutími D-020 a D-021 a vyžadují nové schválení. K rozhodnutí zůstávají také pomocné tabulky, externí zdroje a autoritativní datový snímek.

**Čeká na provozní smoke test:** organizační platby, historický kontext události, telefon a zaokrouhlení pojistného, obnova zálohy a případně fyzický tisk. Podrobnosti jsou v [ACCESS_PARITY_AUDIT_AND_PLAN.md](ACCESS_PARITY_AUDIT_AND_PLAN.md).

## Milestones
- **M1 Funkční náhrada Accessu — stabilizace:** hlavní agendy a roční převod jsou implementované; zbývají odložená historická data, rozhodnutí o pomocných zdrojích a provozní smoke testy.
- **M2 Provozní automatizace — plánováno:** automatické každoroční příkazy a odesílání, bankovní import a párování. Bezpečný roční převod je již dokončen v M1.
- **M3 Ochrana dat — plánováno:** SQLCipher, šifrované zálohy, zamykání, úplný audit.
- **M4 Sestavy — dokončeno pro schválený aktivní rozsah:** OC/ZO/HVP, přihláška, doklad a událost jsou implementované a akceptované; přesná shoda poukázky, obálky a štítku je rozhodnutím D-017 mimo scope.

## Aktuální sprint

`FED-SMOKE-PAYMENTS-01` — aktivní P1 ověřovací sprint. Automatická brána prošla (16 testů, 0 selhání); zbývá zápisový uživatelský smoke test pouze na pracovní kopii databáze.

## Navržené pořadí všech nedokončených sprintů

Tato tabulka obsahuje **všechny známé nedokončené položky** z roadmapy, backlogu, aktuálního parity auditu a Parking Lotu. Pořadí je návrh realizace podle rizika a závislostí. Zápis do tabulky není automatickým schválením sprintu:

- `Čeká na schválení` — backlog položka může začít až po výslovném schválení Product Ownerem.
- `Vyžaduje obnovení` — dříve odloženo rozhodnutím D-020 nebo D-021; implementace je zakázaná bez nového výslovného schválení.
- `Námět P3` — položka z Parking Lotu; před sprintem musí být schválen její přesun do backlogu, priorita i Milestone.

| Pořadí | Sprint | Priorita | Milestone | Výsledek sprintu | Závislosti | Stav schválení |
| ---: | --- | --- | --- | --- | --- | --- |
| 1 | `FED-DATA-AUTHORITY-01` | P0 | M1 | Product Owner potvrdí autoritativní datový snímek a písemně rozhodne, zda vznikne samostatná synchronizace 36 změněných, 2 Access-only a 10 SQLite-only řádků. Sprint nemění data. | dokončený `PARITY-000` | **Dokončeno a akceptováno; autoritativní je produkční Access** |
| 2 | `FED-SMOKE-PAYMENTS-01` | P1 | M1 | Uživatel na provozní kopii ověří individuální platbu, přesné rozdělení organizační platby, ruční nedoplatek a nepřiřazený přeplatek; výsledek bude zapsán. | `PLATBY-002`, `PLATBY-003` | **Aktivní; automatická brána prošla, čeká uživatelský smoke test** |
| 3 | `FED-SMOKE-CLAIM-HISTORY-01` | P1 | M1 | Uživatel ověří, že historická událost používá pojistný záznam podle roku vzniku a při chybějícím roku bezpečně skončí bez fallbacku. | `UDALOSTI-001` | Čeká na schválení smoke testu |
| 4 | `FED-SMOKE-CONTACT-CALC-01` | P1 | M1 | Uživatel ověří uložení telefonu a poměrné pojistné zaokrouhlené nahoru na reprezentativních obdobích. | `CONTACT-CALC-001` | Čeká na schválení smoke testu |
| 5 | `FED-SMOKE-BACKUP-RESTORE-01` | P1 | M1 | Na uživatelské kopii proběhne vytvoření, kontrola a obnova zálohy včetně nouzové zálohy a kontroly výsledných dat. | hotová správa záloh | Čeká na schválení smoke testu |
| 6 | `FED-SMOKE-PRINT-01` | P1 | M1 | Na provozní tiskárně budou ověřeny aktivní PDF dokladu, přihlášky, události a sestav; evidují se pouze skutečné provozní vady. | dokončený dokumentový sprint | Podmíněno potvrzením, že fyzický tisk je akceptační požadavek |
| 7 | `FED-PARITY-AUXILIARY-DATA-01` | P2 | M1 | Read-only report porovná `Seznam_` (4 220), `Odklad` (76) a `Břeclav` (23) s aktivním `Seznam`; Product Owner rozhodne pro každý zdroj archiv/import/vyřazení. Bez automatického merge. | `FED-DATA-AUTHORITY-01` | Čeká na schválení |
| 8 | `FED-PARITY-EXTERNAL-SOURCES-01` | P2 | M1 | Product Owner rozhodne, zda je stále potřeba `Členská základna` a Excel `Email`; pokud ano, vznikne samostatná specifikace bezpečné integrace, nikoli implementace naslepo. | dostupnost externích zdrojů | Čeká na schválení |
| 9 | `FED-PARITY-ARCHIVE-LEGACY-01` | P0 | M1 | Read-only Archiv zpřístupní přesně 4 386 řádků z let 2002–2010, se součty, hledáním, původem záznamu a bez změny zdrojových tabulek. | rozhodnutí D-020, datová autorita | **Vyžaduje obnovení po D-020** |
| 10 | `FED-PARITY-INVOICES-LEGACY-01` | P0 | M1 | Samostatný read-only pohled zpřístupní přesně 4 652 původních faktur/poukázek a zabrání jejich zařazení do nových platebních dávek. | rozhodnutí D-021, potvrzení významu Access `Faktura` | **Vyžaduje obnovení po D-021** |
| 11 | `FEDERACE-MENU-01` | P2 | M1 | „Nový pojištěnec“ bude bezprostředně nad „Seznam pojištěnců“; názvy, routy, ikony a ostatní pořadí zůstanou beze změny. | uzavření M1 stabilizačních rozhodnutí | Schváleno, čeká na aktivaci |
| 12 | `FED-UX-FONT-01` | P2 | M1 | Trvalá volba 100/110/125/150 % bude fungovat na hlavních obrazovkách bez překryvů, oříznutí a rozbití tabulek či dialogů. | audit responzivity | Čeká na schválení Product Ownerem |
| 13 | `FED-AUTOMATION-ANNUAL-ORDERS-01` | P2 | M2 | Řízené vytvoření každoročních příkazů s náhledem, ochranou proti duplicitám, SMTP odesláním a auditem; žádné automatické odeslání bez potvrzení. | roční převod, sazby, SMTP | Schválený backlog, čeká na aktivaci |
| 14 | `FED-BANK-IMPORT-01` | P2 | M2 | Import bankovního výpisu s náhledem, idempotencí, přesným a ručním párováním, nepřiřazenou frontou a auditem. | stabilní platební workflow, schválené formáty bank | Schválený backlog, čeká na aktivaci |
| 15 | `FED-SECURITY-AUDIT-01` | P2 | M3 | Jednotný audit pokryje všechny citlivé zápisy, exporty, obnovu, přihlášení a administraci bez osobních údajů v textu logu. | inventář současných auditních tabulek | Částečně implementováno; čeká na schválení sprintu |
| 16 | `FED-SECURITY-SQLCIPHER-01` | P2 | M3 | Databáze bude šifrovaná, migrace vratná, klíč bezpečně uložený a obnova ověřená bez ztráty dat. | `FED-SECURITY-AUDIT-01`, recovery test, návrh správy klíče | Schválený backlog, čeká na aktivaci |
| 17 | `FED-SECURITY-ENCRYPTED-BACKUPS-01` | P3 | M3 | Zálohy budou šifrované, autentizované, obnovitelné a kompatibilita bude před obnovou ověřena. | návrh správy klíče, SQLCipher/recovery rozhodnutí | Námět P3; čeká na přesun z Parking Lotu |
| 18 | `FED-SECURITY-AUTOLOCK-01` | P3 | M3 | Po schválené době nečinnosti se relace bezpečně zamkne bez ztráty rozpracovaných dat a vyžádá nové ověření. | autentizace, návrh zachování formuláře | Námět P3; čeká na přesun z Parking Lotu |
| 19 | `FED-PAYMENT-REMINDERS-01` | P3 | M2 | Upomínky vzniknou pouze z ověřené platební bilance, nabídnou náhled, audit a ruční potvrzení odeslání. | `FED-BANK-IMPORT-01`, SMTP | Námět P3; čeká na přesun z Parking Lotu |
| 20 | `FED-ANALYTICS-01` | P3 | Budoucí rozvoj | Product Owner nejprve schválí konkrétní metriky; implementace zobrazí jen potřebné agregace bez obecného CRM scope. | produktová specifikace metrik | Námět P3; čeká na přesun z Parking Lotu |

## Definice jednotlivých sprintů

Každý sprint je samostatně schvalovatelný a musí skončit buď doloženým výsledkem, nebo písemným rozhodnutím. Sprint nesmí skrytě absorbovat další položku z tabulky.

### M1 — uzavření parity a provozní stabilizace

- **`FED-DATA-AUTHORITY-01`:** pouze rozhodovací/read-only sprint. Výstupem je identifikace autoritativního souboru, jeho hash a rozhodnutí o rozdílech; synchronizace dat je případný nový sprint.
- **`FED-SMOKE-PAYMENTS-01`:** pouze provozní ověření implementovaného platebního workflow; opravy nalezených vad se evidují samostatně podle závažnosti.
- **`FED-SMOKE-CLAIM-HISTORY-01`:** pouze provozní ověření historického kontextu události.
- **`FED-SMOKE-CONTACT-CALC-01`:** pouze provozní ověření telefonu a výpočtu poměrného pojistného.
- **`FED-SMOKE-BACKUP-RESTORE-01`:** kontrolovaná obnova výhradně na kopii, nikdy na jediné produkční databázi.
- **`FED-SMOKE-PRINT-01`:** provést jen po potvrzení potřeby fyzického tisku; pixelová shoda poukázky, obálky a štítku zůstává podle D-017 mimo scope.
- **`FED-PARITY-AUXILIARY-DATA-01`:** vytvoří důkaz pro rozhodnutí, ale žádná pomocná data nemigruje.
- **`FED-PARITY-EXTERNAL-SOURCES-01`:** rozhodne potřebu a rozhraní; připojení k externím systémům je případný navazující sprint.
- **`FED-PARITY-ARCHIVE-LEGACY-01`:** samostatná read-only dostupnost starých ročních tabulek, bez jejich slučování do aktivní evidence.
- **`FED-PARITY-INVOICES-LEGACY-01`:** samostatná read-only historie s výrazným oddělením od nových faktur a bankovních dávek.
- **`FEDERACE-MENU-01`:** jediná změna pořadí dvou již existujících položek menu.
- **`FED-UX-FONT-01`:** pouze globální škálování typografie a souvisejících rozměrů; žádný redesign.

### M2 — provozní automatizace

- **`FED-AUTOMATION-ANNUAL-ORDERS-01`:** náhled a potvrzení jsou povinné; automatizace nesmí rozeslat neověřenou dávku.
- **`FED-BANK-IMPORT-01`:** musí zachovat původní soubor, zabránit dvojímu importu a nikdy odhadem nepřiřadit nejednoznačnou platbu.
- **`FED-PAYMENT-REMINDERS-01`:** může začít až po stabilním bankovním párování a samostatném produktovém schválení.

### M3 — ochrana dat

- **`FED-SECURITY-AUDIT-01`:** sjednotí auditní model před šifrovací migrací a doloží pokrytí citlivých příkazů.
- **`FED-SECURITY-SQLCIPHER-01`:** před zápisem vyžaduje zálohu, ověřený rollback a recovery test.
- **`FED-SECURITY-ENCRYPTED-BACKUPS-01`:** nesmí znemožnit obnovu starších podporovaných záloh bez jasné migrační cesty.
- **`FED-SECURITY-AUTOLOCK-01`:** musí bezpečně řešit neuložený formulář a opětovné ověření uživatele.

### Budoucí rozvoj

- **`FED-ANALYTICS-01`:** bez schváleného seznamu metrik se sprint neaktivuje.

## Historie sprintů funkční parity

| Pořadí | Sprint | Priorita | Cíl | Stav |
|---:|---|---|---|---|
| 1 | `FED-PARITY-SOURCE-RECONCILE-01` | P0 | Read-only rozdílový audit produkčního Accessu a `dd.sqlite`; určit osm rozdílných řádků bez změny dat. | Dokončeno; 14 370 shodných, 36 změněných, 2 pouze Access, 10 pouze SQLite |
| 2 | `FED-PARITY-DOCUMENTS-01` | P1 | Ověřit a dorovnat aktivní doklad, přihlášku, událost a sestavy OC/ZO/HVP; přesná shoda poukázky, obálky a štítku je rozhodnutím D-017 mimo scope. | Dokončeno, otestováno a akceptováno PO |
| 3 | `FED-PARITY-CLAIMS-MIGRATION-01` | P0 | Zpřístupnit 112 původních pojistných událostí, zachovat ID a oddělit nejasné historické vazby. | Dokončeno, otestováno a akceptováno PO |
| 4 | `FED-PARITY-ARCHIVE-LEGACY-01` | P0 | Zpřístupnit 4 386 archivních záznamů z let 2002–2010 v read-only archivu. | Přeskočeno rozhodnutím D-020; vráceno do backlogu |
| 5 | `FED-PARITY-INVOICES-LEGACY-01` | P0 | Zpřístupnit 4 652 původních faktur/poukázek a potvrdit význam finančního workflow. | Přeskočeno rozhodnutím D-021; vráceno do backlogu |
| 6 | `FED-PARITY-REPORT-HISTORY-01` | P1 | Zpřístupnit historii 141 vystavených sestav. | Dokončeno, otestováno a akceptováno PO |
| 7 | `FED-PARITY-WORKFLOWS-01` | P1 | Side-by-side ověřit pojištěnce, tarify, platby, příkazy a roční převod. | Dokončeno, otestováno a akceptováno PO |
| 8 | `FED-PARITY-AUXILIARY-DATA-01` | P2 | Klasifikovat `Seznam_`, `Odklad`, `Břeclav` a externí zdroje bez automatického slučování. | Čeká na schválení po sprintu 7 |

Historická tabulka dokládá provedené pořadí a rozhodnutí. Neurčuje pořadí nového plánu výše.

## Schválený a sledovaný scope

### UX-001 — Pořadí položek v levém menu

- **Milestone:** M1 — Stabilizace Core
- **Priorita:** P2
- **Stav:** schváleno Product Ownerem, čeká na zařazení po aktuálním sprintu
- **Navržené označení sprintu:** `FEDERACE-MENU-01`
- **Scope:** umístit „Nový pojištěnec“ bezprostředně nad „Seznam pojištěnců“ a zachovat vzájemné pořadí všech ostatních položek.
- **Mimo scope:** změny názvů, ikon, rout, funkcí modulů, vzhledu sidebaru, přidávání nebo odstraňování modulů a jiné UX úpravy.

### PLATBY-001 — Individuální a organizační platby

- **Milestone:** M2 — Provozní automatizace
- **Priorita:** P2
- **Stav:** původní návrh byl zpřesněn rozhodnutím D-014 a dokončen jako `FEDERACE-PLATBY-02` / `PLATBY-002`; samostatně znovu neimplementovat.
- **Implementovaný výsledek:** individuální a organizační platba, přesné automatické rozdělení, ruční rozdělení nedoplatku a nepřiřazený přeplatek. Zbývá uživatelský smoke test.

## Release plán
Vydané verze jsou v [RELEASES.md](RELEASES.md). **Produkční release `v0.24.0` byl dne 2026-08-27 výslovně schválen Product Ownerem.** Balík obsahuje opravu částek při hromadném připisování plateb a opravu vytvoření dokladu z existující platby. Publikace probíhá standardním podepsaným GitHub Actions workflow s testy, EXE/MSI a manifestem Tauri Updateru.

Aktivní položky jsou v [BACKLOG.md](BACKLOG.md), neschválené náměty v [PARKING_LOT.md](PARKING_LOT.md).

## Definition of Done
Schválený scope je implementován, relevantní testy prošly, zdrojová data zůstala nedotčená, dokumentace odpovídá skutečnosti a jsou popsána omezení. Release navíc vyžaduje stabilizaci, RC checklist a schválení Product Ownera.

## Změny a hotfix
Roadmapu, prioritu, Milestone a release plán mění pouze Product Owner; návrh se nejprve zapisuje do Backlogu/Parking Lotu. Hotfix má úzký scope a je povolen jen pro P0, bezpečnost, pád, ztrátu dat nebo updater.
