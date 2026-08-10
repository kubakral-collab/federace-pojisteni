# Sprint FED-PARITY-REPORT-HISTORY-01 — výsledek

Dokončeno 2026-08-10. Priorita P1, milestone M1 — Stabilizace Core.

## Výsledek

Obrazovka **Sestavy a exporty** nově obsahuje samostatnou tabulku **Historie vystavených sestav** v režimu pouze pro čtení.

- `Sestavy`: 3 historické záznamy.
- `SestavyHVP`: 138 historických záznamů.
- Celkem zpřístupněno: 141 záznamů.

Zobrazené údaje: zdroj, původní pořadové číslo, datum vystavení, počet pojištěnců, celková částka, začátek a konec pojistného období a původní poznámka. Historie je vizuálně i datově oddělena od nové tabulky `AuditSestav`, která eviduje současné exporty.

## Databáze a bezpečnost

Nebyla provedena migrace ani zápis do původních tabulek. Backend je načítá přímo a pouze pro čtení. Kvůli historicky poškozenému kódování názvů sloupců používá ověřené pořadí sedmi původních polí, nikoli křehké názvy sloupců.

Kontrolní hash `dd.sqlite` po testu zůstal `F8F90FDEEDEDFED1159D4AA2F24B52AA5B96B94A9726201DD57CDD2F63B93D2D`.

## Testy

- Syntetický test obou Access tabulek a mapování polí: PASS.
- Reálný read-only test `dd.sqlite`: 3 + 138 = 141 záznamů, PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib`: 66 testů PASS.
- `npm.cmd run build`: PASS.

Nebyla změněna logika vytváření ani exportu nových sestav.
