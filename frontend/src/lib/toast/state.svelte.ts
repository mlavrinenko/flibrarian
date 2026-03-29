export type ToastLevel = "success" | "error" | "info";

export interface Toast {
  id: number;
  message: string;
  level: ToastLevel;
}

const DURATION_MS = 4000;

let nextId = 0;

class ToastState {
  items: Toast[] = $state([]);

  show(message: string, level: ToastLevel = "info") {
    const id = nextId++;
    this.items = [...this.items, { id, message, level }];
    setTimeout(() => {
      this.dismiss(id);
    }, DURATION_MS);
  }

  dismiss(id: number) {
    this.items = this.items.filter((t) => t.id !== id);
  }
}

export const toasts = new ToastState();
