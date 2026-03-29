<script lang="ts">
  import { i18n } from "../lib/i18n";

  let { annotation }: { annotation: string } = $props();
</script>

<div class="annotation">
  <span class="label">{i18n.t.bookDetail.annotation}:</span>

  <div class="annotation-text">
    {#each annotation.split("\n\n") as paragraph (paragraph)}
      <p>{paragraph}</p>
    {/each}
  </div>
</div>

<style>
  .label {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--color-text-secondary);
    white-space: nowrap;
  }

  .annotation {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .annotation-text {
    font-size: 0.85rem;
    line-height: 1.5;
    color: var(--color-text-secondary);
  }

  .annotation-text p {
    margin: 0 0 0.4rem;
  }

  .annotation-text p:last-child {
    margin-bottom: 0;
  }
</style>
