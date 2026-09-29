<script lang="ts">
  import { coverViewer } from "../lib/cover";
  import { i18n } from "../lib/i18n";

  function close() {
    coverViewer.close();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) close();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.stopPropagation();
      close();
    }
  }
</script>

{#if coverViewer.open && coverViewer.url}
  <div
    class="backdrop"
    role="dialog"
    aria-modal="true"
    aria-label={coverViewer.title}
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
    tabindex="-1"
  >
    <div class="popup">
      <div class="header">
        <h3>{coverViewer.title}</h3>
        <button
          class="modal-close-btn"
          onclick={close}
          aria-label={i18n.t.close}
        >
          &times;
        </button>
      </div>
      <div class="image-container">
        <img class="cover-full" src={coverViewer.url} alt={coverViewer.title} />
      </div>
    </div>
  </div>
{/if}

<style>
  .popup {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    max-width: 90vw;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 0.5rem 0.5rem 1rem;
    border-bottom: 1px solid var(--color-border-light);
  }

  h3 {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 600;
    max-width: 70vw;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .image-container {
    display: flex;
    padding: 0.75rem;
    min-height: 0;
  }

  .cover-full {
    max-width: calc(90vw - 1.5rem);
    max-height: calc(90vh - 4.5rem);
    object-fit: contain;
    border-radius: 4px;
    border: 1px solid var(--color-border-light);
  }
</style>
