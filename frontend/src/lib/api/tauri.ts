import type {
  ExtractedBook,
  FoundBook,
  LanguageCount,
  Settings,
} from "../types";
import type {
  Api,
  ArchiveInfo,
  IndexingPhase,
  IndexState,
  InfoCallback,
  ProgressCallback,
  SearchFilters,
  WarningCallback,
} from "./types";

export class TauriAdapter implements Api {
  async searchLibrary(
    path: string,
    query: string,
    filters: SearchFilters,
  ): Promise<FoundBook[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<FoundBook[]>("search_library", { path, query, filters });
  }

  async indexLibrary(
    path: string,
    mode: string,
    onProgress?: ProgressCallback,
    onWarning?: WarningCallback,
    onInfo?: InfoCallback,
    signal?: AbortSignal,
    archives?: string[],
  ): Promise<void> {
    const { invoke } = await import("@tauri-apps/api/core");
    const { listen } = await import("@tauri-apps/api/event");

    const unlisteners: (() => void)[] = [];

    if (signal) {
      const onAbort = () => {
        void invoke("cancel_indexing");
      };
      signal.addEventListener("abort", onAbort, { once: true });
      unlisteners.push(() => {
        signal.removeEventListener("abort", onAbort);
      });
    }

    if (onProgress) {
      unlisteners.push(
        await listen<{
          phase: IndexingPhase;
          current: number;
          total: number;
        }>("indexing-progress", (event) => {
          onProgress(
            event.payload.phase,
            event.payload.current,
            event.payload.total,
          );
        }),
      );
    }

    if (onWarning) {
      unlisteners.push(
        await listen<{ message: string }>("indexing-warning", (event) => {
          onWarning(event.payload.message);
        }),
      );
    }

    if (onInfo) {
      unlisteners.push(
        await listen<{ message: string }>("indexing-info", (event) => {
          onInfo(event.payload.message);
        }),
      );
    }

    try {
      await invoke("index_library", { path, mode, archives });
    } finally {
      for (const fn of unlisteners) fn();
    }
    signal?.throwIfAborted();
  }

  async extractBooks(
    path: string,
    bookIds: number[],
    outputDir: string,
  ): Promise<ExtractedBook[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<ExtractedBook[]>("extract_books", {
      path,
      bookIds,
      outputDir,
    });
  }

  downloadBooksToClient(_path: string, _bookIds: number[]): Promise<void> {
    return Promise.reject(
      new Error("Browser download not supported in Tauri — use extractBooks"),
    );
  }

  async getBookCover(path: string, bookId: number): Promise<string | null> {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await invoke<{
      data: string;
      content_type: string;
    } | null>("get_book_cover", { path, bookId });
    if (!result) return null;
    return `data:${result.content_type};base64,${result.data}`;
  }

  async getBookAnnotation(
    path: string,
    bookId: number,
  ): Promise<string | null> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<string | null>("get_book_annotation", { path, bookId });
  }

  async getLanguages(path: string): Promise<LanguageCount[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<LanguageCount[]>("get_languages", { path });
  }

  async getBookCount(path: string): Promise<number> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<number>("get_book_count", { path });
  }

  async listArchives(path: string): Promise<ArchiveInfo[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<ArchiveInfo[]>("list_archives", { path });
  }

  async getIndexState(path: string): Promise<IndexState> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<IndexState>("check_index_state", { path });
  }

  async getSettings(): Promise<Settings> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<Settings>("get_settings");
  }

  async saveSettings(settings: Settings): Promise<void> {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("save_settings", { settings });
  }
}
