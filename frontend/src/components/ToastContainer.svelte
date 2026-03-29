<script lang="ts">
  import { toasts } from "../lib/toast";
  import { i18n } from "../lib/i18n";
</script>

{#if toasts.items.length > 0}
  <div class="toast-container">
    {#each toasts.items as toast (toast.id)}
      <div class="toast level-{toast.level}" role="status">
        <span class="toast-message">{toast.message}</span>
        <button
          class="toast-dismiss"
          onclick={() => {
            toasts.dismiss(toast.id);
          }}
          aria-label={i18n.t.dismiss}>&times;</button
        >
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    z-index: 200;
    max-width: 400px;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.6rem 1rem;
    border-radius: 6px;
    font-size: 0.85rem;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
    animation: slide-in 0.2s ease-out;
  }

  .level-success {
    background: var(--color-success, #2e7d32);
    color: #fff;
  }

  .level-error {
    background: var(--color-error);
    color: #fff;
  }

  .level-info {
    background: var(--color-primary);
    color: #fff;
  }

  .toast-message {
    flex: 1;
  }

  .toast-dismiss {
    border: none;
    background: none;
    color: inherit;
    cursor: pointer;
    font-size: 1.1rem;
    line-height: 1;
    opacity: 0.7;
    padding: 0;
  }

  .toast-dismiss:hover {
    opacity: 1;
  }

  @keyframes slide-in {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }
</style>
