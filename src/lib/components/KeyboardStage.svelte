<script lang="ts">
  import { onMount } from 'svelte';
  import type { HwEffect, Rgb } from '../api';
  import { shade, simPresses, type KeyGeo } from '../preview';

  interface Props {
    keys: KeyGeo[];
    bounds: [number, number, number, number];
    fx: HwEffect;
    lock: boolean;
    /** bump to play the "applied" sweep */
    pulse: number;
    /** live colours from the software engine (layout order); overrides the preview */
    colours?: Rgb[] | null;
  }
  let { keys, bounds, fx, lock, pulse, colours = null }: Props = $props();

  let host: HTMLDivElement;
  let canvas: HTMLCanvasElement;
  const light = document.createElement('canvas');
  let pulseStart = -1;
  let lastPulse = 0;

  $effect(() => {
    if (pulse !== lastPulse) {
      lastPulse = pulse;
      pulseStart = performance.now();
    }
  });

  function roundRect(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number) {
    ctx.beginPath();
    ctx.roundRect(x, y, w, h, r);
  }

  onMount(() => {
    const ctx = canvas.getContext('2d')!;
    const lctx = light.getContext('2d')!;
    let raf = 0;
    let W = 0;
    let H = 0;
    let dpr = 1;

    const resize = () => {
      dpr = window.devicePixelRatio || 1;
      W = host.clientWidth;
      H = host.clientHeight;
      canvas.width = Math.round(W * dpr);
      canvas.height = Math.round(H * dpr);
      light.width = canvas.width;
      light.height = canvas.height;
    };
    const ro = new ResizeObserver(resize);
    ro.observe(host);
    resize();

    const frame = (now: number) => {
      raf = requestAnimationFrame(frame);
      if (document.hidden || W === 0) return;
      const t = now / 1000;
      const [x0, y0, x1, y1] = bounds;
      const bw = x1 - x0;
      const bh = y1 - y0;
      const s = Math.min((W * 0.86) / bw, (H * 0.62) / bh);
      const ox = (W - bw * s) / 2;
      const oy = H * 0.12;
      const px = (x: number) => (ox + (x - x0) * s) * dpr;
      const py = (y: number) => (oy + (y - y0) * s) * dpr;
      const ps = s * dpr;

      const pulseP = pulseStart < 0 ? 1 : (now - pulseStart) / 900;
      const live = colours;
      const presses = live ? [] : simPresses(keys, t);
      const keyColours: Rgb[] = keys.map((k, i) => {
        let c: Rgb = live && live[i] ? [...live[i]] as Rgb : shade(fx, k, t, presses);
        if (lock && k.name === 'Fn') c = [255, 255, 255];
        if (pulseP < 1) {
          const front = 1.15 - pulseP * 1.3;
          const b = Math.max(0, 1 - Math.abs(k.cx - front) * 5) * (1 - pulseP);
          c = [c[0] + (255 - c[0]) * b, c[1] + (255 - c[1]) * b, c[2] + (255 - c[2]) * b];
        }
        return c;
      });

      // light layer: what the LEDs emit, used for glow and the desk reflection
      lctx.clearRect(0, 0, light.width, light.height);
      keys.forEach((k, i) => {
        const [r, g, b] = keyColours[i];
        lctx.fillStyle = `rgb(${r | 0},${g | 0},${b | 0})`;
        for (const [a, bb, c, d] of k.rects) lctx.fillRect(px(a), py(bb), (c - a) * ps, (d - bb) * ps);
      });

      ctx.clearRect(0, 0, canvas.width, canvas.height);
      // desk reflection
      ctx.save();
      ctx.globalAlpha = 0.28;
      ctx.filter = `blur(${26 * dpr}px)`;
      ctx.translate(0, py(y1) * 2 + 14 * dpr);
      ctx.scale(1, -1);
      ctx.drawImage(light, 0, 0);
      ctx.restore();
      // glow halo
      ctx.save();
      ctx.globalAlpha = 0.85;
      ctx.filter = `blur(${18 * dpr}px)`;
      ctx.drawImage(light, 0, 0);
      ctx.restore();

      // plate
      const pad = 9 * ps;
      ctx.fillStyle = '#13102499';
      roundRect(ctx, px(x0) - pad, py(y0) - pad, bw * ps + pad * 2, bh * ps + pad * 2, 14 * dpr);
      ctx.fill();
      ctx.strokeStyle = '#ffffff12';
      ctx.lineWidth = dpr;
      ctx.stroke();

      // keycaps
      const fontPx = Math.max(9, 8.6 * s) * dpr;
      ctx.font = `600 ${fontPx}px 'Manrope Variable', sans-serif`;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      keys.forEach((k, i) => {
        const [r, g, b] = keyColours[i];
        const lum = (r * 0.3 + g * 0.59 + b * 0.11) / 255;
        for (const [a, bb, c, d] of k.rects) {
          const x = px(a);
          const y = py(bb);
          const w = (c - a) * ps;
          const h = (d - bb) * ps;
          const grad = ctx.createLinearGradient(0, y, 0, y + h);
          grad.addColorStop(0, '#272340');
          grad.addColorStop(1, '#1a1730');
          ctx.fillStyle = grad;
          roundRect(ctx, x, y, w, h, 6 * dpr);
          ctx.fill();
          // light shining through the cap
          ctx.fillStyle = `rgba(${r | 0},${g | 0},${b | 0},${0.1 + lum * 0.22})`;
          ctx.fill();
          ctx.fillStyle = `rgba(${r | 0},${g | 0},${b | 0},${0.55 + lum * 0.35})`;
          roundRect(ctx, x + 3 * dpr, y + h - 3.2 * dpr, w - 6 * dpr, 1.6 * dpr, dpr);
          ctx.fill();
          ctx.strokeStyle = '#ffffff10';
          roundRect(ctx, x + 0.5 * dpr, y + 0.5 * dpr, w - dpr, h - dpr, 6 * dpr);
          ctx.stroke();
        }
        const r0 = k.rects[0];
        const lx = px((r0[0] + r0[2]) / 2);
        const ly = py((r0[1] + r0[3]) / 2) - 1.5 * dpr;
        const mix = 0.35 + lum * 0.65;
        ctx.fillStyle = `rgb(${(r * mix + 190 * (1 - mix)) | 0},${(g * mix + 185 * (1 - mix)) | 0},${(b * mix + 215 * (1 - mix)) | 0})`;
        ctx.fillText(k.name, lx, ly);
      });
    };
    raf = requestAnimationFrame(frame);
    return () => {
      cancelAnimationFrame(raf);
      ro.disconnect();
    };
  });
</script>

<div class="stage" bind:this={host}>
  <canvas bind:this={canvas}></canvas>
</div>

<style>
  .stage {
    position: relative;
    width: 100%;
    height: 100%;
    background: radial-gradient(120% 90% at 50% 38%, #1a1630 0%, var(--ink) 70%);
  }
  canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }
</style>
