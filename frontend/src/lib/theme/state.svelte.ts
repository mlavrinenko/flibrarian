const THEMES = ["light", "dark", "custom"] as const;

export type Theme = (typeof THEMES)[number];

const STORAGE_KEY = "theme";
const CUSTOM_COLORS_KEY = "theme-custom-colors";

export interface CustomColors {
  bg: string;
  bgHover: string;
  bgSelected: string;
  bgInput: string;
  bgHeader: string;
  text: string;
  textSecondary: string;
  border: string;
  primary: string;
  error: string;
}

export const DEFAULT_CUSTOM_COLORS: CustomColors = {
  bg: "#faf6f0",
  bgHover: "#f3ede4",
  bgSelected: "#ebe3d6",
  bgInput: "#fefcf9",
  bgHeader: "#f5f0e8",
  text: "#42352a",
  textSecondary: "#6b5d52",
  border: "#d4cdc3",
  primary: "#4a9eff",
  error: "#d32f2f",
};

const CSS_VAR_MAP: Record<keyof CustomColors, string> = {
  bg: "--color-bg",
  bgHover: "--color-bg-hover",
  bgSelected: "--color-bg-selected",
  bgInput: "--color-bg-input",
  bgHeader: "--color-bg-header",
  text: "--color-text",
  textSecondary: "--color-text-secondary",
  border: "--color-border",
  primary: "--color-primary",
  error: "--color-error",
};

function isTheme(value: string): value is Theme {
  return (THEMES as readonly string[]).includes(value);
}

function detectOsTheme(): Theme {
  if (window.matchMedia("(prefers-color-scheme: light)").matches) {
    return "light";
  }
  return "dark";
}

function loadTheme(): Theme {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored && isTheme(stored)) {
    return stored;
  }
  return detectOsTheme();
}

function loadCustomColors(): CustomColors {
  try {
    const raw = localStorage.getItem(CUSTOM_COLORS_KEY);
    if (raw) {
      return {
        ...DEFAULT_CUSTOM_COLORS,
        ...(JSON.parse(raw) as Partial<CustomColors>),
      };
    }
  } catch {
    // ignore
  }
  return { ...DEFAULT_CUSTOM_COLORS };
}

function applyTheme(value: Theme) {
  const root = document.documentElement;
  if (value === "custom") {
    root.setAttribute("data-theme", "custom");
  } else {
    root.setAttribute("data-theme", value);
    for (const cssVar of Object.values(CSS_VAR_MAP)) {
      root.style.removeProperty(cssVar);
    }
  }
}

function applyCustomColors(colors: CustomColors) {
  const root = document.documentElement;
  for (const [key, cssVar] of Object.entries(CSS_VAR_MAP)) {
    root.style.setProperty(cssVar, colors[key as keyof CustomColors]);
  }
}

class ThemeState {
  current: Theme = $state(loadTheme());
  customColors: CustomColors = $state(loadCustomColors());

  constructor() {
    applyTheme(this.current);
    if (this.current === "custom") {
      applyCustomColors(this.customColors);
    }

    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", (e) => {
        if (!localStorage.getItem(STORAGE_KEY)) {
          this.current = e.matches ? "dark" : "light";
          applyTheme(this.current);
        }
      });
  }

  toggle() {
    const order: Theme[] = ["light", "dark", "custom"];
    const idx = order.indexOf(this.current);
    this.setTheme(order[(idx + 1) % order.length]);
  }

  setTheme(next: Theme) {
    this.current = next;
    localStorage.setItem(STORAGE_KEY, next);
    applyTheme(next);
    if (next === "custom") {
      applyCustomColors(this.customColors);
    }
  }

  setCustomColor(key: keyof CustomColors, value: string) {
    this.customColors[key] = value;
    localStorage.setItem(CUSTOM_COLORS_KEY, JSON.stringify(this.customColors));
    if (this.current === "custom") {
      document.documentElement.style.setProperty(CSS_VAR_MAP[key], value);
    }
  }

  setCustomColors(colors: CustomColors) {
    this.customColors = { ...colors };
    localStorage.setItem(CUSTOM_COLORS_KEY, JSON.stringify(this.customColors));
    if (this.current === "custom") {
      applyCustomColors(this.customColors);
    }
  }
}

export const theme = new ThemeState();
