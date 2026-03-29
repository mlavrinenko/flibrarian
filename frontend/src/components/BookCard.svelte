<script lang="ts">
  import type { FoundBook } from "../lib/types";
  import { formatAuthor, formatFileSize, errorMessage } from "../lib/types";
  import { appState } from "../lib/state.svelte";
  import { basket } from "../lib/basket";
  import { downloadBooks } from "../lib/downloads";
  import { i18n } from "../lib/i18n";
  import BasketIcon from "./icons/BasketIcon.svelte";

  let { book, focused = false }: { book: FoundBook; focused?: boolean } =
    $props();

  function handleFilterClick(
    column: "authors" | "genres" | "date" | "lang" | "sequence",
    value: string,
    e: MouseEvent,
  ) {
    e.stopPropagation();
    if (e.ctrlKey || e.metaKey) {
      appState.addColumnFilter(column, value);
    } else {
      appState.filterByColumn(column, value);
    }
  }

  function handleFilterKeydown(
    column: "authors" | "genres" | "date" | "lang" | "sequence",
    value: string,
    e: KeyboardEvent,
  ) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      appState.filterByColumn(column, value);
    }
  }

  function handleRowClick() {
    if (appState.detailBook?.id === book.id) {
      appState.closeBookDetail();
    } else {
      appState.openBookDetail(book);
    }
  }

  let inBasket = $derived(basket.has(book.id));

  async function handleDownload() {
    try {
      await downloadBooks([book.id]);
    } catch (e) {
      appState.error = errorMessage(e);
    }
  }

  function handleBasketToggle(e: MouseEvent) {
    e.stopPropagation();
    if (e.ctrlKey || e.metaKey) {
      void handleDownload();
      return;
    }
    if (inBasket) {
      basket.remove(book.id);
    } else {
      basket.add(book);
    }
  }
</script>

<tr
  class:focused
  class:selected={appState.detailBook?.id === book.id}
  onclick={handleRowClick}
>
  <td class="basket-cell">
    <button
      class="basket-btn"
      class:in-basket={inBasket}
      onclick={handleBasketToggle}
      title={`${inBasket ? i18n.t.basket.removeFromBasket : i18n.t.basket.addToBasket}\n${i18n.t.basket.downloadHint}`}
      aria-label={inBasket
        ? i18n.t.basket.removeFromBasket
        : i18n.t.basket.addToBasket}
    >
      <BasketIcon filled={inBasket} />
    </button>
  </td>
  <td class="id">{book.id}</td>
  <td class="title">{book.title}</td>
  <td class="authors"
    >{#each book.authors as author, i (author.id)}{#if i > 0},
      {/if}<span
        class="clickable"
        role="button"
        tabindex="-1"
        onclick={(e) => {
          handleFilterClick(
            "authors",
            formatAuthor(author, i18n.t.bookCard.anonymous),
            e,
          );
        }}
        onkeydown={(e) => {
          handleFilterKeydown(
            "authors",
            formatAuthor(author, i18n.t.bookCard.anonymous),
            e,
          );
        }}>{formatAuthor(author, i18n.t.bookCard.anonymous)}</span
      >{/each}</td
  >
  <td class="genres"
    >{#each book.genres as genre, i (genre)}{#if i > 0},
      {/if}<span
        class="clickable"
        role="button"
        tabindex="-1"
        onclick={(e) => {
          handleFilterClick("genres", genre, e);
        }}
        onkeydown={(e) => {
          handleFilterKeydown("genres", genre, e);
        }}>{genre}</span
      >{/each}</td
  >
  <td class="date"
    >{#if book.date}<span
        class="clickable"
        role="button"
        tabindex="-1"
        onclick={(e) => {
          handleFilterClick("date", book.date, e);
        }}
        onkeydown={(e) => {
          handleFilterKeydown("date", book.date, e);
        }}>{book.date}</span
      >{/if}</td
  >
  <td class="lang"
    >{#if book.lang}<span
        class="clickable"
        role="button"
        tabindex="-1"
        onclick={(e) => {
          handleFilterClick("lang", book.lang, e);
        }}
        onkeydown={(e) => {
          handleFilterKeydown("lang", book.lang, e);
        }}>{book.lang}</span
      >{/if}</td
  >
  <td class="file-size">{formatFileSize(book.file_size)}</td>
  <td class="sequence"
    >{#if book.sequence}<span
        class="clickable"
        role="button"
        tabindex="-1"
        onclick={(e) => {
          handleFilterClick("sequence", book.sequence, e);
        }}
        onkeydown={(e) => {
          handleFilterKeydown("sequence", book.sequence, e);
        }}>{book.sequence}</span
      >{/if}</td
  >
  <td class="score"
    >{appState.searchQuery.trim() ? book.score.toFixed(2) : ""}</td
  >
</tr>

<style>
  tr {
    cursor: pointer;
  }

  tr:nth-child(even) {
    background: var(--color-bg-hover);
  }

  tr:hover {
    background: var(--color-bg-selected);
  }

  tr.focused {
    outline: 2px solid var(--color-primary);
    outline-offset: -2px;
  }

  tr.selected {
    background: var(--color-bg-selected);
  }

  td {
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--color-border-light);
    font-size: 0.85rem;
    vertical-align: top;
  }

  .id {
    color: var(--color-text-tertiary);
    white-space: nowrap;
  }

  .title {
    font-weight: 500;
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    overflow-wrap: break-word;
    word-break: break-word;
  }

  .authors {
    max-width: 250px;
    overflow: hidden;
    text-overflow: ellipsis;
    overflow-wrap: break-word;
    word-break: break-word;
  }

  .genres {
    max-width: 200px;
    color: var(--color-text-secondary);
    overflow-wrap: break-word;
    word-break: break-word;
  }

  .date {
    white-space: nowrap;
    color: var(--color-text-secondary);
  }

  .lang {
    white-space: nowrap;
    color: var(--color-text-secondary);
  }

  .file-size {
    white-space: nowrap;
    color: var(--color-text-secondary);
    text-align: right;
  }

  .sequence {
    max-width: 200px;
    color: var(--color-text-secondary);
  }

  .score {
    white-space: nowrap;
    color: var(--color-text-tertiary);
    text-align: right;
  }

  .basket-cell {
    width: 40px;
    text-align: center;
  }

  .basket-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.2rem;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--color-text-tertiary);
    cursor: pointer;
  }

  .basket-btn:hover {
    color: var(--color-primary);
    background: var(--color-bg-hover);
  }

  .basket-btn.in-basket {
    color: var(--color-primary);
  }
</style>
