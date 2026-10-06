import { readFile } from "node:fs/promises";

const app = await readFile(new URL("../src/App.tsx", import.meta.url), "utf8");
const backend = await readFile(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
const main = await readFile(new URL("../src/main.tsx", import.meta.url), "utf8");
const fontScale = await readFile(new URL("../src/ui/fontScale.ts", import.meta.url), "utf8");
const css = await readFile(new URL("../src/App.css", import.meta.url), "utf8");
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
  'invoke<string>("create_blank_application_for_printing")',
  'invoke("print_generated_pdf",{path,deleteAfterPrint:true})',
  'invoke("print_generated_pdf",{path,deleteAfterPrint:false})',
  '<Printer/> Tisk',
  'function RegistrationNumberPicker(',
  'invoke<RegistrationNumberOptions>("get_registration_number_options")',
  'registrationNumber}}',
  '<dt>Evidenční číslo</dt><dd>{registrationNumber}</dd>',
  'settingsSection === "font-size"',
  "FONT_SCALE_OPTIONS.map",
  "saveFontScale(option.value)",
  "Seznam pojištěnců – Jan Novák – evidenční číslo 1345",
  'invoke<string[]>("list_certificate_organizations")',
  "Hromadné potvrzení a export",
  "Exportovat jedno PDF",
];

const missing = requiredUiContracts.filter((contract) => !app.includes(contract));
const requiredNavigationOrder = [
  'label: "Hlavní panel"',
  'label: "Nový pojištěnec"',
  'label: "Seznam pojištěnců"',
  'label: "Přidat platbu"',
  'label: "Doklady o zaplacení"',
];
let previousNavigationIndex = -1;
for (const item of requiredNavigationOrder) {
  const index = app.indexOf(item, previousNavigationIndex + 1);
  if (index < 0 || index <= previousNavigationIndex) {
    missing.push(`navigation order: ${requiredNavigationOrder.join(" → ")}`);
    break;
  }
  previousNavigationIndex = index;
}
for (const contract of ["fn create_blank_application_for_printing(", "fn print_generated_pdf(", '"-Verb", "Print"', "fn registration_number_options(", "fn allocate_registration_number(", "get_registration_number_options,", "fn list_certificate_organizations(", "merge_pdf_documents(documents)"]) {
  if (!backend.includes(contract)) missing.push(contract);
}
for (const contract of ["initializeFontScale();"]) {
  if (!main.includes(contract)) missing.push(contract);
}
for (const contract of ["100", "110", "125", "150", "localStorage", 'dataset.fontScale = String(value)']) {
  if (!fontScale.includes(contract)) missing.push(`font scale: ${contract}`);
}
for (const contract of ["zoom: var(--ui-scale)", "overflow-y: auto", ".font-size-preview", 'html[data-font-scale="150"]', ".dashboard-grid", "@media print", "zoom: 1"]) {
  if (!css.includes(contract)) missing.push(`font scale CSS: ${contract}`);
}
if (app.includes('"export_blank_application"') || backend.includes("fn export_blank_application(")) {
  missing.push("legacy blank-application Save dialog flow is still present");
}
if (packageJson.version !== "0.25.1") missing.push("package version 0.25.1");

if (missing.length) {
  console.error(`UI smoke contract is incomplete:\n- ${missing.join("\n- ")}`);
  process.exit(1);
}

console.log(`UI smoke contract PASS (${requiredUiContracts.length} UI/startup contracts, app ${packageJson.version}).`);
