<script lang="ts">
  let { size = 14 }: { size?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 12 12"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
>
  <line x1="3" y1="3" x2="9" y2="9" />
  <line x1="9" y1="3" x2="3" y2="9" />
</svg>
