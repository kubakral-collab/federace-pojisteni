# Backlog

| ID | Název | Priorita | Milestone | Stav | Závislosti | Důvod |
|---|---|---|---|---|---|---|
| UDALOSTI-001 | Historické pojištění podle data vzniku události | P1 | M1 — Stabilizace Core | Implementováno, čeká na uživatelský smoke test | historie `Seznam`; datum vzniku | Zabránit použití současných pojistných údajů u historické události. |
| CONTACT-CALC-001 | Telefon člena a zaokrouhlení poměrného pojistného nahoru | P1 | M1 — Stabilizace Core | Implementováno, čeká na uživatelský smoke test | bezpečná migrace Telefon; tarifní výpočet | Doplnit provozní kontakt a odstranit podhodnocení desetinného výsledku. |
| PLATBY-002 | Individuální a organizační platby s rozúčtováním | P1 | M2 — Provozní automatizace | Implementováno, čeká na uživatelský smoke test | současná evidence členů a plateb; bezpečná migrace | Evidovat jednu přijatou organizační platbu a dohledatelný rozpis bez svévolného dělení nedoplatku či přeplatku. |
| PLATBY-001 | Individuální a organizační platby | P2 | M2 — Provozní automatizace | Schváleno, čeká na zařazení do pořadí sprintů | audit současného modelu plateb; identifikace organizace; bezpečná migrace | Evidovat platby za člena i organizaci přímo z modulu Platby bez vymyšleného rozúčtování. |
| UX-001 | Pořadí položek „Nový pojištěnec“ a „Seznam pojištěnců“ | P2 | M1 — Stabilizace Core | Schváleno, čeká na zařazení po aktuálním sprintu | dokončení nebo uzavření S1; existující konfigurace levého menu | Zjednodušit přirozený začátek práce bez změny funkčnosti navigace. |
| ZMĚNA-003 | Automatické každoroční příkazy a odeslání | P2 | M2 | Schváleno, bez sprintu | rok, sazby, SMTP | Omezit ruční práci. |
| ZMĚNA-004 | Bankovní import a párování | P2 | M2 | Schváleno, bez sprintu | platby, pravidla párování | Odstranit přepis plateb. |
| BEZPEČNOST-002 | SQLCipher | P2 | M3 | Schváleno, bez sprintu | migrace, recovery | Šifrování osobních dat. |
| BEZPEČNOST-004 | Úplný audit citlivých operací | P2 | M3 | Částečně implementováno | jednotný audit | Dohledatelnost změn. |

## Dokončená historie
BUG-001 platební filtr; FUNKCE-001 bezpečný roční převod; REPORT-001 sestavy OC/ZO/HVP; ZMĚNA-001 live search; ZMĚNA-002 akce v detailu; ZMĚNA-005 updater; BEZPEČNOST-001 ruční backup/recovery; BEZPEČNOST-003 přihlášení (zamykání zbývá); pojistné události a přehled pro pojišťovnu jsou implementované. Podrobné důkazy jsou v `ACCESS_PARITY_COMPLETION.md`.
