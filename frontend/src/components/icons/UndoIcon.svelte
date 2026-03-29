<script lang="ts">
  let { size = 16 }: { size?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 16 16"
  fill="none"
  stroke="currentColor"
  stroke-width="1.5"
  stroke-linecap="round"
  stroke-linejoin="round"
>
  <polyline points="4 7 1 4 4 1" />
  <path d="M1 4h9a4 4 0 0 1 0 8H7" />
</svg>
