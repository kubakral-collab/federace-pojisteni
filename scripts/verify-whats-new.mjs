import { readFileSync } from "node:fs";

const packageVersion = JSON.parse(readFileSync("package.json", "utf8")).version;
const changelog = JSON.parse(readFileSync("src/content/whats-new.json", "utf8"));
const appSource = readFileSync("src/App.tsx", "utf8");
const storageSource = readFileSync("src/ui/whatsNew.ts", "utf8");
const forbidden = ["commit", "stack trace", "rowid", "databázová tabulka", "api endpoint", "codex", "jarvis"];

function validate(version, entries) {
  const entry = entries[version];
  if (!entry) throw new Error(`Chybí uživatelský changelog pro verzi ${version}.`);
  if (!entry.title?.trim() || !entry.intro?.trim() || !Array.isArray(entry.sections) || entry.sections.length === 0) {
    throw new Error(`Uživatelský changelog pro verzi ${version} není kompletní.`);
  }
  const text = JSON.stringify(entry).toLocaleLowerCase("cs-CZ");
  if (forbidden.some((word) => text.includes(word))) {
    throw new Error(`Uživatelský changelog pro verzi ${version} obsahuje interní technický text.`);
  }
  for (const section of entry.sections) {
    if (!section.title?.trim() || !Array.isArray(section.items) || section.items.length === 0 || section.items.some((item) => !item?.trim())) {
      throw new Error(`Uživatelský changelog pro verzi ${version} obsahuje prázdnou sekci.`);
    }
  }
}

validate(packageVersion, changelog);
let missingWasRejected = false;
try {
  validate("9999.0.0", changelog);
} catch {
  missingWasRejected = true;
}
if (!missingWasRejected) throw new Error("Release gate neodmítl chybějící changelog.");

const shouldShow = (version, lastSeen, ready = true, authenticated = true) =>
  ready && authenticated && Boolean(changelog[version]) && lastSeen !== version;
if (!shouldShow(packageVersion, "0.26.0")) throw new Error("První spuštění nové verze nezobrazí novinky.");
if (shouldShow(packageVersion, packageVersion)) throw new Error("Potvrzená verze zobrazuje novinky opakovaně.");
if (!shouldShow(packageVersion, "0.24.0")) throw new Error("Přeskočení verzí nezobrazí aktuální novinky.");
if (shouldShow(packageVersion, "0.26.0", false)) throw new Error("Novinky se zobrazí i po neúspěšném startu nebo migraci.");
if (shouldShow(packageVersion, "0.26.0", true, false)) throw new Error("Novinky se zobrazí před přihlášením.");
for (const contract of [
  'window.localStorage.setItem(LAST_SEEN_WHATS_NEW_VERSION_KEY, version)',
  'window.localStorage.getItem(LAST_SEEN_WHATS_NEW_VERSION_KEY) !== version',
]) {
  if (!storageSource.includes(contract)) throw new Error(`Chybí kontrakt uložení potvrzené verze: ${contract}`);
}
for (const contract of [
  'if (!user || preview || memberDetailRowId || !startupStatus?.ready) return',
  'setWhatsNew(whatsNewForVersion(version))',
  'confirmWhatsNewVersion(startupStatus.applicationVersion)',
  'setWhatsNewError("Potvrzení se nepodařilo uložit. Zkuste to prosím znovu.")',
]) {
  if (!appSource.includes(contract)) throw new Error(`Chybí startupový kontrakt novinek: ${contract}`);
}

console.log(`Co je nového ${packageVersion}: startup, potvrzení, přeskočení verzí a release gate PASS.`);
