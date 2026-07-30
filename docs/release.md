# Vydání aplikace

## Předpoklady

- Změny jsou dokončené a otestované v hlavní větvi.
- Verze je shodná v `package.json`, `src-tauri/Cargo.toml` a `src-tauri/tauri.conf.json`.
- Kontrolu lze spustit příkazem `npm run verify:version`.

## Vytvoření nové verze

1. Upravte verzi ve všech třech souborech.
2. Spusťte `npm run verify:version`, `npm run build` a `cargo test --manifest-path src-tauri/Cargo.toml --locked`.
3. Odešlete změny do GitHubu.
4. Vytvořte tag odpovídající verzi, například `git tag v0.17.0`.
5. Odešlete tag příkazem `git push origin v0.17.0`.

Tag ve tvaru `v*` automaticky spustí workflow `.github/workflows/release.yml`. Workflow ověří verze, sestaví Windows x64 aplikaci, NSIS a MSI instalátor, podepíše NSIS updater artefakt a vytvoří GitHub Release až po úspěšném ověření podpisu.

## Ruční spuštění

V GitHubu otevřete **Actions → Windows Release → Run workflow** a zadejte existující nebo nově vytvářený tag odpovídající verzi projektu. Prázdný nebo odlišný tag způsobí chybu.

## Artefakty

Release musí obsahovat samostatné EXE aplikace, NSIS instalátor, český MSI instalátor, podpis NSIS instalátoru s příponou `.sig` a `latest.json`. Tauri 2 používá na Windows jako updater balíček přímo NSIS instalátor; samostatný přípravný ZIP se nepublikuje. `latest.json` obsahuje verzi, poznámky k vydání, URL balíčku a jeho podpis pro platformu `windows-x86_64`.

## Kontrola a instalace aktualizací

Aplikace používá oficiální Tauri Updater. Produkční endpoint ukazuje na `latest.json` v nejnovějším GitHub Release a vzniká v CI podle skutečného názvu repozitáře. Kontrola při spuštění probíhá asynchronně a lze ji vypnout v **Nastavení → Aktualizace**. Ruční kontrola je dostupná také na stránce **O programu**.

Před instalací Tauri ověří podpis balíčku veřejným klíčem z produkční konfigurace. Neplatný, chybějící nebo cizí podpis aktualizaci zastaví. Tento sprint neprovádí databázovou zálohu, migraci ani rollback.

Poznámky zobrazené v aplikaci jsou stejné jako text GitHub Release, protože workflow vytváří `release-body.md` i položku `notes` v `latest.json` ze stejného zdroje.

## Podpis aktualizačního balíčku

Podpis updateru je minisign/Ed25519 podpis používaný Tauri Updaterem. Veřejný klíč je uložen v `src-tauri/updater-public.key` a v produkčním overlayi `src-tauri/tauri.release.conf.json`. Soukromý klíč ani jeho heslo nesmí být v repozitáři, Git historii, instalačním balíčku, CI artefaktech nebo logu.

Tento podpis není Windows Code Signing. Neodstraňuje upozornění SmartScreen a nepotvrzuje Windows vydavatele EXE/MSI. Komerční Windows Code Signing certifikát není součástí tohoto procesu.

## GitHub Secrets

V nastavení repozitáře otevřete **Settings → Secrets and variables → Actions** a vytvořte:

- `TAURI_SIGNING_PRIVATE_KEY` – celý obsah zašifrovaného soukromého Tauri klíče.
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` – heslo soukromého klíče.

Hodnoty vkládejte přímo přes zabezpečené webové rozhraní GitHubu. Nevkládejte je do příkazové řádky, issue, dokumentace ani workflow. Pipeline před buildem ověří jejich přítomnost a při chybějícím Secretu bezpečně skončí. Tajné hodnoty nejsou kopírovány do `release-assets`.

## Lokální podepsaný test

Produkční klíč je lokálně uložen mimo repozitář v uživatelském chráněném úložišti `%APPDATA%\FederacePojisteni\updater-signing`. Heslo je chráněno Windows DPAPI. Po načtení klíče a hesla pouze do proměnných procesu spusťte:

Před lokálním podepsaným buildem vytvořte ignorovaný soubor `src-tauri/tauri.release.generated.conf.json` z `src-tauri/tauri.release.conf.json` a nahraďte `__UPDATER_ENDPOINT__` platnou HTTPS adresou testovacího `latest.json`. Poté spusťte:

`npm run tauri -- build --config src-tauri/tauri.release.generated.conf.json --bundles nsis`

Po skončení proměnné `TAURI_SIGNING_PRIVATE_KEY` a `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` z procesu ihned odstraňte. Běžný lokální build `npm run tauri -- build` produkční klíč nepotřebuje.

## Ověření podpisu

CI používá nástroj v `tools/updater-signature-verifier`. Ručně lze podpis ověřit:

`cargo run --release --locked --manifest-path tools/updater-signature-verifier/Cargo.toml -- src-tauri/updater-public.key <installer.exe> <installer.exe.sig>`

Workflow navíc povinně ověřuje čtyři scénáře: platný podpis projde, pozměněný balíček selže, cizí podpis selže a chybějící podpis selže. Jakékoli selhání zastaví publikaci Release.

## Záloha klíče

> **VAROVÁNÍ: Ztráta soukromého podpisového klíče nebo jeho hesla může znemožnit vydávání aktualizací pro již nainstalované aplikace.**

Před prvním produkčním vydáním musí být zašifrovaný soukromý klíč a obnovitelné heslo zálohovány odděleně od vývojového počítače. Minimálně jedna kopie musí být na fyzicky odpojeném médiu uloženém na bezpečném místě; heslo uložte odděleně, například do firemního správce hesel. Zálohu otestujte podpisem testovacího souboru a následným ověřením veřejným klíčem.

Pokud se heslo ztratí a neexistuje obnovitelná záloha, starý klíč nelze použít. Nevydávejte produkční release, dokud není záloha dokončena a ověřena.

## Kompromitace a rotace

Při podezření na kompromitaci okamžitě zablokujte release workflow, odstraňte oba GitHub Secrets a uchovejte auditní údaje. Vygenerujte nový pár, nahraďte veřejný klíč v aplikaci a soukromou část uložte do nových Secrets. Bezpečná rotace pro již nainstalované klienty musí být provedena aktualizací podepsanou dosavadním důvěryhodným klíčem, která obsahuje nový veřejný klíč. Teprve po jejím rozšíření lze vydávat balíčky pouze novým klíčem. Pokud byl starý klíč skutečně kompromitován, automatický důvěryhodný přechod nemusí být bezpečný a může být nutná ruční reinstalace z ověřeného zdroje.

## Rollback

1. V GitHub Releases označte vadné vydání jako předběžné nebo jej odstraňte.
2. Vraťte opravu do hlavní větve novým commitem; nepřepisujte publikovaný tag.
3. Zvyšte patch verzi, například z `0.17.0` na `0.17.1`.
4. Vytvořte a odešlete nový tag.

## Nejčastější problémy

- **Neshodná verze:** sjednoťte tři verzované soubory a tag `vX.Y.Z`.
- **Chybějící artefakt:** zkontrolujte výstup kroku Tauri build a konfiguraci `bundle.targets`.
- **Selhání MSI:** zkontrolujte log WiX/Tauri na runneru `windows-latest`.
- **Selhání testů:** opravte chybu před vytvořením nového tagu; workflow release nevydá.
- **Chybějící oprávnění Release:** ověřte oprávnění workflow `contents: write` v nastavení repozitáře.
