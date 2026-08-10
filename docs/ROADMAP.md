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

**Implementováno:** audit Accessu; migrace a oddělení aktuálního roku od archivu; dashboard; seznam/detail/nový člen; sazby; platby; archiv; příkaz k úhradě; univerzální hledání; filtr organizace; splatnost a dashboard po splatnosti; první správce a Argon2id; ruční záloha/obnova; doklady (evidence, detail, levé menu, PDF, úplná úhrada, Access šablony); SMTP a Windows Credential Manager; pojistné události; přehled pro pojišťovnu; GitHub, CI/CD, Releases, podepsaný Tauri Updater, EXE/MSI.

**Částečně / stabilizace:** vizuální regrese všech variant dokladů; provozní SMTP end-to-end; úplné sjednocení auditu citlivých operací.

## Milestones
- **M1 Funkční náhrada Accessu — stabilizace:** základní agendy jsou implementované; zbývá potvrdit provozní paritu a RC kvalitu.
- **M2 Provozní automatizace — plánováno:** každoroční příkazy, bankovní import/párování, roční převod.
- **M3 Ochrana dat — plánováno:** SQLCipher, šifrované zálohy, zamykání, úplný audit.
- **M4 Sestavy — plánováno:** OC/ZO/HVP a další provozně schválené výstupy.

## Aktuální sprint

### FED-UDALOSTI-HISTORY-01 — Historické údaje pojištění u pojistných událostí

- **Milestone:** M1 — Stabilizace Core
- **Stav:** implementováno; build a automatické regrese prošly, čeká na uživatelský smoke test
- **Priorita:** P1
- **Scope:** `UDALOSTI-001` — rok události odvodit z data vzniku a použít přesný historický pojistný záznam; při chybějících datech nesmí nastat fallback na aktuální rok.
- **Navazující sprint:** `FED-CONTACT-CALC-01` je rovněž implementován; telefon je dostupný při založení, editaci i v detailu a poměrné pojistné používá finální zaokrouhlení nahoru.

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
