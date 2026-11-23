<script lang="ts">
  import { onMount } from 'svelte';
  import type { EffectInfo, HwEffect } from '../api';
  import { hash, shade, type KeyGeo } from '../preview';

  interface Props {
    info: EffectInfo;
    effect: HwEffect;
    selected: boolean;
    applied: boolean;
    onpick: () => void;
  }
  let { info, effect, selected, applied, onpick }: Props = $props();

  const COLS = 7;
  const ROWS = 3;
  const cells: KeyGeo[] = [];
  for (let r = 0; r < ROWS; r++)
    for (let c = 0; c < COLS; c++)
      cells.push({
        index: r * COLS + c,
        name: '',
        leds: [],
        rects: [],
        cx: (c + 0.5 + r * 0.25) / (COLS + 0.5),
        cy: (r + 0.5) / ROWS,
        seed: hash(r * 97 + c * 13 + 5),
      });

  let canvas: HTMLCanvasElement;
  let hovered = $state(false);

  onMount(() => {
    const ctx = canvas.getContext('2d')!;
    const dpr = window.devicePixelRatio || 1;
    const W = canvas.clientWidth;
    const H = canvas.clientHeight;
    canvas.width = W * dpr;
    canvas.height = H * dpr;
    let raf = 0;
    const cw = (W * dpr) / (COLS + 0.6);
    const ch = (H * dpr) / ROWS;
    let lastStatic = '';
    const draw = (now: number) => {
      raf = requestAnimationFrame(draw);
      const live = hovered || selected;
      if (!live) {
        const key = JSON.stringify(effect.colour) + effect.reverse;
        if (key === lastStatic) return; // static thumbnail already drawn
        lastStatic = key;
      } else {
        lastStatic = '';
      }
      const t = live ? now / 1000 : 1.7;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const e = { ...effect, id: info.id, brightness: 4 };
      for (const k of cells) {
        const [r, g, b] = shade(e, k, t);
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
    raf = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(raf);
  });
</script>

<button
  class="tile"
  class:selected
  aria-pressed={selected}
  onclick={onpick}
  onmouseenter={() => (hovered = true)}
  onmouseleave={() => (hovered = false)}
  onfocus={() => (hovered = true)}
  onblur={() => (hovered = false)}
>
  <canvas bind:this={canvas}></canvas>
  <span class="name">{info.name}</span>
  {#if applied}<span class="dot" title="Klavyede şu an bu efekt var"></span>{/if}
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
