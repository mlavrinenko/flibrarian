export type {
  Api,
  ArchiveInfo,
  IndexingPhase,
  IndexState,
  InfoCallback,
  ProgressCallback,
  SearchFilters,
  WarningCallback,
} from "./types";

import type { Api } from "./types";
import { TauriAdapter } from "./tauri";
import { WebAdapter } from "./web";

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export const api: Api = isTauri() ? new TauriAdapter() : new WebAdapter();

export async function confirmDialog(message: string): Promise<boolean> {
  if (isTauri()) {
    const { ask } = await import("@tauri-apps/plugin-dialog");
    return ask(message);
  }
  return confirm(message);
}
