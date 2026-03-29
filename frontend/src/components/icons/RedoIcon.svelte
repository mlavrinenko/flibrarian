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
  <polyline points="12 7 15 4 12 1" />
  <path d="M15 4H6a4 4 0 0 0 0 8h2" />
</svg>
