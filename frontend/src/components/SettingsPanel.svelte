<script lang="ts">
  import { untrack } from "svelte";
  import { settings } from "../lib/settings";
  import { i18n, type Locale } from "../lib/i18n";
  import { theme, type Theme, type CustomColors } from "../lib/theme";
  import { isTauri } from "../lib/api";
  import { errorMessage } from "../lib/types";
  import SunIcon from "./icons/SunIcon.svelte";
  import MoonIcon from "./icons/MoonIcon.svelte";
  import PaletteIcon from "./icons/PaletteIcon.svelte";
  import RuFlagIcon from "./icons/RuFlagIcon.svelte";
  import EnFlagIcon from "./icons/EnFlagIcon.svelte";
  import SettingsPathField from "./SettingsPathField.svelte";
  import SettingsColorPicker from "./SettingsColorPicker.svelte";

  let libraryPath = $state(settings.libraryPath);
  let defaultSaveFolder = $state(settings.defaultSaveFolder);
  let saving = $state(false);
  let error: string | null = $state(null);
  let savedTheme: Theme = $state(theme.current);
  let savedLocale: Locale = $state(i18n.locale);
  let savedCustomColors: CustomColors = $state({ ...theme.customColors });

  $effect(() => {
    if (settings.open) {
      libraryPath = settings.libraryPath;
      defaultSaveFolder = settings.defaultSaveFolder;
      savedTheme = untrack(() => theme.current);
      savedLocale = untrack(() => i18n.locale);
      savedCustomColors = untrack(() => ({ ...theme.customColors }));
      error = null;
    }
  });

  async function browseFolder(target: "library" | "save") {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({ directory: true });
    if (selected) {
      if (target === "library") {
        libraryPath = selected;
      } else {
        defaultSaveFolder = selected;
      }
    }
  }

  async function handleSave() {
    saving = true;
    error = null;
    try {
      settings.libraryPath = libraryPath;
      settings.defaultSaveFolder = defaultSaveFolder;
      await settings.save();
      settings.open = false;
    } catch (e) {
      error = errorMessage(e);
    } finally {
      saving = false;
    }
  }

  function handleCancel() {
    theme.setTheme(savedTheme);
    theme.setCustomColors(savedCustomColors);
    i18n.setLocale(savedLocale);
    settings.open = false;
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      handleCancel();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      handleCancel();
    }
  }

  const tauriEnv = isTauri();

  const themeOptions: {
    value: Theme;
    icon: typeof SunIcon;
    label: () => string;
  }[] = [
    {
      value: "light",
      icon: SunIcon,
      label: () => i18n.t.themeCustomization.light,
    },
    {
      value: "dark",
      icon: MoonIcon,
      label: () => i18n.t.themeCustomization.dark,
    },
    {
      value: "custom",
      icon: PaletteIcon,
      label: () => i18n.t.themeCustomization.custom,
    },
  ];
</script>

{#if settings.open}
  <div
    class="backdrop"
    role="dialog"
    aria-modal="true"
    aria-label={i18n.t.settings.title}
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
    tabindex="-1"
  >
    <div class="modal">
      <h2>{i18n.t.settings.title}</h2>

      <div class="section-label">{i18n.t.themeCustomization.title}</div>
      <div class="theme-row">
        {#each themeOptions as opt (opt.value)}
          <button
            class="theme-btn"
            class:active={theme.current === opt.value}
            onclick={() => {
              theme.setTheme(opt.value);
            }}
            aria-label={opt.label()}
            title={opt.label()}
          >
            <opt.icon />
            <span>{opt.label()}</span>
          </button>
        {/each}
      </div>

      {#if theme.current === "custom"}
        <SettingsColorPicker />
      {/if}

      <div class="section-label">{i18n.t.languageSwitcher}</div>
      <div class="theme-row">
        <button
          class="theme-btn"
          class:active={i18n.locale === "ru"}
          onclick={() => {
            i18n.setLocale("ru");
          }}
          aria-label="Русский"
        >
          <RuFlagIcon />
          <span>Русский</span>
        </button>
        <button
          class="theme-btn"
          class:active={i18n.locale === "en"}
          onclick={() => {
            i18n.setLocale("en");
          }}
          aria-label="English"
        >
          <EnFlagIcon />
          <span>English</span>
        </button>
      </div>

      <SettingsPathField
        id="settings-library-path"
        label={i18n.t.settings.libraryPathLabel}
        placeholder={i18n.t.settings.libraryPathPlaceholder}
        bind:value={libraryPath}
        {tauriEnv}
        onbrowse={() => browseFolder("library")}
      />

      {#if tauriEnv}
        <SettingsPathField
          id="settings-save-folder"
          label={i18n.t.settings.defaultSaveFolderLabel}
          placeholder={i18n.t.settings.defaultSaveFolderPlaceholder}
          bind:value={defaultSaveFolder}
          {tauriEnv}
          onbrowse={() => browseFolder("save")}
        />
      {/if}

      {#if error}
        <p class="error" role="alert">{error}</p>
      {/if}

      <div class="actions">
        <button class="cancel-btn" onclick={handleCancel} disabled={saving}>
          {i18n.t.settings.cancelButton}
        </button>
        <button class="save-btn" onclick={handleSave} disabled={saving}>
          {i18n.t.settings.saveButton}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 1.5rem;
    width: 480px;
    max-width: 90vw;
    max-height: 85vh;
    overflow-y: auto;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
  }

  h2 {
    margin: 0 0 1.25rem;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .section-label {
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--color-text-secondary);
    margin-bottom: 0.5rem;
  }

  .theme-row {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .theme-btn {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.4rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg-input);
    color: var(--color-text);
    cursor: pointer;
    font-size: 0.85rem;
  }

  .theme-btn:hover {
    border-color: var(--color-primary);
  }

  .theme-btn.active {
    border-color: var(--color-primary);
    background: var(--color-primary-ring);
    color: var(--color-primary);
    font-weight: 600;
  }

  .error {
    color: var(--color-error);
    background: var(--color-bg-error);
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    font-size: 0.85rem;
    margin: 0;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1.25rem;
  }

  .cancel-btn {
    padding: 0.5rem 1rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg-input);
    color: var(--color-text);
    cursor: pointer;
    font-size: 0.9rem;
  }

  .cancel-btn:hover:not(:disabled) {
    border-color: var(--color-primary);
  }

  .save-btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 6px;
    background: var(--color-primary);
    color: white;
    cursor: pointer;
    font-size: 0.9rem;
  }

  .save-btn:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }

  .save-btn:disabled,
  .cancel-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
