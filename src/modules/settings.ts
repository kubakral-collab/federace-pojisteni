export type SettingsModule = {
  id: "font-size" | "tariffs" | "payments" | "email" | "receipts" | "updates" | "limits" | "organizations" | "users" | "database" | "backups";
  label: string;
  enabled: boolean;
};

export const SETTINGS_MODULES: SettingsModule[] = [
  { id: "font-size", label: "Velikost písma", enabled: true },
  { id: "tariffs", label: "Sazby pojistného", enabled: true },
  { id: "payments", label: "Platební údaje", enabled: true },
  { id: "email", label: "E-mail (SMTP)", enabled: true },
  { id: "receipts", label: "Doklady o zaplacení", enabled: true },
  { id: "updates", label: "Aktualizace", enabled: true },
  { id: "limits", label: "Limity pojištění", enabled: false },
  { id: "organizations", label: "Organizace", enabled: false },
  { id: "users", label: "Uživatelé", enabled: false },
  { id: "database", label: "Databáze", enabled: false },
  { id: "backups", label: "Zálohy", enabled: true },
];
