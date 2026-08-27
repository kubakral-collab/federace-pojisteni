# Sprint DOKLADY-001 — Oprava podkladů a doklad z existující platby

Datum dokončení: 2026-08-27

Priorita: P1

Milestone: M1 — Stabilizace Core

## Příčina

Frontend po volbě člena volá endpoint `get_payment_document_basis` s `rowId` vybraného pojistného záznamu. Endpoint doplní aktivní rok a načte podklady v `receipts::load_basis`.

U případu ev. č. 394 byl `Seznam.Identifikátor` uložen jako SQLite `INTEGER` (`16831`), zatímco Rust očekával `String`. SQL výraz bez explicitního převodu proto skončil chybou typu a UI zobrazilo obecnou hlášku „Podklady dokladu se nepodařilo načíst.“. Ostatní data případu byla správná: pojistná částka 600 000 Kč, pojistné 1 502 Kč a úhrada 1 502 Kč.

## Oprava

- SQL převádí `Identifikátor` přes `CAST(... AS TEXT)`, takže podporuje číselné i textové historické hodnoty.
- Částka dokladu se bere ze skutečné úhrady: součet `PlatbyClenu.Castka`, případně původní souhrn `Seznam.SkutÚhrada`, pokud detailní platební kniha ještě neexistuje.
- Při vytvoření dokladu se původní souhrnná úhrada materializuje existující idempotentní kompatibilní funkcí do jednoho technického platebního záznamu. Nevzniká druhá finanční hodnota, `SkutÚhrada` se nemění a doklad dostane platné `IdPlatby`.
- U nové/detailní platby je `DatumUhrady` její skutečné `DatumPrijeti`. U původní souhrnné evidence, která datum úhrady neobsahuje, zůstává již zavedené kompatibilní datum 1. ledna daného pojistného roku.
- `DatumVystaveni` vzniká samostatně v okamžiku vytvoření dokladu.
- Ochrana proti duplicitě respektuje existující doménu: jeden doklad člena za pojistný rok (`UNIQUE IdentifikatorClena, PojistnyRok`). Opakovaná akce vrátí existující doklad.
- Vložení dokladu a auditního záznamu zůstává v jedné databázové transakci.
- Frontend zapisuje konkrétní technickou chybu do vývojové konzole bez osobních údajů; produkční UI zachovává bezpečnou obecnou hlášku.
- Tlačítko je aktivní pouze při načtených podkladech a dostatečné úhradě.

Vyhledávání členů nebylo změněno.

## Změněné soubory

- `src-tauri/src/receipts.rs` — oprava typu, vazba na existující platbu, validace a regresní testy;
- `src/App.tsx` — vývojový log a správná dostupnost tlačítka;
- `docs/ROADMAP.md`, `docs/BACKLOG.md`, `docs/DECISIONS.md` — řízení sprintu;
- tento závěrečný záznam.

## Databáze

Bez změny schématu a bez hromadné migrace. Při vystavení dokladu může být idempotentně vytvořen technický detail již existující původní souhrnné úhrady, pokud dosud chybí; částka člena se tím nemění.

## Testy

- cílené testy Dokladů: 19/19 PASS;
- všechny Rust testy: 70/70 PASS;
- TypeScript + Vite produkční build: PASS;
- ev. č. 394: podklady 1 502 Kč se načtou, doklad má 1 502 Kč, `IdPlatby > 0`, opakování ponechá jeden doklad a jednu platbu a `SkutÚhrada` zůstane 1 502 Kč.

