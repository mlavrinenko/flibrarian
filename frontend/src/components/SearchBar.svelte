<script lang="ts">
  import { appState } from "../lib/state.svelte";
  import { i18n } from "../lib/i18n";
  import CloseIcon from "./icons/CloseIcon.svelte";
  import SpinnerIcon from "./icons/SpinnerIcon.svelte";
</script>

<div class="search-bar" role="search">
  <div class="field">
    <label for="search-query">{i18n.t.search.searchLabel}</label>
    <div class="input-wrapper">
      <input
        id="search-query"
        type="text"
        placeholder={i18n.t.search.searchPlaceholder}
        bind:value={appState.searchQuery}
      />
      {#if appState.loading}
        <span class="loading-indicator" aria-label="Searching…">
          <SpinnerIcon />
        </span>
      {:else if appState.searchQuery}
        <button
          class="clear-btn"
          onclick={() => (appState.searchQuery = "")}
          aria-label={i18n.t.bookList.filter.clearFilter}
        >
          <CloseIcon />
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .search-bar {
    display: flex;
    gap: 0.75rem;
    align-items: flex-end;
    padding: 1rem 0;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    flex: 1;
  }

  label {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    font-size: 0.9rem;
    background: var(--color-bg-input);
    color: var(--color-text);
    box-sizing: border-box;
  }

  input:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px var(--color-primary-ring);
  }

  .input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }

  .input-wrapper input {
    padding-right: 1.8rem;
  }

  .clear-btn {
    position: absolute;
    right: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2px;
    border: none;
    border-radius: 3px;
    background: transparent;
    color: var(--color-text-tertiary);
    cursor: pointer;
    line-height: 0;
  }

  .clear-btn:hover {
    color: var(--color-text);
    background: var(--color-bg-hover);
  }

  .loading-indicator {
    position: absolute;
    right: 6px;
    display: flex;
    align-items: center;
    color: var(--color-text-tertiary);
    line-height: 0;
  }
</style>
