# Roadmapa

Aktualizováno 2026-08-02. Jediný zdroj pravdy pro plán vývoje.

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

## Release plán
Vydané verze jsou v [RELEASES.md](RELEASES.md). **Navrhovaný další stabilizační release (verze neurčena) čeká na schválení Product Ownerem:** regresní a vizuální testy dokladů, SMTP, updateru, záloh a migrací. Návrh není schválený sprint ani oprávnění k vydání.

Aktivní položky jsou v [BACKLOG.md](BACKLOG.md), neschválené náměty v [PARKING_LOT.md](PARKING_LOT.md).

## Definition of Done
Schválený scope je implementován, relevantní testy prošly, zdrojová data zůstala nedotčená, dokumentace odpovídá skutečnosti a jsou popsána omezení. Release navíc vyžaduje stabilizaci, RC checklist a schválení Product Ownera.

## Změny a hotfix
Roadmapu, prioritu, Milestone a release plán mění pouze Product Owner; návrh se nejprve zapisuje do Backlogu/Parking Lotu. Hotfix má úzký scope a je povolen jen pro P0, bezpečnost, pád, ztrátu dat nebo updater.
