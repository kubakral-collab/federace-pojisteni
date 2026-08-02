# SPRINT POJISTENI-03 — Scope hlavního pracovního toku

> Historický referenční scope; aktuální plán určuje výhradně `ROADMAP.md`.

> **Stav: SCHVÁLENO A UZAMČENO**
>
> Referenční scope pro implementaci Sprintu POJISTENI-03. Změny rozsahu, terminologie, workflow nebo obchodní logiky vyžadují nové výslovné schválení.

## Rozhodnutí o workflow

Pro první implementaci je vybrán uzavřený pracovní tok **založení nového pojištěnce**.

`Vstup → Switchboard → Pojištěnci → Editace → Seznam → návrat na Switchboard`

Formulář `Pojištěnci` je první hlavní datový vstup dostupný ze `Switchboard` přes `Příkaz30_Click`. Tok zahrnuje zadání, výpočty, validaci, uložení i storno. Vyhledávání, úhrady, pojistné události a sestavy jsou samostatné větve mimo tento sprint.

Cílem projektu není vytvořit lepší verzi Accessu, ale vytvořit moderní aplikaci se 100% funkční paritou vůči původnímu systému. Jakékoli nové funkce budou řešeny až po dokončení kompletní migrace.

## Výchozí Access formulář

### `Vstup`

- Po otevření nastaví fokus na `heslo`.
- `OK` ověří heslo. Při úspěchu zavře `Vstup` a otevře `Switchboard`.
- Neúspěch zobrazí `Neplatné heslo - přístup není povolen!` s titulkem `Nesprávné heslo`.
- `Příkaz8` zobrazí `Program bude ukončen!` s titulkem `Ukončení` a ukončí aplikaci.
- `Příkaz7` provede `FindNext`.
- `Text1_GotFocus` a `Text3_GotFocus` přesunou fokus na `OK`.

Hodnota původního hesla nebude uvedena v dokumentaci ani rozhraní.

## Navázané formuláře

### `Switchboard`

- Hlavní rozcestník aplikace.
- `Příkaz30` otevře `Pojištěnci` a skryje `Switchboard`.
- Ostatní větve rozcestníku nejsou součástí workflow.

### `Pojištěnci`

- Při otevření skryje `Switchboard`.
- Inicializuje `LastEv` a `LastPříj` na první dostupnou hodnotu.
- Umožňuje nový záznam uložit, uložit a zavřít nebo stornovat.
- Při zavření znovu zobrazí `Switchboard`.

## Použité dotazy

Žádný uložený Access dotaz se v tomto workflow přímo nespouští podle názvu. Používají se vložené SQL příkazy pro `Editace` a `Seznam`, zdroje řádků, `Requery` pomocných prvků a vyhledání záznamu podle `RočPojistné`.

## Použité tabulky

| Tabulka | Účel | Přístup |
|---|---|---|
| `Seznam` | Hlavní evidence pojištěnců | Čtení a vložení |
| `Editace` | Dočasná mezitabulka před přenosem do `Seznam` | Vložení a vyčištění |
| `Kategorie` | Podpůrný číselník | Pouze čtení |

Uložení bude atomické; po úspěchu ani chybě nesmí zůstat rozpracovaný obsah v `Editace`.

## Pole a pořadí

1. `Titul`
2. `Příjmení`
3. `Jméno`
4. `RodnéČíslo`
5. `ZO`
6. `OdbPříslušnost`
7. `Město`
8. `Adresa`
9. `PSČ`
10. `Stát`
11. `Poznámka`
12. `PojištěníOd`
13. `PojištěníDo`
14. `RočPojistné`
15. `Kategorie`
16. `Ztráta`
17. `PojistnáČástka`
18. `SkutÚhrada`
19. `KódOC`
20. `EvČíslo`
21. `E-mail`

Původní aliasy: `Počátek` → `PojištěníOd`, `Konec` → `PojištěníDo`, `RočPoj` → `RočPojistné`, `Kat` → `Kategorie`, `Částka` → `PojistnáČástka`.

Vazba prvku `OC` na pozici `OdbPříslušnost` v původním ukládacím SQL musí být před implementací ověřena proti skutečným datům; `KódOC` je současně samostatné pole.

## Tlačítka a akce

| Formulář | Prvek | Akce |
|---|---|---|
| `Vstup` | `OK` | Ověří heslo a otevře `Switchboard`, nebo zobrazí původní chybu |
| `Vstup` | `Příkaz7` | `FindNext` |
| `Vstup` | `Příkaz8` | Oznámí ukončení a ukončí aplikaci |
| `Switchboard` | `Příkaz30` | Otevře `Pojištěnci` a skryje rozcestník |
| `Pojištěnci` | `Nový` | Uloží přes `Editace` do `Seznam`, vyčistí `Editace` a otevře čistý formulář |
| `Pojištěnci` | `Zavřít` | Stejně uloží a zavře formulář |
| `Pojištěnci` | `Storno` | Zavře bez ukládacích SQL příkazů |
| `Pojištěnci` | `Příkaz38` | Zobrazí poslední zavedené evidenční číslo |
| `Pojištěnci` | `Seznam73` | Najde první záznam se zvoleným `RočPojistné` |

Další zachované události:

- `KódOC_AfterUpdate`: při `KódOC = 1` nastaví `OdbPříslušnost` na `FVČ`, jinak `FV`.
- `OC_AfterUpdate`: obnoví `ZO`.
- `Rok_AfterUpdate`: obnoví `LastEv` a `LastPříj`.
- `Měs_AfterUpdate`: obnoví `měs` a `částka`.
- `Kat_AfterUpdate`, `Kategorie_AfterUpdate`, `RočPoj_AfterUpdate`, `Ztráta_AfterUpdate`, `Konec_AfterUpdate` a `Počátek_AfterUpdate` přepočítají nebo obnoví `měs`, `částka` a `Pojistné`.
- `KódOC_GotFocus`, `LastEv_GotFocus`, `LastPříj_GotFocus` přesunou fokus na `Titul`.
- `Text25_GotFocus` přesune fokus na `SkutÚhrada`.

Návrhový export formuláře potvrdil, že z dříve nejasných názvů je ovládacím prvkem pouze `Příkaz38` s titulkem `Poslední`, nápovědou `Zobrazit poslední evidenční číslo` a událostí `[Event Procedure]`. `Příkaz18`, `Příkaz19`, `Příkaz20`, `Příkaz37` a `Příkaz43` v aktuálním návrhu formuláře neexistují a jejich osiřelé VBA procedury se neimplementují. Úplný nález je v `docs/pojistenci-controls-export.md`.

## Validace

Původní tarifní matice se převezme beze změn:

| `RočPojistné` | B / 0 | B / -1 | A / 0 | A / -1 | C / 0 | C / -1 |
|---:|---:|---:|---:|---:|---:|---:|
| 200000 | 495 | 594 | 950 | 1146 | 1901 | 2294 |
| 240000 | 536 | 644 | 1030 | 1311 | 2059 | 2624 |
| 280000 | 578 | 743 | 1242 | 1427 | 2485 | 2855 |
| 320000 | 710 | 858 | 1361 | 1641 | 2723 | 3284 |
| 360000 | 849 | 981 | 1509 | 1856 | 3020 | 3713 |
| 400000 | 950 | 1146 | 1674 | 2276 | 3350 | 4455 |

Sloupce kombinují `Kategorie` a `Ztráta`. Výsledek se zapisuje do původního prvku `Pojistné` nebo `Částka` podle konkrétní události.

Tarifní matice musí být v implementaci definována právě na jednom centrálním místě: v databázové tabulce, konfiguračním souboru nebo jediném centrálním modulu. Komponenty a funkce smějí tento zdroj pouze používat. Duplicitní `switch`/`case`, více kopií matice a ručně přepisované sazby na více místech jsou nepřípustné.

- Datová pravidla se převezmou pouze z vlastností Access formuláře, tabulek a VBA.
- Chybějící hodnoty se zpracují bez pádu a bez vymyšleného doplňování.
- Uživatelské chyby budou česky a bez technických detailů.
- Před prvním zápisem vznikne záloha databáze.
- Vložení do `Editace`, přenos do `Seznam`, vyčištění `Editace` a auditní záznam proběhnou v jedné transakci.
- Chyba vrátí celou transakci zpět.
- Workflow nemaže existujícího pojištěnce. Vyčištění technické tabulky `Editace` není uživatelské mazání.

## Auditní log

Auditní log slouží pouze jako provozní evidence změn. Není požadováno ukládat jednotlivé hodnoty všech polí.

Každý pokus o zápis musí obsahovat alespoň:

- datum a čas,
- přihlášeného uživatele,
- typ operace `INSERT`,
- identifikátor pojištěnce,
- výsledek operace `OK` nebo `ERROR`.

Úspěšný auditní záznam `OK` vznikne v transakci společně s uložením pojištěnce. Záznam `ERROR` nesmí narušit návrat neúspěšné datové transakce ani vystavit uživateli technické detaily.

## Tisky a exporty

Tento pracovní tok nespouští žádnou sestavu, tisk ani export. Ve Sprintu 03 se proto žádný takový výstup nepřidá.

## Přesný nový ekvivalent

1. Aplikace otevře `Vstup` s fokusem na `heslo`.
2. `OK` zachová původní úspěšný i neúspěšný průběh.
3. Po přihlášení se zobrazí `Switchboard`.
4. Ekvivalent `Příkaz30` otevře `Pojištěnci` a skryje rozcestník.
5. `Pojištěnci` zachová pole, pořadí, pomocné prvky, fokus, obnovování a tarifní výpočty.
6. `Nový` provede validaci, zálohu před prvním zápisem, transakční uložení a auditní záznam a otevře čistý formulář.
7. `Zavřít` provede stejné uložení, zavře formulář a zobrazí `Switchboard`.
8. `Storno` zavře bez uložení a zobrazí `Switchboard`.
9. Selhání vrátí zápis zpět a zobrazí srozumitelnou českou zprávu.

Záloha, transakce a auditní log nevytvoří nový uživatelský krok.

## Objekty mimo rozsah

- `Hledat`, `Přehled`, `PřPod1`, `PřPod2`, `PřPod3`, `SestHVP`
- `Doklad`, `PoštPoukázka`, `Úhrada`, `Faktury`
- `Poj_udalost`, `Poj_Udalost_mail`, `Poj_Udalost_upr`
- `Archivace`, `PřenosDat`, `Převod_data`, `Heslo`
- všechny sestavy, tisky a exportní větve
- ostatní příkazy `Switchboard`
- historické a pravděpodobně nepoužívané objekty
- vyhledávání, editace a mazání klientů, úhrady a pojistné události

## Implementační brána

**Stav: UZAVŘENO**

Otevřené body byly read-only analýzou původního Accessu vyřešeny:

1. Z původně nejasných prvků existuje v návrhu pouze `Příkaz38`; osiřelé procedury se neimplementují.
2. `OC` se ukládá do `Seznam.OdbPříslušnost`.
3. `KódOC` se ukládá do `Seznam.KódOC`.
4. `ZO` se ukládá do `Seznam.ZO`.

Scope byl schválen a implementace Sprintu POJISTENI-03 byla povolena.

## Implementační disciplína

- Access databáze je jediný zdroj pravdy pro chování původní aplikace.
- SQLite je datový zdroj nové aplikace, nikoli zdroj obchodních pravidel migrace.
- Chování, které není jednoznačně doloženo v Accessu, se nesmí domýšlet.
- Každá nejasnost se nejprve ověří v původní aplikaci a teprve poté implementuje.
- Zachová se terminologie, pořadí kroků, workflow a obchodní logika.
- Modernizovat se smí pouze technické provedení v Tauri, Rustu a SQLite.
- Nové funkce a změny workflow jsou mimo migraci a mohou se řešit až po dosažení úplné funkční parity.
