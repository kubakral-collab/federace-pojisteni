import { readFile } from "node:fs/promises";

const app = await readFile(new URL("../src/App.tsx", import.meta.url), "utf8");
const packageJson = JSON.parse(await readFile(new URL("../package.json", import.meta.url), "utf8"));

const requiredUiContracts = [
  'invoke<StartupStatus>("get_startup_status")',
  'invoke<{ initialized: boolean }>("get_auth_status")',
  'authInitialized ? "login" : "initialize_admin"',
  'invoke<DashboardInfo>("get_dashboard")',
  'invoke<MemberPage>("list_members"',
  'invoke<Member>("get_member"',
  'invoke<MemberPage>("list_members", {',
  '"Nový člen"',
  "<h1>Platby</h1>",
  '"Pojistné události"',
  'screen === "Doklady o zaplacení"',
  '"Nastavení"',
  '"O programu"',
  "dashboard?.programVersion",
  "databaseSchemaVersion",
  "startupStatus.smokeMode",
  "startupStatus.databasePath",
];

const missing = requiredUiContracts.filter((contract) => !app.includes(contract));
if (packageJson.version !== "0.25.0") missing.push("package version 0.25.0");

if (missing.length) {
  console.error(`UI smoke contract is incomplete:\n- ${missing.join("\n- ")}`);
  process.exit(1);
}

console.log(`UI smoke contract PASS (${requiredUiContracts.length} UI/startup contracts, app ${packageJson.version}).`);
