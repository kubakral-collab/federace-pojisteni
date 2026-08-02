# Pravidla práce

Před prací: přečíst README a ROADMAP; ověřit schválený sprint, prioritu a aktuální Milestone; auditovat implementaci; zachovat funkční stav. Požadavek mimo ROADMAP neimplementovat ani z něj nevytvářet sprint, ale navrhnout Backlog/Parking Lot a čekat na Product Ownera.

Během práce: žádný nesouvisející refactoring, redesign ani funkce mimo scope; neměnit Access ani zdrojovou `dd.sqlite`; nevytvářet fiktivní provozní data; používat pracovní databázi a idempotentní migrace.

Po práci: spustit testy, aktualizovat dokumentaci, uvést změněné soubory, testy, omezení a otevřené body. Release nepublikovat mimo schválený cyklus; sprint ani commit není release.
