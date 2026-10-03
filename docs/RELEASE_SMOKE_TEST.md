# Ruční UI smoke test — Federace 0.25.0

Použijte pouze smoke build spuštěný přes `scripts/start-release-smoke.ps1`. Na přihlašovací obrazovce nejprve ověřte, že banner **SMOKE DB** ukazuje absolutní cestu končící `smoke-data\dd-smoke.sqlite`. Pokud ne, aplikaci zavřete a test nepokračujte.

- [ ] **Aplikace se spustí** — zobrazí se přihlašovací obrazovka bez chyby migrace a s bannerem SMOKE DB.
- [ ] **Přihlášení funguje** — platné přihlašovací údaje otevřou aplikaci; chybné údaje jsou odmítnuty.
- [ ] **Dashboard se načte** — souhrnné karty a aktuální data se zobrazí bez chyby.
- [ ] **Seznam členů se načte** — zobrazí se existující členové a stránkování reaguje.
- [ ] **Vyhledávání člena funguje** — známé jméno nebo evidenční číslo vrátí odpovídající záznam.
- [ ] **Detail člena se otevře** — osobní, organizační a pojistné údaje vybraného člena jsou dostupné.
- [ ] **Platby se otevřou** — agenda nebo záložka plateb se načte bez chyby.
- [ ] **Pojistné události se otevřou** — seznam událostí se načte a lze otevřít existující položku.
- [ ] **Dokumenty se otevřou** — doklady/dokumenty člena se načtou bez chyby.
- [ ] **Nový pojištěnec se otevře** — formulář nového pojištěnce zobrazí všechna povinná pole; testovací záznam není nutné uložit.
- [ ] **Nastavení se otevře** — obrazovka nastavení a její aktivní moduly se načtou.
- [ ] **Aktualizace fungují** — ruční kontrola aktualizací doběhne s výsledkem nebo srozumitelnou síťovou zprávou.
- [ ] **O programu zobrazuje 0.25.0** — karta O programu uvádí verzi aplikace `0.25.0`.
- [ ] **DB schema zobrazuje 1** — karta O programu uvádí verzi databázového schématu `1`.
- [ ] **Nejsou viditelné chyby migrace** — nikde se nezobrazuje diagnostika neúspěšné nebo částečné migrace.
- [ ] **Restart aplikace proběhne bez další migrace** — po zavření a opětovném spuštění se aplikace přihlásí nad stejnou smoke DB a nevytvoří další migrační zálohu.

Po testu zapište výsledek, datum a případné chyby. Produkční vydání zůstává blokované, dokud nejsou všechny body potvrzené.
