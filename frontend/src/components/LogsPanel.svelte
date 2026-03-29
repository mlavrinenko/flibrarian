<script lang="ts">
  import { logs } from "../lib/logs";
  import { i18n } from "../lib/i18n";
  import type { LogEntry, LogLevel } from "../lib/logs/state.svelte";

  function close() {
    logs.markRead();
    logs.open = false;
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) close();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") close();
  }

  function formatTime(ts: number): string {
    const d = new Date(ts);
    return d.toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }

  function formatDate(ts: number): string {
    const d = new Date(ts);
    return d.toLocaleDateString([], {
      month: "short",
      day: "numeric",
    });
  }

  function sourceLabel(entry: LogEntry): string {
    const key =
      `source${entry.source[0].toUpperCase()}${entry.source.slice(1)}` as
        | "sourceIndexing"
        | "sourceJs"
        | "sourceApp";
    return i18n.t.logs[key];
  }

  let newThreshold = $state(0);

  $effect(() => {
    if (logs.open) {
      newThreshold = logs.lastReadTimestamp;
    }
  });
</script>

<svelte:window onkeydown={logs.open ? handleKeydown : undefined} />

{#if logs.open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="backdrop" onmousedown={handleBackdropClick}>
    <div class="panel" role="dialog" aria-label={i18n.t.logs.title}>
      <div class="panel-header">
        <h2>{i18n.t.logs.title}</h2>
        <div class="header-actions">
          <button
            class="clear-btn"
            onclick={() => {
              logs.clear();
            }}
          >
            {i18n.t.logs.clear}
          </button>
          <button class="close-btn" onclick={close}>&times;</button>
        </div>
      </div>

      <div class="filter-row">
        <div class="level-filters">
          {#each ["info", "warn", "error"] as level (level)}
            <button
              class="level-filter-btn"
              class:active={logs.minLevel === level}
              onclick={() => (logs.minLevel = level as LogLevel)}
            >
              {i18n.t.logs[
                `level${level[0].toUpperCase()}${level.slice(1)}` as keyof typeof i18n.t.logs
              ]}
            </button>
          {/each}
        </div>
        <input
          type="text"
          class="filter-input"
          placeholder={i18n.t.logs.filterPlaceholder}
          bind:value={logs.filter}
        />
      </div>

      <div class="log-entries">
        {#if logs.filtered.length === 0}
          <p class="empty">{i18n.t.logs.empty}</p>
        {:else}
          {#each logs.filtered as entry, i (i)}
            <div
              class="log-entry level-{entry.level}"
              class:new-entry={entry.timestamp > newThreshold}
            >
              <span class="ts"
                >{formatDate(entry.timestamp)}
                {formatTime(entry.timestamp)}</span
              >
              <span class="badge level-badge-{entry.level}">{entry.level}</span>
              <span class="badge source-badge">{sourceLabel(entry)}</span>
              <span class="msg">{entry.message}</span>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .panel {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    width: 90vw;
    height: 90vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--color-border);
  }

  .panel-header h2 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .header-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .clear-btn {
    padding: 0.25rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-bg-input);
    color: var(--color-text);
    cursor: pointer;
    font-size: 0.8rem;
  }

  .clear-btn:hover {
    border-color: var(--color-primary);
  }

  .close-btn {
    padding: 0.1rem 0.5rem;
    border: none;
    background: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 1.3rem;
    line-height: 1;
  }

  .close-btn:hover {
    color: var(--color-text);
  }

  .filter-row {
    padding: 0.5rem 1rem;
    border-bottom: 1px solid var(--color-border-light);
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .level-filters {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
  }

  .level-filter-btn {
    padding: 0.3rem 0.5rem;
    border: 1px solid var(--color-border);
    background: var(--color-bg-input);
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 0.75rem;
    white-space: nowrap;
  }

  .level-filter-btn:first-child {
    border-radius: 4px 0 0 4px;
  }

  .level-filter-btn:last-child {
    border-radius: 0 4px 4px 0;
  }

  .level-filter-btn.active {
    background: var(--color-primary);
    color: #fff;
    border-color: var(--color-primary);
  }

  .filter-input {
    width: 100%;
    padding: 0.4rem 0.6rem;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-bg-input);
    color: var(--color-text);
    font-size: 0.85rem;
    box-sizing: border-box;
  }

  .filter-input:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px var(--color-primary-ring);
  }

  .log-entries {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem 1rem;
    font-family: monospace;
    font-size: 0.8rem;
  }

  .empty {
    color: var(--color-text-muted);
    text-align: center;
    padding: 2rem;
  }

  .log-entry {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    padding: 0.2rem 0;
    border-bottom: 1px solid var(--color-border-light);
    line-height: 1.4;
  }

  .log-entry.new-entry {
    background: var(--color-bg-selected);
    border-left: 3px solid var(--color-primary);
    padding-left: 0.4rem;
  }

  .ts {
    color: var(--color-text-muted);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .badge {
    padding: 0 0.35rem;
    border-radius: 3px;
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .level-badge-warn {
    background: var(--color-warning);
    color: #000;
  }

  .level-badge-error {
    background: var(--color-error);
    color: #fff;
  }

  .level-badge-info {
    background: var(--color-primary);
    color: #fff;
  }

  .source-badge {
    background: var(--color-bg-hover);
    color: var(--color-text-secondary);
  }

  .msg {
    word-break: break-word;
    min-width: 0;
  }
</style>
