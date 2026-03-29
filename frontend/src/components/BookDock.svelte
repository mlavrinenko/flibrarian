<script lang="ts">
  import { errorMessage } from "../lib/types";
  import { appState } from "../lib/state.svelte";
  import { basket } from "../lib/basket";
  import { api } from "../lib/api";
  import { downloadBooks } from "../lib/downloads";
  import { i18n } from "../lib/i18n";
  import { settings } from "../lib/settings";
  import BasketIcon from "./icons/BasketIcon.svelte";
  import DownloadIcon from "./icons/DownloadIcon.svelte";
  import BookDockFields from "./BookDockFields.svelte";
  import BookDockAnnotation from "./BookDockAnnotation.svelte";

  const MIN_HEIGHT = 100;
  const MAX_HEIGHT_VH = 70;

  let book = $derived(appState.detailBook);
  let inBasket = $derived(book ? basket.has(book.id) : false);
  let coverUrl: string | null = $state(null);
  let coverLoading = $state(false);
  let annotation: string | null = $state(null);
  let dockHeight = $derived(settings.dockHeight);
  let resizing = $state(false);

  $effect(() => {
    dockHeight = settings.dockHeight;
  });

  $effect(() => {
    const currentBook = book;
    if (!currentBook || !appState.libraryPath) {
      coverUrl = null;
      coverLoading = false;
      annotation = null;
      return;
    }
    coverLoading = true;
    coverUrl = null;
    annotation = null;
    const libraryPath = appState.libraryPath;

    api
      .getBookCover(libraryPath, currentBook.id)
      .then((url) => {
        if (appState.detailBook?.id === currentBook.id) {
          coverUrl = url;
        }
      })
      .catch(() => {
        if (appState.detailBook?.id === currentBook.id) {
          coverUrl = null;
        }
      })
      .finally(() => {
        if (appState.detailBook?.id === currentBook.id) {
          coverLoading = false;
        }
      });

    api
      .getBookAnnotation(libraryPath, currentBook.id)
      .then((text) => {
        if (appState.detailBook?.id === currentBook.id) {
          annotation = text;
        }
      })
      .catch(() => {
        if (appState.detailBook?.id === currentBook.id) {
          annotation = null;
        }
      });
  });

  function close() {
    appState.closeBookDetail();
  }

  function handleFilterClick(
    column: "authors" | "genres" | "date" | "lang" | "sequence",
    value: string,
    e: MouseEvent,
  ) {
    close();
    if (e.ctrlKey || e.metaKey) {
      appState.addColumnFilter(column, value);
    } else {
      appState.filterByColumn(column, value);
    }
  }

  function handleBasketToggle() {
    if (!book) return;
    if (inBasket) {
      basket.remove(book.id);
    } else {
      basket.add(book);
    }
  }

  async function handleDownload() {
    if (!book) return;
    try {
      await downloadBooks([book.id]);
    } catch (e) {
      appState.error = errorMessage(e);
    }
  }

  function maxHeight(): number {
    return Math.floor((window.innerHeight * MAX_HEIGHT_VH) / 100);
  }

  function clampHeight(h: number): number {
    return Math.max(MIN_HEIGHT, Math.min(h, maxHeight()));
  }

  function handleResizeStart(e: MouseEvent) {
    e.preventDefault();
    resizing = true;
    const startY = e.clientY;
    const startHeight = dockHeight;

    function onMouseMove(ev: MouseEvent) {
      dockHeight = clampHeight(startHeight + (startY - ev.clientY));
    }

    function onMouseUp() {
      resizing = false;
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
      settings.dockHeight = dockHeight;
      void settings.save();
    }

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
  }
</script>

{#if book}
  <div
    class="dock"
    class:resizing
    role="complementary"
    aria-label={book.title}
    style="height: {dockHeight}px"
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="resize-handle"
      role="separator"
      aria-orientation="horizontal"
      onmousedown={handleResizeStart}
    ></div>
    <div class="dock-content">
      {#if coverLoading}
        <div class="cover-placeholder"></div>
      {:else if coverUrl}
        <div class="cover">
          <img src={coverUrl} alt={book.title} />
        </div>
      {/if}

      <div class="details">
        <div class="title-row">
          <h3>{book.title}</h3>
          <div class="actions">
            <button
              class="action-btn"
              class:in-basket={inBasket}
              onclick={handleBasketToggle}
              title={inBasket
                ? i18n.t.basket.removeFromBasket
                : i18n.t.basket.addToBasket}
              aria-label={inBasket
                ? i18n.t.basket.removeFromBasket
                : i18n.t.basket.addToBasket}
            >
              <BasketIcon filled={inBasket} />
            </button>
            <button
              class="action-btn"
              onclick={() => void handleDownload()}
              title={i18n.t.basket.downloadBook}
              aria-label={i18n.t.basket.downloadBook}
            >
              <DownloadIcon />
            </button>
            <button
              class="modal-close-btn"
              onclick={close}
              aria-label={i18n.t.close}
            >
              &times;
            </button>
          </div>
        </div>

        <BookDockFields {book} onfilterclick={handleFilterClick} />

        {#if annotation}
          <BookDockAnnotation {annotation} />
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .dock {
    border-top: 2px solid var(--color-border-strong);
    background: var(--color-bg-header);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .resize-handle {
    height: 4px;
    cursor: ns-resize;
    flex-shrink: 0;
  }

  .resize-handle:hover {
    background: var(--color-primary);
    opacity: 0.4;
  }

  .dock.resizing .resize-handle {
    background: var(--color-primary);
    opacity: 0.6;
  }

  .dock-content {
    display: flex;
    gap: 1rem;
    align-items: flex-start;
    padding: 0.5rem 1rem 0.75rem;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .cover {
    flex-shrink: 0;
  }

  .cover img {
    max-width: 120px;
    max-height: 180px;
    object-fit: contain;
    border-radius: 4px;
    border: 1px solid var(--color-border-light);
  }

  .cover-placeholder {
    width: 120px;
    height: 160px;
    flex-shrink: 0;
    border-radius: 4px;
    background: var(--color-bg-hover);
  }

  .details {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .title-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 0.75rem;
  }

  h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    line-height: 1.3;
    word-break: break-word;
  }

  .actions {
    display: flex;
    gap: 0.25rem;
    align-items: center;
    flex-shrink: 0;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--color-text-tertiary);
    cursor: pointer;
  }

  .action-btn:hover {
    color: var(--color-primary);
    background: var(--color-bg-hover);
  }

  .action-btn.in-basket {
    color: var(--color-primary);
  }
</style>
