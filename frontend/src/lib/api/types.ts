import type {
  ExtractedBook,
  FoundBook,
  LanguageCount,
  Settings,
} from "../types";

export interface SearchFilters {
  title?: string;
  authors?: string;
  genres?: string;
  date?: string;
  lang?: string;
  file_size?: string;
  sequence?: string;
}

export type IndexingPhase =
  | "Counting"
  | "Parsing"
  | "Writing"
  | "BuildingSearchIndex"
  | "CreatingFtsIndex";

export interface ProgressCallback {
  (phase: IndexingPhase, current: number, total: number): void;
}

export interface WarningCallback {
  (message: string): void;
}

export interface InfoCallback {
  (message: string): void;
}

export interface IndexState {
  archives_indexed: number;
  archives_pending: number;
  archives_new: number;
  search_index_valid: boolean;
  total_books: number;
}

export interface ArchiveInfo {
  name: string;
  status: string;
}

export interface Api {
  searchLibrary(
    path: string,
    query: string,
    filters: SearchFilters,
  ): Promise<FoundBook[]>;
  indexLibrary(
    path: string,
    mode: string,
    onProgress?: ProgressCallback,
    onWarning?: WarningCallback,
    onInfo?: InfoCallback,
    signal?: AbortSignal,
    archives?: string[],
  ): Promise<void>;
  listArchives(path: string): Promise<ArchiveInfo[]>;
  extractBooks(
    path: string,
    bookIds: number[],
    outputDir: string,
  ): Promise<ExtractedBook[]>;
  downloadBooksToClient(path: string, bookIds: number[]): Promise<void>;
  getBookCover(path: string, bookId: number): Promise<string | null>;
  getBookAnnotation(path: string, bookId: number): Promise<string | null>;
  getLanguages(path: string): Promise<LanguageCount[]>;
  getBookCount(path: string): Promise<number>;
  getIndexState(path: string): Promise<IndexState>;
  getSettings(): Promise<Settings>;
  saveSettings(settings: Settings): Promise<void>;
}
