import { api } from "../api";
import type { Settings } from "../types";

const DEFAULT_DOCK_HEIGHT = 250;

class SettingsState {
  libraryPath = $state("");
  defaultSaveFolder = $state("");
  dockHeight = $state(DEFAULT_DOCK_HEIGHT);
  columnWidths = $state<Record<string, number>>({});
  open = $state(false);
  loaded = $state(false);

  async load() {
    try {
      const s = await api.getSettings();
      this.libraryPath = s.library_path ?? "";
      this.defaultSaveFolder = s.default_save_folder ?? "";
      this.dockHeight = s.ui?.dock_height ?? DEFAULT_DOCK_HEIGHT;
      this.columnWidths = s.ui?.column_widths ?? {};
    } catch {
      // keep defaults on error
    }
    this.loaded = true;
  }

  async save() {
    const s: Settings = {
      library_path: this.libraryPath || null,
      default_save_folder: this.defaultSaveFolder || null,
      ui: {
        dock_height: this.dockHeight,
        column_widths:
          Object.keys(this.columnWidths).length > 0
            ? this.columnWidths
            : undefined,
      },
    };
    await api.saveSettings(s);
  }
}

export const settings = new SettingsState();
