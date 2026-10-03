# Audit Access vs. program Federace Pojištění

Datum kontroly: 2026-09-08
Kontrolovaná verze aplikace: `0.24.0`

## Verdikt

Nový program funkčně pokrývá všech 46 zdokumentovaných obchodních oblastí původního Accessu: 30 přímo a 16 bezpečnějším nebo modernějším řešením. Automatické technické ověření současného pracovního stromu prošlo bez chyby (`npm run build`; Rust 70/70 testů).

Program ale ještě není úplnou náhradou Accessu z hlediska historických dat a provozní akceptace. Největší mezery jsou nedostupný archiv 2002–2010, nedostupná historie původních faktur a neprovedená synchronizace rozdílů mezi autoritativním Accessem a SQLite.

## Realizace po auditu

Doplněno 2026-09-08: archiv 2002–2010 a původní tabulka `Faktura` jsou nově dostupné v přihlášené obrazovce **Historická data**. Jde o stránkované read-only zobrazení s hledáním; povolen je pouze pevný seznam historických tabulek a původní faktury nemají zápisovou vazbu do nové platební dávky. Tím jsou původní mezery PARITY-002 a PARITY-003 implementačně odstraněny; před vydáním ještě vyžadují uživatelskou akceptaci nad pracovní kopií provozní databáze.

## Co funguje

| Oblast | Stav | Důkaz |
| --- | --- | --- |
| Přihlášení, role, navigace a diagnostika | Funguje | Implementace; build a testy prošly |
| Pojištěnci: seznam, detail, hledání, filtry, založení, editace, historie a řízené storno | Funguje | Regresní testy seznamu, detailu, editace, filtrů a storna prošly |
| Tarify a poměrné pojistné | Funguje technicky | Testy sazeb, historie a zaokrouhlení prošly; zbývá uživatelský smoke test |
| Individuální a organizační platby | Funguje technicky | Testy CRUD, přesného rozdělení, nedoplatku a přeplatku prošly; uživatelský smoke test není uzavřen |
| Nové faktury, příkazy a platební dávky | Funguje pro nové záznamy | Test atomické dávky a zákazu dvojího exportu prošel |
| Doklady, potvrzení, přihlášky a aktivní PDF sestavy | Funguje technicky | PDF regresní testy prošly; fyzický tisk není potvrzen |
| Pojistné události | Funguje | Všech 112 historických událostí je dostupných; 98 přesně napojených, 14 bezpečně read-only |
| Sestavy OC, ZO, HVP a provozní exporty | Funguje | Náhled/PDF/CSV testy prošly |
| Historie vystavených sestav | Funguje | 141 historických záznamů je dostupných read-only |
| Roční převod | Funguje technicky | Testuje se záloha, transakce, nové sazby, nulová úhrada a idempotence |
| CSV import členů | Funguje | Testuje se náhled, validace, duplicity, transakce, záloha a audit |
| Záloha a obnova | Funguje technicky | Automatické testy integrity, obnovy a nouzové zálohy prošly |

## Co chybí proti úplné náhradě Accessu

| Priorita | Mezera | Dopad | Potřebná práce |
| --- | --- | --- | --- |
| P0 | Archivní tabulky 2002–2010 | Implementováno po auditu; 4 386 řádků je dostupných read-only | Provést uživatelský smoke test a akceptaci |
| P0 | Původní faktury/poukázky | Implementováno po auditu; 4 652 řádků je dostupných read-only a oddělených od nových dávek | Provést uživatelský smoke test a akceptaci |
| P0 | Datové rozdíly autoritativního Accessu a SQLite nejsou synchronizované | Dřívější audit zjistil 14 370 shod, 36 změněných, 2 jen v Accessu a 10 jen v SQLite | Schválit pravidla a provést vratný migrační sprint; automatické sloučení není bezpečné |
| P2 | Není určen význam tabulek `Seznam_`, `Odklad`, `Břeclav` | 4 220 + 76 + 23 řádků je zachováno, ale jejich provozní role není rozhodnuta | Read-only porovnání duplicit a rozhodnutí: archiv/import/vyřazení |
| P2 | Chybí rozhodnutí o externích zdrojích `Členská základna` a Excel `Email` | Původní Access je používal, zdroje ale nebyly dodány | Potvrdit, zda jsou ještě provozně potřeba; případně navrhnout samostatnou integraci |

## Co je implementované, ale nelze ještě označit za provozně ověřené

1. Individuální a organizační platby na pracovní kopii reálných dat.
2. Historická událost a výběr pojistného záznamu podle data vzniku.
3. Uložení telefonu a poměrné pojistné zaokrouhlené nahoru.
4. Obnova skutečné zálohy na uživatelské kopii.
5. Fyzický tisk aktivních PDF na provozní tiskárně, pokud je požadován.

Dokud tyto smoke testy neproběhnou, správné označení je „automaticky ověřeno“, nikoli „provozně potvrzeno“.

## Co nefunguje

Při této kontrole nebyla automatickými testy prokázána žádná rozbitá implementovaná funkce. Prokazatelně nefunguje pouze uživatelský přístup k výše uvedeným historickým datům, protože příslušné obrazovky neexistují. U neprovedených smoke testů nelze tvrdit ani „funguje“, ani „nefunguje“.

## Co je nový rozvoj, nikoli mezera proti Accessu

- import bankovního výpisu a automatické/ruční párování;
- automatické každoroční příkazy a řízené odesílání;
- SQLCipher a šifrované zálohy;
- sjednocený audit všech citlivých operací;
- automatický zámek aplikace;
- uživatelská velikost písma a schválená změna pořadí menu.

## Datová kontrola lokální `dd.sqlite`

- `PRAGMA integrity_check`: `ok`;
- `Seznam`: 14 416 řádků;
- roční tabulky 2002–2010: 4 386 řádků;
- `Faktura`: 4 652 řádků;
- `Poj_udalost`: 112 řádků;
- `Sestavy` + `SestavyHVP`: 141 řádků;
- `Seznam_`: 4 220, `Odklad`: 76, `Břeclav`: 23.

Lokálně dostupná referenční kopie Accessu nemá hash autoritativního produkčního souboru z rozhodnutí D-032. Proto tato kontrola neopakuje aktuální řádkové porovnání autoritativního Accessu; pro rozdíly používá poslední schválený read-only audit. K novému definitivnímu porovnání je nutná aktuální kopie produkčního Accessu s SHA-256 `DF176160A48085BDA225FCAB3D8A9194371E2EBEE43C9EB27CA497D696E0DEDC`.

## Doporučené pořadí

1. Dokončit P1 uživatelské smoke testy, začít platbami a obnovou zálohy.
2. Rozhodnout a bezpečně vyřešit datové rozdíly Access/SQLite.
3. Obnovit P0 sprinty pro archiv 2002–2010 a historické faktury.
4. Klasifikovat pomocné tabulky a externí zdroje.
5. Teprve potom řešit bankovní automatizaci, šifrování a UX rozvoj.

## Použité důkazy

- `FUNCTION_MAP.md` a exportovaný inventář Access objektů;
- `docs/ACCESS_PARITY_AUDIT_AND_PLAN.md`, `BACKLOG.md`, `ROADMAP.md`, `DECISIONS.md`;
- zdrojový kód Tauri/React a jeho testy;
- lokální `dd.sqlite` v režimu čtení;
- aktuální běh `npm.cmd run build` a `cargo test --all-targets` dne 2026-09-08.
