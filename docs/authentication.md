# Přihlášení a účet správce

## První spuštění

Aplikace neobsahuje výchozí heslo ani jeho hash. Při prvním spuštění pracovní databáze uživatel vytvoří účet správce a zadá vlastní heslo o délce alespoň 12 znaků.

Heslo se neukládá. Aplikace vytvoří náhodnou sůl a uloží pouze Argon2id hash do tabulky `AppUsers` v pracovní SQLite databázi v aplikačním datovém adresáři. Uživatelské jméno se převezme z přihlášeného účtu Windows.

## Další spuštění

Přihlášení načte aktivní účet z pracovní databáze a ověří zadané heslo proti uloženému Argon2id hashi. Hodnota hesla ani hash se nezapisují do aplikačních logů.

## Zálohy a obnova

Účet je součástí pracovní databáze, a proto je zahrnut do úplné zálohy `.fvcbackup`. Zálohu je nutné chránit stejně jako produkční databázi. Tento sprint neimplementuje šifrování záloh.

## Obnova přístupu

V této verzi není implementováno výchozí ani servisní heslo. Ztracené heslo nelze získat ze zdrojového kódu. Obnova přístupu musí být řešena obnovením důvěryhodné úplné zálohy nebo budoucím samostatným administrátorským workflow.
