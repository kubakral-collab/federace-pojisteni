# Funkční mapa migrace Access → Tauri

## Účel a metodika

Tento dokument porovnává funkce původní aplikace s aktuálním stavem Tauri aplikace v tomto repozitáři. **Jediným zdrojem informací o původní aplikaci je dodaný `ACCESS_AUDIT_PACKAGE.zip`** (zejména `ACCESS_AUDIT_SUMMARY.md`, `FORMS.md`, `BUTTONS.md`, `MACROS.md`, `VBA.md`, `QUERIES.md`, `REPORTS.md`, `TABLES_RELATIONS.md` a exporty v `evidence/`). Access databáze nebyla znovu otevřena ani auditována. Stav Tauri je určen z aktuálního zdrojového kódu a jeho testů.

Stavy:

- **Hotovo** — v Tauri existuje funkčně odpovídající workflow.
- **Chybí** — doložená provozní funkce Accessu nemá funkční ekvivalent nebo je implementována jen část nezbytného workflow.
- **Nahrazeno** — účel je pokryt jiným, zpravidla bezpečnějším nebo jednodušším řešením; nejde o doslovnou kopii Accessu.
- **Vyřazeno** — záměrně odstraněno se schválením. V dodaném balíčku takové schválení není, proto tento stav není přidělen žádné funkci.

Priorita je uvedena pouze u chybějících funkcí: **P0** blokuje bezpečnou provozní paritu nebo roční cyklus, **P1** je důležitá každodenní agenda, **P2** méně časté workflow nebo optimalizace, **P3** historická či diagnostická funkce k potvrzení.

## Souhrn

| Stav | Počet funkčních položek |
| --- | ---: |
| Hotovo | 30 |
| Nahrazeno | 16 |
| Chybí | 0 |
| Vyřazeno | 0 |

Všechny doložené funkční oblasti Accessu mají v Tauri aplikaci provozní ekvivalent. Označení „Hotovo“ znamená shodu účelu, nikoli pixelovou nebo technickou kopii formuláře Accessu; nahrazené technické mechanismy jsou výslovně označeny stavem „Nahrazeno“.

## Funkční mapa

### Přístup, navigace a správa aplikace

| ID | Funkce původního Accessu | Důkaz v balíčku | Ekvivalent v Tauri | Stav | Priorita / poznámka |
| --- | --- | --- | --- | --- | --- |
| FM-01 | Vstupní brána chráněná heslem | `Vstup`, `Heslo`; tlačítko ověřovalo pevné heslo | Prvotní založení správce, Argon2id přihlášení, relace a odhlášení | Nahrazeno | Pevné heslo bylo nahrazeno bezpečnou autentizací. |
| FM-02 | Hlavní přepínací panel a otevření agend | `Switchboard` a jeho tlačítka/VBA | Levá navigace mezi Přehledem, členy, platbami, doklady, událostmi, archivem, zálohami a nastavením | Hotovo | — |
| FM-03 | Ukončení aplikace | `Switchboard` / ukončovací akce | Příkaz `quit_application` | Hotovo | — |
| FM-04 | Servisní minimalizace, otevření objektů a sekvence kláves | makro `Kontr` | Diagnostika integrity databáze, verze, aktivního roku, počtů a záloh | Nahrazeno | Bez nebezpečných `SendKeys` a bez zpřístupnění databázových objektů. |

### Pojištěnci a pojistné záznamy

| ID | Funkce původního Accessu | Důkaz v balíčku | Ekvivalent v Tauri | Stav | Priorita / poznámka |
| --- | --- | --- | --- | --- | --- |
| FM-05 | Seznam aktuálních pojištěnců a detail záznamu | `Seznam`, `SeznamPod`, `Přehled`, `_PřehledOC` | Stránkovaný seznam a detail člena | Hotovo | — |
| FM-06 | Živé hledání člena | `Hledat`, vyhledávací události | Hledání v seznamu a ve výběrech agend | Hotovo | — |
| FM-07 | Filtrování podle OC/ZO, roku, kategorie, ztráty, pojistného, úhrady a ukončení | `Přehled`, `PřPod1–3`, `_PřehledOC` | Filtry seznamu a archivu včetně správného porovnání předepsané a skutečně uhrazené částky | Hotovo | Regresní test pokrývá nulovou, částečnou, úplnou a přeplatkovou úhradu. |
| FM-08 | Založení pojištěnce, výpočet sazby, uložení a „uložit a nový“ | `Pojištěnci`; tarifní dotazy `200T–400T` | Formulář Pojištěnci, databázové uložení, verzované sazby a automatický výpočet | Hotovo | „Uložit a nový“ lze pokrýt opakovaným založením; obchodní výsledek je zachován. |
| FM-09 | Editace aktuálního záznamu | pracovní tabulka `Editace`, `Pojištěnci`, `SeznamPod` | Editace aktuálního pojistného záznamu s rolí správce a auditem | Nahrazeno | Dočasná tabulka Accessu není potřebná. |
| FM-10 | Historie pojištění jedné osoby | roční záznamy v `Seznam` a roční tabulky | Historie člena a detail archivního záznamu | Hotovo | — |
| FM-11 | Nevratné odstranění pojištěnce/záznamu | tlačítka `Del` v `Přehled` a `_PřehledOC`; `KlientDelete` | Řízené storno aktuálního záznamu s důvodem, potvrzením, historií a auditem | Nahrazeno | Hard delete není uživateli zpřístupněn. |
| FM-12 | Kontrola duplicit pojištěnců | `Najít duplicity pro Seznam` | Parametrická kontrolní sestava duplicit a blokace duplicit při importu | Hotovo | Ověřeno integračním testem importu. |
| FM-13 | Připojení/import odložených či externích dat | `Odklad_přidat`, `Dotaz6`, `PřipojDatakSeznamu`, `PropojitData` | CSV UTF-8 import s náhledem, validací, detekcí duplicit, zálohou, transakcí a auditem | Nahrazeno | Pomocné Access tabulky nejsou potřeba. |

### Platby, příkazy, faktury a doklady

| ID | Funkce původního Accessu | Důkaz v balíčku | Ekvivalent v Tauri | Stav | Priorita / poznámka |
| --- | --- | --- | --- | --- | --- |
| FM-14 | Záznam úhrady pojištěnce | `Úhrada` | Přidání, seznam a odstranění dílčích plateb člena; součet do skutečné úhrady | Hotovo | — |
| FM-15 | Rozlišení nedoplatku/přeplatku a platebního stavu | `Úhrada`, výrazy ve formulářích/sestavách | Stav platby, nedoplatek a přeplatek v detailu | Hotovo | Stejná definice platebního stavu je použita i ve filtrech. |
| FM-16 | Vytvoření příkazu k úhradě z faktur, vložení/vyjmutí položek | `_Příkaz`, tabulky `Faktura`/`Příkaz` | Kniha faktur a jednorázový dávkový CSV export se součtem, vazbou a auditem | Hotovo | Test ověřuje atomickou dávku a zákaz opakovaného exportu. |
| FM-17 | Individuální podklady k platbě / platební instrukce | část účelu `_Příkaz` a fakturačních údajů | Náhled a PDF příkazu k úhradě, otevření souboru a audit tisku | Nahrazeno | Modernizovaný individuální workflow. |
| FM-18 | Evidence a aktualizace vydaných faktur/poukázek | `Faktury`, `Faktura`, `Faktura_copy`, `Copy_fakt2`, `Copy_faktur`, `Dotaz3`, `KlientDelete` | Kniha faktur s číselnou řadou, bankovními údaji, splatností, stavem, dávkou a auditem | Hotovo | — |
| FM-19 | Doklad o zaplacení: vytvořit, zobrazit, exportovat PDF, tisknout a odeslat e-mailem | dílčí účel `Doklad_tisk`, makra `Makro2/3` | Agenda Doklady o zaplacení včetně PDF, náhledu, tisku, SMTP a auditu | Hotovo | — |
| FM-20 | Pojistný doklad/potvrzení a hromadný tisk podle data, OC nebo ZO | `Doklad`, `Doklad_`, `Doklad_1`; sestavy `Doklad`, `Doklad_` | Třístránkové potvrzení a dávkové vytvoření/export podle data, OC a ZO | Hotovo | Opakované vytvoření je idempotentní a export auditovaný. |
| FM-21 | Přihláška a formulář dat klienta | režimy `Doklad`; `Přihláška`, `Přihláška_`, `Přihláška_Old`, `Form_DataKlientů` | PDF přihláška z detailu člena s evidencí a auditem | Hotovo | — |
| FM-22 | Obálka, adresní seznamy a poštovní štítky | `AdresPojPodm`, `ObálkaDL`, `NálepkyAdr`, `NálepkyForm`, `NálepkyZO` | PDF obálka a adresní štítek; adresní seznam v centru sestav | Hotovo | — |
| FM-23 | Poštovní poukázky jednotlivě i dávkově a současný zápis faktury | `PoštPoukázka`; sestavy `PoštPoukázka`, `_`, `_S`; `Dotaz2` | PDF poukázka atomicky propojená s fakturou člena | Hotovo | Vazbu ověřuje automatický test. |
| FM-24 | Automatické vytvoření dokladu po způsobilé úhradě | v Accessu ruční vazba úhrady/dokladu | `create_if_eligible` po uložení platby a nastavitelné automatické vytváření | Nahrazeno | Zjednodušení oproti ručnímu Access workflow. |

### Pojistné události

| ID | Funkce původního Accessu | Důkaz v balíčku | Ekvivalent v Tauri | Stav | Priorita / poznámka |
| --- | --- | --- | --- | --- | --- |
| FM-25 | Evidence nové pojistné události | `Poj_udalost`, `Dotaz1/5`, pomocná tabulka `Poj_udalost_zdroj` | Založení události s validací a auditem | Nahrazeno | Přímá transakce nahrazuje pracovní tabulku. |
| FM-26 | Přehled a detail událostí | `Poj_udalost` a dotaz `test` | Centrální přehled a události v detailu člena | Hotovo | — |
| FM-27 | Úprava údajů události včetně stavu, škody a plnění | `Poj_Udalost_upr` | Editace události | Hotovo | — |
| FM-28 | Úprava kontaktního e-mailu při práci s událostí | `Poj_Udalost_mail` | E-mail se upravuje v detailu člena | Nahrazeno | Jediný zdroj kontaktního údaje. |
| FM-29 | Tisk/zobrazení hlášení pojistné události a souhrnné sestavy | `PojUdálost`, `PojUdálost_sest`; tlačítko „Uložit a zobrazit“ | Filtrovaná HVP/událostní sestava s náhledem, PDF, CSV a auditem | Hotovo | — |

### Přehledy, sestavy a exporty

| ID | Funkce původního Accessu | Důkaz v balíčku | Ekvivalent v Tauri | Stav | Priorita / poznámka |
| --- | --- | --- | --- | --- | --- |
| FM-30 | Interaktivní přehled pro pojišťovnu | `_PřehledOC`, `SestPoj` | Obrazovka Přehled pro pojišťovnu s výběrem a přechodem do člena | Hotovo | — |
| FM-31 | Tisk/export přehledu pro pojišťovnu | `_PřehledOC`, `SestPoj`, `SestPojEmail` | Stejný filtrovaný dataset v náhledu, PDF a CSV | Hotovo | — |
| FM-32 | Sestava OC | `Sestava`, `SestavaOC`, `SestKontr` | Parametr OC/rok, náhled, PDF a CSV | Hotovo | — |
| FM-33 | Sestava ZO včetně potvrzení, počtů a odesílací varianty | `SestavaZO`, `SestavaZOpod`, `SestavaZO_Old`; `SestavaZO`, `_Potvrz`, `_Počty`, `Sest_Odes`; `Export_kontrol` | Parametr ZO/rok, náhled, PDF a CSV | Hotovo | — |
| FM-34 | Sestava HVP a evidence vystavených sestav | `SestHVP`, `SestHVPpod`, `SestPřehl`; `SestPřehl`; tabulky `Sestavy`, `SestavyHVP`; `Kon`, `OdstrSes` | Parametrická HVP sestava s auditem exportu | Hotovo | — |
| FM-35 | Přehled počátků a ukončení pojištění | `Přehled`; `SestP2`, `SestP2_Adrs`; `SeznamOd`, `SeznamKonec`, `Seznam_podsest` | Samostatné sestavy počátků a ukončení s obdobím | Hotovo | — |
| FM-36 | Export začátků/ukončení do Excelu | `PřenosDat`; `Přev`, `Ukon` | CSV UTF-8 pro Excel a PDF přes dialog uložení | Nahrazeno | Bez pevných cest a pomocných tabulek. |
| FM-37 | Export ZO do XLS/RTF/DOC/PDF | `Makro1`, `test`, převedená makra; `Export_kontrol` | Sjednocený PDF a CSV UTF-8 export | Nahrazeno | Historické RTF/DOC formáty nebyly kopírovány. |
| FM-38 | E-mailový seznam pojištěnců | `Email_2015`, `SestPojEmail` | Parametrický kontaktní seznam v náhledu, PDF a CSV | Hotovo | — |
| FM-39 | Kontrolní sestava a seznam prázdných evidenčních čísel | `SestKontr`, `Prázdná čísla`, dotazy duplicit | Kontrola povinných polí, duplicit, pojistného a přeplatků | Hotovo | — |
| FM-40 | Jednorázové osobní/roční sestavy | `Procházka Jiří 2022`, `Email_2015`, `opravy` | Parametrická sestava podle roku, jména, ev. nebo rodného čísla | Nahrazeno | Osobní/roční názvy nejsou zakódovány. |

### Archivace, roční cyklus, zálohy a audit

| ID | Funkce původního Accessu | Důkaz v balíčku | Ekvivalent v Tauri | Stav | Priorita / poznámka |
| --- | --- | --- | --- | --- | --- |
| FM-41 | Prohlížení historických let | roční tabulky `2002–2010`, `Archivace`, `Seznam_` | Archiv podle pojistného roku, hledání, filtry a detail | Nahrazeno | Jedna časová datová struktura nahrazuje fyzické roční tabulky. |
| FM-42 | Ruční archivace/pojmenování kopie dat | `Archivace` | Verifikované `.fvcbackup` zálohy, seznam a obnova s nouzovou zálohou | Nahrazeno | Bezpečnější provozní ekvivalent archivní kopie. |
| FM-43 | Převod dat do dalšího období, nové datum, nulová úhrada a přepočet sazeb | `Převod_data`, `Přenos`, `PřipojDatakSeznamu`, `200T–400T`, `PojAkt/PojAkt_` | Automatický roční převod se zálohou, transakcí, idempotencí, vyloučením ukončených záznamů a přepočtem sazeb | Nahrazeno | Integrační test ověřuje zálohu, nulovou úhradu, nové období, sazby a zákaz duplicitního převodu. |
| FM-44 | Ruční vytvoření a obnova zálohy | účel `Archivace` | Správa záloh s kontrolou kompatibility a integrity | Hotovo | — |
| FM-45 | Audit citlivých operací | Access jen dílčí příznaky tisku/dokladu | Audit členů, plateb, dokladů, příkazů a událostí | Nahrazeno | Rozšíření proti Accessu; úplnost všech citlivých operací je ještě samostatný bezpečnostní backlog. |
| FM-46 | Správa tarifů místo pevných aktualizačních dotazů | `200T–400T`, výpočty ve `Pojištěnci` a `Převod_data` | Verze sazeb s platností, aktivací a validací překryvů | Hotovo | — |

## Backlog chybějících funkcí

K datu této kontroly nemá žádná doložená funkční položka stav **Chybí**. Další práce patří do stabilizace, provozní akceptace a nových produktových požadavků; stav **Vyřazeno** nebyl bez doloženého schválení Product Ownerem použit.

## Pokrytí Access objektů

Následující index zajišťuje, že žádný objekt z inventáře balíčku nezůstal mimo mapu. Pomocné formuláře, dotazy a sestavy jsou přiřazeny k obchodní funkci; jejich technický mechanismus se nemusí v Tauri kopírovat.

### Formuláře (33/33)

| Funkční ID | Access formuláře |
| --- | --- |
| FM-01 | `Vstup`, `Heslo` |
| FM-02–FM-03 | `Switchboard` |
| FM-05–FM-07, FM-11, FM-30 | `Seznam`, `SeznamPod`, `Přehled`, `PřPod1`, `PřPod2`, `PřPod3`, `_PřehledOC`, `Hledat` |
| FM-08–FM-09 | `Pojištěnci` |
| FM-14–FM-15 | `Úhrada` |
| FM-16–FM-17 | `_Příkaz` |
| FM-18 | `Faktury` |
| FM-19–FM-22 | `Doklad`, `Doklad_`, `Doklad_1`, `Doklad_tisk` |
| FM-23 | `PoštPoukázka` |
| FM-25–FM-29 | `Poj_udalost`, `Poj_Udalost_mail`, `Poj_Udalost_upr` |
| FM-32 | `Sestava` |
| FM-33 | `SestavaZO`, `SestavaZO_Old`, `SestavaZOpod` |
| FM-34 | `SestHVP`, `SestHVPpod`, `SestPřehl` |
| FM-36 | `PřenosDat` |
| FM-41–FM-43 | `Archivace`, `Převod_data` |

### Makra (5/5) a standardní VBA moduly (16/16)

| Funkční ID | Objekt |
| --- | --- |
| FM-04, FM-39 | makro `Kontr` |
| FM-37 | makra `Makro1`, `test` |
| FM-19–FM-20 | makra `Makro2`, `Makro3` |
| FM-17, FM-20, FM-22, FM-31, FM-33, FM-35, FM-37 | modul `global`; `Převedené makro- Makro1` až `Převedené makro- Makro10`; `Převedené makro- test`, `test1`, `test2`, `test3`, `test4` |

Formulářové VBA procedury (29 modulů, celkem 285 procedur spolu se standardními moduly dle souhrnu balíčku) jsou pokryty přes odpovídající formulář a funkční ID výše. Události čistě řídící fokus, viditelnost, `Requery`, otevření/zavření formuláře a obsluhu chyb jsou UI mechanismus, ne samostatná obchodní funkce; v Tauri jsou považovány za absorbované navigací a reaktivním stavem příslušné funkce.

### Uložené dotazy (35/35)

| Funkční ID | Access dotazy |
| --- | --- |
| FM-43, FM-46 | `200T`, `240T`, `280T`, `320T`, `360T`, `400T`, `Přenos`, `PřipojDatakSeznamu`, `PojAkt`, `PojAkt_` |
| FM-18 | `Copy_fakt2`, `Copy_faktur`, `Dotaz3`, `Duplicitní hodnoty v tabulce Faktura`, `KlientDelete` |
| FM-25–FM-29 | `Dotaz1`, `Dotaz5`, `test` |
| FM-23 | `Dotaz2` |
| FM-13 | `Dotaz6`, `Odklad_přidat`, `PropojitData` |
| FM-33, FM-37 | `Export_kontrol` |
| FM-38, FM-40 | `Email_2015`, `opravy` |
| FM-34 | `Kon`, `OdstrSes` |
| FM-12, FM-39 | `Najít duplicity pro Seznam` |
| FM-35, FM-36 | `Přev`, `SestP2`, `SestP2_Adrs`, `Ukon` |
| FM-41–FM-43 | `OdstrUkon` |
| FM-20, FM-43 | `PojTest` |
| FM-41 | `Sest` |

### Sestavy (31/31)

| Funkční ID | Access sestavy |
| --- | --- |
| FM-20 | `Doklad`, `Doklad_` |
| FM-21 | `Form_DataKlientů`, `Přihláška`, `Přihláška_`, `Přihláška_Old` |
| FM-22 | `AdresPojPodm`, `NálepkyAdr`, `NálepkyForm`, `NálepkyZO`, `ObálkaDL` |
| FM-23 | `PoštPoukázka`, `PoštPoukázka_`, `PoštPoukázka_S` |
| FM-29 | `PojUdálost`, `PojUdálost_sest` |
| FM-31 | `SestPoj`, `SestPojEmail` |
| FM-32, FM-39 | `SestavaOC`, `SestKontr` |
| FM-33 | `SestavaZO`, `SestavaZO_Potvrz`, `SestavaZO_Počty`, `Sest_Odes` |
| FM-34 | `SestPřehl` |
| FM-35 | `SeznamOd`, `SeznamKonec`, `Seznam_podsest`, `Seznam_podsest_` |
| FM-39 | `Prázdná čísla` |
| FM-40 | `Procházka Jiří 2022` |

## Implementační zásady pro chybějící funkce

1. Tiskové výstupy nejprve porovnat s definicemi a screenshoty v balíčku; Access nespouštět. Referenční očekávání převést do anonymizovaných fixture dat a snapshot testů PDF.
2. Nepřenášet pevné cesty `C:\Pojistka`, `C:\Federace` nebo `D:\Pojistka`; používat dialog uložení a aplikační datový adresář.
3. Pomocné make-table/append workflow nahradit parametrizovaným SQL a transakcemi. Dočasné tabulky vytvářet jen tam, kde je prokazatelně nutný stabilní snapshot.
4. Každá dávková nebo destruktivní operace musí mít náhled dopadu, potvrzení, roli správce, automatickou zálohu tam, kde mění roční data, a audit bez osobních údajů v textu logu.
5. Historické objekty se jmény `Old`, konkrétním rokem či osobou neoznačovat automaticky jako vyřazené. Nejprve získat rozhodnutí Product Ownera; do té doby jsou pokryty obecným backlogem P3.
