import { settings } from "./settings/state.svelte";

const MIN_COL_WIDTH = 60;

const RESIZABLE_COLUMNS = [
  "title",
  "authors",
  "genres",
  "date",
  "lang",
  "file_size",
  "sequence",
] as const;

export type ResizableColumn = (typeof RESIZABLE_COLUMNS)[number];

class ColumnResizeState {
  widths = $state<Record<string, number | null>>({});
  #resizing: { col: string; startX: number; startWidth: number } | null = null;
  #initialized = false;

  init() {
    if (this.#initialized) return;
    this.#initialized = true;
    $effect(() => {
      const saved = settings.columnWidths;
      const w: Record<string, number | null> = {};
      for (const col of RESIZABLE_COLUMNS) {
        w[col] = saved[col] ?? null;
      }
      this.widths = w;
    });
  }

  style(col: string): string | undefined {
    const w = this.widths[col];
    return w ? `width: ${w}px; min-width: ${w}px` : undefined;
  }

  start(col: string, e: MouseEvent) {
    e.preventDefault();
    const th = (e.target as HTMLElement).parentElement;
    if (!th) return;
    const startWidth = Math.round(th.getBoundingClientRect().width);
    this.#resizing = { col, startX: e.clientX, startWidth };

    const onMouseMove = (ev: MouseEvent) => {
      if (!this.#resizing) return;
      const delta = Math.round(ev.clientX - this.#resizing.startX);
      this.widths[this.#resizing.col] = Math.max(
        MIN_COL_WIDTH,
        this.#resizing.startWidth + delta,
      );
    };

    const onMouseUp = () => {
      this.#resizing = null;
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
      this.#save();
    };

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
  }

  #save() {
    const toSave: Record<string, number> = {};
    for (const [col, w] of Object.entries(this.widths)) {
      if (w != null) toSave[col] = w;
    }
    settings.columnWidths = toSave;
    void settings.save();
  }
}

export const columnResize = new ColumnResizeState();
