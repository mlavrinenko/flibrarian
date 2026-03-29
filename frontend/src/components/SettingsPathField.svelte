<script lang="ts">
  import { i18n } from "../lib/i18n";

  let {
    id,
    label,
    placeholder,
    value = $bindable(),
    tauriEnv,
    onbrowse,
  }: {
    id: string;
    label: string;
    placeholder: string;
    value?: string;
    tauriEnv: boolean;
    onbrowse?: () => void;
  } = $props();
</script>

<div class="field">
  <label for={id}>{label}</label>
  <div class="input-row">
    <input {id} type="text" {placeholder} bind:value />
    {#if tauriEnv && onbrowse}
      <button type="button" class="browse-btn" onclick={onbrowse}>
        {i18n.t.settings.browseButton}
      </button>
    {/if}
  </div>
</div>

<style>
  .field {
    margin-bottom: 1rem;
  }

  label {
    display: block;
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    margin-bottom: 0.25rem;
  }

  .input-row {
    display: flex;
    gap: 0.5rem;
  }

  input {
    flex: 1;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    font-size: 0.9rem;
    background: var(--color-bg-input);
    color: var(--color-text);
  }

  input:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px var(--color-primary-ring);
  }

  .browse-btn {
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-bg-input);
    color: var(--color-text);
    cursor: pointer;
    font-size: 0.85rem;
    white-space: nowrap;
  }

  .browse-btn:hover {
    border-color: var(--color-primary);
  }
</style>
