<script lang="ts">
  import { appState } from "../lib/state.svelte";
  import { i18n } from "../lib/i18n";
  import CloseIcon from "./icons/CloseIcon.svelte";
  import ChevronDownIcon from "./icons/ChevronDownIcon.svelte";

  const NOT_SPECIFIED = "\0";

  let open = $state(false);
  let search = $state("");
  let buttonRef: HTMLButtonElement | undefined = $state();
  let searchRef: HTMLInputElement | undefined = $state();

  let selected = $derived(appState.selectedLanguages);
  let hasSelection = $derived(selected.length > 0);

  let notSpecifiedCount = $derived(
    Math.max(
      0,
      appState.bookCount -
        appState.availableLanguages.reduce((sum, lc) => sum + lc.count, 0),
    ),
  );

  let options = $derived<{ value: string; label: string; count?: number }[]>(
    (() => {
      const q = search.trim().toLowerCase();
      const langs = appState.availableLanguages
        .filter((lc) => !q || lc.lang.toLowerCase().includes(q))
        .map((lc) => ({ value: lc.lang, label: lc.lang, count: lc.count }));

      const notSpecifiedLabel = i18n.t.bookList.filter.notSpecified;
      const showNotSpecified =
        !q || notSpecifiedLabel.toLowerCase().includes(q);

      return showNotSpecified
        ? [
            {
              value: NOT_SPECIFIED,
              label: notSpecifiedLabel,
              count: notSpecifiedCount || undefined,
            },
            ...langs,
          ]
        : langs;
    })(),
  );

  function isSelected(value: string): boolean {
    return selected.includes(value);
  }

  function toggle(value: string) {
    appState.toggleLanguage(value);
  }

  function clear(e: MouseEvent) {
    e.stopPropagation();
    appState.clearColumnFilter("lang");
  }

  function handleToggle() {
    open = !open;
    if (open) {
      search = "";
      requestAnimationFrame(() => searchRef?.focus());
    }
  }

  function handleClickOutside(e: MouseEvent) {
    const target = e.target as Node;
    if (buttonRef && !buttonRef.closest(".lang-filter")?.contains(target)) {
      open = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      open = false;
      buttonRef?.focus();
    }
  }

  $effect(() => {
    if (open) {
      document.addEventListener("click", handleClickOutside, true);
      return () => {
        document.removeEventListener("click", handleClickOutside, true);
      };
    }
  });

  function displayText(): string {
    if (selected.length === 0) return "";
    return selected
      .map((v) =>
        v === NOT_SPECIFIED ? i18n.t.bookList.filter.notSpecified : v,
      )
      .join(", ");
  }
</script>

<div class="lang-filter">
  <div class="trigger-row">
    <button
      bind:this={buttonRef}
      class="trigger"
      class:active={hasSelection}
      onclick={handleToggle}
      onkeydown={handleKeydown}
      type="button"
    >
      <span class="trigger-text" class:placeholder={!hasSelection}>
        {hasSelection ? displayText() : i18n.t.bookList.filter.placeholder}
      </span>
      <ChevronDownIcon size={10} />
    </button>
    {#if hasSelection}
      <button
        class="clear-btn"
        onclick={clear}
        type="button"
        aria-label={i18n.t.bookList.filter.clearFilter}
      >
        <CloseIcon size={12} />
      </button>
    {/if}
  </div>

  {#if open}
    <div
      class="dropdown"
      role="listbox"
      aria-label={i18n.t.bookList.columns.lang}
    >
      <div class="search-row">
        <input
          bind:this={searchRef}
          class="search-input"
          type="text"
          placeholder={i18n.t.bookList.filter.placeholder}
          bind:value={search}
          onkeydown={handleKeydown}
        />
      </div>
      <div class="options-list">
        {#each options as opt (opt.value)}
          <button
            class="option"
            class:selected={isSelected(opt.value)}
            role="option"
            aria-selected={isSelected(opt.value)}
            type="button"
            onclick={() => {
              toggle(opt.value);
            }}
          >
            <span class="checkbox">{isSelected(opt.value) ? "\u2713" : ""}</span
            >
            <span
              class="option-label"
              class:not-specified={opt.value === NOT_SPECIFIED}
              >{opt.label}</span
            >
            {#if opt.count != null}
              <span class="option-count">{opt.count}</span>
            {/if}
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .lang-filter {
    position: relative;
    width: 100%;
  }

  .trigger-row {
    display: flex;
    align-items: center;
    position: relative;
  }

  .trigger {
    width: 100%;
    padding: 0.25rem 1.4rem 0.25rem 0.4rem;
    border: 1px solid var(--color-border);
    border-radius: 3px;
    font-size: 0.75rem;
    background: var(--color-bg-input);
    color: var(--color-text);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 2px;
    box-sizing: border-box;
    text-align: left;
    line-height: 1.4;
  }

  .trigger.active {
    border-color: var(--color-primary);
    background: var(--color-bg-selected);
  }

  .trigger:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 1px var(--color-primary-ring);
  }

  .trigger-text {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .trigger-text.placeholder {
    color: var(--color-text-tertiary);
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

  .dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    min-width: 160px;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 3px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 100;
    margin-top: 1px;
    display: flex;
    flex-direction: column;
  }

  .search-row {
    padding: 0.3rem;
    border-bottom: 1px solid var(--color-border-light);
  }

  .search-input {
    width: 100%;
    padding: 0.25rem 0.4rem;
    border: 1px solid var(--color-border);
    border-radius: 3px;
    font-size: 0.75rem;
    background: var(--color-bg-input);
    color: var(--color-text);
    box-sizing: border-box;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 1px var(--color-primary-ring);
  }

  .options-list {
    max-height: 200px;
    overflow-y: auto;
  }

  .option {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 0.3rem 0.5rem;
    border: none;
    background: transparent;
    color: var(--color-text);
    font-size: 0.75rem;
    cursor: pointer;
    text-align: left;
  }

  .option:hover {
    background: var(--color-bg-hover);
  }

  .option.selected {
    background: var(--color-bg-selected);
  }

  .checkbox {
    width: 14px;
    flex-shrink: 0;
    text-align: center;
    font-size: 0.7rem;
    color: var(--color-primary);
  }

  .option-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .not-specified {
    font-style: italic;
    color: var(--color-text-secondary);
  }

  .option-count {
    flex-shrink: 0;
    font-size: 0.65rem;
    color: var(--color-text-tertiary);
  }
</style>
