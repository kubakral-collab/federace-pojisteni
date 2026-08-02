# Inventář sestav Access

> Historická forenzní reference; není aktuální produktovou roadmapou.

Inventář obsahuje všech **31 sestav** z kontejneru DAO `Reports`. U každé sestavy jsou uvedené vytěžené události VBA a vazby na formuláře nebo tisk/export.

## `AdresPojPodm`

- **Zařazení:** Technická pomocná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | Forms!Doklad.Visible = True |
| `Report_NoData` | MsgBox "Nejsou k dispozici žádná data, sestava bude stornována, tisk štítků je zastaven", vbInformation, "Storno akce" |
| `Report_Open` | MsgBox "Vlož do formulář s pojistnými podmínkami (adresní rámeček vpravo nahoře)", vbExclamation, "Příprava k tisku adres na formuláře" · Forms!Doklad.Visible = False |

## `Doklad`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Doklady a poukázky – tisk
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | [neaktivní] Forms!Doklad.Visible = True · [neaktivní] Forms!Doklad.PopUp = True |
| `Report_NoData` | MsgBox "Nejsou k dispozici žádná data, sestava bude stornována, tisk je zastaven", vbInformation, "Storno akce" |
| `Report_Open` | [neaktivní] MsgBox "Vlož do tiskárny formuláře dokladu o pojištění", vbExclamation, "Příprava k tisku dokladu" · [neaktivní] Forms!Doklad.Visible = False · [neaktivní] Forms!Doklad.PopUp = False |

## `Doklad_`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Doklady a poukázky – tisk
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | [neaktivní] Forms!Doklad.Visible = True · [neaktivní] Forms!Doklad.PopUp = True |
| `Report_NoData` | MsgBox "Nejsou k dispozici žádná data, sestava bude stornována, tisk je zastaven", vbInformation, "Storno akce" |
| `Report_Open` | [neaktivní] MsgBox "Vlož do tiskárny formuláře dokladu o pojištění", vbExclamation, "Příprava k tisku dokladu" · [neaktivní] Forms!Doklad.Visible = False · [neaktivní] Forms!Doklad.PopUp = False |

## `Form_DataKlientů`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | Forms!Doklad.Visible = True |
| `Report_NoData` | MsgBox "Nejsou k dispozici žádná data, sestava bude stornována, tisk štítků je zastaven", vbInformation, "Storno akce" |
| `Report_Open` | Forms!Doklad.Visible = False |

## `NálepkyAdr`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | Forms!Doklad.Visible = True · [neaktivní] Forms!Doklad.PopUp = True |
| `Report_NoData` | MsgBox "Nejsou k dispozici žádná data, sestava bude stornována, tisk štítků je zastaven", vbInformation, "Storno akce" |
| `Report_Open` | MsgBox "Vlož do tiskárny samolepící štítky", vbExclamation, "Příprava k tisku štítků" · Forms!Doklad.Visible = False · [neaktivní] Forms!Doklad.PopUp = False |

## `NálepkyForm`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | Forms!Doklad.Modal = True · Forms!Doklad!Tisk.Enabled = True · Forms!Doklad!Storno.Enabled = True |
| `Report_NoData` | MsgBox "Nejsou k dispozici žádná data, sestava bude stornována, tisk štítků je zastaven", vbInformation, "Storno akce" |
| `Report_Open` | Forms!Doklad.Modal = False · Forms!Doklad!Od.SetFocus · Forms!Doklad!Tisk.Enabled = False · Forms!Doklad!Storno.Enabled = False |

## `NálepkyZO`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_NoData` | MsgBox "Nejsou k dispozici žádná data, sestava bude stornována, tisk štítků je zastaven", vbInformation, "Storno akce" |

## `ObálkaDL`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | [neaktivní] Forms!Doklad.Visible = True |
| `Report_NoData` | MsgBox "Nejsou k dispozici žádná data, sestava bude stornována, tisk štítků je zastaven", vbInformation, "Storno akce" |
| `Report_Open` | MsgBox "Vlož do tiskárny obálky DL, lícem dolů, chlopní vlevo", vbExclamation, "Příprava k tisku adres na obálky" · [neaktivní] Forms!Doklad.Visible = False |

## `PojUdálost`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Pojistné události – tisk
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Open` | If Forms!Poj_Udalost!ID = 1 Then · Me.RecordSource = "SELECT Seznam.Identifikátor, [Titul] & ' ' & [Příjmení] & ' ' & [Jméno] AS Pojištěnec, Seznam.RodnéČíslo, [KódOC] & IIf(Len([EvČíslo])=4,[EvČíslo],IIf(Len([EvČíslo])=3,'0' & [EvČíslo],IIf(Len([EvČíslo])=2,'00' & [EvČíslo],IIf(Len([EvČíslo])=1,'000' & [EvČíslo])))) AS Evid, [Město] & '; ' & [Adresa] & '; ' & [PSČ] AS Adrs, Poj_udalost_zdroj.Telefon, Seznam.[e-mail], Poj_udalost_zdroj.Povolání, Poj_udalost_zdroj.Zaměstnavatel, 'Kategorie ' & [Kategorie] & IIf([Ztráta]=0,' ','+ ztráta') & '; roční pojistné ' & [RočPojistné] & '; skutečná úhrada ' & [SkutÚhrada] & ' Kč' AS Typ_pojištění, [PojištěníOd] & ' - ' & [PojištěníDo] AS Poj_Období, Poj_udalost_zdroj.Vznik_PU, Poj_udalost_zdroj.Oznámení_PU, Poj_udalost_zdroj.Popis_Události, Poj_udalost_zdroj.Poznámka1, Poj_udalost_zdroj.Poznámka2, Poj_udalost_Zdroj.Pojistné_plnění, Poj_udalost_zdroj.Ukončeno," _ · If Forms!Poj_Udalost!ID = 2 Then · Me.RecordSource = "SELECT Poj_udalost.ID, [Titul] & ' ' & [Příjmení] & ' ' & [Jméno] AS Pojištěnec, Seznam.RodnéČíslo, [KódOC] & IIf(Len([EvČíslo])=4,[EvČíslo],IIf(Len([EvČíslo])=3,'0' & [EvČíslo],IIf(Len([EvČíslo])=2,'00' & [EvČíslo],IIf(Len([EvČíslo])=1,'000' & [EvČíslo])))) AS Evid, [Město] & '; ' & [Adresa] & '; ' & [PSČ] AS Adrs, Seznam.[e-mail], Poj_udalost.Telefon, 'Kategorie ' & [Kategorie] & IIf([Ztráta]=0,' ','+ ztráta') & '; roční pojistné ' & [RočPojistné] & '; skutečná úhrada ' & [SkutÚhrada] & ' Kč' AS Typ_pojištění, [PojištěníOd] & ' - ' & [PojištěníDo] AS Poj_Období, Poj_udalost.Řeší_pojišťovna, Poj_udalost.Zaměstnavatel, Poj_udalost.Povolání, Poj_udalost.Vznik_PU, Poj_udalost.Oznámení_PU, Poj_udalost.Popis_Události, Poj_udalost.Poznámka1, Poj_udalost.Poznámka2, Poj_udalost.Zjištěná_škoda, Poj_udalost.Pojistné_plnění, Poj_udalost.Ukončeno, Poj_udalost.Řeší_pojišťovna, Poj_udalost.Pojistné_plnění, Poj_udalost.Ukončeno" _ |

## `PojUdálost_sest`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Pojistné události – tisk
- **Kódový modul:** ne

- Bez vlastní VBA události; sestava může být plně řízena zdrojem dat a volajícím formulářem.

## `PoštPoukázka`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Doklady a poukázky – tisk
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | Forms!PoštPoukázka.Visible = True |
| `Report_Open` | Forms!PoštPoukázka.Visible = False |

## `PoštPoukázka_`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Doklady a poukázky – tisk
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | Forms!PoštPoukázka.Visible = True |
| `Report_Open` | Forms!PoštPoukázka.Visible = False |

## `PoštPoukázka_S`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Doklady a poukázky – tisk
- **Kódový modul:** ano

- Bez vlastní VBA události; sestava může být plně řízena zdrojem dat a volajícím formulářem.

## `Prázdná čísla`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | Forms!switchboard.Visible = True |
| `Report_Open` | Forms!switchboard.Visible = False |

## `Procházka Jiří 2022`

- **Zařazení:** Historická / pravděpodobně nepoužívaná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | [neaktivní] Forms!Doklad.Visible = True · [neaktivní] Forms!Doklad.PopUp = True |
| `Report_NoData` | MsgBox "Nejsou k dispozici žádná data, sestava bude stornována, tisk je zastaven", vbInformation, "Storno akce" |
| `Report_Open` | [neaktivní] MsgBox "Vlož do tiskárny formuláře dokladu o pojištění", vbExclamation, "Příprava k tisku dokladu" · [neaktivní] Forms!Doklad.Visible = False · [neaktivní] Forms!Doklad.PopUp = False |

## `Přihláška`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Doklady a poukázky – tisk
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | Forms!Doklad.Visible = True |

## `Přihláška_`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Doklady a poukázky – tisk
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | Forms!Doklad.Visible = True |

## `Přihláška_Old`

- **Zařazení:** Historická / pravděpodobně nepoužívaná
- **Nové místo:** Doklady a poukázky – tisk
- **Kódový modul:** ne

- Bez vlastní VBA události; sestava může být plně řízena zdrojem dat a volajícím formulářem.

## `Sest_Odes`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | Forms!SestavaZO.Modal = True |
| `Report_NoData` | MsgBox "Žádná data nejsou k dispozici, sestava bude stornována" |
| `Report_Open` | [neaktivní] If MsgBox("Chceš zobrazit diferencované počty klientů?", vbYesNo, "Počty v sestavě") = vbYes Then · [neaktivní] Me!Počty.Visible = True · [neaktivní] Me!Počty.Visible = False |

## `SestavaOC`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | Forms!SestavaZO.Modal = True |
| `Report_NoData` | MsgBox "Žádná data nejsou k dispozici, sestava bude stornována" |

## `SestavaZO`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | Forms!SestavaZO.Modal = True |
| `Report_NoData` | MsgBox "Žádná data nejsou k dispozici, sestava bude stornována" |
| `Report_Open` | [neaktivní] If MsgBox("Chceš zobrazit diferencované počty klientů?", vbYesNo, "Počty v sestavě") = vbYes Then · [neaktivní] Me!Počty.Visible = True · [neaktivní] Me!Počty.Visible = False |

## `SestavaZO_Počty`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ne

- Bez vlastní VBA události; sestava může být plně řízena zdrojem dat a volajícím formulářem.

## `SestavaZO_Potvrz`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | Forms!SestavaZO.Modal = True |
| `Report_NoData` | MsgBox "Žádná data nejsou k dispozici, sestava bude stornována" |

## `SestKontr`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | Forms!switchboard.Visible = True |
| `Report_NoData` | MsgBox "Žádná data nejsou k dispozici, sestava bude stornována" |

## `SestPoj`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_NoData` | MsgBox "Žádná data nejsou k dispozici, sestava bude stornována" |

## `SestPojEmail`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_NoData` | MsgBox "Žádná data nejsou k dispozici, sestava bude stornována" |

## `SestPřehl`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_NoData` | MsgBox "Žádná data nejsou k dispozici, sestava bude stornována" |

## `Seznam_podsest`

- **Zařazení:** Technická pomocná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ne

- Bez vlastní VBA události; sestava může být plně řízena zdrojem dat a volajícím formulářem.

## `Seznam_podsest_`

- **Zařazení:** Technická pomocná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ne

- Bez vlastní VBA události; sestava může být plně řízena zdrojem dat a volajícím formulářem.

## `SeznamKonec`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_NoData` | MsgBox "Žádná data nejsou k dispozici, sestava bude stornována" |

## `SeznamOd`

- **Zařazení:** Používaná nebo provozně významná
- **Nové místo:** Sestavy a tiskové výstupy
- **Kódový modul:** ano

| Událost | Akce |
|---|---|
| `Report_Close` | Forms!Přehled.Visible = True |
| `Report_NoData` | MsgBox "Žádná data nejsou k dispozici, sestava bude stornována" |
