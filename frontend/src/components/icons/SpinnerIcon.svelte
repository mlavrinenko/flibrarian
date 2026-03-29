<script lang="ts">
  let { size = 14 }: { size?: number } = $props();
</script>

<svg
  class="spinner"
  width={size}
  height={size}
  viewBox="0 0 16 16"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
>
  <path d="M8 1a7 7 0 1 0 7 7" />
</svg>

<style>
  .spinner {
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
