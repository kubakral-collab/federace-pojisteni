# FED-SMOKE-PAYMENTS-01 — provozní ověření plateb

Datum zahájení: 2026-09-01

Priorita: P1

Milestone: M1 — Funkční náhrada Accessu / stabilizace

Stav: aktivní; automatická brána prošla, uživatelský smoke test čeká

## Cíl

Ověřit již implementované individuální a organizační platby bez přidávání nové funkčnosti. Zápisový smoke test smí proběhnout pouze na pracovní kopii databáze.

## Scénáře

1. Individuální platba se uloží k vybranému členovi a správně změní jeho bilanci.
2. Organizační platba přesně odpovídající součtu se rozdělí podle uloženého pojistného členů.
3. Nedoplatek vyžaduje ruční rozdělení s přesně shodným součtem.
4. Přeplatek se členům svévolně nerozdělí a zůstane nepřiřazený na hlavní platbě.
5. U členů se stejnou pojistnou částkou, ale rozdílným uloženým pojistným, se použije skutečné pojistné daného záznamu.
6. Platby a rozdělení jsou dohledatelné a auditované.

## Bezpečnostní hranice

- žádný zápis do jediné produkční databáze;
- před smoke testem vytvořit pracovní kopii nebo ověřenou zálohu;
- nevytvářet fiktivní transakci v ostré evidenci;
- po testu pracovní kopii zahodit nebo jednoznačně označit jako testovací;
- nalezená vada je nový samostatně evidovaný bug, nikoli příležitost rozšířit scope sprintu.

## Automatická brána

- [x] přesná organizační platba vytvoří hlavní platbu, rozdělení a členské platby;
- [x] nedoplatek vyžaduje přesné ruční rozdělení;
- [x] přeplatek zůstává nepřiřazený;
- [x] výpočet zbývající částky pokrývá nulovou, částečnou, úplnou a přeplatkovou úhradu;
- [x] rozdílné uložené pojistné se nezamění za pojistnou částku;
- [x] individuální CRUD platby přepočítá součet a zapíše audit;
- [x] platební filtr správně rozlišuje zaplaceno a nezaplaceno.

Ověření 2026-09-01: `cargo test payment` — 16 testů prošlo, 0 selhalo, 54 odfiltrováno.

## Uživatelský smoke test

- [ ] uživatel otevře modul Platby nad pracovní kopií;
- [ ] ověří individuální platbu;
- [ ] ověří přesnou organizační platbu;
- [ ] ověří ruční nedoplatek;
- [ ] ověří nepřiřazený přeplatek;
- [ ] zkontroluje bilanci člena, detail organizace a audit;
- [ ] výsledek akceptuje nebo uvede konkrétní odchylku.

## Definition of Done

- [x] automatická brána prošla;
- [ ] uživatelský smoke test proběhl na kopii;
- [ ] produkční data zůstala nezměněná;
- [ ] výsledek je zapsán v roadmapě, backlogu a rozhodnutích.
