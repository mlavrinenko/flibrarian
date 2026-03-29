import type { FoundBook } from "../types";

const STORAGE_KEY = "basket";

function loadBasket(): FoundBook[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? (JSON.parse(raw) as FoundBook[]) : [];
  } catch {
    return [];
  }
}

function saveBasket(books: FoundBook[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(books));
  } catch {
    // storage full or unavailable — ignore
  }
}

class BasketState {
  books: FoundBook[] = $state(loadBasket());
  open = $state(false);

  get count(): number {
    return this.books.length;
  }

  has(bookId: number): boolean {
    return this.books.some((b) => b.id === bookId);
  }

  add(book: FoundBook) {
    if (this.has(book.id)) return;
    this.books = [...this.books, book];
    saveBasket(this.books);
  }

  addMany(newBooks: FoundBook[]) {
    const existing = new Set(this.books.map((b) => b.id));
    const toAdd = newBooks.filter((b) => !existing.has(b.id));
    if (toAdd.length === 0) return;
    this.books = [...this.books, ...toAdd];
    saveBasket(this.books);
  }

  remove(bookId: number) {
    this.books = this.books.filter((b) => b.id !== bookId);
    saveBasket(this.books);
  }

  clear() {
    this.books = [];
    saveBasket(this.books);
  }

  bookIds(): number[] {
    return this.books.map((b) => b.id);
  }
}

export const basket = new BasketState();
