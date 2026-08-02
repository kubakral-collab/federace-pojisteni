# Audit původní Access aplikace

> Historický forenzní dokument; není aktuální produktovou roadmapou.

## Identifikace zdroje

- **Zdroj pravdy:** `C:\Users\kubak\Desktop\Federace\Access_do_SQLite_OPRAVA\Pojištění.accdb`
- **SHA-256 originálu:** `72C717E3D9C41E510F01CC814A78C451443275B89BF811224A76A7A59C5FDE32`
- **Způsob auditu:** bitově shodná pracovní kopie; DAO 12.0 katalog; ODBC pouze pro čtení; rekonstrukce VBA projektu z `MSysAccessStorage`; vizuální kontrola spuštění v Accessu s blokovaným aktivním obsahem.
- **SQLite:** pro audit objektů ani workflow nebyla použita; zůstává pouze zdrojem dat nové aplikace.

## Souhrn objektů

| Typ | Počet | Úplnost |
|---|---:|---|
| Tabulky a připojené tabulky | 26 | DAO katalog |
| Uložené dotazy | 35 | DAO QueryDefs |
| Formuláře | 33 | DAO Forms + VBA |
| Sestavy | 31 | DAO Reports + VBA |
| Makra | 6 | DAO Scripts |
| Standardní moduly | 16 | DAO Modules + VBA |
| Vytěžené kódové moduly celkem | 75 | 338 procedur |
| Definované relace DAO | 0 | V souboru nejsou deklarované relace |

## Tabulky

| Tabulka | Účel | Zařazení | Sloupce |
|---|---|---|---:|
| `2002` | Historický roční snímek evidence klientů. | Historická / pravděpodobně nepoužívaná | 19 |
| `2003` | Historický roční snímek evidence klientů. | Historická / pravděpodobně nepoužívaná | 21 |
| `2004` | Historický roční snímek evidence klientů. | Historická / pravděpodobně nepoužívaná | 21 |
| `2005` | Historický roční snímek evidence klientů. | Historická / pravděpodobně nepoužívaná | 21 |
| `2006` | Historický roční snímek evidence klientů. | Historická / pravděpodobně nepoužívaná | 22 |
| `2007` | Historický roční snímek evidence klientů. | Historická / pravděpodobně nepoužívaná | 22 |
| `2008` | Historický roční snímek evidence klientů. | Historická / pravděpodobně nepoužívaná | 22 |
| `2009` | Historický roční snímek evidence klientů. | Historická / pravděpodobně nepoužívaná | 22 |
| `2010` | Historický roční snímek evidence klientů. | Historická / pravděpodobně nepoužívaná | 23 |
| `Břeclav` | Lokální import nebo historický výřez klientů z Břeclavi. | Používaná nebo provozně významná | 27 |
| `Členská základna` | Připojená členská základna; dotaz PropojitData do ní zapisuje příznak pojištění. | Používaná nebo provozně významná | 0 |
| `Editace` | Prázdná pomocná tabulka pro editaci. | Používaná nebo provozně významná | 22 |
| `Email` | Připojený zdroj e-mailových údajů. | Používaná nebo provozně významná | 0 |
| `Faktura` | Evidence faktur a platebních údajů. | Používaná nebo provozně významná | 10 |
| `Faktura_copy` | Pracovní kopie faktur pro přenos nebo export. | Historická / pravděpodobně nepoužívaná | 9 |
| `Kategorie` | Číselník kategorií. | Používaná nebo provozně významná | 6 |
| `Odklad` | Odložené záznamy připravené k doplnění do evidence. | Používaná nebo provozně významná | 16 |
| `Poj_udalost` | Evidence pojistných událostí. | Používaná nebo provozně významná | 16 |
| `Poj_udalost_` | Starší nebo pracovní kopie pojistných událostí. | Používaná nebo provozně významná | 15 |
| `Poj_udalost_zdroj` | Dočasný zdroj pro vytvoření a tisk pojistné události. | Technická pomocná | 15 |
| `Převod_` | Pracovní nebo historická tabulka pro převod dat. | Používaná nebo provozně významná | 24 |
| `Sestavy` | Pomocná data o vytvořených sestavách. | Technická pomocná | 7 |
| `SestavyHVP` | Pomocná data pro sestavy HVP. | Technická pomocná | 7 |
| `Seznam` | Hlavní evidence pojištěnců a jejich pojištění. | Používaná nebo provozně významná | 27 |
| `Seznam_` | Starší nebo pracovní kopie evidence pojištěnců. | Používaná nebo provozně významná | 27 |
| `Switchboard Items` | Technická konfigurace staršího Access switchboardu. | Technická pomocná | 5 |

## Uložené dotazy

| Dotaz | Typ | Funkce | Zařazení |
|---|---|---|---|
| `200T` | aktualizační | Aktualizace hodnot v DISTINCTROW. | Používaná nebo provozně významná |
| `240T` | aktualizační | Aktualizace hodnot v DISTINCTROW. | Používaná nebo provozně významná |
| `280T` | aktualizační | Aktualizace hodnot v DISTINCTROW. | Používaná nebo provozně významná |
| `320T` | aktualizační | Aktualizace hodnot v DISTINCTROW. | Používaná nebo provozně významná |
| `360T` | aktualizační | Aktualizace hodnot v DISTINCTROW. | Používaná nebo provozně významná |
| `400T` | aktualizační | Aktualizace hodnot v DISTINCTROW. | Používaná nebo provozně významná |
| `Copy_fakt2` | vytvářecí | Vytvoření pracovní tabulky Fakt_copy. | Historická / pravděpodobně nepoužívaná |
| `Copy_faktur` | přidávací | Přidání záznamů do Fakt_copy. | Historická / pravděpodobně nepoužívaná |
| `Dotaz1` | přidávací | Přidání záznamů do Poj_udalost. | Používaná nebo provozně významná |
| `Dotaz2` | přidávací | Přidání záznamů do Faktura. | Používaná nebo provozně významná |
| `Dotaz3` | vytvářecí | Vytvoření pracovní tabulky Faktura_copy. | Používaná nebo provozně významná |
| `Dotaz5` | přidávací | Přidání záznamů do Poj_udalost_zdroj. | Používaná nebo provozně významná |
| `Dotaz6` | přidávací | Přidání záznamů do Seznam. | Používaná nebo provozně významná |
| `Duplicitní hodnoty v tabulce Faktura` | výběrový | Výběr nebo kontrolní pohled nad Faktura. | Používaná nebo provozně významná |
| `Email_2015` | výběrový | Výběr nebo kontrolní pohled nad daty. | Historická / pravděpodobně nepoužívaná |
| `Export_kontrol` | výběrový | Výběr nebo kontrolní pohled nad daty. | Používaná nebo provozně významná |
| `KlientDelete` | přidávací | Přidání záznamů do Faktura. | Používaná nebo provozně významná |
| `Kon` | přidávací | Přidání záznamů do Sestavy. | Používaná nebo provozně významná |
| `Najít duplicity pro Seznam` | výběrový | Výběr nebo kontrolní pohled nad Seznam. | Používaná nebo provozně významná |
| `Odklad_přidat` | přidávací | Přidání záznamů do Seznam. | Používaná nebo provozně významná |
| `OdstrSes` | odstraňovací | Mazání záznamů v Sestavy.PořČíslo. | Používaná nebo provozně významná |
| `OdstrUkon` | odstraňovací | Mazání záznamů v Seznam1.*. | Používaná nebo provozně významná |
| `opravy` | výběrový | Výběr nebo kontrolní pohled nad daty. | Historická / pravděpodobně nepoužívaná |
| `PojAkt` | aktualizační | Aktualizace hodnot v DISTINCTROW. | Používaná nebo provozně významná |
| `PojAkt_` | aktualizační | Aktualizace hodnot v DISTINCTROW. | Používaná nebo provozně významná |
| `PojTest` | aktualizační | Aktualizace hodnot v DISTINCTROW. | Historická / pravděpodobně nepoužívaná |
| `PropojitData` | aktualizační | Aktualizace hodnot v Členská. | Používaná nebo provozně významná |
| `Přenos` | vytvářecí | Vytvoření pracovní tabulky Převod. | Používaná nebo provozně významná |
| `Přev` | vytvářecí | Vytvoření pracovní tabulky Převod. | Používaná nebo provozně významná |
| `PřipojDatakSeznamu` | přidávací | Přidání záznamů do Seznam. | Používaná nebo provozně významná |
| `Sest` | přidávací | Přidání záznamů do Seznam1. | Používaná nebo provozně významná |
| `SestP2` | výběrový | Výběr nebo kontrolní pohled nad daty. | Používaná nebo provozně významná |
| `SestP2_Adrs` | výběrový | Výběr nebo kontrolní pohled nad daty. | Používaná nebo provozně významná |
| `test` | výběrový | Výběr nebo kontrolní pohled nad daty. | Historická / pravděpodobně nepoužívaná |
| `Ukon` | vytvářecí | Vytvoření pracovní tabulky převod. | Používaná nebo provozně významná |

### Parametry a vazby dotazů

Dotazy odkazují zejména na formuláře `PoštPoukázka`, `Faktury`, `SestavaZO`, `SestHVP`, `SestPřehl`, `Přehled`, `PřenosDat`, `Převod_data`, `Poj_Udalost` a `Úhrada`. Akční dotazy mění tabulky `Seznam`, `Faktura`, `Poj_udalost`, `Poj_udalost_zdroj`, `Převod`, `Sestavy` a připojenou `Členská základna`; v nové aplikaci musí být volané jen ze stejného workflow a až po samostatném schválení zápisové fáze.

## Makra

| Makro | Zařazení | Poznámka |
|---|---|---|
| `~TMPCLPMacro` | Technická pomocná | Dočasný systémový objekt. |
| `Kontr` | Používaná nebo provozně významná | Původní pojmenování zachovat; související převedené makro je uvedeno mezi moduly. |
| `Makro1` | Používaná nebo provozně významná | Původní pojmenování zachovat; související převedené makro je uvedeno mezi moduly. |
| `Makro2` | Používaná nebo provozně významná | Původní pojmenování zachovat; související převedené makro je uvedeno mezi moduly. |
| `Makro3` | Používaná nebo provozně významná | Původní pojmenování zachovat; související převedené makro je uvedeno mezi moduly. |
| `test` | Historická / pravděpodobně nepoužívaná | Původní pojmenování zachovat; související převedené makro je uvedeno mezi moduly. |

## VBA moduly

| Modul | Zařazení | Funkce |
|---|---|---|
| `global` | Používaná nebo provozně významná | `Doklad`, `DokladJedn`, `ŠtítkyDoklad` |
| `Převedené makro- Makro1` | Používaná nebo provozně významná | `Makro1` |
| `Převedené makro- Makro10` | Používaná nebo provozně významná | `Makro18` |
| `Převedené makro- Makro2` | Používaná nebo provozně významná | `Makro11` |
| `Převedené makro- Makro3` | Používaná nebo provozně významná | `Makro12` |
| `Převedené makro- Makro4` | Používaná nebo provozně významná | `Makro13` |
| `Převedené makro- Makro5` | Používaná nebo provozně významná | `Makro14` |
| `Převedené makro- Makro6` | Používaná nebo provozně významná | `Makro15` |
| `Převedené makro- Makro7` | Používaná nebo provozně významná | `Makro2` |
| `Převedené makro- Makro8` | Používaná nebo provozně významná | `Makro16` |
| `Převedené makro- Makro9` | Používaná nebo provozně významná | `Makro17` |
| `Převedené makro- test` | Historická / pravděpodobně nepoužívaná | `test` |
| `Převedené makro- test1` | Historická / pravděpodobně nepoužívaná | `test1` |
| `Převedené makro- test2` | Historická / pravděpodobně nepoužívaná | `test2` |
| `Převedené makro- test3` | Historická / pravděpodobně nepoužívaná | `test3` |
| `Převedené makro- test4` | Historická / pravděpodobně nepoužívaná | `test_test` |

## Filtrování a vyhledávání

- Formulář `Hledat` vyhledává a otevírá konkrétní záznamy; přepíná kontext přes hodnotu `typ`.
- `Přehled`, `SestPřehl`, `SestavaZO`, `Úhrada`, `Doklad`, `PoštPoukázka` a `Poj_udalost` používají navázané seznamy, `Requery`, filtry a dvojklik pro přechod do detailu.
- `Přehled` filtruje podle `DatumOd`/`DatumDo`; `SestavaZO` podle `OC`, `ZO` a `Rok`; `Úhrada` podle počátku pojištění, OC a ZO.
- Accessové standardní hledání používají procedury `DoCmd.FindNext`; přesná událost je uvedena ve `forms-inventory.md`.

## Tiskové a exportní výstupy

- Náhled/tisk přes `DoCmd.OpenReport`; PDF přes `DoCmd.OutputTo`; e-mail přes `DoCmd.SendObject`; Excel přes `TransferSpreadsheet` nebo `OutputTo` dotazu.
- Kód obsahuje pevné cesty `D:\Pojistka\...`, `C:\Pojistka\...` a `C:\Federace\...`; jde o zachovávané provozní chování, nikoli návrh nové cesty.
- Úplný seznam 31 výstupů a jejich událostí je v `reports-inventory.md`.

## Vazby formulářů

- `Vstup` → `Switchboard`; `Switchboard` je hlavní rozcestník.
- `Switchboard` otevírá klienty, hledání, přehledy, úhrady, doklady, poukázky, sestavy, archivaci, přenos dat a heslo.
- `Přehled`/`SestavaZO` → `Hledat` přes dvojklik na klienta.
- `Poj_udalost` → `Poj_Udalost_mail` a `Poj_Udalost_upr`; používá sestavu `PojUdálost`.
- `Doklad` řídí více režimů tisku (`Přihláška`, `Doklad`, adresní výstupy) podle hodnoty `ID`.
- Pomocné formuláře `PřPod1`–`PřPod3`, `SeznamPod`, `SestavaZOpod`, `SestHVPpod` jsou podformuláře nadřazených obrazovek.

## Klasifikace a omezení auditu

- **Používané:** objekty dosažitelné ze `Switchboard`, formulářů a aktivního VBA.
- **Historické:** roční tabulky, názvy `Old`, konkrétní roční/osobní sestavy a staré exporty.
- **Technické pomocné:** podformuláře, dočasné tabulky, zdrojové tabulky a konfigurace switchboardu.
- **Pravděpodobně nepoužívané:** objekty `test`, `Copy_*`, `opravy` a objekty odkazující na chybějící tabulky; definitivní vyřazení vyžaduje potvrzení uživatele.
- Aktivní obsah byl při vizuální kontrole blokovaný, aby se nespustily zápisy. Události a akce byly proto auditovány staticky z VBA; názvy maker jsou úplné, ale jejich případné deklarativní kroky je před implementací nutné ještě potvrdit v licencovaném návrhovém režimu Accessu.

## Akce standardních a převedených modulů

Tato tabulka zachovává procedury a rozpoznané akce standardních modulů, včetně modulů vzniklých převodem maker.

| Modul | Procedura | Akce |
|---|---|---|
| `global` | `Doklad` | DoCmd.OpenForm "Doklad", acNormal, , , acEdit, acWindowNormal · Forms!Doklad!popOd.ControlSource = "='Počátek pojištění:'" · Forms!Doklad!Od.Value = "" · Forms!Doklad!Od.Format = "Short Date" · Forms!Doklad!ID.Value = 2 · Forms!Doklad.Caption = "Tisk dokladu hromadně" |
| `global` | `DokladJedn` | DoCmd.OpenForm "Doklad", acNormal, , , acEdit, acWindowNormal, "Tisk dokladu" · Forms!Doklad!popOd.ControlSource = "='Evidenční číslo:'" · Forms!Doklad!Od.Value = "" · Forms!Doklad!ID.Value = 1 · Forms!Doklad.Caption = "Tisk jednotlivého dokladu" |
| `global` | `ŠtítkyDoklad` | Forms!switchboard.Visible = False · MsgBox "Vlož do tiskárny poštovní štítky", vbExclamation, "Příprava k tisku štítků" · DoCmd.OpenForm "Doklad", , , , , acNormal · Forms!Doklad!ID.Value = 3 · Forms!Doklad.Caption = "Tisk poštovních štítků" · Forms!Doklad!popOd.Value = "Pojištění od" · Forms!Doklad!OC.Visible = True · Forms!Doklad!Pouk.Visible = True · Forms!Doklad!OC.Value = Forms!Doklad!OC.ItemData(0) |
| `Převedené makro- test4` | `test_test` | DoCmd.OutputTo acOutputReport, "SestavaZO", "Excel97-Excel2003Workbook(*.xls)", "c:\Pojistka\SestZO\ZO_.xls", False, "", 0, acExportQualityPrint · MsgBox Error$ |
| `Převedené makro- Makro7` | `Makro2` | DoCmd.SendObject acReport, Forms!Doklad_tisk!Příjmení & Forms!Doklad_tisk!rok, "PDFFormat(*.pdf)", Forms!Doklad_tisk!Pošta, "", "", "Doklad o pojištění", "Přílohou zasíláme Doklad o pojištění z odpovědnosti", True, "" · MsgBox Error$ |
| `Převedené makro- Makro5` | `Makro14` | DoCmd.OutputTo acOutputReport, "AdresPojPodm", "Excel97-Excel2003Workbook(*.xls)", "C:\Pojistka\sestavy_P2\sest_Adrs.xls", True, "", 0, acExportQualityPrint · MsgBox Error$ |
| `Převedené makro- test1` | `test1` | DoCmd.FindRecord "Forms!Úhrada!Hledat", acStart, False, acDown, False, acCurrent, False · MsgBox Error$ |
| `Převedené makro- test2` | `test2` | DoCmd.OutputTo acReport, "SestavaZO", "RichTextFormat(*.rtf)", "c:\federace\pojistka\seshvp\SestZO.rtf", True, "", 0 · MsgBox Error$ |
| `Převedené makro- Makro2` | `Makro11` | DoCmd.OutputTo acOutputReport, "SeznamOd", "PDFFormat(*.pdf)", "", False, "", 0, acExportQualityScreen · MsgBox Error$ |
| `Převedené makro- test3` | `test3` | Forms!Doklad.Modal = False · MsgBox Error$ |
| `Převedené makro- Makro10` | `Makro18` | DoCmd.OutputTo acOutputReport, "SestavaZO", "RichTextFormat(*.rtf)", """D:\Pojistka\sestZO\&Forms!SestavaZO!ZO&"".rtf""", True, "", 0, acExportQualityPrint · MsgBox Error$ |
| `Převedené makro- Makro8` | `Makro16` | DoCmd.OutputTo acOutputQuery, "Export_kontrol", "Excel97-Excel2003Workbook(*.xls)", "D:\Pojistka\sestZO\" & Forms!SestavaZO!ZO & ".xls", True, "", 0, acExportQualityPrint · MsgBox Error$ |
| `Převedené makro- Makro1` | `Makro1` | DoCmd.OutputTo acOutputReport, "SeznamOd", "PDFFormat(*.pdf)", "", False, "", 0, acExportQualityScreen · MsgBox Error$ |
| `Převedené makro- Makro3` | `Makro12` | DoCmd.OutputTo acOutputReport, "SeznamOd", "PDFFormat(*.pdf)", "C:\Pojistka\sestavy_P2\sest_P2.pdf", True, "", 0, acExportQualityScreen · MsgBox Error$ |
| `Převedené makro- Makro9` | `Makro17` | DoCmd.OutputTo acOutputReport, "SestavaZO", "RichTextFormat(*.rtf)", """D:\Pojistka\sestZO\&Forms!SestavaZO!ZO&"".doc""", True, "", 0, acExportQualityPrint · MsgBox Error$ |
| `Převedené makro- Makro4` | `Makro13` | DoCmd.OutputTo acOutputReport, "SeznamOd", "PDFFormat(*.pdf)", "C:\Pojistka\sestavy_P2\sest_P2.pdf", True, "", 0, acExportQualityScreen · MsgBox Error$ |
| `Převedené makro- Makro6` | `Makro15` | DoCmd.OutputTo acOutputReport, "AdresPojPodm", "Excel97-Excel2003Workbook(*.xls)", "C:\Pojistka\sestavy_P2\sest_Adrs.xls", True, "", 0, acExportQualityPrint · MsgBox Error$ |
| `Převedené makro- test` | `test` | [neaktivní] DoCmd.OutputTo acOutputQuery, "SELECT [KódOC] & IIf(Len([evČíslo])=1,'000' & [Evčíslo],IIf(Len([evčíslo])=2,'00' & [Evčíslo],IIf(Len([evčíslo])=3,'0' & [Evčíslo],IIf(Len([evčíslo])=4,[Evčíslo])))) AS Evidčíslo, nz([Titul] & ' ' & [Príjmení] & ' ' & [Jméno]) AS Klient, Seznam.Rodnéčíslo, Seznam.PojištěníOd, Seznam.PojištěníDo, Seznam.Ukoncení, Seznam.Pojistnáčástka AS RočníPojistné, Seznam.PojistNespotř, [RočPojistné]/2 AS LimitPlnení, Seznam.RočPojistné AS RočníLimitPlnění, Seznam.Kategorie, IIf([Ztráta]=-1,'Poj. na ztrátu') AS Poznámka FROM Seznam WHERE (((Seznam.PojišteníOd)=[Forms]![Přehled]![DatumOd]) AND ((Seznam.SkutÚhrada)>0)) OR (((Seznam.Ukončení)=[Forms]![Přehled]![DatumDo]));", , "Excel97-Excel2003Workbook(*.xls)", "c:\Pojistka\SestZO\SestP2" & Date & ".xls", False, "", 0, acExportQualityPrint · DoCmd.OutputTo acOutputQuery, "SELECT [KódOC] & IIf(Len([evČíslo])=1,'000' & [Evčíslo],IIf(Len([evčíslo])=2,'00' & [Evčíslo],IIf(Len([evčíslo])=3,'0' & [Evčíslo],IIf(Len([evčíslo])=4,[Evčíslo])))) AS Evidčíslo, nz([Titul] & ' ' & [Príjmení] & ' ' & [Jméno]) AS Klient, Seznam.Rodnéčíslo, Seznam.PojištěníOd, Seznam.PojištěníDo, Seznam.Ukoncení, Seznam.Pojistnáčástka AS RočníPojistné, Seznam.PojistNespotř, [RočPojistné]/2 AS LimitPlnení, Seznam.RočPojistné AS RočníLimitPlnění, Seznam.Kategorie, IIf([Ztráta]=-1,'Poj. na ztrátu') AS Poznámka FROM Seznam WHERE (((Seznam.PojišteníOd)=[Forms]![Přehled]![DatumOd]) AND ((Seznam.SkutÚhrada)>0)) OR (((Seznam.Ukončení)=[Forms]![Přehled]![DatumDo]));", , "Excel97-Excel2003Workbook(*.xls)", "c:\Pojistka\SestZO\SestP2" & Date & ".xls", False, "", 0, acExportQualityPrint · MsgBox Error$ |
