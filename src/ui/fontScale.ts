export const FONT_SCALE_STORAGE_KEY = "federace.uiFontScale";

export const FONT_SCALE_OPTIONS = [
  { value: 100, label: "100 % – Výchozí" },
  { value: 110, label: "110 % – Větší" },
  { value: 125, label: "125 % – Velké" },
  { value: 150, label: "150 % – Velmi velké" },
] as const;

export type FontScale = (typeof FONT_SCALE_OPTIONS)[number]["value"];

export function isFontScale(value: number): value is FontScale {
  return FONT_SCALE_OPTIONS.some((option) => option.value === value);
}

export function readFontScale(): FontScale {
  try {
    const value = Number(window.localStorage.getItem(FONT_SCALE_STORAGE_KEY));
    return isFontScale(value) ? value : 100;
  } catch {
    return 100;
  }
}

export function applyFontScale(value: FontScale): void {
  document.documentElement.style.setProperty("--ui-scale", String(value / 100));
  document.documentElement.dataset.fontScale = String(value);
}

export function saveFontScale(value: FontScale): void {
  window.localStorage.setItem(FONT_SCALE_STORAGE_KEY, String(value));
  applyFontScale(value);
}

export function initializeFontScale(): FontScale {
  const value = readFontScale();
  applyFontScale(value);
  return value;
}
