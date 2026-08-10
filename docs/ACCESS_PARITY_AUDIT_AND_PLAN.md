# Audit funkční shody Access → Federace a pracovní plán

Datum auditu: 2026-08-10  
Stav: audit zdrojů a statické porovnání dokončeno; implementace nápravných sprintů nezahájena

## Cíl

Cílem projektu je provozní náhrada původní databáze Access. Federace musí při stejných vstupních datech zachovat datový význam, výpočty, důležité pracovní postupy a výsledné dokumenty. Nemusí kopírovat technická omezení Accessu, pevná hesla, `SendKeys`, pracovní tabulky ani pevné cesty na disku.

Za „shodné“ se považuje pouze chování doložené zdrojem Accessu a ověřené v kódu nebo testu Federace. Pouhá existence podobně nazvané obrazovky nestačí.

## Auditované zdroje

### Původní Access

- pracovní kopie `_scope-work/sprint12-audit.accdb`;
- auditní balíček `_scope-work/access-audit-package`;
- 33 formulářů, 97 tlačítek, 285 VBA procedur, 5 maker, 35 uložených dotazů, 31 sestav a 26 tabulek;
- exporty definic, screenshoty návrhového zobrazení a DAO katalog;
- převodní report `dd_report.json`.

### Federace

- frontend `src/App.tsx` a `src/App.css`;
- backendové moduly v `src-tauri/src`;
- pracovní SQLite `dd.sqlite`;
- 58 automatických Rust testů a frontend build verze `v0.22.0`;
- dosavadní `FUNCTION_MAP.md` a `docs/ACCESS_PARITY_COMPLETION.md`.

## Zásadní korekce předchozího hodnocení

Předchozí funkční mapa uváděla 30 položek „Hotovo“, 16 „Nahrazeno“ a 0 „Chybí“. Toto hodnocení potvrzovalo hlavně existenci provozního ekvivalentu. Nepotvrzovalo, že:

- všechny původní záznamy jsou dostupné v nových obrazovkách;
- sestavy mají stejné filtry, součty a rozložení;
- nové finanční modely mají stejný význam jako původní tabulky;
- všechny pracovní postupy byly ověřeny vedle sebe na stejné kopii dat.

Proto se stav „funkční parita dokončena“ nahrazuje stavem **částečná parita, probíhá stabilizační audit a akceptace**.

## Souhrnný stav

| Oblast | Co již máme | Co ještě chybí | Stav | Priorita |
|---|---|---|---|---|
| Zdroj pravdy dat | produkční Access má ověřený hash `DF176160…E0DEDC`; SQLite má vlastní převodní report | `dd.sqlite` vznikla z jiné/starší kopie: `Seznam` má 14 416 řádků proti 14 408 v produkčním Accessu; před další migrací je nutný read-only rozdílový report | Blokující rozpor | P0 |
| Převod zdrojových tabulek | 24 lokálních Access tabulek bylo převedeno do SQLite se shodným počtem řádků a integritou `ok` | ověřit význam pomocných a historických tabulek; dvě externě připojené tabulky nebyly součástí převodu | Částečně shodné | P1 |
| Aktuální pojištěnci | seznam, detail, založení, editace, hledání, filtry, historie a řízené storno | společný provozní test nad reálnými scénáři Accessu | Téměř shodné | P1 |
| Archiv 2011–2026 | data v `Seznam`, roční archiv, hledání a detail | validovat neplatná/chybějící data období | Částečně shodné | P1 |
| Archiv 2002–2010 | zdrojové roční tabulky zůstaly v SQLite | současný Archiv je nečte; 4 386 záznamů není uživatelsky dostupných | Chybí | P0 |
| Pomocné členské zdroje | `Seznam_` 4 220 řádků, `Odklad` 76 řádků a `Břeclav` 23 řádků jsou zachovány | určit duplicity a provozní význam; neslučovat automaticky | K rozhodnutí | P2 |
| Tarify a výpočet pojistného | verzované sazby, výpočet délky podle Access logiky, finální zaokrouhlení nahoru, automatické testy | datová sada s reprezentativními příklady všech kategorií a roků | Téměř shodné | P1 |
| Platby členů | původní součet `SkutÚhrada` zůstává zachován; nové dílčí a organizační platby mají audit | původní evidence neobsahuje datum jednotlivých plateb; převod vytváří technický záznam k 1. lednu; porovnat hromadné workflow OC/ZO | Bezpečně rozšířeno, částečně ověřeno | P1 |
| Původní faktury/poukázky | tabulka `Faktura` se 4 652 řádky zůstala v SQLite | nová obrazovka čte pouze novou tabulku `VydaneFaktury`, takže historii nezobrazuje; nový model „dodavatel/účet“ nemá dosud prokázanou shodu s významem Accessu | Chybí / významově neověřeno | P0 |
| Příkazy k úhradě | individuální příkaz, PDF, bankovní údaje a audit; nová dávka CSV | porovnat přesný význam `_Příkaz`, vkládání/vyjímání položek a finanční směr operace | Částečně shodné | P1 |
| Doklady o zaplacení | třístránkové PDF, dokladová evidence, SMTP, dávkové vytvoření a regresní testy | vizuální porovnání všech variant proti Access sestavám a reálnému tisku | Téměř shodné | P1 |
| Přihláška, obálka, štítek a poukázka | generování PDF a evidence existují | současné dokumenty jsou obecné PDF; přesné rozložení, pole a tiskové rozměry Accessu nejsou potvrzené snapshoty | Částečně shodné | P1 |
| Pojistné události – nové | založení, úprava, stav z data ukončení, historický rok, audit a přehled | detailní PDF a přesné workflow „Uložit a zobrazit“ nejsou potvrzené proti Accessu | Částečně shodné | P1 |
| Pojistné události – historie | původní `Poj_udalost` se 112 řádky zůstala v SQLite | nová agenda čte pouze prázdnou/novou tabulku `PojistneUdalosti`; původní události nejsou zobrazené | Chybí | P0 |
| Historická vazba událostí | pro většinu událostí lze dohledat přesný pojistný rok | audit našel případy bez přesné roční vazby, jednu nejednoznačnou vazbu a dvě události bez data; nesmí se přiřadit odhadem | K řízenému dořešení | P1 |
| Sestavy OC/ZO/HVP a přehledy | parametrické náhledy, PDF/CSV a základní test datasetu | současné PDF je obecná tabulka; varianty, součty, pořadí, hlavičky a tiskový vzhled 31 Access sestav nejsou jednotlivě akceptované | Částečně shodné | P1 |
| Historie vystavených sestav | původní `Sestavy` má 3 a `SestavyHVP` 138 řádků | nové `AuditSestav` eviduje pouze nové exporty a původní historii nezobrazuje | Chybí | P1 |
| Roční převod | záloha, transakce, nulová úhrada, nové období, nové sazby a idempotence mají automatický test | side-by-side test na kopii reálných dat a kontrolní součty proti Access výsledku | Téměř shodné | P1 |
| Import/připojená data | CSV import s náhledem, validací, zálohou a auditem | původní externí `Členská základna` a Excel `Email` nebyly dodány; rozhodnout, zda je živé propojení stále provozní požadavek | Bezpečně nahrazeno / k rozhodnutí | P2 |
| Zálohy a obnova | ověřené balíčky, kontrola integrity a nouzová záloha | provozní obnovovací zkouška na uživatelské kopii | Lepší než Access, čeká akceptace | P1 |
| Přihlášení a audit | Argon2id, relace, role a audit citlivých operací | sjednotit zbývající auditní operace podle bezpečnostního backlogu | Bezpečně nahrazeno | P2 |
| Navigace | všechny hlavní nové agendy jsou dostupné | pořadí menu `UX-001`; nejde o blokaci datové parity | Funkční | P2 |

## Ověřené datové skutečnosti

- Produkční Access `C:\Users\kubak\Downloads\Pojistka\Pojistka\Pojištění.accdb` existuje a jeho SHA-256 `DF176160A48085BDA225FCAB3D8A9194371E2EBEE43C9EB27CA497D696E0DEDC` přesně odpovídá původnímu auditnímu protokolu.
- `dd_report.json` dokládá, že `dd.sqlite` vznikla z jiné dřívější kopie Accessu. Převodní report potvrzuje shodné počty řádků vůči této kopii a `PRAGMA integrity_check = ok`, nikoli však úplnou shodu s později auditovaným produkčním souborem.
- Produkční Access má v `Seznam` 14 408 řádků, zatímco `dd.sqlite` a pracovní `sprint12-audit.accdb` mají 14 416 řádků. Ostatní kontrolované počty (`Poj_udalost` 112, `Faktura` 4 652, `Sestavy` 3, `SestavyHVP` 138, `Odklad` 76, `Seznam_` 4 220 a roky 2002–2010) se shodují.
- `Seznam` obsahuje převážně období 2011–2026; obsahuje také chybějící a dva zjevně neplatné roky, které vyžadují report kvality dat.
- Roční tabulky 2002–2010 obsahují celkem 4 386 záznamů a nejsou součástí současného archivního dotazu.
- `Poj_udalost` obsahuje 112 původních událostí. Nová tabulka `PojistneUdalosti` se vytváří odděleně bez migrace původních řádků.
- `Poj_udalost_` obsahuje 9 řádků s ID 1–9; stejná ID existují v hlavní tabulce, proto se nesmějí automaticky přidat jako nové události.
- `Faktura` obsahuje 4 652 původních řádků. Nová tabulka `VydaneFaktury` vzniká odděleně bez migrace nebo společného read-only pohledu.
- `Sestavy` a `SestavyHVP` obsahují dohromady 141 historických záznamů, které nový audit sestav nepřebírá.
- Původní Access neměl telefon v tabulce člena `Seznam`; telefon byl pouze u pojistné události. Telefon člena je schválené rozšíření, nikoli původní parita.

## Pracovní plán

### Fáze 0 — Zmrazení rozsahu a důkazů

1. `FED-PARITY-SOURCE-RECONCILE-01`: vytvořit read-only rozdílový report produkčního Accessu proti `dd.sqlite`, zejména přesně identifikovat osm rozdílných řádků `Seznam`; bez automatické změny kteréhokoli souboru.
2. Product Owner potvrdí, která databáze a časový okamžik jsou autoritativním zdrojem pro další migraci.
3. Uchovat kontrolní SHA-256 potvrzeného Access zdroje a `dd.sqlite`.
4. Pro všechny opravné testy používat pracovní kopie; zdrojové soubory nikdy neměnit.
5. Vytvořit anonymizované fixture scénáře odvozené z reálných struktur.
6. Každou odchylku navázat na formulář, dotaz, VBA proceduru nebo sestavu Accessu.

### Fáze 1 — P0 dostupnost historických dat

1. `FED-PARITY-CLAIMS-MIGRATION-01`: bezpečně a idempotentně zpřístupnit všech 112 událostí; zachovat původní ID a všechna pole; nejasné roční vazby označit, neodhadovat.
2. `FED-PARITY-ARCHIVE-LEGACY-01`: zpřístupnit roky 2002–2010 v jednotném read-only Archivu; zachovat původní názvy polí a kontrolní počty 4 386.
3. `FED-PARITY-INVOICES-LEGACY-01`: zpřístupnit 4 652 původních faktur/poukázek a oddělit jejich význam od nových finančních záznamů; historické řádky nesmějí vstoupit do nové dávky omylem.
4. Pro každý převod dodat report: zdrojový počet, cílový počet, přeskočené/nejednoznačné řádky, kontrolní součty a idempotence.

### Fáze 2 — P1 obchodní logika a workflow

1. Pojištěnci: tabulka scénářů založení, editace, ukončení, hledání a filtrů proti Accessu.
2. Tarify: golden testy pro každou pojistnou částku, kategorii, ztrátu, celý rok i poměrnou část roku.
3. Platby: potvrdit význam hromadné úhrady OC/ZO, nedoplatku, přeplatku a návaznosti na doklady.
4. Finance: přesně popsat rozdíl mezi Access `Faktura`, formulářem `Faktury`, `_Příkaz` a novými `VydaneFaktury`/`PlatebniDavky`.
5. Roční převod: spustit oba systémy na izolované kopii stejného roku a porovnat počty, osoby, sazby, období, ukončení a nulové úhrady.

### Fáze 3 — P1 dokumenty a sestavy

1. Pro všech 31 Access sestav vytvořit matici: aktivní, historická, nahrazená nebo čekající na rozhodnutí.
2. U aktivních výstupů porovnat zdrojový dataset, filtry, řazení, skupiny, mezisoučty, celkové součty a stránkování.
3. Renderovat referenční Access PDF a Federace PDF nad stejnými anonymizovanými daty.
4. Zavést snapshot/regresní testy pro doklad, přihlášku, poukázku, obálku, štítek, událost, OC, ZO a HVP.
5. Zpřístupnit nebo bezpečně označit historickou evidenci 141 vystavených sestav.

### Fáze 4 — P2 pomocná data a bezpečné náhrady

1. Porovnat `Seznam_`, `Odklad`, `Břeclav` a aktivní `Seznam` podle identifikátoru, rodného a evidenčního čísla; vytvořit pouze report, ne automatický merge.
2. Rozhodnout o externích zdrojích `Členská základna` a `Email`.
3. Potvrdit Product Ownerem bezpečné náhrady: autentizace, zálohy, CSV místo XLS/RTF/DOC, transakce místo pracovních tabulek a řízené storno místo hard delete.
4. Teprve po uzavření parity pokračovat v čistě nových UX funkcích `UX-001` a `UX-002`.

## Povinné akceptační brány

Každý opravný sprint musí splnit:

- původní zdrojová data zůstala bitově nezměněná;
- migrace je idempotentní a má kontrolní součty;
- žádný nejasný řádek není přiřazen odhadem;
- stejný scénář dává v Accessu a Federaci stejný obchodní výsledek;
- rozdílný technický postup je písemně označen jako bezpečná náhrada;
- automatické testy, frontend build a uživatelský smoke test prošly;
- Product Owner schválil případné záměrné rozdíly.

## Rozhodnutí vyžadující Product Ownera

1. Mají být `Seznam_`, `Odklad` a `Břeclav` pouze archivní důkaz, nebo aktivní zdroj dat?
2. Je živé propojení na externí `Členská základna` a Excel e-mailů stále potřebné?
3. Které historické varianty sestav `Old`, osobní sestavy a RTF/DOC exporty se stále provozně používají?
4. Jak ručně vyřešit události bez jednoznačného pojistného záznamu daného roku?
5. Je schváleno zachovat bezpečné modernizace místo doslovného kopírování rizikového chování Accessu?

## Doporučené pořadí

Dokud nejsou uzavřeny Fáze 0–3, je hlavní produktová priorita **M1 — funkční a datová shoda Accessu**. Nové UX a automatizace zůstávají v Backlogu. První sprint má být read-only `FED-PARITY-SOURCE-RECONCILE-01`; teprve po potvrzení autoritativního zdroje následuje `FED-PARITY-CLAIMS-MIGRATION-01`.
