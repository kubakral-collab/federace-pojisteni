# Roadmapa

Aktualizováno 2026-08-10. Jediný zdroj pravdy pro plán vývoje.

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
Ověřeno proti tagům `v0.17.0`–`v0.21.0`, kódu a testům.

**Implementováno:** technický audit Access objektů; bezeztrátový převod 24 lokálních tabulek do SQLite; oddělení aktuálního roku od archivu; dashboard; seznam/detail/nový člen; sazby; platby; archiv nad tabulkou `Seznam`; příkaz k úhradě; hledání; filtry; přihlášení; zálohy; doklady; SMTP; nové pojistné události; přehledy; CI/CD a podepsaný updater.

**Částečně / stabilizace:** úplná provozní parita Accessu není potvrzená. Audit odhalil nezpřístupněné historické události, faktury, roky 2002–2010 a historii sestav; dokumenty a sestavy nemají dokončené side-by-side a vizuální regrese. Závazný stav a plán je v [ACCESS_PARITY_AUDIT_AND_PLAN.md](ACCESS_PARITY_AUDIT_AND_PLAN.md).

## Milestones
- **M1 Funkční náhrada Accessu — stabilizace:** základní agendy jsou implementované; zbývá potvrdit provozní paritu a RC kvalitu.
- **M2 Provozní automatizace — plánováno:** každoroční příkazy, bankovní import/párování, roční převod.
- **M3 Ochrana dat — plánováno:** SQLCipher, šifrované zálohy, zamykání, úplný audit.
- **M4 Sestavy — plánováno:** OC/ZO/HVP a další provozně schválené výstupy.

## Aktuální sprint

### Žádný aktivní sprint — čeká se na schválení Product Ownerem

- **Milestone:** M1 — Stabilizace Core
- **Poslední dokončený sprint:** `FED-PARITY-CLAIMS-MIGRATION-01`, akceptován rozhodnutím D-020.
- **Přeskočený sprint:** `FED-PARITY-ARCHIVE-LEGACY-01` byl rozhodnutím D-020 vrácen do backlogu.
- **Navržený další sprint:** `FED-PARITY-INVOICES-LEGACY-01` — zatím neschválen, implementace nezačala.

## Pořadí sprintů funkční parity

| Pořadí | Sprint | Priorita | Cíl | Stav |
|---:|---|---|---|---|
| 1 | `FED-PARITY-SOURCE-RECONCILE-01` | P0 | Read-only rozdílový audit produkčního Accessu a `dd.sqlite`; určit osm rozdílných řádků bez změny dat. | Dokončeno; 14 370 shodných, 36 změněných, 2 pouze Access, 10 pouze SQLite |
| 2 | `FED-PARITY-DOCUMENTS-01` | P1 | Ověřit a dorovnat aktivní doklad, přihlášku, událost a sestavy OC/ZO/HVP; přesná shoda poukázky, obálky a štítku je rozhodnutím D-017 mimo scope. | Dokončeno, otestováno a akceptováno PO |
| 3 | `FED-PARITY-CLAIMS-MIGRATION-01` | P0 | Zpřístupnit 112 původních pojistných událostí, zachovat ID a oddělit nejasné historické vazby. | Dokončeno, otestováno a akceptováno PO |
| 4 | `FED-PARITY-ARCHIVE-LEGACY-01` | P0 | Zpřístupnit 4 386 archivních záznamů z let 2002–2010 v read-only archivu. | Přeskočeno rozhodnutím D-020; vráceno do backlogu |
| 5 | `FED-PARITY-INVOICES-LEGACY-01` | P0 | Zpřístupnit 4 652 původních faktur/poukázek a potvrdit význam finančního workflow. | Navržený další sprint; čeká na samostatné schválení PO |
| 6 | `FED-PARITY-REPORT-HISTORY-01` | P1 | Zpřístupnit historii 141 vystavených sestav. | Čeká na schválení po sprintu 5 |
| 7 | `FED-PARITY-WORKFLOWS-01` | P1 | Side-by-side ověřit pojištěnce, tarify, platby, příkazy a roční převod. | Čeká na schválení po sprintu 6 |
| 8 | `FED-PARITY-AUXILIARY-DATA-01` | P2 | Klasifikovat `Seznam_`, `Odklad`, `Břeclav` a externí zdroje bez automatického slučování. | Čeká na schválení po sprintu 7 |

Pořadí může změnit pouze Product Owner. Schválení procesu není automatickým schválením sprintů 2–8.

## Schválený budoucí scope

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
- **Stav:** schváleno Product Ownerem, čeká na zařazení do pořadí sprintů
- **Navržené označení sprintu:** `FEDERACE-PLATBY-01`
- **Scope:** umožnit na stránce Platby vytvořit individuální platbu navázanou na člena nebo organizační platbu navázanou na organizaci; evidovat typ platby, zobrazit příslušného člena či organizaci a členy vybrané organizace; zachovat kompatibilitu existujících plateb bezpečnou migrací.
- **Mimo scope:** automatické rozúčtování organizační částky mezi členy, změny pojistného, sazeb, dokladů, certifikátů, členů, detailu člena nebo levého menu.
- **Otevřené rozhodnutí:** pravidla rozúčtování organizační platby vyžadují samostatné schválení Product Ownera; nesmějí být odvozena ani implementována bez něj.

## Release plán
Vydané verze jsou v [RELEASES.md](RELEASES.md). **Produkční release `v0.22.0` byl dne 2026-08-10 výslovně schválen Product Ownerem.** Balík zahrnuje dokončení provozní parity Accessu, individuální a organizační platby, historické údaje pojistných událostí, telefon člena a opravu zaokrouhlování poměrného pojistného. Publikace probíhá standardním podepsaným GitHub Actions workflow s testy, EXE/MSI a manifestem Tauri Updateru.

Aktivní položky jsou v [BACKLOG.md](BACKLOG.md), neschválené náměty v [PARKING_LOT.md](PARKING_LOT.md).

## Definition of Done
Schválený scope je implementován, relevantní testy prošly, zdrojová data zůstala nedotčená, dokumentace odpovídá skutečnosti a jsou popsána omezení. Release navíc vyžaduje stabilizaci, RC checklist a schválení Product Ownera.

## Změny a hotfix
Roadmapu, prioritu, Milestone a release plán mění pouze Product Owner; návrh se nejprve zapisuje do Backlogu/Parking Lotu. Hotfix má úzký scope a je povolen jen pro P0, bezpečnost, pád, ztrátu dat nebo updater.
