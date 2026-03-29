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

export class WebAdapter implements Api {
  private baseUrl = "/api";

  async searchLibrary(
    path: string,
    query: string,
    filters: SearchFilters,
  ): Promise<FoundBook[]> {
    const params = new URLSearchParams({ path, query });
    if (filters.title) params.set("filter_title", filters.title);
    if (filters.authors) params.set("filter_authors", filters.authors);
    if (filters.genres) params.set("filter_genres", filters.genres);
    if (filters.date) params.set("filter_date", filters.date);
    if (filters.lang) params.set("filter_lang", filters.lang);
    if (filters.file_size) params.set("filter_file_size", filters.file_size);
    if (filters.sequence) params.set("filter_sequence", filters.sequence);
    const response = await fetch(`${this.baseUrl}/search?${params}`);
    if (!response.ok) throw new Error("Search failed");
    return (await response.json()) as FoundBook[];
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
    const response = await fetch(`${this.baseUrl}/index`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ path, mode, archives }),
      signal,
    });
    if (!response.ok) throw new Error("Indexing failed");

    const reader = response.body?.getReader();
    if (!reader) return;

    const decoder = new TextDecoder();
    let buffer = "";
    let currentEvent = "";

    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });
      const segments = buffer.split("\n");
      buffer = segments.pop() ?? "";

      for (const line of segments) {
        if (line.startsWith("event: ")) {
          currentEvent = line.slice(7).trim();
          continue;
        }

        if (line.startsWith("data: ")) {
          const payload = line.slice(6);

          if (currentEvent === "error") {
            throw new Error(payload || "Indexing error");
          }

          if (currentEvent === "warning" && onWarning) {
            onWarning(payload);
            currentEvent = "";
            continue;
          }

          if (currentEvent === "info" && onInfo) {
            onInfo(payload);
            currentEvent = "";
            continue;
          }

          if (onProgress) {
            try {
              const data: unknown = JSON.parse(payload);
              if (
                typeof data === "object" &&
                data !== null &&
                "phase" in data &&
                "current" in data &&
                "total" in data &&
                typeof (data as { current: unknown }).current === "number" &&
                typeof (data as { total: unknown }).total === "number"
              ) {
                const d = data as {
                  phase: IndexingPhase;
                  current: number;
                  total: number;
                };
                onProgress(d.phase, d.current, d.total);
              }
            } catch {
              // non-JSON data line, skip
            }
          }

          currentEvent = "";
        }
      }
    }
  }

  async extractBooks(
    path: string,
    bookIds: number[],
    outputDir: string,
  ): Promise<ExtractedBook[]> {
    const response = await fetch(`${this.baseUrl}/extract`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ path, book_ids: bookIds, output_dir: outputDir }),
    });
    if (!response.ok) throw new Error("Extraction failed");
    return (await response.json()) as ExtractedBook[];
  }

  async downloadBooksToClient(path: string, bookIds: number[]): Promise<void> {
    const response = await fetch(`${this.baseUrl}/download`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ path, book_ids: bookIds }),
    });
    if (!response.ok) throw new Error("Download failed");

    const disposition = response.headers.get("Content-Disposition");
    const utf8Match = disposition?.match(/filename\*=UTF-8''(.+)/);
    const plainMatch = disposition?.match(/filename="(.+)"/);
    const filename = utf8Match?.[1]
      ? decodeURIComponent(utf8Match[1])
      : (plainMatch?.[1] ?? "books.zip");

    const blob = await response.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  async getBookCover(path: string, bookId: number): Promise<string | null> {
    const params = new URLSearchParams({ path, book_id: String(bookId) });
    const response = await fetch(`${this.baseUrl}/cover?${params}`);
    if (response.status === 404) return null;
    if (!response.ok) return null;
    const blob = await response.blob();
    return new Promise((resolve) => {
      const reader = new FileReader();
      reader.onloadend = () => {
        resolve(reader.result as string);
      };
      reader.onerror = () => {
        resolve(null);
      };
      reader.readAsDataURL(blob);
    });
  }

  async getBookAnnotation(
    path: string,
    bookId: number,
  ): Promise<string | null> {
    const params = new URLSearchParams({ path, book_id: String(bookId) });
    const response = await fetch(`${this.baseUrl}/annotation?${params}`);
    if (!response.ok) return null;
    return (await response.json()) as string | null;
  }

  async getLanguages(path: string): Promise<LanguageCount[]> {
    const params = new URLSearchParams({ path });
    const response = await fetch(`${this.baseUrl}/languages?${params}`);
    if (!response.ok) return [];
    return (await response.json()) as LanguageCount[];
  }

  async getBookCount(path: string): Promise<number> {
    const params = new URLSearchParams({ path });
    const response = await fetch(`${this.baseUrl}/book-count?${params}`);
    if (!response.ok) return 0;
    return (await response.json()) as number;
  }

  async listArchives(path: string): Promise<ArchiveInfo[]> {
    const params = new URLSearchParams({ path });
    const response = await fetch(`${this.baseUrl}/archives?${params}`);
    if (!response.ok) throw new Error("Failed to list archives");
    return (await response.json()) as ArchiveInfo[];
  }

  async getIndexState(path: string): Promise<IndexState> {
    const params = new URLSearchParams({ path });
    const response = await fetch(`${this.baseUrl}/index-state?${params}`);
    if (!response.ok) throw new Error("Failed to get index state");
    return (await response.json()) as IndexState;
  }

  async getSettings(): Promise<Settings> {
    const response = await fetch(`${this.baseUrl}/settings`);
    if (!response.ok) throw new Error("Failed to load settings");
    return (await response.json()) as Settings;
  }

  async saveSettings(settings: Settings): Promise<void> {
    const response = await fetch(`${this.baseUrl}/settings`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(settings),
    });
    if (!response.ok) throw new Error("Failed to save settings");
  }
}
