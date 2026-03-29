const STORAGE_KEY = "flibrarian-logs";
const MAX_ENTRIES = 1000;

export type LogLevel = "info" | "warn" | "error";

export interface LogEntry {
  timestamp: number;
  level: LogLevel;
  source: "indexing" | "js" | "app";
  message: string;
}

const LEVEL_PRIORITY: Record<LogLevel, number> = {
  info: 0,
  warn: 1,
  error: 2,
};

export class LogsState {
  entries = $state<LogEntry[]>([]);
  open = $state(false);
  filter = $state("");
  minLevel = $state<LogLevel>("warn");
  unreadCount = $state(0);
  lastReadTimestamp = $state(Date.now());

  constructor() {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        this.entries = JSON.parse(stored) as LogEntry[];
      }
    } catch {
      // ignore
    }

    $effect.root(() => {
      $effect(() => {
        try {
          localStorage.setItem(STORAGE_KEY, JSON.stringify(this.entries));
        } catch {
          // ignore
        }
      });
    });
  }

  add(entry: Omit<LogEntry, "timestamp">) {
    this.entries = [...this.entries, { ...entry, timestamp: Date.now() }].slice(
      -MAX_ENTRIES,
    );

    if (!this.open && LEVEL_PRIORITY[entry.level] >= LEVEL_PRIORITY["warn"]) {
      this.unreadCount++;
    }
  }

  clear() {
    this.entries = [];
    this.unreadCount = 0;
  }

  markRead() {
    this.lastReadTimestamp = Date.now();
    this.unreadCount = 0;
  }

  get filtered(): LogEntry[] {
    const minPriority = LEVEL_PRIORITY[this.minLevel];
    let source = this.entries.filter(
      (e) => LEVEL_PRIORITY[e.level] >= minPriority,
    );
    if (this.filter) {
      const lower = this.filter.toLowerCase();
      source = source.filter(
        (e) =>
          e.message.toLowerCase().includes(lower) ||
          e.level.includes(lower) ||
          e.source.includes(lower),
      );
    }
    return [...source].reverse();
  }
}
