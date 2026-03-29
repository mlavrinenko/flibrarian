<script lang="ts">
  import SearchBar from "./components/SearchBar.svelte";
  import IndexingPanel from "./components/IndexingPanel.svelte";
  import BookList from "./components/BookList.svelte";
  import BookDock from "./components/BookDock.svelte";
  import SettingsPanel from "./components/SettingsPanel.svelte";
  import BasketPopup from "./components/BasketPopup.svelte";
  import BasketIcon from "./components/icons/BasketIcon.svelte";
  import SettingsIcon from "./components/icons/SettingsIcon.svelte";
  import LogsIcon from "./components/icons/LogsIcon.svelte";
  import UndoIcon from "./components/icons/UndoIcon.svelte";
  import RedoIcon from "./components/icons/RedoIcon.svelte";
  import LogsPanel from "./components/LogsPanel.svelte";
  import ToastContainer from "./components/ToastContainer.svelte";
  import { appState } from "./lib/state.svelte";
  import { basket } from "./lib/basket";
  import { logs } from "./lib/logs";
  import { i18n } from "./lib/i18n";
  import { settings } from "./lib/settings";
  import { isTauri, confirmDialog } from "./lib/api";

  void settings.load();

  if (isTauri()) {
    void import("@tauri-apps/api/window").then(({ getCurrentWindow }) => {
      void getCurrentWindow().onCloseRequested(async (event) => {
        if (
          appState.indexing &&
          !(await confirmDialog(i18n.t.indexing.closeConfirm))
        ) {
          event.preventDefault();
          return;
        }
        appState.cancelIndex();
      });
    });
  }

  function handleBeforeUnload(e: BeforeUnloadEvent) {
    if (appState.indexing) {
      e.preventDefault();
    }
  }

  function isInputFocused(): boolean {
    const tag = document.activeElement?.tagName;
    return tag === "INPUT" || tag === "SELECT" || tag === "TEXTAREA";
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
    const mod = e.ctrlKey || e.metaKey;
    if (mod && e.key === "z" && !e.shiftKey) {
      e.preventDefault();
      appState.undo();
      return;
    }
    if (mod && (e.key === "Z" || (e.key === "z" && e.shiftKey))) {
      e.preventDefault();
      appState.redo();
      return;
    }
    if (e.key === "/" && !isInputFocused()) {
      e.preventDefault();
      document.getElementById("search-query")?.focus();
    }
    if (e.key === "Escape") {
      if (appState.detailBook) {
        appState.closeBookDetail();
      } else {
        if (document.activeElement instanceof HTMLElement) {
          document.activeElement.blur();
        }
        appState.focusedIndex = -1;
      }
    }
  }
</script>

<svelte:window
  onkeydown={handleGlobalKeydown}
  onbeforeunload={handleBeforeUnload}
/>

<div class="app-layout">
  <header>
    <div class="header-title">
      <h1>flibrarian</h1>
      {#if appState.bookCount > 0}
        <span class="book-count"
          >{i18n.t.header.keepsBooks(appState.bookCount)}</span
        >
      {/if}
    </div>
    <div class="header-controls">
      <div class="undo-redo">
        <button
          class="icon-btn"
          onclick={() => {
            appState.undo();
          }}
          disabled={!appState.canUndo}
          aria-label={i18n.t.header.undo}
          title={i18n.t.header.undo}
        >
          <UndoIcon size={16} />
        </button>
        <button
          class="icon-btn"
          onclick={() => {
            appState.redo();
          }}
          disabled={!appState.canRedo}
          aria-label={i18n.t.header.redo}
          title={i18n.t.header.redo}
        >
          <RedoIcon size={16} />
        </button>
      </div>
      <IndexingPanel />
      <button
        class="icon-btn basket-btn"
        onclick={() => (basket.open = true)}
        aria-label={i18n.t.basket.title}
        title={i18n.t.basket.title}
      >
        <BasketIcon size={18} />
        {#if basket.count > 0}
          <span class="basket-badge">{basket.count}</span>
        {/if}
      </button>
      <button
        class="icon-btn logs-btn"
        onclick={() => (logs.open = true)}
        aria-label={i18n.t.logs.title}
        title={i18n.t.logs.title}
      >
        <LogsIcon size={18} />
        {#if logs.unreadCount > 0}
          <span class="logs-badge">{logs.unreadCount}</span>
        {/if}
      </button>
      <button
        class="icon-btn"
        onclick={() => (settings.open = true)}
        aria-label={i18n.t.settings.title}
        title={i18n.t.settings.title}
      >
        <SettingsIcon />
      </button>
    </div>
  </header>

  <div class="search-area">
    <SearchBar />

    {#if appState.error}
      <p class="error" role="alert">{appState.error}</p>
    {/if}
  </div>

  <main>
    <BookList books={appState.searchResults} />
  </main>

  <BookDock />

  <SettingsPanel />
  <BasketPopup />
  <LogsPanel />
  <ToastContainer />
</div>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 2rem;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg-header);
    flex-shrink: 0;
  }

  .search-area {
    padding: 0.5rem 2rem;
    flex-shrink: 0;
  }

  main {
    flex: 1;
    overflow: auto;
    padding: 0 2rem 1rem;
  }

  .header-title {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
  }

  h1 {
    font-size: 1.5rem;
    font-weight: 600;
    margin: 0;
  }

  .book-count {
    font-size: 0.8rem;
    font-weight: 400;
    color: var(--color-text-secondary);
  }

  .header-controls {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .undo-redo {
    display: flex;
    gap: 0.25rem;
  }

  .icon-btn:disabled {
    opacity: 0.3;
    cursor: default;
    pointer-events: none;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.4rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg-input);
    color: var(--color-text);
    cursor: pointer;
  }

  .icon-btn:hover {
    border-color: var(--color-primary);
  }

  .basket-btn,
  .logs-btn {
    position: relative;
  }

  .logs-badge {
    position: absolute;
    top: -6px;
    right: -6px;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    border-radius: 8px;
    background: var(--color-warning);
    color: #000;
    font-size: 0.65rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }

  .basket-badge {
    position: absolute;
    top: -6px;
    right: -6px;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    border-radius: 8px;
    background: var(--color-primary);
    color: white;
    font-size: 0.65rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }

  .error {
    color: var(--color-error);
    background: var(--color-bg-error);
    padding: 0.5rem 1rem;
    border-radius: 4px;
    font-size: 0.9rem;
  }
</style>
