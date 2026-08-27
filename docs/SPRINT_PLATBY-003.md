# Sprint PLATBY-003 — Oprava částek při hromadném připisování plateb

Datum dokončení: 2026-08-27

Priorita: P1

Milestone: M1 — Stabilizace Core

## Výsledek

Obrazovka organizační platby nyní odděluje pojistnou částku, skutečné pojistné a úhradu. Chyba vznikala tím, že backend používal Access sloupec `RočPojistné` jako očekávanou platbu. V migrovaném modelu však tento sloupec představuje pojistnou částku/limit; skutečné uložené pojistné je ve sloupci `PojistnáČástka`.

Nové mapování:

- **Pojistná částka:** `Seznam.RočPojistné` z řádku člena vybraného roku;
- **Pojistné:** `Seznam.PojistnáČástka` ze stejného řádku;
- **Uhrazeno:** `Seznam.SkutÚhrada` ze stejného řádku;
- **Připsat:** `max(pojistné - uhrazeno, 0)`.

Tabulka má pořadí `Vybrat`, `Ev. číslo`, `Člen`, `Pojistná částka`, `Pojistné`, `Uhrazeno`, `Připsat`. Pole Připsat zůstává editovatelné. Automatická hodnota se znovu nastaví pouze při výběru organizace nebo explicitní akci „Vybrat jen nezaplacené“. Změna roku vyčistí dříve načtenou organizaci a členy, takže nelze omylem kombinovat údaje různých období.

Backend při uložení znovu načte členy vybrané organizace a roku, ověří jejich existenci a nepovolí zápornou částku ani částku vyšší než zbývající pojistné. Operace nadále vytváří záznam platby a nemění pojistnou částku.

## Změněné soubory

- `src-tauri/src/organization_payments.rs` — správné mapování uložených částek, výpočet doplatku, backendová validace a regresní testy;
- `src/App.tsx` — oddělené sloupce, správné předvyplnění a výběr nezaplacených;
- `docs/ROADMAP.md`, `docs/BACKLOG.md`, `docs/DECISIONS.md` — schválení a stav sprintu;
- tento závěrečný záznam.

## Databáze

Bez změny schématu a bez migrace. Existující data nebyla přepsána.

## Testy

- cílené testy organizačních plateb: 5/5 PASS;
- všechny Rust testy: 68/68 PASS;
- TypeScript + Vite produkční build: PASS;
- pokryto: plná úhrada, nulová úhrada, částečná úhrada, přeplatek a rozdílné pojistné při stejné pojistné částce.

