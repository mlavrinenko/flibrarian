import { api, isTauri } from "./api";
import { settings } from "./settings";
import { appState } from "./state.svelte";
import { i18n } from "./i18n";
import { toasts } from "./toast";

export async function resolveSaveFolder(): Promise<string | null> {
  if (settings.defaultSaveFolder) return settings.defaultSaveFolder;

  if (isTauri()) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({ directory: true });
    if (!selected) return null;
    settings.defaultSaveFolder = selected;
    await settings.save();
    return selected;
  }

  appState.error = i18n.t.settings.saveFolderRequired;
  return null;
}

export async function downloadBooks(bookIds: number[]): Promise<boolean> {
  if (!appState.libraryPath) return false;

  if (!isTauri()) {
    await api.downloadBooksToClient(appState.libraryPath, bookIds);
    toasts.show(i18n.t.basket.downloadSuccess(bookIds.length), "success");
    return true;
  }

  const dir = await resolveSaveFolder();
  if (!dir) return false;

  await api.extractBooks(appState.libraryPath, bookIds, dir);
  toasts.show(i18n.t.basket.downloadSuccess(bookIds.length), "success");
  return true;
}
