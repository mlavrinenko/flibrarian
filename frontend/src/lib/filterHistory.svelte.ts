export interface FilterSnapshot {
  searchQuery: string;
  columnFilters: Record<string, string>;
}

const MAX_HISTORY = 50;
const HISTORY_DEBOUNCE_MS = 600;

function snapshotsEqual(a: FilterSnapshot, b: FilterSnapshot): boolean {
  if (a.searchQuery !== b.searchQuery) return false;
  const keys = Object.keys(a.columnFilters);
  return keys.every((k) => a.columnFilters[k] === b.columnFilters[k]);
}

function snapshotEmpty(s: FilterSnapshot): boolean {
  return (
    !s.searchQuery.trim() &&
    Object.values(s.columnFilters).every((v) => !v.trim())
  );
}

export class FilterHistory {
  private stack: FilterSnapshot[] = $state([]);
  private index: number = $state(-1);
  private timer: ReturnType<typeof setTimeout> | null = null;

  constructor(initial: FilterSnapshot) {
    this.stack = [initial];
    this.index = 0;
  }

  get canUndo(): boolean {
    return this.index > 0;
  }

  get canRedo(): boolean {
    return this.index < this.stack.length - 1;
  }

  schedulePush(snapshot: FilterSnapshot) {
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      if (snapshotsEqual(this.stack[this.index], snapshot)) return;
      if (snapshotEmpty(snapshot)) return;

      this.stack.splice(this.index + 1);
      this.stack.push({
        searchQuery: snapshot.searchQuery,
        columnFilters: { ...snapshot.columnFilters },
      });
      if (this.stack.length > MAX_HISTORY) {
        this.stack.shift();
      }
      this.index = this.stack.length - 1;
    }, HISTORY_DEBOUNCE_MS);
  }

  undo(): FilterSnapshot | null {
    if (!this.canUndo) return null;
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }
    this.index--;
    return this.stack[this.index];
  }

  redo(): FilterSnapshot | null {
    if (!this.canRedo) return null;
    this.index++;
    return this.stack[this.index];
  }
}
