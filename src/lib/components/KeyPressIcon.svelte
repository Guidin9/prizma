<script lang="ts">
  /** A keycap being pressed: marks effects that only light up when a key is pressed. */
  let { size = 14 }: { size?: number } = $props();
</script>

<svg viewBox="0 0 16 16" width={size} height={size} aria-hidden="true">
  <rect x="2.5" y="6" width="11" height="8" rx="2" />
  <path d="M5.5 10.5h5" />
  <path d="M8 1.2v2.6M3.9 2.4l1.3 1.9M12.1 2.4l-1.3 1.9" />
</svg>

<style>
  svg {
    flex: none;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
