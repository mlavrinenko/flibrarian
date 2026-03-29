<script lang="ts">
  import { appState } from "../lib/state.svelte";
  import { api } from "../lib/api";
  import type { ArchiveInfo } from "../lib/api";
  import { SvelteSet } from "svelte/reactivity";
  import { i18n } from "../lib/i18n";

  let {
    open,
    onclose,
  }: {
    open: boolean;
    onclose: () => void;
  } = $props();

  let archives: ArchiveInfo[] = $state([]);
  let selected = new SvelteSet<string>();
  let loading = $state(false);

  $effect(() => {
    if (open && appState.libraryPath) {
      void loadArchives(appState.libraryPath);
    }
  });

  async function loadArchives(path: string) {
    loading = true;
    try {
      archives = await api.listArchives(path);
      selected.clear();
    } catch {
      archives = [];
    } finally {
      loading = false;
    }
  }

  function toggle(name: string) {
    if (selected.has(name)) {
      selected.delete(name);
    } else {
      selected.add(name);
    }
  }

  function selectAll() {
    for (const a of archives) selected.add(a.name);
  }

  function deselectAll() {
    selected.clear();
  }

  function handleReindex() {
    if (selected.size === 0) return;
    onclose();
    void appState.index("pick", [...selected]);
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onclose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onclose();
    }
  }

  const allSelected = $derived(
    archives.length > 0 && selected.size === archives.length,
  );
</script>

{#if open}
  <div
    class="backdrop"
    role="dialog"
    aria-modal="true"
    aria-label={i18n.t.indexing.pickerTitle}
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
    tabindex="-1"
  >
    <div class="popup">
      <div class="header">
        <h3>{i18n.t.indexing.pickerTitle}</h3>
        <button
          class="modal-close-btn"
          onclick={() => (open = false)}
          aria-label={i18n.t.close}
        >
          &times;
        </button>
      </div>

      {#if loading}
        <p class="empty">{i18n.t.indexing.pickerLoading}</p>
      {:else if archives.length === 0}
        <p class="empty">{i18n.t.indexing.pickerEmpty}</p>
      {:else}
        <div class="toolbar">
          <button
            class="link-btn"
            onclick={allSelected ? deselectAll : selectAll}
          >
            {allSelected
              ? i18n.t.indexing.pickerDeselectAll
              : i18n.t.indexing.pickerSelectAll}
          </button>
          <span class="count">{selected.size} / {archives.length}</span>
        </div>
        <ul class="archive-list">
          {#each archives as archive (archive.name)}
            <li>
              <label class="archive-item">
                <input
                  type="checkbox"
                  checked={selected.has(archive.name)}
                  onchange={() => {
                    toggle(archive.name);
                  }}
                />
                <span class="archive-name">{archive.name}</span>
                <span class="archive-status status-{archive.status}"
                  >{archive.status}</span
                >
              </label>
            </li>
          {/each}
        </ul>
      {/if}

      <div class="footer">
        <button
          class="reindex-btn"
          disabled={selected.size === 0}
          onclick={handleReindex}
        >
          {i18n.t.indexing.pickerReindex} ({selected.size})
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .popup {
    background: var(--color-bg);
    border-radius: 8px;
    width: 480px;
    max-width: 90vw;
    max-height: 70vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--color-border);
  }

  .header h3 {
    margin: 0;
    font-size: 1rem;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid var(--color-border);
  }

  .link-btn {
    background: none;
    border: none;
    color: var(--color-success);
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0;
  }

  .link-btn:hover {
    text-decoration: underline;
  }

  .count {
    font-size: 0.8rem;
    color: var(--color-text-secondary);
  }

  .archive-list {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .archive-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 1rem;
    cursor: pointer;
  }

  .archive-item:hover {
    background: var(--color-bg-hover);
  }

  .archive-name {
    flex: 1;
    font-size: 0.85rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .archive-status {
    font-size: 0.75rem;
    padding: 0.1rem 0.4rem;
    border-radius: 3px;
    white-space: nowrap;
  }

  .status-indexed {
    color: var(--color-success);
  }

  .status-indexing {
    color: var(--color-warning, #e6a700);
  }

  .status-new {
    color: var(--color-text-secondary);
  }

  .footer {
    padding: 0.75rem 1rem;
    border-top: 1px solid var(--color-border);
    display: flex;
    justify-content: flex-end;
  }

  .reindex-btn {
    padding: 0.5rem 1.25rem;
    background: var(--color-success);
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 0.9rem;
    cursor: pointer;
  }

  .reindex-btn:hover:not(:disabled) {
    background: var(--color-success-hover);
  }

  .reindex-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .empty {
    padding: 2rem 1rem;
    text-align: center;
    color: var(--color-text-secondary);
  }
</style>
