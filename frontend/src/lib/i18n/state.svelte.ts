import type { Translation } from "./types";
import ru from "./ru";
import en from "./en";

const translations = { ru, en } satisfies Record<string, Translation>;

export type Locale = keyof typeof translations;

const STORAGE_KEY = "locale";
const DEFAULT_LOCALE: Locale = "ru";

function loadLocale(): Locale {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored && stored in translations) {
    return stored as Locale;
  }
  return DEFAULT_LOCALE;
}

class I18nState {
  locale: Locale = $state(loadLocale());

  get t(): Translation {
    return translations[this.locale];
  }

  setLocale(next: Locale) {
    this.locale = next;
    localStorage.setItem(STORAGE_KEY, next);
  }

  toggle() {
    const locales = Object.keys(translations) as Locale[];
    const idx = locales.indexOf(this.locale);
    this.setLocale(locales[(idx + 1) % locales.length]);
  }
}

export const i18n = new I18nState();
