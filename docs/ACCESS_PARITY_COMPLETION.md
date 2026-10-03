# Dokončení funkční parity Access → Tauri

> **Historický implementační záznam, nikoli aktuální potvrzení úplné parity.** Aktuální stav k verzi `v0.24.0`, včetně odložených historických dat a čekajících akceptací, je v [ACCESS_PARITY_AUDIT_AND_PLAN.md](ACCESS_PARITY_AUDIT_AND_PLAN.md). Počty testů níže platí pouze k datu tohoto historického záznamu.

Datum ověření: 2026-08-05

Tento dokument je auditní stopou implementace chybějících oblastí z `FUNCTION_MAP.md`. Databáze Access nebyla měněna. Nové operace v Tauri používají pracovní SQLite databázi, role, validace, transakce, zálohy a auditní tabulky podle dopadu operace.

## Implementované oblasti

| Oblast | Výsledek v Tauri | Ověření |
| --- | --- | --- |
| Platební filtr | Uhrazeno porovnává skutečnou úhradu s předepsaným pojistným; Neuhrazeno zahrnuje nulovou a částečnou úhradu. | `payment_status_filter_compares_paid_and_prescribed_amounts` |
| Roční převod | Automatická záloha, transakční převod, nové období, nulová úhrada, nové sazby, vynechání ukončených a idempotence. | `yearly_roll_forward_is_transactional_backed_up_and_idempotent` |
| Hromadné pojistné doklady | Filtr data úhrady, OC a ZO; pouze aktivní a plně uhrazené záznamy; opakované spuštění nevytváří duplicity; dávkové PDF a audit. | testy dávkových potvrzení v `lib.rs` |
| Sestavy pojišťovny, OC, ZO a HVP | Společný parametrický náhled, PDF, CSV UTF-8 a audit vydání. | `every_operational_report_previews_and_exports_csv` |
| Počátky, ukončení a kontakty | Filtry roku/období/organizace, náhled, PDF a CSV. | stejný integrační test sestav |
| Duplicity a kvalita dat | Parametrické kontrolní sestavy pro duplicity, povinná pole, pojistné a platební nesoulady. | stejný integrační test sestav |
| Osobní historická sestava | Obecná parametrická sestava podle jména, evidenčního nebo rodného čísla a roku. | stejný integrační test sestav |
| Faktury | Číselná řada, bankovní údaje, splatnost, stav, vazba na dávku a audit. | `invoice_batch_is_atomic_and_not_exported_twice` |
| Platební dávka | Jednorázový CSV export připravených faktur, kontrolní součet, součet částek, vazba a ochrana před druhým exportem. | `invoice_batch_is_atomic_and_not_exported_twice` |
| Přihláška, obálka a štítek | PDF z aktuálního záznamu člena a evidence vydání. | `all_member_document_pdfs_are_created_and_voucher_is_linked` |
| Poštovní poukázka | PDF a atomicky propojená faktura člena se stejnou částkou. | `voucher_and_member_invoice_are_linked_atomically` |
| CSV import členů | Náhled, validace, mapování českých sloupců, detekce duplicit, záloha, transakce a audit. | `member_import_previews_inserts_and_audits` |
| Řízené storno | Pouze správce, povinný důvod, ukončení aktuálního záznamu bez hard delete a audit. | `controlled_deactivation_ends_only_current_record_and_audits` |
| Diagnostika | Read-only integrita databáze, verze, aktivní rok, provozní počty a počet záloh bez Access `SendKeys`. | kompilace příkazu a kontrola UI |

## Umístění implementace

- `src-tauri/src/lib.rs` — aplikační příkazy, oprávnění, zálohy, dávkové doklady a integrační testy.
- `src-tauri/src/reports.rs` — parametrické sestavy, PDF/CSV a audit.
- `src-tauri/src/financial_documents.rs` — faktury, platební dávky a členské dokumenty.
- `src-tauri/src/member_import.rs` — validovaný import členů.
- `src/App.tsx` — obrazovky a uživatelské akce.

## Stav parity

Funkční mapa obsahuje 46 položek: 30 má stav **Hotovo**, 16 stav **Nahrazeno**, 0 stav **Chybí** a 0 stav **Vyřazeno**. Stav Vyřazeno nebyl použit, protože nebylo doloženo schválení Product Ownerem.

Zbývající činnosti jsou provozní akceptace na kopii produkčních dat, vizuální porovnání tiskových výstupů a případné produktové změny. Nejde o chybějící funkční oblasti z auditu.

## Závěrečné technické ověření

- Backend: 53 testů prošlo, 0 selhalo.
- Frontend: TypeScript kontrola a produkční Vite build prošly.
- Desktop: Tauri debug build bez balíčkování prošel a vytvořil `src-tauri/target/debug/pojisteni-klienti.exe`.
- Formátování Rust zdrojů: `cargo fmt` aplikován a následná kontrola změn je bez chyb formátu patchů.
