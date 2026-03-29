export interface Author {
  id: string;
  first_name: string | null;
  middle_name: string | null;
  last_name: string | null;
  nickname: string | null;
}

export interface Book {
  id: number;
  title: string;
  genres: string[];
  authors: Author[];
  date: string;
  lang: string;
  file_size: number;
  sequence: string;
}

export interface FoundBook extends Book {
  score: number;
}

export interface LanguageCount {
  lang: string;
  count: number;
}

export interface ExtractedBook {
  id: number;
  title: string;
  author: string;
  output_path: string;
}

export interface UiSettings {
  dock_height?: number;
  column_widths?: Record<string, number>;
}

export interface Settings {
  library_path: string | null;
  default_save_folder: string | null;
  ui?: UiSettings;
}

export function formatAuthor(author: Author, anonymous: string): string {
  const parts = [
    author.first_name,
    author.middle_name,
    author.last_name,
    author.nickname,
  ].filter((s): s is string => s !== null && s !== "");
  return parts.length > 0 ? parts.join(" ") : anonymous;
}

export function formatAuthors(authors: Author[], anonymous: string): string {
  return authors.map((a) => formatAuthor(a, anonymous)).join(", ");
}

export function formatFileSize(bytes: number): string {
  const KB = 1024;
  const MB = 1024 * 1024;
  if (bytes >= MB) return `${(bytes / MB).toFixed(1)} MB`;
  if (bytes >= KB) return `${(bytes / KB).toFixed(1)} KB`;
  return `${bytes} B`;
}

export function errorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}
