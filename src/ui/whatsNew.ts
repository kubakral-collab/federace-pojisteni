import changelog from "../content/whats-new.json";

export const LAST_SEEN_WHATS_NEW_VERSION_KEY = "federace.lastSeenWhatsNewVersion";

export type WhatsNewEntry = {
  title: string;
  intro: string;
  sections: Array<{ title: string; items: string[] }>;
};

const entries = changelog as Record<string, WhatsNewEntry>;

export function whatsNewForVersion(version: string): WhatsNewEntry | null {
  return entries[version] ?? null;
}

export function shouldShowWhatsNew(version: string, lastSeenVersion: string | null): boolean {
  return whatsNewForVersion(version) !== null && lastSeenVersion !== version;
}

export function readLastSeenWhatsNewVersion(): string | null {
  try {
    return window.localStorage.getItem(LAST_SEEN_WHATS_NEW_VERSION_KEY);
  } catch {
    return null;
  }
}

export function confirmWhatsNewVersion(version: string): void {
  window.localStorage.setItem(LAST_SEEN_WHATS_NEW_VERSION_KEY, version);
  if (window.localStorage.getItem(LAST_SEEN_WHATS_NEW_VERSION_KEY) !== version) {
    throw new Error("Potvrzení se nepodařilo uložit.");
  }
}
