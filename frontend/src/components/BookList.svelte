<script lang="ts">
  import type { FoundBook } from "../lib/types";
  import {
    appState,
    FILTERABLE_COLUMNS,
    type FilterableColumn,
  } from "../lib/state.svelte";
  import {
    columnResize,
    type ResizableColumn,
  } from "../lib/columnResize.svelte";
  import { i18n } from "../lib/i18n";
  import BookCard from "./BookCard.svelte";
  import LangFilter from "./LangFilter.svelte";
  import CloseIcon from "./icons/CloseIcon.svelte";

  let { books }: { books: FoundBook[] } = $props();

  columnResize.init();

  let tableRef: HTMLTableElement | undefined = $state();

  const COL_LABELS: Record<ResizableColumn, () => string> = {
    title: () => i18n.t.bookList.columns.title,
    authors: () => i18n.t.bookList.columns.authors,
    genres: () => i18n.t.bookList.columns.genres,
    date: () => i18n.t.bookList.columns.date,
    lang: () => i18n.t.bookList.columns.lang,
    file_size: () => i18n.t.bookList.columns.fileSize,
    sequence: () => i18n.t.bookList.columns.sequence,
  };

  const RESIZABLE_COLS = Object.keys(COL_LABELS) as ResizableColumn[];

  function handleFilterInput(column: FilterableColumn, e: Event) {
    const value = (e.target as HTMLInputElement).value;
    appState.setColumnFilter(column, value);
  }

  function scrollFocusedIntoView() {
    if (!tableRef || appState.focusedIndex < 0) return;
    const rows = tableRef.querySelectorAll("tbody tr");
    const row = rows[appState.focusedIndex] as Element | undefined;
    row?.scrollIntoView({ block: "nearest" });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (books.length === 0) return;

    switch (e.key) {
      case "ArrowDown":
      case "j": {
        e.preventDefault();
        appState.focusedIndex = Math.min(
          appState.focusedIndex + 1,
          books.length - 1,
        );
        scrollFocusedIntoView();
        break;
      }
      case "ArrowUp":
      case "k": {
        e.preventDefault();
        appState.focusedIndex = Math.max(appState.focusedIndex - 1, 0);
        scrollFocusedIntoView();
        break;
      }
      case "Enter": {
        e.preventDefault();
        if (
          appState.focusedIndex >= 0 &&
          appState.focusedIndex < books.length
        ) {
          appState.openBookDetail(books[appState.focusedIndex]);
        }
        break;
      }
      case "Home": {
        e.preventDefault();
        appState.focusedIndex = 0;
        scrollFocusedIntoView();
        break;
      }
      case "End": {
        e.preventDefault();
        appState.focusedIndex = books.length - 1;
        scrollFocusedIntoView();
        break;
      }
    }
  }
</script>

<div class="table-wrapper">
  <table
    bind:this={tableRef}
    tabindex="0"
    role="grid"
    aria-label={i18n.t.bookList.ariaLabel}
    onkeydown={handleKeydown}
  >
    <thead>
      <tr>
        <th class="basket-col"></th>
        <th class="fixed-col">{i18n.t.bookList.columns.id}</th>
        {#each RESIZABLE_COLS as col (col)}
          <th style={columnResize.style(col)}
            >{COL_LABELS[col]()}<button
              class="resize-handle"
              aria-label={i18n.t.bookList.filter.resizeColumn}
              tabindex="-1"
              onmousedown={(e) => {
                columnResize.start(col, e);
              }}
            ></button></th
          >
        {/each}
        <th class="fixed-col">{i18n.t.bookList.columns.score}</th>
      </tr>
      <tr class="filter-row">
        <th></th>
        <th></th>
        {#each FILTERABLE_COLUMNS as col (col)}
          <th>
            {#if col === "lang"}
              <LangFilter />
            {:else}
              <div class="filter-cell">
                <input
                  type="text"
                  class="filter-input"
                  class:active={appState.columnFilters[col].trim() !== ""}
                  placeholder={i18n.t.bookList.filter.placeholder}
                  value={appState.columnFilters[col]}
                  oninput={(e: Event) => {
                    handleFilterInput(col, e);
                  }}
                />
                {#if appState.columnFilters[col].trim() !== ""}
                  <button
                    class="clear-btn"
                    onclick={() => {
                      appState.clearColumnFilter(col);
                    }}
                    aria-label={i18n.t.bookList.filter.clearFilter}
                  >
                    <CloseIcon size={12} />
                  </button>
                {/if}
              </div>
            {/if}
          </th>
        {/each}
        <th></th>
      </tr>
    </thead>
    <tbody>
      {#each books as book, i (book.id)}
        <BookCard {book} focused={appState.focusedIndex === i} />
      {/each}
    </tbody>
  </table>
  {#if books.length === 0 && !appState.loading && (appState.searchQuery.trim() || Object.values(appState.columnFilters).some( (v) => v.trim(), ))}
    <p class="no-results">{i18n.t.bookList.noResults}</p>
  {/if}
</div>

<style>
  .table-wrapper {
    overflow-x: clip;
  }

  table {
    width: 100%;
    border-collapse: separate;
    border-spacing: 0;
    outline: none;
    table-layout: auto;
  }

  table:focus-visible {
    box-shadow: 0 0 0 2px var(--color-primary-focus);
    border-radius: 4px;
  }

  thead th {
    padding: 0.35rem 0.75rem;
    text-align: left;
    font-size: 0.8rem;
    line-height: 1.2;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--color-text-secondary);
    border-bottom: none;
    box-shadow: inset 0 -1px 0 var(--color-border-strong);
    white-space: nowrap;
    position: sticky;
    top: 0;
    background: var(--color-bg);
    z-index: 3;
    overflow: hidden;
  }

  thead th:last-child {
    text-align: right;
  }

  .basket-col {
    width: 40px;
  }

  .fixed-col {
    width: 1px;
    white-space: nowrap;
  }

  .filter-row th {
    padding: 0.25rem 0.75rem;
    border-bottom: none;
    box-shadow:
      inset 0 -1px 0 var(--color-border),
      0 -2px 0 0 var(--color-bg);
    position: sticky;
    top: 1.66rem;
    background: var(--color-bg);
    z-index: 2;
    overflow: visible;
  }

  .filter-cell {
    position: relative;
    display: flex;
    align-items: center;
  }

  .filter-input {
    width: 100%;
    padding: 0.2rem 1.4rem 0.2rem 0.4rem;
    border: 1px solid var(--color-border);
    border-radius: 3px;
    font-size: 0.75rem;
    background: var(--color-bg-input);
    color: var(--color-text);
    box-sizing: border-box;
  }

  .filter-input.active {
    border-color: var(--color-primary);
    background: var(--color-bg-selected);
  }

  .filter-input:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 1px var(--color-primary-ring);
  }

  .clear-btn {
    position: absolute;
    right: 2px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1px;
    border: none;
    border-radius: 2px;
    background: transparent;
    color: var(--color-text-tertiary);
    cursor: pointer;
    line-height: 0;
  }

  .clear-btn:hover {
    color: var(--color-text);
    background: var(--color-bg-hover);
  }

  .resize-handle {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    width: 4px;
    padding: 0;
    border: none;
    border-radius: 0;
    background: transparent;
    cursor: col-resize;
    user-select: none;
  }

  .resize-handle:hover {
    background: var(--color-primary);
    opacity: 0.4;
  }

  .resize-handle:focus {
    outline: none;
  }

  .no-results {
    text-align: center;
    color: var(--color-text-tertiary);
    font-size: 0.9rem;
    padding: 2rem 0;
    margin: 0;
  }
</style>
