# FED-DATA-AUTHORITY-01 — autoritativní datový snímek

Datum zahájení: 2026-09-01

Priorita: P0

Milestone: M1 — Funkční náhrada Accessu / stabilizace

Stav: dokončeno a akceptováno Product Ownerem

## Cíl

Písemně určit autoritativní datový snímek pro případné budoucí synchronizace. Sprint je výhradně read-only a nesmí měnit Access, `dd.sqlite` ani pracovní databázi aplikace.

## Ověřené zdroje

| Zdroj | SHA-256 | Řádky `Seznam` | Role |
| --- | --- | ---: | --- |
| Produkční `Pojištění.accdb` | `DF176160A48085BDA225FCAB3D8A9194371E2EBEE43C9EB27CA497D696E0DEDC` | 14 408 | Později auditovaný produkční Access; původní audit jej označil jako hlavní databázi. |
| Repozitářová `dd.sqlite` | `F8F90FDEEDEDFED1159D4AA2F24B52AA5B96B94A9726201DD57CDD2F63B93D2D` | 14 416 | Pracovní SQLite vytvořená z jiné, dřívější kopie Accessu. |

## Read-only důkaz z 2026-09-01

Nástroj `scripts/audit-access-sqlite-parity.py` byl spuštěn dvakrát nad stejnými soubory.

| Výsledek | Počet |
| --- | ---: |
| Přesně shodné řádky | 14 370 |
| Stejná identita, změněná pole | 36 |
| Pouze v produkčním Accessu | 2 |
| Pouze v `dd.sqlite` | 10 |
| Nejednoznačné skupiny | 0 |

Oba běhy skončily kódem 0 a daly stejný výsledek. SHA-256 obou databází před a po bězích zůstal totožný. Nebyl proveden žádný zápis ani synchronizace.

## Doporučení

Jako autoritativní zdroj doporučujeme **produkční Access s hashem `DF176160…E0DEDC`**, protože:

1. původní forenzní audit jej označil jako hlavní produkční databázi;
2. jde o pozdější produkční snímek než zdroj, ze kterého vznikla `dd.sqlite`;
3. deklarovaným cílem Federace je odpovídat původní produkční aplikaci Access;
4. rozdíly jsou úplně klasifikované a nezůstala žádná nejednoznačná skupina.

Toto doporučení samo o sobě neopravňuje změnit `dd.sqlite` ani uživatelskou pracovní databázi.

## Rozhodnutí požadované od Product Ownera

Product Owner zvolil první možnost:

- **Produkční Access s hashem `DF176160A48085BDA225FCAB3D8A9194371E2EBEE43C9EB27CA497D696E0DEDC` je autoritativní.**

Rozhodnutí neurčuje automatickou synchronizaci. Případná synchronizace 36 změněných, 2 Access-only a 10 SQLite-only řádků vyžaduje nový samostatný, vratný migrační sprint.

## Hranice a bezpečnost

- žádné `INSERT`, `UPDATE`, `DELETE` ani změna schématu;
- žádný merge podle odhadu;
- žádné zveřejnění osobních hodnot v dokumentaci;
- případná synchronizace musí mít vlastní scope, zálohu, náhled změn, idempotenci, kontrolní součty a rollback;
- starý i nový snímek musí být zachován nejméně do akceptace případné migrace.

## Definition of Done

- [x] ověřeny identifikátory a hashe obou zdrojů;
- [x] read-only audit spuštěn dvakrát se stejným výsledkem;
- [x] potvrzeno 14 370 / 36 / 2 / 10 a 0 nejednoznačných skupin;
- [x] potvrzeno, že oba zdroje zůstaly bitově nezměněné;
- [x] zapsáno doporučení a bezpečnostní hranice;
- [x] Product Owner písemně určil autoritativní variantu;
- [x] rozhodnutí je zapsáno v `DECISIONS.md` a návazné dokumentaci.
