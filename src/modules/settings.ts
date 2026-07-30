export type SettingsModule = {
  id: "tariffs" | "payments" | "updates" | "limits" | "organizations" | "users" | "database" | "backups";
  label: string;
  enabled: boolean;
};

export const SETTINGS_MODULES: SettingsModule[] = [
  { id: "tariffs", label: "Sazby pojistného", enabled: true },
  { id: "payments", label: "Platební údaje", enabled: true },
  { id: "updates", label: "Aktualizace", enabled: true },
  { id: "limits", label: "Limity pojištění", enabled: false },
  { id: "organizations", label: "Organizace", enabled: false },
  { id: "users", label: "Uživatelé", enabled: false },
  { id: "database", label: "Databáze", enabled: false },
  { id: "backups", label: "Zálohy", enabled: true },
];
