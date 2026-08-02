# Bezpečnost

Rodná čísla a osobní údaje se nesmějí dostat do logů, repozitáře ani mimo schválené workflow. Žádná funkce nesmí snížit ochranu dat.

**Implementováno:** první správce bez výchozího hesla; Argon2id; SMTP heslo přes Windows Credential Manager/DPAPI (`keyring` Windows native); kontrolované zálohy a nouzová obnova; GitHub Secrets; updater signing key mimo repo a ověření podpisu v CI.

**Částečně:** audit vybraných změn/plateb/dokladů/událostí; přihlášení bez automatického zamykání; zálohy s integritou, ale bez šifrování.

**Plánováno:** SQLCipher, šifrované zálohy a správa klíčů, automatické zamykání, úplný audit citlivých operací. Únik dat, bezpečnostní chyba, ztráta dat či narušení updateru je P0.
