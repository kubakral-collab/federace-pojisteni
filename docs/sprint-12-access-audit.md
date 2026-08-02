# Sprint 12 — Audit pojistných událostí v Accessu

> Historický forenzní scope; aktuální plán určuje výhradně `ROADMAP.md`.

## Stav

**Fáze A dokončena. Implementace ani návrh migračního schématu nebyly zahájeny.**

Audit proběhl nad databází `Pojištění.accdb` pouze pro čtení. Návrhy formulářů a sestav byly exportovány z bitově shodné pracovní kopie. Původní Access ani zdrojová `dd.sqlite` nebyly změněny.

## Závěr auditu

Původní Access obsahuje aktivně používaný modul pojistných událostí se 112 záznamy z let 2012–2026. Záznamy jsou vedeny v tabulce `Poj_udalost`, připravovaný nový záznam prochází pomocnou tabulkou `Poj_udalost_zdroj` a tabulka `Poj_udalost_` je starší kopie s 9 záznamy.

Access nepoužívá samostatný číselník typů ani stavů. Číslem události je automatické číselné pole `ID`. Stav je odvozen pouze z vyplnění data `Ukončeno`; žádný samostatný sloupec stavu neexistuje.

Událost je spojena s tabulkou `Seznam` přes `Identifikátor`. Vazba neobsahuje jednoznačný odkaz na konkrétní roční pojistný záznam. To je hlavní otevřený bod pro Fázi B.

## Datové tabulky

### `Poj_udalost`

- Účel: hlavní historická a provozní evidence pojistných událostí.
- Počet záznamů: **112**.
- Primární klíč: `ID`, datový typ AutoNumber/Long, jedinečný.
- Vazba na člena: `Identifikátor` → `Seznam.Identifikátor`.
- Access nemá deklarovanou relaci ani cizí klíč; vazba vzniká pouze v SQL formulářů a sestav.

| Pole | Typ Access | Povinné | Význam podle formuláře |
|---|---|---:|---|
| `ID` | AutoNumber | ne | Interní číslo pojistné události |
| `Identifikátor` | Long | ne | Vazba na člena/pojistný záznam v `Seznam` |
| `Povolání` | Text(255) | ne | Povolání pojištěnce |
| `Zaměstnavatel` | Text(255) | ne | Zaměstnavatel |
| `Oznámení_PU` | DateTime | ne | Datum oznámení pojistné události |
| `Vznik_PU` | DateTime | ne | Datum vzniku pojistné události |
| `Popis_Události` | Long Text | ne | Popis pojistné události |
| `Poznámka1` | Long Text | ne | Poznámky |
| `Poznámka2` | Long Text | ne | Doplňky a informace |
| `Zjištěná_škoda` | Double | ne | Zjištěná škoda |
| `Pojistné_plnění` | Double | ne | Pojistné plnění |
| `Ukončeno` | DateTime | ne | Vypořádání/ukončení |
| `Řeší_pojišťovna` | Text(255) | ne | Volný text „Řeší makléř s pojišťovnou“ |
| `Poloha v sestavě` | Text(255) | ne | Umístění textu ve výstupní sestavě |
| `Telefon` | Text(255) | ne | Kontaktní telefon pro událost |
| `E-mail` | Text(255) | ne | E-mail; hlavní formulář však čte e-mail také ze `Seznam` |

### `Poj_udalost_zdroj`

- Účel: jednořádková technická tabulka pro přípravu náhledu a tisku nové události.
- Počet záznamů při auditu: **1**.
- Nemá primární klíč.
- Před vytvořením nové události ji VBA vymaže, vloží do ní hodnoty formuláře, vytvoří PDF a teprve potom data přidá do `Poj_udalost`.
- Obsahuje stejná provozní pole jako hlavní tabulka kromě `ID`.

### `Poj_udalost_`

- Účel: starší nebo pracovní kopie hlavní tabulky.
- Počet záznamů: **9**.
- Struktura odpovídá starší variantě `Poj_udalost`; chybí pole `E-mail`.
- Žádný nalezený aktivní formulář, dotaz ani VBA tuto tabulku nepoužívá.
- Klasifikace: **historický nebo pravděpodobně nepoužívaný objekt**.
- Ve Fázi B musí být její záznamy porovnány s hlavní tabulkou; nesmí být automaticky importovány jako nové události bez kontroly duplicit.

## Uložené dotazy

| Dotaz | Typ | Funkce |
|---|---|---|
| `Dotaz1` | Přidávací | Kopíruje připravený záznam z `Poj_udalost_zdroj` do `Poj_udalost`. |
| `Dotaz5` | Přidávací | Plní `Poj_udalost_zdroj` hodnotami z formuláře `Poj_Udalost`. |
| `~sq_fPoj_Udalost_mail` | Systémový formulářový | Načítá `Identifikátor` a e-mail člena ze `Seznam`. |
| `~sq_fPoj_Udalost_upr` | Systémový formulářový | Načítá vybranou událost podle `Poj_udalost.ID`. |
| `~sq_rPojUdálost_sest` | Systémový sestavový | Spojuje události se členy a vytváří roční seznam pro tisk. |
| `test` | Výběrový | Detail jedné události spojený s členskými a pojistnými údaji. |

Aktivní VBA používá stejné SQL také přímo jako vložené příkazy, nikoli pouze přes názvy `Dotaz1` a `Dotaz5`.

## Formuláře

### `Poj_udalost`

Jeden formulář slouží ve dvou režimech:

1. `ID = 1` — založení nové události.
2. `ID = 2` — prohlížení, úprava a tisk existující události nebo roční sestavy.

#### Výběr člena

- `Rok` načítá roky z `Seznam.PojištěníOd` při založení a z `Poj_udalost.Vznik_PU` v archivu.
- `OC` načítá `KódOC` a `OdbPříslušnost`.
- `ZO` je filtrováno podle `KódOC`.
- `ZOsez` zobrazuje člena nebo existující událost; výběr se dokončuje dvojklikem.
- Při založení je zdrojem seznam členů `Seznam`.
- Při prohlížení je zdrojem `Poj_udalost INNER JOIN Seznam`.

#### Zobrazené členské a pojistné údaje

- Pojištěnec
- Rodné číslo
- Pojistné období
- Typ pojištění
- Evidenční číslo
- Bydliště
- Odborová příslušnost
- E-mail
- Interně skrytý `Identifikátor`

#### Editovatelná pole události

- Telefon
- Poloha v sestavě
- Zaměstnavatel
- Povolání
- Datum vzniku PU
- Datum oznámení PU
- Zjištěná škoda
- Pojistné plnění
- Popis PU
- Poznámky
- Doplňky a informace
- Vypořádání – ukončení
- Řeší makléř s pojišťovnou

#### Tlačítka a akce

| Prvek | Viditelný text | Akce |
|---|---|---|
| `Zobr` | Uložit a zobrazit | V novém režimu připraví pomocný záznam, vytvoří PDF a přidá událost do hlavní tabulky. V archivním režimu vytiskne detail nebo roční sestavu. |
| `Storno` | Zavřít | Zavře formulář a znovu zobrazí hlavní nabídku. |
| `ZázUpr` | Upravit záznamy | Otevře `Poj_Udalost_upr`. |
| `EmailUpr` | Upravit e-mail | Otevře `Poj_Udalost_mail`. |
| `Sest` | Tisk sestavy | Přepíná detailní tisk a roční sestavu. |
| `ZobrOC` | Zobrazit OC | Omezuje archiv a sestavu na vybrané OC. |

### `Poj_Udalost_upr`

- Navázaný formulář pro přímou úpravu vybraného řádku `Poj_udalost`.
- Obsahuje všechna provozní pole události kromě vazby na člena a pole `Řeší_pojišťovna`.
- Uložení probíhá standardním navázaným Access formulářem při zavření.
- Tlačítko `Zavřít` uloží změny, vrátí hlavní formulář a provede `Requery`.

### `Poj_Udalost_mail`

- Navázaný formulář nad `Seznam`.
- Umožňuje upravit e-mail člena.
- Tlačítko `Zavřít` uloží změnu a vrátí hlavní formulář.
- Tato funkce mění členský záznam, nikoli pojistnou událost.

## Hlavní nabídka

Formulář `Switchboard` obsahuje dvě samostatné větve:

- `Příkaz78_Click`: otevře novou pojistnou událost (`ID = 1`).
- `Příkaz80_Click`: otevře přehled a úpravy existujících událostí (`ID = 2`).

Toto potvrzuje požadavek na dvě samostatné položky nové navigace.

## Číslování

- Access nepoužívá formát `PU-rok-pořadí`.
- Jediným číslem události je `Poj_udalost.ID`.
- `ID` je AutoNumber a primární klíč.
- V aktuálních datech je 112 jedinečných hodnot v rozsahu 1–115; mezery jsou důsledkem smazaných nebo neuložených AutoNumber hodnot.
- Ve Fázi B musí být původní `ID` zachováno jako historické číslo. Nový formát čísla se nesmí zavést bez výslovného schválení Federace.

## Stavy a typy

### Stav

- Samostatný stavový sloupec ani číselník neexistuje.
- `Ukončeno IS NULL` odpovídá otevřené/nevypořádané události.
- Vyplněné `Ukončeno` odpovídá uzavřené/vypořádané události.
- Otevřených podle tohoto pravidla je **39**, uzavřených **73**.
- Doporučené stavy ze zadání se nesmí implementovat jako parita Accessu.

### Typ události

- Samostatné pole ani číselník typu události neexistuje.
- Povaha události je zachycena pouze volným textem `Popis_Události`, poznámkami a částkami.
- Filtr „typ události“ nelze implementovat bez nového obchodního pravidla schváleného Federací.

## Historická data

| Rok vzniku | Počet |
|---:|---:|
| 2026 | 1 |
| 2025 | 11 |
| 2024 | 22 |
| 2023 | 4 |
| 2022 | 7 |
| 2021 | 11 |
| 2020 | 8 |
| 2019 | 10 |
| 2018 | 3 |
| 2017 | 11 |
| 2016 | 6 |
| 2015 | 7 |
| 2014 | 1 |
| 2013 | 7 |
| 2012 | 1 |
| Bez data vzniku | 2 |

Další kvalita dat:

- 112/112 záznamů má `ID` a `Identifikátor`.
- 110/112 má datum vzniku.
- 106/112 má datum oznámení.
- 95/112 má zjištěnou škodu.
- 71/112 má pojistné plnění.
- Všechny události lze přes `Identifikátor` spojit alespoň s jedním řádkem `Seznam`.
- 99 událostí má právě dostupnou vazbu na pojistný rok odpovídající roku `Vznik_PU`.
- 13 událostí nemá pojistný řádek stejného roku.
- U jedné události odpovídá roku a identifikátoru více než jeden řádek `Seznam`.
- Tyto případy musí být ve Fázi B zařazeny do migračního reportu a nesmí být automaticky přiřazeny odhadem.

## Tiskové výstupy

### `PojUdálost`

- Detail jedné nové nebo existující události.
- Spojuje členské údaje, pojistné období a údaje události.
- Nová událost se tiskne z `Poj_udalost_zdroj`, existující z `Poj_udalost`.
- VBA ukládá PDF do pevné historické cesty `D:\Pojistka\Škodní událost\`.
- Název detailního PDF je odvozen pouze od jména pojištěnce, takže může kolidovat.

### `PojUdálost_sest`

- Roční seznam pojistných událostí.
- Filtruje podle roku `Vznik_PU`.
- Volitelně filtruje podle `Seznam.KódOC`.
- Výstupní soubor má pevný název `SestavaPU.pdf`.

## E-mailové workflow a dokumenty

- V modulech pojistných událostí nebylo nalezeno `DoCmd.SendObject`, automatizace Outlooku ani jiné automatické odesílání.
- Formulář `Poj_Udalost_mail` pouze ručně upravuje e-mail uložený v `Seznam`.
- E-mail je použit jako údaj v sestavě, nikoli jako automatický příjemce.
- Nebyla nalezena tabulka příloh, pole cesty k dokumentu ani workflow připojování souborů.
- Automatické odesílání e-mailu ani správa dokumentů proto nejsou součástí doložené funkční parity.

## Přesné původní workflow

### Nová událost

1. Uživatel otevře větev nové pojistné události.
2. Vybere rok, OC, ZO a člena.
3. Dvojklikem načte členské a pojistné údaje.
4. Doplní kontaktní a pracovní údaje a údaje události.
5. Stiskne `Uložit a zobrazit`.
6. Access vymaže `Poj_udalost_zdroj`.
7. Access vloží do pomocné tabulky aktuální hodnoty formuláře.
8. Access vytvoří a zobrazí PDF sestavy `PojUdálost`.
9. Access zkopíruje data do `Poj_udalost`.
10. Access zavře sestavu.

VBA nepoužívá explicitní transakci. Nová implementace musí tento tok modernizovat transakcí, ale nesmí měnit jeho datový význam.

### Archiv a úprava

1. Uživatel otevře větev existujících pojistných událostí.
2. Vybere rok vzniku.
3. Volitelně omezí seznam na OC.
4. Dvojklikem vybere událost.
5. Může vytisknout detail, roční sestavu, upravit událost nebo upravit e-mail člena.

## Klasifikace objektů

| Objekt | Klasifikace | Důvod |
|---|---|---|
| `Poj_udalost` | Používaný | Hlavní evidence, formuláře, dotazy a sestavy. |
| `Poj_udalost_zdroj` | Technický pomocný | Jednořádková příprava nové události a tisku. |
| `Poj_udalost_` | Historický / pravděpodobně nepoužívaný | Není odkazována aktivním workflow. |
| `Poj_udalost` formulář | Používaný | Nová událost i archiv. |
| `Poj_Udalost_upr` | Používaný | Úprava existující události. |
| `Poj_Udalost_mail` | Používaný pomocný | Úprava e-mailu člena. |
| `PojUdálost` | Používaný | Detailní PDF. |
| `PojUdálost_sest` | Používaný | Roční/OC sestava. |
| `Dotaz1`, `Dotaz5` | Používané nebo zdvojené technické | Stejná SQL logika je také vložena ve VBA. |
| Systémové `~sq_*` dotazy | Technické pomocné | Uložené zdroje formulářů a sestav. |

## Otevřené body před Fází B

1. Potvrdit, zda nové uživatelské číslo události má zůstat původní číselné `ID`, nebo zda Federace schválí nový formát. Access formát `PU-2026-0001` nepoužívá.
2. Rozhodnout, jak přiřadit 13 událostí bez pojistného záznamu stejného roku a jednu nejednoznačnou roční vazbu.
3. Potvrdit, zda tabulka `Poj_udalost_` představuje zálohu, nebo obsahuje historické záznamy, které v hlavní tabulce chybí.
4. Potvrdit, zda stav má zůstat pouze `Otevřená`/`Uzavřená` podle data `Ukončeno`. Podrobnější seznam stavů není v Accessu doložen.
5. Potvrdit, zda má být zaveden nový číselník typu události. Access typ neeviduje.
6. Potvrdit, zda změna e-mailu člena z modulu událostí má být zachována. Jde o změnu členských dat mimo samotnou událost.
7. Potvrdit požadovaný význam pole `Poloha v sestavě`, které je v datech i formuláři, ale není obchodně popsáno.

Do vyřešení těchto bodů nesmí začít Fáze B ani implementace.
