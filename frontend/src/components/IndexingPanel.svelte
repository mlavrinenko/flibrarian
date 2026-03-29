<script lang="ts">
  import { appState } from "../lib/state.svelte";
  import { i18n } from "../lib/i18n";
  import { confirmDialog } from "../lib/api";
  import type { IndexingPhase } from "../lib/api";
  import ChevronDownIcon from "./icons/ChevronDownIcon.svelte";
  import ArchivePickerPopup from "./ArchivePickerPopup.svelte";

  type Mode = "new" | "full" | "search" | "pick";

  let mode = $state<Mode>("new");
  let menuOpen = $state(false);
  let pickerOpen = $state(false);

  const TOTAL_PHASES = 5;

  const phaseKeys: Record<IndexingPhase, keyof typeof i18n.t.indexing> = {
    Counting: "phaseCounting",
    Parsing: "phaseParsing",
    Writing: "phaseWriting",
    BuildingSearchIndex: "phaseBuildingSearchIndex",
    CreatingFtsIndex: "phaseCreatingFtsIndex",
  };

  const phaseStep: Record<IndexingPhase, number> = {
    Counting: 1,
    Parsing: 2,
    Writing: 3,
    BuildingSearchIndex: 4,
    CreatingFtsIndex: 5,
  };

  function progressTooltip(): string {
    const { phase, current, total } = appState.indexingProgress;
    const step = phaseStep[phase];
    const phaseName = i18n.t.indexing[phaseKeys[phase]];
    return `(${step}/${TOTAL_PHASES}) ${phaseName} — ${current} / ${total}`;
  }

  function progressPercent(): number {
    const { current, total } = appState.indexingProgress;
    return total > 0 ? (current / total) * 100 : 0;
  }

  function handleIndex() {
    if (mode === "pick") {
      pickerOpen = true;
      return;
    }
    void appState.index(mode);
  }

  async function handleCancel() {
    if (await confirmDialog(i18n.t.indexing.cancelConfirm)) {
      appState.cancelIndex();
    }
  }

  function selectMode(m: Mode) {
    mode = m;
    menuOpen = false;
  }

  function handleClickOutside(e: MouseEvent) {
    if (!(e.target as Element).closest(".split-btn")) {
      menuOpen = false;
    }
  }

  const indexing = $derived(appState.indexing);
  const hasProgress = $derived(indexing && appState.indexingProgress.total > 0);

  const newCount = $derived(
    appState.indexState
      ? appState.indexState.archives_new + appState.indexState.archives_pending
      : null,
  );

  const modes: Mode[] = ["new", "full", "search", "pick"];

  const modeLabels: Record<Mode, () => string> = {
    new: () =>
      i18n.t.indexing.modeNew + (newCount !== null ? ` (${newCount})` : ""),
    full: () => i18n.t.indexing.modeFull,
    search: () => i18n.t.indexing.modeSearch,
    pick: () => i18n.t.indexing.modePick,
  };

  const modeTips: Record<Mode, () => string> = {
    new: () => i18n.t.indexing.tipNew,
    full: () => i18n.t.indexing.tipFull,
    search: () => i18n.t.indexing.tipSearch,
    pick: () => i18n.t.indexing.tipPick,
  };

  const buttonLabel = $derived(modeLabels[mode]());
  const buttonTip = $derived(modeTips[mode]());

  const indexDisabled = $derived(
    !appState.libraryPath ||
      (mode === "new" && newCount === 0) ||
      (mode === "search" && appState.indexState?.total_books === 0),
  );
</script>

<svelte:window onclick={handleClickOutside} />

<div class="indexing-panel">
  {#if hasProgress}
    <span class="progress-text">{progressTooltip()}</span>
  {/if}
  <span class="index-label">{i18n.t.indexing.label}</span>
  <div class="split-btn">
    {#if indexing}
      <button
        class="split-main cancel"
        onclick={handleCancel}
        title={i18n.t.indexing.inProgress}
        role={hasProgress ? "progressbar" : undefined}
        aria-valuenow={hasProgress
          ? appState.indexingProgress.current
          : undefined}
        aria-valuemin={hasProgress ? 0 : undefined}
        aria-valuemax={hasProgress
          ? appState.indexingProgress.total
          : undefined}
      >
        {#if hasProgress}
          <div class="progress-fill" style="width: {progressPercent()}%"></div>
        {/if}
        <span class="btn-label">{i18n.t.indexing.cancel}</span>
      </button>
    {:else}
      <button
        class="split-main"
        onclick={handleIndex}
        disabled={indexDisabled}
        title={buttonTip}
      >
        <span class="btn-label">{buttonLabel}</span>
      </button>
      <button
        class="split-toggle"
        onclick={() => (menuOpen = !menuOpen)}
        aria-label={i18n.t.indexing.modeAriaLabel}
      >
        <ChevronDownIcon />
      </button>
    {/if}
    {#if menuOpen}
      <div class="split-menu">
        {#each modes as m (m)}
          <button
            class="split-menu-item"
            class:active={mode === m}
            title={modeTips[m]()}
            onclick={() => {
              selectMode(m);
            }}
          >
            {modeLabels[m]()}
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<ArchivePickerPopup
  open={pickerOpen}
  onclose={() => {
    pickerOpen = false;
  }}
/>

<style>
  .indexing-panel {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .progress-text {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    white-space: nowrap;
  }

  .index-label {
    font-size: 0.85rem;
    color: var(--color-text-secondary);
    white-space: nowrap;
  }

  .split-btn {
    display: flex;
    position: relative;
  }

  .split-main {
    position: relative;
    overflow: hidden;
    padding: 0.5rem 1rem;
    background: transparent;
    color: var(--color-text);
    border: 1px solid var(--color-border);
    border-radius: 6px 0 0 6px;
    font-size: 0.9rem;
    cursor: pointer;
    white-space: nowrap;
    text-align: left;
  }

  .split-main:hover:not(:disabled) {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .split-main:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .split-main.cancel {
    background: var(--color-progress-track);
    border-color: var(--color-border);
    border-radius: 6px;
  }

  .split-main.cancel:hover {
    background: var(--color-error);
    border-color: var(--color-error);
    color: white;
  }

  .progress-fill {
    position: absolute;
    inset: 0;
    width: 0;
    background: var(--color-primary);
    opacity: 0.35;
    transition: width 0.2s ease;
  }

  .btn-label {
    position: relative;
    z-index: 1;
  }

  .split-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.5rem 0.4rem;
    background: transparent;
    color: var(--color-text);
    border: 1px solid var(--color-border);
    border-left: none;
    border-radius: 0 6px 6px 0;
    cursor: pointer;
  }

  .split-toggle:hover:not(:disabled) {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .split-toggle:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .split-menu {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 2px;
    background: var(--color-bg-input);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    z-index: 10;
    min-width: 100%;
  }

  .split-menu-item {
    display: block;
    width: 100%;
    padding: 0.5rem 0.75rem;
    background: none;
    border: none;
    color: var(--color-text);
    font-size: 0.85rem;
    text-align: left;
    cursor: pointer;
    white-space: nowrap;
  }

  .split-menu-item:hover {
    background: var(--color-bg-hover);
  }

  .split-menu-item.active {
    color: var(--color-primary);
    font-weight: 600;
  }
</style>
