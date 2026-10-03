# Databázové migrace

## Vlastník schématu

Jediným vlastníkem změn základního databázového schématu je `src-tauri/src/migrations.rs`.
Aktuální podporovaná verze je `DB_SCHEMA_VERSION = 1` a je uložena v
`PRAGMA user_version`. Migrační manager se spustí v Tauri `setup` před tím, než
frontend zpřístupní přihlášení a běžné moduly.

## Inventář původních runtime změn

| Oblast | Původní změna | Klasifikace | Stav ve verzi 1 |
|---|---|---|---|
| `lib.rs` | `Telefon` v `Seznam`/`Editace`, `AuditLog`, `AppUsers` | bootstrap | přesunuto do baseline migrace |
| `current_insurance_year.rs` | `PojistnaObdobi`, `NeprevadetCleny`, `Telefon` | bootstrap + runtime data | DDL přesunuto; seed/roční převod zůstává řízená datová operace |
| `tariffs.rs` | `sazby_pojistneho` + index | bootstrap | centrální baseline |
| `payments.rs` | nastavení, příkazy + index | bootstrap | centrální baseline |
| `member_payments.rs` | platby, audit, index, doplnění vazby organizace | bootstrap / kompatibilizační migrace | centrální baseline |
| `organization_payments.rs` | organizační platby, rozpis + index | bootstrap | centrální baseline |
| `claims.rs` | události, vazební fronta, index, import Access záznamů | bootstrap + datová migrace | atomicky v baseline 0→1 |
| `email_service.rs` | nastavení e-mailu | bootstrap | centrální baseline |
| `receipts.rs` | nastavení, doklady, audit + index | bootstrap | centrální baseline |
| `applications.rs` | přihlášky, index, immutable triggery | bootstrap | centrální baseline |
| `financial_documents.rs` | faktury, dávky, audit, členské doklady | bootstrap | centrální baseline |
| `reports.rs` | audit sestav | bootstrap | centrální baseline |

Funkce `ensure_schema` zůstávají jako interní stavební blok definice baseline a
pro izolované unit testy. Produkční operace je již nevolají při otevření
obrazovky nebo při prvním zápisu.

## Podporovaný baseline 0

Verze 0 je přijata pouze po kontrole integrity a známého současného tvaru:
základní tabulky `Seznam`, `Editace`, `Kategorie`, jejich povinné sloupce a
indexy identity/organizačního kódu. Pokud už jsou přítomné aplikační tabulky
0.24.0, kontrolují se i jejich povinné sloupce. Neznámý nebo neúplný tvar je
odmítnut bez změny databáze.

Historické databáze aplikací 0.2.0–0.16.x nejsou prohlášeny za podporované,
protože nejsou k dispozici reprezentativní fixture. Registry migrací umožňuje
později přidat explicitní importer nebo další ověřenou cestu.

## Přidání další migrace

Nový krok musí být samostatná položka registru se zdrojovou a cílovou verzí,
deterministickou transakční funkcí a testem. `user_version` se mění až uvnitř
úspěšně dokončené transakce. Před změnou existující databáze manager nejprve
vytvoří a znovu otevře `.fvcbackup`; tím ověří formát, checksum, SQLite
integritu, počet členů a původní schema version.
