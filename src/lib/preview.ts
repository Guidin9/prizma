// Approximate on-screen previews of the firmware effects. The keyboard renders its own
// animation; these only show the character of each effect on the stage.
import type { HwEffect, Layout, Rgb } from './api';
import type { L } from './i18n.svelte';

export interface KeyGeo {
  index: number;
  /** stable key id (LayoutKey.ansi), used by layer key masks */
  id: string;
  name: string;
  leds: number[];
  rects: [number, number, number, number][];
  cx: number; // 0..1 across the board
  cy: number; // 0..1 down the board
  seed: number; // 0..1, stable per key
}

export function geometry(layout: Layout): { keys: KeyGeo[]; bounds: [number, number, number, number] } {
  const all = layout.keys.flatMap((k) => k.rects);
  const x0 = Math.min(...all.map((r) => r[0]));
  const y0 = Math.min(...all.map((r) => r[1]));
  const x1 = Math.max(...all.map((r) => r[2]));
  const y1 = Math.max(...all.map((r) => r[3]));
  const keys = layout.keys.map((k, index) => {
    const rx0 = Math.min(...k.rects.map((r) => r[0]));
    const rx1 = Math.max(...k.rects.map((r) => r[2]));
    const ry0 = Math.min(...k.rects.map((r) => r[1]));
    const ry1 = Math.max(...k.rects.map((r) => r[3]));
    return {
      index,
      id: k.ansi,
      name: k.name,
      leds: k.leds,
      rects: k.rects,
      cx: ((rx0 + rx1) / 2 - x0) / (x1 - x0),
      cy: ((ry0 + ry1) / 2 - y0) / (y1 - y0),
      seed: hash(index * 7919 + 17),
    };
  });
  return { keys, bounds: [x0, y0, x1, y1] };
}

export function hash(n: number): number {
  let x = Math.imul(n ^ 0x9e3779b9, 0x85ebca6b);
  x ^= x >>> 13;
  x = Math.imul(x, 0xc2b2ae35);
  x ^= x >>> 16;
  return (x >>> 0) / 4294967296;
}

export function hsv(h: number, s = 1, v = 1): Rgb {
  h = ((h % 1) + 1) % 1;
  const i = Math.floor(h * 6);
  const f = h * 6 - i;
  const p = v * (1 - s);
  const q = v * (1 - f * s);
  const t = v * (1 - (1 - f) * s);
  const [r, g, b] = [
    [v, t, p],
    [q, v, p],
    [p, v, t],
    [p, q, v],
    [t, p, v],
    [v, p, q],
  ][i % 6];
  return [r * 255, g * 255, b * 255];
}

const scale = (c: Rgb, k: number): Rgb => [c[0] * k, c[1] * k, c[2] * k];
const clamp01 = (v: number) => Math.max(0, Math.min(1, v));

/** A (simulated) key press for the reactive effects' previews. */
export interface Press {
  x: number;
  y: number;
  t: number;
  hue: number;
}

/** Firmware effects that stay dark until a key is pressed (seen on the device). */
export const REACTIVE = new Set([0x04, 0x07, 0x0c, 0x14]);

/** Deterministic "someone is typing" presses so the reactive previews have something to show. */
export function simPresses(keys: KeyGeo[], t: number, every = 0.45): Press[] {
  const out: Press[] = [];
  if (!keys.length) return out;
  const n0 = Math.floor(t / every);
  for (let n = Math.max(0, n0 - 6); n <= n0; n++) {
    const k = keys[Math.floor(hash(n * 104729 + 7) * keys.length)];
    out.push({ x: k.cx, y: k.cy, t: n * every, hue: hash(n * 31 + 3) });
  }
  return out;
}

/**
 * Colour of one key for firmware effect `e` at time `t` (seconds). Modelled on photos of the real
 * keyboard (desk camera, every effect in multicolour mode); still an approximation.
 */
export function shade(e: HwEffect, k: KeyGeo, t: number, presses: Press[] = []): Rgb {
  const speed = 0.35 + (e.speed - 1) * 0.3;
  const dir = e.reverse ? -1 : 1;
  const tt = t * speed;
  const level = 0.25 + e.brightness * 0.1875; // firmware 0..4 -> 0.25..1
  const base = (hue: number): Rgb => (e.colour ? e.colour : hsv(hue));
  const x = e.reverse ? 1 - k.cx : k.cx;
  // keys are ~1/15 of the board wide; the board is ~3x wider than tall
  const dist = (p: Press) => Math.hypot(k.cx - p.x, (k.cy - p.y) * 0.34);
  const age = (p: Press) => (t - p.t) * speed;
  const angle = () => Math.atan2((k.cy - 0.5) * 0.34, k.cx - 0.5) / (Math.PI * 2);
  let c: Rgb = [0, 0, 0];
  let v = 1;
  switch (e.id) {
    case 0x01: // fixed: a still rainbow across the board in multicolour
      c = base(k.cx * 0.9);
      break;
    case 0x02: // respire: the whole board breathes, the colour moves on with each breath
      c = base(0.62 - k.cx * 0.15 + Math.floor((tt * 2.2) / (Math.PI * 2)) * 0.17);
      v = 0.08 + 0.92 * (0.5 - 0.5 * Math.cos(tt * 2.2));
      break;
    case 0x03: // rainbow: the whole board shifts hue together
      c = base(0.16 - tt * 0.12);
      break;
    case 0x04: {
      // flash away: the pressed key's row lights up from the key outwards and fades
      v = 0;
      for (const p of presses) {
        const a = age(p);
        if (a < 0 || a > 1.1 || Math.abs(k.cy - p.y) > 0.08) continue;
        if (Math.abs(k.cx - p.x) <= a * 1.6 + 0.04) v = Math.max(v, 1 - a / 1.1);
      }
      c = base(k.cx * 0.9 + tt * 0.05);
      break;
    }
    case 0x05: {
      // raindrops: single keys light up at random, each in its own colour, and fade
      const ph = tt * 0.55 + k.seed * 11.3;
      const f = ph % 1;
      c = base(hash(Math.floor(ph) * 977 + k.index));
      v = f < 0.18 ? 1 - f / 0.18 : 0;
      break;
    }
    case 0x06: // rainbow wheel: colours turn around the middle of the board
      c = base(angle() * dir - tt * 0.35);
      break;
    case 0x07: {
      // ripples: rings spread from the pressed key across the whole board
      v = 0;
      let hue = 0;
      for (const p of presses) {
        const a = age(p);
        if (a < 0 || a > 1.6) continue;
        const ring = clamp01(1 - Math.abs(dist(p) - a * 0.55) * 22) * (1 - a / 1.6);
        if (ring > v) {
          v = ring;
          hue = p.hue + dist(p) * 2;
        }
      }
      c = base(hue);
      break;
    }
    case 0x08: {
      // stars: random keys twinkle
      const cyc = tt / 2.2 + k.seed * 3.7;
      const ph = cyc % 1;
      c = base(hash(Math.floor(cyc) * 131 + k.index));
      v = ph < 0.12 ? ph / 0.12 : Math.max(0, 1 - (ph - 0.12) / 0.35);
      v *= v;
      break;
    }
    case 0x09: // shadow disappear: every key lit in its own colour, drifting slowly
      c = base(k.seed + tt * 0.04);
      break;
    case 0x0a: {
      // retro snake: a short multicolour snake runs through the rows
      const rows = 5;
      const row = Math.min(rows - 1, Math.floor(k.cy * rows));
      const order = row + (row % 2 ? 1 - x : x);
      const head = (tt * 2.4) % (rows + 1);
      const d = head - order;
      c = base(d * 1.5 + tt * 0.1);
      v = d >= 0 && d < 0.4 ? 1 - d / 0.4 : 0;
      break;
    }
    case 0x0b: // neon stream: a wide rainbow flows across the whole board
      c = base(x * 0.55 - tt * 0.35);
      break;
    case 0x0c: {
      // reaction: only the pressed key lights up, then fades
      v = 0;
      let hue = 0;
      for (const p of presses) {
        const a = age(p);
        if (a < 0 || a > 0.9 || dist(p) > 0.02) continue;
        if (1 - a / 0.9 > v) {
          v = 1 - a / 0.9;
          hue = p.hue;
        }
      }
      c = base(hue);
      break;
    }
    case 0x0d: {
      // sine wave: a thin line of keys rides a moving sine curve
      const yc = 0.5 + 0.4 * Math.sin((x - tt * 0.35) * Math.PI * 2);
      c = base(tt * 0.08 + x * 0.3);
      v = clamp01(1 - Math.abs(k.cy - yc) / 0.14);
      break;
    }
    case 0x0e: {
      // scanning: a wide bar sweeps back and forth, changing colour
      const pos = 0.5 + 0.6 * Math.sin(tt * 1.4);
      c = base(0.6 - tt * 0.06);
      v = clamp01(1 - Math.abs(k.cx - pos) * 3.5);
      break;
    }
    case 0x0f: // windmill: coloured slices turn around the middle, no gaps
      c = base(angle() * 2 * dir - tt * 0.3);
      break;
    case 0x10: // waterfall: each row its own colour, flowing down the board
      c = base(k.cy * 0.7 - tt * 0.4);
      break;
    case 0x11: // blossom: colour rings open out from the middle and fill the board
      c = base(Math.hypot(k.cx - 0.5, (k.cy - 0.5) * 0.34) * 2.6 - tt * 0.4);
      break;
    case 0x12: {
      // rotating storm: scattered keys burst out from the middle, then the board goes dark
      const cyc = (tt * 0.33) % 1;
      const d = Math.hypot(k.cx - 0.5, (k.cy - 0.5) * 0.34);
      c = base(k.seed + angle() + tt * 0.1);
      v = k.seed > 0.4 && cyc < 0.75 ? clamp01(1 - Math.abs(d - cyc * 0.75) * 7) : 0;
      break;
    }
    case 0x13: {
      // collision: two colourful fronts run in from the sides and meet in the middle
      const p = (tt * 0.5) % 1;
      const left = clamp01(1 - Math.abs(k.cx - p * 0.5) * 7);
      const right = clamp01(1 - Math.abs(k.cx - (1 - p * 0.5)) * 7);
      const burst = p > 0.85 ? clamp01(1 - Math.abs(k.cx - 0.5) * 3) * (1 - (p - 0.85) / 0.15) : 0;
      c = base(k.seed * 0.6 + (k.cx < 0.5 ? 0 : 0.5));
      v = Math.max(left, right, burst) * (k.seed > 0.25 ? 1 : 0.3);
      break;
    }
    case 0x14: {
      // perfect: the pressed key and its neighbours light up in a small cross
      v = 0;
      let hue = 0;
      for (const p of presses) {
        const a = age(p);
        if (a < 0 || a > 1) continue;
        const dx = Math.abs(k.cx - p.x);
        const dy = Math.abs(k.cy - p.y);
        const reach = 0.035 + a * 0.12;
        const onCross = (dy < 0.06 && dx < reach) || (dx < 0.04 && dy < reach * 3);
        if (onCross && 1 - a > v) {
          v = 1 - a;
          hue = p.hue + dx * 3;
        }
      }
      c = base(hue);
      break;
    }
    case 0x16: // off
      v = 0;
      break;
    default:
      c = base(k.cx);
  }
  return scale(c, v * level);
}

/** What each firmware effect looks like on the keyboard (checked with a camera, multicolour mode). */
export const DESCRIPTIONS: Record<number, L> = {
  0x01: { tr: 'Bütün tuşlar sabit yanar. Çok renkte klavye boyunca duran bir gökkuşağı olur.', en: 'Every key glows steadily. In multicolour it is a still rainbow across the board.' },
  0x02: { tr: 'Bütün klavye nefes alır gibi yanıp söner; çok renkte her nefeste renk değişir.', en: 'The whole board fades in and out like breathing; in multicolour the colour moves on with every breath.' },
  0x03: { tr: 'Bütün klavye aynı anda yavaşça renkten renge geçer.', en: 'The whole board drifts slowly from colour to colour, all keys together.' },
  0x04: { tr: 'Tuşa basınca o tuşun bütün satırı yanar ve söner. Basmazsan klavye karanlık kalır.', en: 'Pressing a key lights up its whole row, which then fades. Dark until you press a key.' },
  0x05: { tr: 'Rastgele tuşlar tek tek, her biri kendi renginde yanıp söner.', en: 'Random single keys flash, each in its own colour.' },
  0x06: { tr: 'Renkler klavyenin ortası etrafında döner.', en: 'Colours turn around the middle of the board.' },
  0x07: { tr: 'Bastığın tuştan bütün klavyeye halka halka ışık yayılır. Basmazsan karanlık kalır.', en: 'Rings of light spread across the board from the key you press. Dark until you press a key.' },
  0x08: { tr: 'Rastgele tuşlar yıldız gibi pırıldar.', en: 'Random keys twinkle like stars.' },
  0x09: { tr: 'Her tuş kendi renginde yanar, renkler yavaşça kayar.', en: 'Every key glows in its own colour; the colours drift slowly.' },
  0x0a: { tr: 'Kısa, renkli bir ışık yılanı satırlar boyunca hızla koşar.', en: 'A short, colourful snake of light races along the rows.' },
  0x0b: { tr: 'Geniş bir gökkuşağı bütün klavyede yana doğru akar.', en: 'A wide rainbow flows sideways across the whole board.' },
  0x0c: { tr: 'Yalnızca bastığın tuş yanar, sonra söner. Basmazsan karanlık kalır.', en: 'Only the key you press lights up, then fades. Dark until you press a key.' },
  0x0d: { tr: 'İnce bir ışık çizgisi dalga gibi kıvrılarak ilerler.', en: 'A thin line of light weaves across the board like a wave.' },
  0x0e: { tr: 'Geniş bir ışık şeridi sağa sola tarar, rengi değişir.', en: 'A wide bar of light sweeps left and right, changing colour.' },
  0x0f: { tr: 'Renkli dilimler yel değirmeni gibi ortada döner.', en: 'Coloured slices spin around the middle like a windmill.' },
  0x10: { tr: 'Her satır ayrı renkte; renkler şelale gibi aşağı akar.', en: 'Each row has its own colour and the colours pour down like a waterfall.' },
  0x11: { tr: 'Renk halkaları ortadan açılıp bütün klavyeyi doldurur.', en: 'Rings of colour open out from the middle and fill the board.' },
  0x12: { tr: 'Dağınık ışıklar ortadan dışarı patlar, sonra söner.', en: 'Scattered lights burst out from the middle, then fade.' },
  0x13: { tr: 'İki renkli ışık grubu yanlardan gelip ortada çarpışır.', en: 'Two colourful groups of light run in from the sides and collide in the middle.' },
  0x14: { tr: 'Bastığın tuş ve çevresi artı şeklinde yanar. Basmazsan karanlık kalır.', en: 'The key you press and its neighbours light up in a small cross. Dark until you press a key.' },
  0x16: { tr: 'Arka ışık kapanır.', en: 'The backlight is off.' },
};
