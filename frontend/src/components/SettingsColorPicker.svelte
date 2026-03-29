<script lang="ts">
  import {
    theme,
    DEFAULT_CUSTOM_COLORS,
    type CustomColors,
  } from "../lib/theme";
  import { i18n } from "../lib/i18n";

  const COLOR_KEYS: (keyof CustomColors)[] = [
    "bg",
    "bgHover",
    "bgSelected",
    "bgInput",
    "bgHeader",
    "text",
    "textSecondary",
    "border",
    "primary",
    "error",
  ];

  const colorLabelMap: Record<keyof CustomColors, () => string> = {
    bg: () => i18n.t.themeCustomization.colorBg,
    bgHover: () => i18n.t.themeCustomization.colorBgHover,
    bgSelected: () => i18n.t.themeCustomization.colorBgSelected,
    bgInput: () => i18n.t.themeCustomization.colorBgInput,
    bgHeader: () => i18n.t.themeCustomization.colorBgHeader,
    text: () => i18n.t.themeCustomization.colorText,
    textSecondary: () => i18n.t.themeCustomization.colorTextSecondary,
    border: () => i18n.t.themeCustomization.colorBorder,
    primary: () => i18n.t.themeCustomization.colorPrimary,
    error: () => i18n.t.themeCustomization.colorError,
  };
</script>

<div class="color-grid">
  {#each COLOR_KEYS as key (key)}
    <label class="color-field">
      <input
        type="color"
        value={theme.customColors[key]}
        oninput={(e) => {
          theme.setCustomColor(key, (e.target as HTMLInputElement).value);
        }}
      />
      <span>{colorLabelMap[key]()}</span>
    </label>
  {/each}
</div>
<button
  class="reset-btn"
  onclick={() => {
    theme.setCustomColors({ ...DEFAULT_CUSTOM_COLORS });
  }}
>
  {i18n.t.themeCustomization.resetDefaults}
</button>

<style>
  .color-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .color-field {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.8rem;
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  .color-field input[type="color"] {
    -webkit-appearance: none;
    appearance: none;
    width: 28px;
    height: 28px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 2px;
    cursor: pointer;
    background: var(--color-bg-input);
    flex-shrink: 0;
  }

  .color-field input[type="color"]::-webkit-color-swatch-wrapper {
    padding: 0;
  }

  .color-field input[type="color"]::-webkit-color-swatch {
    border: none;
    border-radius: 4px;
  }

  .color-field input[type="color"]::-moz-color-swatch {
    border: none;
    border-radius: 4px;
  }

  .reset-btn {
    padding: 0.3rem 0.6rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg-input);
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 0.8rem;
    margin-bottom: 1.25rem;
  }

  .reset-btn:hover {
    border-color: var(--color-primary);
    color: var(--color-text);
  }
</style>
