import {
  api,
  type IndexingPhase,
  type IndexState,
  type SearchFilters,
} from "./api";
import { FilterHistory, type FilterSnapshot } from "./filterHistory.svelte";
import { logs } from "./logs";
import { settings } from "./settings";
import type { FoundBook, LanguageCount } from "./types";
import { errorMessage } from "./types";

export const FILTERABLE_COLUMNS = [
  "title",
  "authors",
  "genres",
  "date",
  "lang",
  "file_size",
  "sequence",
] as const;

export type FilterableColumn = (typeof FILTERABLE_COLUMNS)[number];

type ColumnFilters = Record<FilterableColumn, string>;

function emptyFilters(): ColumnFilters {
  return Object.fromEntries(
    FILTERABLE_COLUMNS.map((c) => [c, ""]),
  ) as ColumnFilters;
}

const DEBOUNCE_MS = 350;

class AppState {
  get libraryPath() {
    return settings.libraryPath;
  }
  set libraryPath(value: string) {
    settings.libraryPath = value;
  }

  searchQuery = $state("");
  searchResults: FoundBook[] = $state([]);
  loading = $state(false);
  error: string | null = $state(null);

  indexing = $state(false);
  indexingProgress = $state<{
    phase: IndexingPhase;
    current: number;
    total: number;
  }>({ phase: "Parsing", current: 0, total: 0 });

  bookCount: number = $state(0);
  availableLanguages: LanguageCount[] = $state([]);
  indexState: IndexState | null = $state(null);
  focusedIndex: number = $state(-1);

  detailBook: FoundBook | null = $state(null);

  columnFilters: ColumnFilters = $state(emptyFilters());

  private debounceTimer: ReturnType<typeof setTimeout> | null = null;
  private indexAbort: AbortController | null = null;
  private history = new FilterHistory({
    searchQuery: "",
    columnFilters: emptyFilters(),
  });
  private restoringFromHistory = false;

  get canUndo(): boolean {
    return this.history.canUndo;
  }

  get canRedo(): boolean {
    return this.history.canRedo;
  }

  constructor() {
    $effect.root(() => {
      $effect(() => {
        const query = this.searchQuery;
        const filters = { ...this.columnFilters };
        const hasFilters = Object.values(filters).some((v) => v.trim() !== "");

        if (!this.libraryPath) return;

        if (!query.trim() && !hasFilters) {
          this.searchResults = [];
          this.error = null;
          return;
        }

        this.scheduleSearch(query, this.libraryPath, filters);
      });

      $effect(() => {
        if (this.libraryPath) {
          void this.loadBookCount(this.libraryPath);
          void this.loadLanguages(this.libraryPath);
          void this.loadIndexState(this.libraryPath);
          void this.checkAndResumeIndexing(this.libraryPath);
        }
      });

      $effect(() => {
        const snapshot = {
          searchQuery: this.searchQuery,
          columnFilters: { ...this.columnFilters },
        };
        if (this.restoringFromHistory) return;
        this.history.schedulePush(snapshot);
      });
    });
  }

  private async checkAndResumeIndexing(libraryPath: string) {
    if (this.indexing) return;
    try {
      const state = await api.getIndexState(libraryPath);
      if (state.archives_pending > 0 || state.archives_new > 0) {
        logs.add({
          level: "info",
          source: "indexing",
          message: `Resuming indexing: ${state.archives_pending} interrupted, ${state.archives_new} new archives`,
        });
        await this.index("new");
      }
    } catch {
      // DB doesn't exist yet or other error — ignore
    }
  }

  private async loadBookCount(libraryPath: string) {
    try {
      this.bookCount = await api.getBookCount(libraryPath);
    } catch {
      this.bookCount = 0;
    }
  }

  private async loadLanguages(libraryPath: string) {
    try {
      this.availableLanguages = await api.getLanguages(libraryPath);
    } catch {
      this.availableLanguages = [];
    }
  }

  private async loadIndexState(libraryPath: string) {
    try {
      this.indexState = await api.getIndexState(libraryPath);
    } catch {
      this.indexState = null;
    }
  }

  private scheduleSearch(
    query: string,
    libraryPath: string,
    filters: ColumnFilters,
  ) {
    if (this.debounceTimer) clearTimeout(this.debounceTimer);
    this.debounceTimer = setTimeout(() => {
      void this.executeSearch(query, libraryPath, filters);
    }, DEBOUNCE_MS);
  }

  private async executeSearch(
    query: string,
    libraryPath: string,
    filters: ColumnFilters,
  ) {
    this.loading = true;
    this.error = null;
    this.focusedIndex = -1;

    const searchFilters: SearchFilters = {};
    for (const col of FILTERABLE_COLUMNS) {
      const v = filters[col].trim();
      if (!v) continue;
      if (col === "lang") {
        const parts = v.split("|").map((part) => (part === "\0" ? "" : part));
        searchFilters[col] =
          parts.length === 1 && parts[0] === "" ? "|" : parts.join("|");
      } else {
        searchFilters[col] = v;
      }
    }

    try {
      this.searchResults = await api.searchLibrary(
        libraryPath,
        query,
        searchFilters,
      );
    } catch (e) {
      this.error = errorMessage(e);
      this.searchResults = [];
    } finally {
      this.loading = false;
    }
  }

  setColumnFilter(column: FilterableColumn, value: string) {
    this.columnFilters = { ...this.columnFilters, [column]: value };
  }

  clearColumnFilter(column: FilterableColumn) {
    this.columnFilters = { ...this.columnFilters, [column]: "" };
  }

  clearAllFilters() {
    this.columnFilters = emptyFilters();
  }

  private restoreSnapshot(snapshot: FilterSnapshot | null) {
    if (!snapshot) return;
    this.restoringFromHistory = true;
    this.searchQuery = snapshot.searchQuery;
    this.columnFilters = { ...snapshot.columnFilters } as ColumnFilters;
    this.restoringFromHistory = false;
  }

  undo() {
    this.restoreSnapshot(this.history.undo());
  }

  redo() {
    this.restoreSnapshot(this.history.redo());
  }

  get selectedLanguages(): string[] {
    const raw = this.columnFilters.lang;
    if (!raw.trim()) return [];
    return raw.split("|");
  }

  toggleLanguage(lang: string) {
    const current = this.selectedLanguages;
    const idx = current.indexOf(lang);
    if (idx >= 0) {
      current.splice(idx, 1);
    } else {
      current.push(lang);
    }
    this.columnFilters = {
      ...this.columnFilters,
      lang: current.join("|"),
    };
  }

  filterByColumn(column: FilterableColumn, value: string) {
    this.searchQuery = "";
    if (column === "lang") {
      this.columnFilters = { ...emptyFilters(), lang: value };
    } else {
      this.columnFilters = { ...emptyFilters(), [column]: `=${value}` };
    }
  }

  addColumnFilter(column: FilterableColumn, value: string) {
    this.searchQuery = "";
    if (column === "lang") {
      const current = this.selectedLanguages;
      if (!current.includes(value)) {
        current.push(value);
      }
      this.columnFilters = {
        ...this.columnFilters,
        lang: current.join("|"),
      };
    } else {
      this.columnFilters = { ...this.columnFilters, [column]: `=${value}` };
    }
  }

  async index(mode: string, archives?: string[]) {
    if (!this.libraryPath) return;

    this.indexAbort = new AbortController();
    this.indexing = true;
    this.indexingProgress = { phase: "Parsing", current: 0, total: 0 };
    this.error = null;

    try {
      await api.indexLibrary(
        this.libraryPath,
        mode,
        (phase, current, total) => {
          this.indexingProgress = { phase, current, total };
        },
        (message) => {
          logs.add({ level: "warn", source: "indexing", message });
        },
        (message) => {
          logs.add({ level: "info", source: "indexing", message });
        },
        this.indexAbort.signal,
        archives,
      );
    } catch (e) {
      if (this.indexAbort.signal.aborted) return;
      this.error = errorMessage(e);
    } finally {
      this.indexAbort = null;
      this.indexing = false;
      if (this.libraryPath) {
        void this.loadBookCount(this.libraryPath);
        void this.loadLanguages(this.libraryPath);
        void this.loadIndexState(this.libraryPath);
      }
    }
  }

  cancelIndex() {
    this.indexAbort?.abort();
  }

  openBookDetail(book: FoundBook) {
    this.detailBook = book;
  }

  closeBookDetail() {
    this.detailBook = null;
  }
}

export const appState = new AppState();
