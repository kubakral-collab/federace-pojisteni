# Aktuální stav funkční a datové parity Access → Federace

Aktualizováno: 2026-08-31

Ověřená verze: `v0.24.0`

Stav: funkční oblasti jsou implementované; úplná historická datová parita není dokončena.

## Jak tento dokument číst

Toto je aktuální zdroj pravdy pro stav migrace z Accessu. Rozlišuje tři věci, které starší dokumentace směšovala:

1. **Funkční pokrytí** — zda v nové aplikaci existuje provozní ekvivalent funkce Accessu.
2. **Datová dostupnost** — zda jsou v nové aplikaci uživatelsky dostupné všechny historické záznamy.
3. **Provozní akceptace** — zda byl scénář vyzkoušen uživatelem na provozní kopii dat a případně na konkrétní tiskárně.

`FUNCTION_MAP.md` eviduje funkční pokrytí: 30 položek **Hotovo**, 16 **Nahrazeno**, 0 **Chybí**. Neznamená to, že jsou zpřístupněna všechna historická data. Plán a pořadí dalšího vývoje zůstávají v [ROADMAP.md](ROADMAP.md), aktivní práce v [BACKLOG.md](BACKLOG.md) a schválená rozhodnutí v [DECISIONS.md](DECISIONS.md).

## Ověření aktuálního kódu

Ověřeno 2026-08-31 nad pracovním stromem verze `v0.24.0`:

- `npm.cmd run build` — prošlo;
- `cargo test --all-targets` — **70 testů prošlo, 0 selhalo**;
- verze v `package.json`, `src-tauri/Cargo.toml` a `src-tauri/tauri.conf.json` je `0.24.0`.

## Jednoznačný souhrn

| Oblast | Funkce | Historická data | Akceptace / rozhodnutí | Celkový stav |
| --- | --- | --- | --- | --- |
| Přihlášení, role a navigace | Hotovo | Neuplatní se | Produkčně vydáno | Hotovo |
| Pojištěnci: seznam, detail, založení, editace, hledání, filtry, historie a storno | Hotovo | `Seznam` dostupný pro roky obsažené ve sjednocené tabulce | Základní workflow vydáno | Hotovo |
| Tarify a výpočet pojistného | Hotovo, bezpečně nahrazuje pevné Access sazby | Historické částky se nepřepočítávají | Regresní testy prošly | Hotovo |
| Individuální a organizační platby | Hotovo | Původní součet `SkutÚhrada` zachován | Implementováno; organizační workflow čeká na uživatelský smoke test | Implementováno, čeká smoke test |
| Příkazy, nová kniha faktur a platební dávky | Hotovo | Původních 4 652 řádků `Faktura` není v nové knize zobrazeno | Historie odložena D-021 | Funkce hotová, historická data chybí |
| Doklady o zaplacení a pojistná potvrzení | Hotovo | Aktivní data dostupná | Opravy v `v0.24.0` akceptovány | Hotovo |
| Přihláška | Hotovo | Neuplatní se | Regresní PDF test prošel | Hotovo |
| Poukázka, obálka a štítek | Základní PDF existuje | Neuplatní se | Přesná tisková shoda vyřazena z cíle rozhodnutím D-017 | Schváleně nahrazeno |
| Nové pojistné události | Hotovo | Neuplatní se | Regresní testy prošly | Hotovo |
| Historické pojistné události | Hotovo | Všech 112 řádků dostupných; 98 přesně napojeno, 14 záměrně read-only | Migrační sprint akceptován | Hotovo s označenými nejasnostmi |
| Sestavy OC/ZO/HVP a provozní přehledy | Hotovo | Aktivní dataset dostupný | Dokumentový sprint akceptován; pixelová identita není cíl | Hotovo |
| Historie vystavených sestav | Hotovo | Všech 141 řádků dostupných read-only | Sprint akceptován | Hotovo |
| Roční převod | Hotovo, transakční a se zálohou | Zachovává historii | Side-by-side workflow ověřeno a akceptováno | Hotovo |
| CSV import členů | Hotovo, bezpečná náhrada pomocných Access tabulek | Externí zdroje nebyly součástí migrace | Potřeba živých vazeb čeká na rozhodnutí | Funkce hotová, externí integrace nerozhodnutá |
| Zálohy a obnova | Hotovo | Neuplatní se | Automatické testy prošly; provozní obnova čeká na smoke test | Implementováno, čeká smoke test |
| Archiv 2011–2026 | Hotovo nad tabulkou `Seznam` | Dostupná datovaná období | Testy stránkování a hledání prošly | Hotovo |
| Archiv 2002–2010 | Čtecí UI pro tyto tabulky chybí | **4 386 řádků zachováno, ale uživatelsky nedostupných** | Odloženo D-020; bez nového schválení neimplementovat | Chybějící historická dostupnost |
| Pomocné tabulky `Seznam_`, `Odklad`, `Břeclav` | Obecný import existuje | 4 220 + 76 + 23 řádků zachováno, význam nerozhodnut | Čeká na report duplicit a rozhodnutí Product Ownera | K rozhodnutí |
| Externí `Členská základna` a Excel `Email` | CSV náhrada existuje | Externí zdroje nebyly dodány | Čeká na potvrzení provozní potřeby | K rozhodnutí |
| Audit citlivých operací | Částečně implementován napříč agendami | Neuplatní se | Sjednocení zůstává v bezpečnostním backlogu | Částečně hotovo |
| Šifrování databáze SQLCipher | Chybí | Neuplatní se | Schválený budoucí bezpečnostní scope | Plánováno |
| Bankovní import a automatické párování | Chybí | Neuplatní se | Schválený budoucí scope M2 | Plánováno |
| Automatické každoroční příkazy a odesílání | Chybí | Neuplatní se | Schválený budoucí scope M2 | Plánováno |
| UX pořadí menu a velikost písma | Funkčně neblokuje provoz | Neuplatní se | `FEDERACE-MENU-01` schválen a čeká na aktivaci; `FED-UX-FONT-01` čeká na schválení | Plánováno / k rozhodnutí |

## Co je opravdu dokončeno

- bezpečné přihlášení, role, navigace a diagnostika;
- evidence pojištěnců včetně historie, filtrů, sazeb a řízeného storna;
- individuální a organizační platby včetně rozúčtování;
- nové finanční dokumenty, platební dávky a příkazy;
- doklady, potvrzení, přihláška a základní poštovní výstupy;
- nové i všech 112 původních pojistných událostí;
- aktivní sestavy OC/ZO/HVP a historie 141 vystavených sestav;
- bezpečný roční převod;
- validovaný CSV import;
- zálohování a obnova;
- opravy plateb a dokladů vydané v `v0.24.0`.

## Co opravdu zbývá

### Odložená historická data — vyžadují nové schválení

1. **PARITY-002:** zpřístupnit 4 386 archivních záznamů z let 2002–2010 (D-020).
2. **PARITY-003:** zpřístupnit 4 652 původních faktur a poukázek odděleně od nové knihy (D-021).

### Rozhodnutí Product Ownera

1. Autoritativní datový snímek byl potvrzen rozhodnutím D-032: produkční Access s hashem `DF176160…E0DEDC`. Rozdílový audit našel 14 370 shodných, 36 změněných, 2 pouze v Accessu a 10 pouze v SQLite; synchronizace nebyla schválena ani provedena.
2. Určit význam `Seznam_`, `Odklad` a `Břeclav`; data neslučovat automaticky.
3. Rozhodnout, zda jsou stále potřebné externí vazby `Členská základna` a Excel `Email`.
4. Schválit pořadí nebo aktivaci návrhů sprintů vypsaných v [ROADMAP.md](ROADMAP.md); UX-002 samostatně čeká na produktové schválení.

### Uživatelské smoke testy

1. organizační platba a její ruční rozúčtování;
2. historická událost používající pojistný záznam podle data vzniku;
3. telefon člena a poměrné pojistné zaokrouhlené nahoru;
4. obnova zálohy na uživatelské kopii;
5. tisk aktivních PDF na provozní tiskárně, pokud je fyzický výstup součástí běžného procesu.

### Budoucí funkce, nikoli dluh parity Accessu

- automatické každoroční příkazy a odesílání;
- bankovní import a párování;
- SQLCipher;
- úplně sjednocený audit citlivých operací;
- UX pořadí menu a nastavení velikosti písma.

## Ověřené datové skutečnosti

- Auditovaný produkční Access měl 14 408 řádků `Seznam`; pracovní SQLite 14 416.
- Rozdílový audit: 14 370 přesně shodných, 36 změněných, 2 pouze v Accessu a 10 pouze v SQLite.
- Roční tabulky 2002–2010 obsahují 4 386 řádků.
- Původní `Faktura` obsahuje 4 652 řádků.
- Původních pojistných událostí je 112; všechny jsou zpřístupněny, 14 bez bezpečně prokazatelné přesné vazby zůstává read-only.
- `Sestavy` a `SestavyHVP` obsahují dohromady 141 historických záznamů; všechny jsou zpřístupněny read-only.
- `Seznam_` obsahuje 4 220, `Odklad` 76 a `Břeclav` 23 řádků.
- Původní externí tabulky nebyly součástí lokální migrace.

## Závěr

Aplikace je funkční náhradou hlavních Access workflow a aktuální vydání prochází automatickými kontrolami. Projekt ale nelze označit za úplně datově uzavřený, dokud Product Owner nerozhodne o odloženém archivu, původních fakturách, pomocných tabulkách a autoritativním datovém snímku. Tyto položky nesmějí být skryty tvrzením `0 Chybí` z funkční mapy.
