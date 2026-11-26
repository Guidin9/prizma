<script lang="ts">
  import { onMount } from 'svelte';
  import type { EffectInfo, HwEffect } from '../api';
  import { REACTIVE, hash, shade, simPresses, type KeyGeo } from '../preview';
  import KeyPressIcon from './KeyPressIcon.svelte';
  import { t } from '../i18n.svelte';
  import { hwName } from '../swcatalog';

  interface Props {
    info: EffectInfo;
    effect: HwEffect;
    selected: boolean;
    applied: boolean;
    onpick: () => void;
  }
  let { info, effect: fx, selected, applied, onpick }: Props = $props();

  const COLS = 7;
  const ROWS = 3;
  const cells: KeyGeo[] = [];
  for (let r = 0; r < ROWS; r++)
    for (let c = 0; c < COLS; c++)
      cells.push({
        index: r * COLS + c,
        id: '',
        name: '',
        leds: [],
        rects: [],
        cx: (c + 0.5 + r * 0.25) / (COLS + 0.5),
        cy: (r + 0.5) / ROWS,
        seed: hash(r * 97 + c * 13 + 5),
      });

  let canvas: HTMLCanvasElement;
  let hovered = $state(false);

  let paint: ((t: number) => void) | null = $state.raw(null);

  onMount(() => {
    const ctx = canvas.getContext('2d')!;
    const dpr = window.devicePixelRatio || 1;
    const W = canvas.clientWidth;
    const H = canvas.clientHeight;
    canvas.width = W * dpr;
    canvas.height = H * dpr;
    const cw = (W * dpr) / (COLS + 0.6);
    const ch = (H * dpr) / ROWS;
    paint = (t: number) => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const e = { ...fx, id: info.id, brightness: 4 };
      const presses = simPresses(cells, t, 0.35);
      for (const k of cells) {
        const [r, g, b] = shade(e, k, t, presses);
        const col = k.index % COLS;
        const row = Math.floor(k.index / COLS);
        const x = (col + row * 0.3) * cw + cw * 0.08;
        const y = row * ch + ch * 0.1;
        ctx.shadowColor = `rgb(${r | 0},${g | 0},${b | 0})`;
        ctx.shadowBlur = 10 * dpr;
        ctx.fillStyle = `rgb(${(r * 0.85 + 18) | 0},${(g * 0.85 + 16) | 0},${(b * 0.85 + 30) | 0})`;
        ctx.beginPath();
        ctx.roundRect(x, y, cw * 0.84, ch * 0.78, 3 * dpr);
        ctx.fill();
      }
    };
    return () => (paint = null);
  });

  // Only the hovered or selected tile animates; the others are drawn once (a page full of
  // animation loops keeps the web view busy even when nothing changes).
  $effect(() => {
    const live = hovered || selected;
    void fx.colour, fx.speed, fx.reverse; // redraw when the stored settings change
    if (!paint) return;
    if (!live) {
      paint(1.7);
      return;
    }
    let raf = 0;
    let last = 0;
    const loop = (now: number) => {
      raf = requestAnimationFrame(loop);
      if (now - last < 32) return;
      last = now;
      paint?.(now / 1000);
    };
    raf = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(raf);
  });
</script>

<button
  class="tile"
  class:selected
  aria-label={REACTIVE.has(info.id) ? t('tile.pressAria', { name: hwName(info) }) : undefined}
  aria-pressed={selected}
  onclick={onpick}
  onmouseenter={() => (hovered = true)}
  onmouseleave={() => (hovered = false)}
  onfocus={() => (hovered = true)}
  onblur={() => (hovered = false)}
>
  <canvas bind:this={canvas}></canvas>
  {#if REACTIVE.has(info.id)}
    <span class="press" title={t('tile.pressHint')}>
      <KeyPressIcon size={12} />{t('tile.press')}
    </span>
  {/if}
  <span class="name">{hwName(info)}</span>
  {#if applied}<span class="dot" title={t('tile.current')}></span>{/if}
</button>

<style>
  .tile {
    position: relative;
    display: grid;
    gap: var(--s2);
    padding: var(--s3) var(--s3) var(--s2);
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 12px;
    cursor: pointer;
    text-align: left;
    transition: border-color 160ms, background 160ms, transform 160ms;
  }
  .tile:hover {
    background: var(--raised);
    border-color: var(--line-strong);
  }
  .tile.selected {
    background: var(--raised);
    border-color: transparent;
  }
  .tile.selected::before {
    content: '';
    position: absolute;
    inset: -1px;
    border-radius: 12px;
    padding: 1px;
    background: var(--spectrum);
    -webkit-mask: linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0);
    mask: linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0);
    -webkit-mask-composite: xor;
    mask-composite: exclude;
    pointer-events: none;
  }
  canvas {
    width: 100%;
    height: 46px;
    display: block;
  }
  .name {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .press {
    position: absolute;
    top: 9px;
    left: 9px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 7px 2px 5px;
    border-radius: 6px;
    border: 1px solid #ffffff2e;
    background: #0e0c18d9;
    color: var(--text);
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.01em;
    pointer-events: none;
  }
  .dot {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--ok);
    box-shadow: 0 0 8px var(--ok);
  }
</style>
