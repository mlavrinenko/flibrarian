<script lang="ts">
  import type { FoundBook } from "../lib/types";
  import { formatAuthor, formatFileSize } from "../lib/types";
  import { i18n } from "../lib/i18n";

  type FilterColumn = "authors" | "genres" | "date" | "lang" | "sequence";

  let {
    book,
    onfilterclick,
  }: {
    book: FoundBook;
    onfilterclick: (column: FilterColumn, value: string, e: MouseEvent) => void;
  } = $props();

  function handleKeydown(
    column: FilterColumn,
    value: string,
    e: KeyboardEvent,
  ) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onfilterclick(column, value, e as unknown as MouseEvent);
    }
  }
</script>

<div class="fields">
  <div class="field">
    <span class="label">{i18n.t.bookList.columns.authors}:</span>
    <span class="value"
      >{#each book.authors as author, i (author.id)}{#if i > 0}<span
            class="separator"
            >,
          </span>{/if}<span
          class="clickable"
          role="button"
          tabindex="-1"
          onclick={(e: MouseEvent) => {
            onfilterclick(
              "authors",
              formatAuthor(author, i18n.t.bookCard.anonymous),
              e,
            );
          }}
          onkeydown={(e: KeyboardEvent) => {
            handleKeydown(
              "authors",
              formatAuthor(author, i18n.t.bookCard.anonymous),
              e,
            );
          }}>{formatAuthor(author, i18n.t.bookCard.anonymous)}</span
        >{/each}</span
    >
  </div>

  <div class="field">
    <span class="label">{i18n.t.bookList.columns.genres}:</span>
    <span class="value"
      >{#each book.genres as genre, i (genre)}{#if i > 0}<span class="separator"
            >,
          </span>{/if}<span
          class="clickable"
          role="button"
          tabindex="-1"
          onclick={(e: MouseEvent) => {
            onfilterclick("genres", genre, e);
          }}
          onkeydown={(e: KeyboardEvent) => {
            handleKeydown("genres", genre, e);
          }}>{genre}</span
        >{/each}</span
    >
  </div>

  {#if book.date}
    <div class="field">
      <span class="label">{i18n.t.bookList.columns.date}:</span>
      <span class="value">
        <span
          class="clickable"
          role="button"
          tabindex="-1"
          onclick={(e: MouseEvent) => {
            onfilterclick("date", book.date, e);
          }}
          onkeydown={(e: KeyboardEvent) => {
            handleKeydown("date", book.date, e);
          }}>{book.date}</span
        >
      </span>
    </div>
  {/if}

  {#if book.sequence}
    <div class="field">
      <span class="label">{i18n.t.bookList.columns.sequence}:</span>
      <span class="value">
        <span
          class="clickable"
          role="button"
          tabindex="-1"
          onclick={(e: MouseEvent) => {
            onfilterclick("sequence", book.sequence, e);
          }}
          onkeydown={(e: KeyboardEvent) => {
            handleKeydown("sequence", book.sequence, e);
          }}>{book.sequence}</span
        >
      </span>
    </div>
  {/if}

  {#if book.lang}
    <div class="field">
      <span class="label">{i18n.t.bookList.columns.lang}:</span>
      <span class="value">
        <span
          class="clickable"
          role="button"
          tabindex="-1"
          onclick={(e: MouseEvent) => {
            onfilterclick("lang", book.lang, e);
          }}
          onkeydown={(e: KeyboardEvent) => {
            handleKeydown("lang", book.lang, e);
          }}>{book.lang}</span
        >
      </span>
    </div>
  {/if}

  <div class="field">
    <span class="label">{i18n.t.bookList.columns.fileSize}:</span>
    <span class="value">{formatFileSize(book.file_size)}</span>
  </div>

  <div class="field meta">
    <span>{i18n.t.bookDetail.id}: {book.id}</span>
    {#if book.score}
      <span class="separator">|</span>
      <span>{i18n.t.bookDetail.score}: {book.score.toFixed(2)}</span>
    {/if}
  </div>
</div>

<style>
  .fields {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem 1.5rem;
  }

  .field {
    display: flex;
    align-items: baseline;
    gap: 0.3rem;
    font-size: 0.85rem;
  }

  .label {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--color-text-secondary);
    white-space: nowrap;
  }

  .value {
    line-height: 1.4;
  }

  .separator {
    color: var(--color-text-tertiary);
  }

  .meta {
    color: var(--color-text-tertiary);
    font-size: 0.8rem;
  }
</style>
