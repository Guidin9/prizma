// Approximate on-screen previews of the firmware effects. The keyboard renders its own
// animation; these only show the character of each effect on the stage.
import type { HwEffect, Layout, Rgb } from './api';

export interface KeyGeo {
  index: number;
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

/** Colour of one key for effect `e` at time `t` (seconds). */
export function shade(e: HwEffect, k: KeyGeo, t: number): Rgb {
  const speed = 0.35 + (e.speed - 1) * 0.3;
  const dir = e.reverse ? -1 : 1;
  const tt = t * speed;
  const level = 0.25 + e.brightness * 0.1875; // firmware 0..4 -> 0.25..1
  const base = (hue: number): Rgb => (e.colour ? e.colour : hsv(hue));
  const x = e.reverse ? 1 - k.cx : k.cx;
  let c: Rgb;
  let v = 1;
  switch (e.id) {
    case 0x01: // fixed
      c = base(k.cx * 0.9);
      break;
    case 0x02: // respire
      c = base(tt * 0.15);
      v = 0.15 + 0.85 * (0.5 - 0.5 * Math.cos(tt * 2.2));
      break;
    case 0x03: // rainbow wave
      c = e.colour ? e.colour : hsv(x * 0.9 - tt * 0.5);
      v = e.colour ? 0.35 + 0.65 * (0.5 + 0.5 * Math.sin((x - tt * 0.5) * Math.PI * 2)) : 1;
      break;
    case 0x04: // flash away
    case 0x08: {
      // stars: random keys flare and fade
      const period = e.id === 0x08 ? 2.6 : 1.4;
      const ph = (tt / period + k.seed * 3.7) % 1;
      c = base(k.seed + tt * 0.05);
      v = ph < 0.12 ? ph / 0.12 : Math.max(0, 1 - (ph - 0.12) / 0.45);
      v = e.id === 0x08 ? v * v : v;
      break;
    }
    case 0x05: {
      // raindrops falling per column
      const col = Math.floor(k.cx * 14);
      const drop = (tt * 0.9 + hash(col * 31) * 5) % 1.6;
      const d = drop - k.cy;
      c = base(hash(col * 13) + tt * 0.03);
      v = d >= 0 && d < 0.45 ? 1 - d / 0.45 : 0.05;
      break;
    }
    case 0x06: {
      // rainbow wheel around the centre
      const a = Math.atan2((k.cy - 0.5) * 0.45, k.cx - 0.5) / (Math.PI * 2);
      c = e.colour ? e.colour : hsv(a * dir - tt * 0.35);
      if (e.colour) v = 0.3 + 0.7 * (0.5 + 0.5 * Math.sin((a * dir - tt * 0.35) * Math.PI * 2));
      break;
    }
    case 0x07:
    case 0x11: {
      // ripples / blossom: rings from the centre
      const d = Math.hypot(k.cx - 0.5, (k.cy - 0.5) * 0.4);
      const r = (tt * 0.45) % 0.8;
      c = base(d * 1.4 - tt * 0.3);
      v = e.id === 0x11 ? clamp01(1 - Math.abs(d - r) * 6) * 0.9 + 0.1 : clamp01(1 - Math.abs(d - r) * 9);
      break;
    }
    case 0x09: {
      // shadow disappear: lit keys fade away in a sweep
      const sweep = (tt * 0.4) % 1.3;
      c = base(k.cx);
      v = clamp01((x - sweep) * 4 + 1);
      break;
    }
    case 0x0a: {
      // snake running along rows
      const order = k.cy * 5 + (Math.floor(k.cy * 5) % 2 ? 1 - k.cx : k.cx);
      const head = (tt * 1.1) % 5.5;
      const d = head - order;
      c = base(order * 0.18);
      v = d >= 0 && d < 0.9 ? 1 - d / 0.9 : 0.04;
      break;
    }
    case 0x0b: // neon stream: diagonal flow
      c = e.colour ? e.colour : hsv((x + k.cy * 0.35) * 0.7 - tt * 0.4);
      v = e.colour ? 0.3 + 0.7 * (0.5 + 0.5 * Math.sin((x + k.cy * 0.35 - tt * 0.4) * Math.PI * 4)) : 1;
      break;
    case 0x0c: {
      // reaction: dark until a key is "pressed"
      const ph = (tt * 0.5 + k.seed * 9.1) % 3;
      c = base(k.seed);
      v = ph < 0.6 ? 1 - ph / 0.6 : 0.03;
      break;
    }
    case 0x0d: // sine wave
      c = base(x * 0.6 - tt * 0.2);
      v = clamp01(0.5 + 0.5 * Math.sin((x * 2 - tt) * Math.PI * 2 + k.cy * 3));
      break;
    case 0x0e: {
      // scanning bar back and forth
      const pos = 0.5 + 0.55 * Math.sin(tt * 1.6);
      c = base(0.55 + pos * 0.3);
      v = clamp01(1 - Math.abs(k.cx - pos) * 5);
      break;
    }
    case 0x0f: {
      // windmill blades
      const a = Math.atan2((k.cy - 0.5) * 0.45, k.cx - 0.5);
      c = base(a / (Math.PI * 2) + tt * 0.1);
      v = clamp01(Math.cos(a * 4 - tt * dir * 2.4) * 1.5);
      break;
    }
    case 0x10: // waterfall
      c = e.colour ? e.colour : hsv(k.cy * 0.6 - tt * 0.4 + k.cx * 0.15);
      v = e.colour ? 0.3 + 0.7 * (0.5 + 0.5 * Math.sin((k.cy - tt * 0.6) * Math.PI * 3)) : 1;
      break;
    case 0x12: {
      // rotating storm: spiral
      const a = Math.atan2((k.cy - 0.5) * 0.45, k.cx - 0.5) / (Math.PI * 2);
      const d = Math.hypot(k.cx - 0.5, (k.cy - 0.5) * 0.45);
      c = e.colour ? e.colour : hsv(a * dir + d * 2 - tt * 0.5);
      if (e.colour) v = 0.25 + 0.75 * (0.5 + 0.5 * Math.sin((a * dir + d * 2 - tt * 0.5) * Math.PI * 4));
      break;
    }
    case 0x13: {
      // collision: two fronts meet in the middle
      const p = (tt * 0.5) % 1;
      const left = clamp01(1 - Math.abs(k.cx - p * 0.5) * 8);
      const right = clamp01(1 - Math.abs(k.cx - (1 - p * 0.5)) * 8);
      const burst = p > 0.85 ? clamp01(1 - Math.abs(k.cx - 0.5) * 3) * (1 - (p - 0.85) / 0.15) : 0;
      c = base(k.cx < 0.5 ? 0.0 : 0.6);
      v = Math.max(left, right, burst);
      break;
    }
    case 0x14: // perfect: rainbow that breathes
      c = base(k.cx * 0.9 - tt * 0.3);
      v = 0.35 + 0.65 * (0.5 - 0.5 * Math.cos(tt * 1.4));
      break;
    case 0x16: // off
      c = [0, 0, 0];
      v = 0;
      break;
    default:
      c = base(k.cx);
  }
  return scale(c, v * level);
}

/** Short, honest descriptions of what each firmware effect does. */
export const DESCRIPTIONS: Record<number, string> = {
  0x01: 'Bütün tuşlar sabit yanar.',
  0x02: 'Işık yavaşça yanıp söner, nefes alır gibi.',
  0x03: 'Renkler klavye boyunca dalga halinde akar.',
  0x04: 'Tuşlar tek tek parlayıp uzaklaşır.',
  0x05: 'Işık damlaları yukarıdan aşağı düşer.',
  0x06: 'Renkler klavyenin ortası etrafında döner.',
  0x07: 'Halkalar ortadan dışarı yayılır.',
  0x08: 'Rastgele tuşlar yıldız gibi pırıldar.',
  0x09: 'Yanan tuşlar bir taraftan kaybolur.',
  0x0a: 'Bir ışık yılanı satırlar boyunca kıvrılır.',
  0x0b: 'Neon renkler çapraz akar.',
  0x0c: 'Bastığın tuş yanar, sonra söner.',
  0x0d: 'Parlaklık sinüs dalgası gibi ilerler.',
  0x0e: 'Bir ışık şeridi sağa sola tarar.',
  0x0f: 'Işık kanatları yel değirmeni gibi döner.',
  0x10: 'Renkler şelale gibi akar.',
  0x11: 'Işık ortadan çiçek gibi açılır.',
  0x12: 'Renkler sarmal halinde döner.',
  0x13: 'İki ışık cephesi ortada çarpışır.',
  0x14: 'Gökkuşağı nefes alır gibi parlar.',
  0x16: 'Arka ışık kapanır.',
};
