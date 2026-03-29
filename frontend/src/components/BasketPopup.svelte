<script lang="ts">
  import type { FoundBook } from "../lib/types";
  import { formatAuthors, errorMessage } from "../lib/types";
  import { basket } from "../lib/basket";
  import { appState } from "../lib/state.svelte";
  import { i18n } from "../lib/i18n";
  import { downloadBooks } from "../lib/downloads";

  let downloading = $state(false);

  function handleBookClick(book: FoundBook) {
    basket.open = false;
    appState.openBookDetail(book);
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      basket.open = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      basket.open = false;
    }
  }

  async function downloadAll() {
    if (basket.count === 0) return;

    downloading = true;
    appState.error = null;

    try {
      const ok = await downloadBooks(basket.bookIds());
      if (ok) {
        basket.clear();
        basket.open = false;
      }
    } catch (e) {
      appState.error = errorMessage(e);
    } finally {
      downloading = false;
    }
  }
</script>

{#if basket.open}
  <div
    class="backdrop basket-backdrop"
    role="dialog"
    aria-modal="true"
    aria-label={i18n.t.basket.title}
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
    tabindex="-1"
  >
    <div class="popup">
      <div class="header">
        <h3>{i18n.t.basket.title}</h3>
        <div class="header-actions">
          {#if basket.count > 0}
            <button
              class="clear-btn"
              onclick={() => {
                basket.clear();
              }}
            >
              {i18n.t.basket.clearBasket}
            </button>
          {/if}
          <button
            class="modal-close-btn"
            onclick={() => (basket.open = false)}
            aria-label={i18n.t.close}
          >
            &times;
          </button>
        </div>
      </div>

      {#if basket.count === 0}
        <p class="empty">{i18n.t.basket.empty}</p>
      {:else}
        <ul class="book-list">
          {#each basket.books as book (book.id)}
            <li>
              <div class="book-info">
                <span
                  class="book-title clickable"
                  role="button"
                  tabindex="-1"
                  onclick={() => {
                    handleBookClick(book);
                  }}
                  onkeydown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      handleBookClick(book);
                    }
                  }}>{book.title}</span
                >
                <span class="book-authors"
                  >{formatAuthors(
                    book.authors,
                    i18n.t.bookCard.anonymous,
                  )}</span
                >
              </div>
              <button
                class="remove-btn"
                onclick={() => {
                  basket.remove(book.id);
                }}
                title={i18n.t.basket.removeFromBasket}
                aria-label={i18n.t.basket.removeFromBasket}
              >
                &times;
              </button>
            </li>
          {/each}
        </ul>

        <div class="footer">
          <button
            class="download-btn"
            onclick={() => downloadAll()}
            disabled={downloading || !appState.libraryPath}
          >
            {downloading
              ? i18n.t.basket.downloading
              : i18n.t.basket.downloadAll}
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  :global(.backdrop).basket-backdrop {
    align-items: flex-start;
    padding-top: 4rem;
  }

  .popup {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    width: 500px;
    max-width: 90vw;
    max-height: 70vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--color-border-light);
  }

  h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .header-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .clear-btn {
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--color-error);
    border-radius: 4px;
    background: transparent;
    color: var(--color-error);
    font-size: 0.75rem;
    cursor: pointer;
  }

  .clear-btn:hover {
    background: var(--color-error);
    color: white;
  }

  .empty {
    text-align: center;
    color: var(--color-text-tertiary);
    padding: 2rem 1rem;
    margin: 0;
  }

  .book-list {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    flex: 1;
  }

  .book-list li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid var(--color-border-light);
    gap: 0.5rem;
  }

  .book-list li:last-child {
    border-bottom: none;
  }

  .book-list li:hover {
    background: var(--color-bg-hover);
  }

  .book-info {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
    flex: 1;
  }

  .book-title {
    font-size: 0.85rem;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .book-authors {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .remove-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--color-text-tertiary);
    font-size: 1.1rem;
    cursor: pointer;
    flex-shrink: 0;
    line-height: 1;
  }

  .remove-btn:hover {
    color: var(--color-error);
    background: var(--color-bg-error);
  }

  .footer {
    padding: 0.75rem 1rem;
    border-top: 1px solid var(--color-border-light);
    display: flex;
    justify-content: flex-end;
  }

  .download-btn {
    padding: 0.5rem 1.25rem;
    background: var(--color-success);
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 0.85rem;
    cursor: pointer;
    white-space: nowrap;
  }

  .download-btn:hover:not(:disabled) {
    background: var(--color-success-hover);
  }

  .download-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
