# FED-PARITY-SOURCE-RECONCILE-01

## Výsledek

Read-only porovnání potvrdilo, že produkční Access a `dd.sqlite` nejsou stejný datový snímek. Zdrojové soubory nebyly změněny.

| Kontrola | Produkční Access | `dd.sqlite` |
|---|---:|---:|
| SHA-256 | `DF176160A48085BDA225FCAB3D8A9194371E2EBEE43C9EB27CA497D696E0DEDC` | `F8F90FDEEDEDFED1159D4AA2F24B52AA5B96B94A9726201DD57CDD2F63B93D2D` |
| Řádky `Seznam` | 14 408 | 14 416 |
| Přesně shodné řádky | 14 370 | 14 370 |

Po odečtení přesně shodných řádků:

- 36 záznamů se stejným interním identifikátorem a evidenčním číslem má změněná pole;
- 2 záznamy existují pouze v produkčním Accessu;
- 10 záznamů existuje pouze v `dd.sqlite`;
- nezůstala žádná nejednoznačná skupina, kterou by porovnávač nedokázal klasifikovat.

Čistý rozdíl `+2 −10` odpovídá rozdílu osmi řádků mezi databázemi.

## Změněná pole

| Pole | Počet změněných záznamů |
|---|---:|
| `PojištěníOd` | 35 |
| `PojistnáČástka` | 35 |
| `PojištěníDo` | 3 |
| `Příjmení` | 3 |
| `SkutÚhrada` | 3 |
| `Adresa` | 1 |
| `Jméno` | 1 |
| `Město` | 1 |
| `PSČ` | 1 |
| `RodnéČíslo` | 1 |
| `RočPojistné` | 1 |
| `e-mail` | 1 |

Report záměrně neobsahuje původní ani nové hodnoty osobních údajů.

## Záznamy pouze na jedné straně

Pouze produkční Access — interní identifikátor / evidenční číslo:

- `17490 / 1345`
- `17491 / 1346`

Pouze `dd.sqlite` — interní identifikátor / evidenční číslo:

- `16761 / 297`
- `16885 / 478`
- `16897 / 494`
- `16900 / 499`
- `16912 / 518`
- `17121 / 861`
- `17135 / 885`
- `17232 / 1056`
- `17293 / 1138`
- `17294 / 1139`

## Metoda

Opakovatelný nástroj je v `scripts/audit-access-sqlite-parity.py`.

1. Produkční Access se zkopíruje do dočasného adresáře a kopie se otevře přes ODBC v režimu read-only.
2. SQLite se otevře pomocí URI `mode=ro`.
3. Nejprve se porovnají celé normalizované řádky jako multiset.
4. Zbývající řádky se párují podle `Identifikátor + EvČíslo`.
5. Výstup uvádí pouze interní klíče a názvy změněných polí, nikoli osobní hodnoty.

Opakované spuštění dává shodný výsledek. Nástroj neprovádí `INSERT`, `UPDATE`, `DELETE`, změnu schématu ani zápis do porovnávaných databází.

## Rozhodnutí a doporučení

Původní forenzní audit označil produkční soubor s hashem `DF176160…E0DEDC` jako hlavní databázi. Product Owner zároveň stanovil, že Federace má odpovídat původní databázi Access. Proto je doporučeným autoritativním datovým snímkem právě tento produkční Access.

Tento sprint žádná data nesynchronizuje. Budoucí migrační sprint musí nejprve vytvořit zálohu, náhled přesných změn a kontrolní report; teprve poté lze změny aplikovat se samostatným schválením.

## Definition of Done

- [x] ověřen kontrolní hash produkčního Accessu;
- [x] oba zdroje otevřeny pouze pro čtení;
- [x] přesně klasifikováno 14 408/14 416 řádků;
- [x] rozdíl osmi řádků vysvětlen jako 2 pouze v Accessu a 10 pouze v SQLite;
- [x] klasifikováno 36 změněných záznamů bez zveřejnění osobních hodnot;
- [x] vytvořen opakovatelný auditní nástroj;
- [x] zdrojové databáze zůstaly nezměněné.
