# Ruční záloha a obnova databáze

## Umístění databáze

Pracovní SQLite databáze je uložena v aplikačním datovém adresáři systému Windows. Soubor dodávaný s instalací slouží pouze jako výchozí zdroj při prvním spuštění. Uživatel databázový soubor přímo nevybírá ani nekopíruje.

## Formát `.fvcbackup`

Záloha je jeden ZIP kompatibilní kontejner s vlastní příponou `.fvcbackup`. Obsahuje přesně:

- `manifest.json` – verze formátu, aplikace a schématu, čas vytvoření, počet unikátních členů, velikost a SHA-256 databáze.
- `database.sqlite` – konzistentní snapshot celé databáze vytvořený SQLite Online Backup API.
- `checksum.sha256` – kontrolní součet souboru `database.sqlite`.

Formát je verzovaný položkou `formatVersion`. Další poskytovatelé, například budoucí cloudové úložiště, mohou být přidáni bez změny databáze.

## Vytvoření zálohy

V nabídce **Soubor → Vytvořit zálohu…** uživatel vybere cílové umístění. Aplikace vytvoří snapshot, zkontroluje `PRAGMA quick_check`, sestaví balíček, znovu ověří jeho strukturu, SHA-256, velikost, verzi schématu a počet členů. Neúplný dočasný balíček se nepublikuje.

## Obnova

Před potvrzením aplikace ověří strukturu balíčku, checksum, čitelnost SQLite a shodu verze databázového schématu. Před každou obnovou automaticky vytvoří `Emergency_Backup_YYYY-MM-DD_HH-mm.fvcbackup` v aplikačním adresáři záloh.

Obnova používá SQLite Backup API proti pracovní databázi. Pokud obnovení nebo následná kontrola integrity selže, aplikace se pokusí vrátit obsah z nouzového snapshotu a zobrazí pouze srozumitelnou chybu. Po úspěchu se aplikace automaticky restartuje.

## Omezení sprintu

Zálohy nejsou šifrované. Neobsahují cloudovou synchronizaci, plánování, inkrementální ukládání ani export jednotlivých tabulek. Soubor zálohy proto musí být uložen na zabezpečeném místě s odpovídajícími přístupovými právy.
