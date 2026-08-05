<script lang="ts">
  let {
    data,
    max,
    height = 32,
    color = "var(--dockl-accent)",
  }: {
    data: number[];
    max?: number;
    height?: number;
    color?: string;
  } = $props();

  const width = 160;

  let points = $derived.by(() => {
    if (data.length === 0) return "";
    const effectiveMax = max ?? Math.max(...data, 1);
    const stepX = data.length > 1 ? width / (data.length - 1) : 0;
    return data
      .map((v, i) => {
        const x = i * stepX;
        const y = height - (Math.min(v, effectiveMax) / effectiveMax) * height;
        return `${x.toFixed(1)},${y.toFixed(1)}`;
      })
      .join(" ");
  });
</script>

<svg class="sparkline" viewBox={`0 0 ${width} ${height}`} {height} preserveAspectRatio="none">
  {#if points}
    <polyline fill="none" stroke={color} stroke-width="1.5" stroke-linejoin="round" {points} />
  {/if}
</svg>

<style>
  .sparkline {
    display: block;
    width: 100%;
  }
</style>
