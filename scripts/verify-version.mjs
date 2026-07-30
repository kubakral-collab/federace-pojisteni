import { readFileSync } from "node:fs";

const packageVersion = JSON.parse(readFileSync("package.json", "utf8")).version;
const tauriVersion = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8")).version;
const cargoText = readFileSync("src-tauri/Cargo.toml", "utf8");
const cargoVersion = cargoText.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
const versions = { package: packageVersion, tauri: tauriVersion, cargo: cargoVersion };

if (!cargoVersion || new Set(Object.values(versions)).size !== 1) {
  console.error("Verze aplikace nejsou shodné:", versions);
  process.exit(1);
}

const tagIndex = process.argv.indexOf("--tag");
const tag = tagIndex >= 0 ? process.argv[tagIndex + 1] : "";
if (tag && tag !== `v${packageVersion}`) {
  console.error(`Git tag ${tag} neodpovídá verzi v projektu v${packageVersion}.`);
  process.exit(1);
}

console.log(`Ověřena jednotná verze ${packageVersion}${tag ? ` pro tag ${tag}` : ""}.`);
