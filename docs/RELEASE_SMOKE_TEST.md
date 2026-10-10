# Ruční UI smoke test — FED-TEST-001

Spusťte pouze smoke build příkazem uvedeným na konci dokumentu. Na přihlašovací obrazovce musí banner **SMOKE DB** ukazovat absolutní cestu `C:\Users\kubak\Documents\Federace Pojištění\smoke-data\dd-smoke.sqlite`. Pokud ukazuje jinou cestu, aplikaci ihned zavřete.

1. [ ] **Start a přihlášení** — aplikace se spustí bez migrační chyby, přihlášení funguje a O programu ukazuje aplikaci `0.26.0`, DB schema `2`.
2. [ ] **Hlavní panel a rok** — je vidět přepínač Aktivní pojistný rok a tlačítko Založit nový pojistný rok; samotné přepnutí nic nevytváří.
3. [ ] **Ruční založení roku** — potvrzovací dialog nabídne následující rok; Zrušit nic nezmění; potvrzení vytvoří rok a druhý pokus je odmítnut.
4. [ ] **Přepínání 2026 → 2027 → 2026** — dashboard, seznam, platby, doklady a přehledy vždy zobrazují zvolený rok.
5. [ ] **Převod a platby** — běžný člen a člen s datem bez checkboxu jsou převedeni; člen s datem i checkboxem není; nový rok má 0 Kč, starý zachová úhrady.
6. [ ] **Seznam a samostatný detail** — kliknutí otevře nativní okno, nikoli detail pod seznamem; po zavření zůstávají filtry, hledání a stránka.
7. [ ] **Uložení detailu** — lze uložit Datum ukončení pojištění i checkbox Pojištění ukončeno a po novém otevření zůstávají hodnoty zachované.
8. [ ] **Neuložené změny – Zůstat** — Zpět i zavření okna zobrazí dialog a Zůstat a pokračovat zachová rozepsané hodnoty.
9. [ ] **Neuložené změny – Uložit** — Uložit a odejít zavře okno pouze po úspěšném uložení; při validační chybě zůstane otevřené.
10. [ ] **Neuložené změny – Zahodit** — Odejít bez uložení změny zahodí; bez změn se dialog nezobrazuje.
11. [ ] **Přihláška a událost** — ochrana rozepsaných změn funguje ve Vyplnit přihlášku a v pojistné události.
12. [ ] **Přehled pro pojišťovnu** — načte se bez SQL chyby, datum filtruje podle PojištěníOd a výsledky odpovídají aktivnímu roku.
13. [ ] **Historická data** — položka není v levém menu; Archiv je dostupný.
14. [ ] **Archivní událost** — změna povoleného údaje starší události se uloží a událost zůstane ve svém původním roce.
15. [ ] **Názvy PDF** — individuální doklad obsahuje rok a jméno; hromadný rok a organizaci; diakritika a zakázané Windows znaky vytvoření neblokují.
16. [ ] **Regrese a tisk** — Nový pojištěnec, elektronická přihláška, EvČísla, platby, doklady, hromadná potvrzení, události, Archiv, font 100/110/125/150 % a Aktualizace fungují; Vytisknout přihlášku otevře systémový dialog s tiskárnou, orientací, stránkami a tlačítky Tisk/Zrušit, nikoli Uložit jako. Po Zrušit zavřete samostatné okno Tisk přihlášky a ověřte bezpečný návrat bez změny dat.
17. [ ] **Restart** — po restartu zůstává stejná smoke DB, schema 2 a nevznikne další pojistný rok ani opakovaná migrace.

## Spuštění

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start-release-smoke.ps1 -SourceDatabase ".\smoke-data\FED-TEST-001\source-readonly.sqlite" -ResetSmokeCopy
```

Po dokončení zapište datum, výsledek každého bodu a případnou chybu. Release zůstává blokovaný, dokud nejsou všechny checkboxy potvrzené.
