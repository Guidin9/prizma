import type { Blend, Layer, Rgb, SwEffect } from './api';
import { tx, type L } from './i18n.svelte';

export type Group = 'anim' | 'react' | 'audio' | 'screen';

export interface SwInfo {
  kind: string;
  name: L;
  group: Group;
  desc: L;
  /** controls this effect uses */
  uses: { colours?: boolean; direction?: string[]; scale?: boolean };
  defaults: Partial<SwEffect>;
  /** static thumbnail */
  thumb: string;
}

export const GROUPS: { id: Group; name: L }[] = [
  { id: 'anim', name: { tr: 'Animasyonlar', en: 'Animations' } },
  { id: 'react', name: { tr: 'Tuşa basınca', en: 'On key press' } },
  { id: 'audio', name: { tr: 'Müzik', en: 'Music' } },
  { id: 'screen', name: { tr: 'Ekran', en: 'Screen' } },
];

const SPECTRUM = 'linear-gradient(90deg,#ff3d6e,#ff9d2e,#ffe14d,#4dff91,#3dd6ff,#6b5cff,#d24dff)';

export const SW: SwInfo[] = [
  { kind: 'solid', name: { tr: 'Düz renk', en: 'Solid colour' }, group: 'anim', desc: { tr: 'Seçtiğin tuşlar tek renkte sabit yanar. WASD gibi vurgular için.', en: 'The keys you pick glow in one steady colour. Made for highlights like WASD.' }, uses: { colours: true }, defaults: { colours: [[255, 40, 60]] }, thumb: 'linear-gradient(90deg,#ff2840,#ff2840)' },
  { kind: 'wave', name: { tr: 'Gökkuşağı dalgası', en: 'Rainbow wave' }, group: 'anim', desc: { tr: 'Renkler seçtiğin yönde akar.', en: 'Colours flow in the direction you pick.' }, uses: { colours: true, direction: ['right', 'left', 'up', 'down', 'diag', 'out', 'in'], scale: true }, defaults: { direction: 'right' }, thumb: SPECTRUM },
  { kind: 'spiral', name: { tr: 'Sarmal', en: 'Spiral' }, group: 'anim', desc: { tr: 'Renkler ortadan sarmal çizerek döner.', en: 'Colours spin out from the middle in a spiral.' }, uses: { colours: true, direction: ['right', 'left'], scale: true }, defaults: {}, thumb: 'conic-gradient(from 90deg,#ff3d6e,#ffe14d,#4dff91,#3dd6ff,#d24dff,#ff3d6e)' },
  { kind: 'aurora', name: { tr: 'Aurora', en: 'Aurora' }, group: 'anim', desc: { tr: 'Kuzey ışıkları gibi yumuşak renk perdeleri.', en: 'Soft curtains of colour, like the northern lights.' }, uses: { colours: true, scale: true }, defaults: { speed: 0.8 }, thumb: 'linear-gradient(120deg,#0b3,#14ffa0 30%,#2878ff 60%,#aa3cff)' },
  { kind: 'plasma', name: { tr: 'Plazma', en: 'Plasma' }, group: 'anim', desc: { tr: 'Kıvrılan, iç içe geçen renk dalgaları.', en: 'Swirling, interlocking waves of colour.' }, uses: { colours: true, scale: true }, defaults: {}, thumb: 'radial-gradient(circle at 30% 40%,#ff3dcb,transparent 45%),radial-gradient(circle at 70% 60%,#3dd6ff,transparent 45%),#2a1a5a' },
  { kind: 'breathe', name: { tr: 'Nefes gradyanı', en: 'Breathing gradient' }, group: 'anim', desc: { tr: 'Renkler arasında nefes alır gibi geçiş yapar.', en: 'Breathes slowly from one colour to the next.' }, uses: { colours: true }, defaults: { colours: [[255, 61, 110], [61, 214, 255]] }, thumb: 'linear-gradient(90deg,#ff3d6e,#1a1630 50%,#3dd6ff)' },
  { kind: 'stars', name: { tr: 'Yıldız tozu', en: 'Stardust' }, group: 'anim', desc: { tr: 'Tuşlar rastgele yıldız gibi parlar.', en: 'Keys sparkle at random, like stars.' }, uses: { colours: true }, defaults: {}, thumb: 'radial-gradient(circle at 20% 30%,#fff 0 2px,transparent 3px),radial-gradient(circle at 70% 60%,#ffe14d 0 2px,transparent 3px),radial-gradient(circle at 45% 80%,#3dd6ff 0 2px,transparent 3px),#120f20' },
  { kind: 'matrix', name: { tr: 'Matrix yağmuru', en: 'Matrix rain' }, group: 'anim', desc: { tr: 'Yeşil karakter yağmuru gibi düşen ışık.', en: 'Light falling like green code rain.' }, uses: { colours: true }, defaults: {}, thumb: 'repeating-linear-gradient(90deg,#062,#062 6px,#28ff5a 7px,#062 9px)' },
  { kind: 'fire', name: { tr: 'Ateş', en: 'Fire' }, group: 'anim', desc: { tr: 'Alt sıradan yükselen alevler.', en: 'Flames rising from the bottom row.' }, uses: { colours: true, scale: true }, defaults: {}, thumb: 'linear-gradient(0deg,#ffd23c,#ff2800 45%,#300)' },
  { kind: 'ripple', name: { tr: 'Dalga', en: 'Ripple' }, group: 'react', desc: { tr: 'Bastığın tuştan halka yayılır.', en: 'A ring spreads out from every key you press.' }, uses: { colours: true, scale: true }, defaults: {}, thumb: 'radial-gradient(circle,transparent 30%,#3dd6ff 36%,transparent 42%,transparent 60%,#d24dff 66%,transparent 72%),#120f20' },
  { kind: 'trail', name: { tr: 'İz', en: 'Trail' }, group: 'react', desc: { tr: 'Bastığın tuş yanar, yavaşça söner.', en: 'Pressed keys light up and slowly fade.' }, uses: { colours: true }, defaults: {}, thumb: 'linear-gradient(90deg,#120f20,#6b5cff,#fff)' },
  { kind: 'heatmap', name: { tr: 'Isı haritası', en: 'Heatmap' }, group: 'react', desc: { tr: 'En çok bastığın tuşlar kırmızıya döner.', en: 'The keys you use most turn red.' }, uses: {}, defaults: {}, thumb: 'linear-gradient(90deg,#001e,#2878ff,#4dff91,#ffe14d,#ff2800)' },
  { kind: 'spectrum', name: { tr: 'Spektrum', en: 'Spectrum' }, group: 'audio', desc: { tr: 'Her sütun bir frekansı, yükseklik sesi gösterir.', en: 'Each column is a frequency band; its height is the loudness.' }, uses: { colours: true }, defaults: {}, thumb: 'linear-gradient(0deg,#4dff91 0 30%,transparent 30%) 0 0/12% 100% no-repeat,linear-gradient(0deg,#ffe14d 0 70%,transparent 70%) 20% 0/12% 100% no-repeat,linear-gradient(0deg,#ff3d6e 0 50%,transparent 50%) 40% 0/12% 100% no-repeat,linear-gradient(0deg,#3dd6ff 0 85%,transparent 85%) 60% 0/12% 100% no-repeat,linear-gradient(0deg,#d24dff 0 40%,transparent 40%) 80% 0/12% 100% no-repeat,#120f20' },
  { kind: 'bass', name: { tr: 'Bas vuruşu', en: 'Bass pulse' }, group: 'audio', desc: { tr: 'Bas vurdukça klavye ortadan patlar.', en: 'The keyboard bursts from the middle on every bass hit.' }, uses: { colours: true }, defaults: {}, thumb: 'radial-gradient(circle,#ff3d6e,#6b5cff 50%,#120f20 75%)' },
  { kind: 'vu', name: { tr: 'Ses seviyesi', en: 'VU meter' }, group: 'audio', desc: { tr: 'Ses yükseldikçe ışık ortadan yanlara uzar.', en: 'Light grows from the middle outwards as the sound gets louder.' }, uses: { colours: true }, defaults: {}, thumb: 'linear-gradient(90deg,#ff2800,#ffe14d 25%,#4dff91 45%,#4dff91 55%,#ffe14d 75%,#ff2800)' },
  { kind: 'ambient', name: { tr: 'Ekran rengi', en: 'Screen colour' }, group: 'screen', desc: { tr: 'Ekranın her bölgesinin rengi altındaki tuşlara geçer.', en: 'Each part of the screen lends its colour to the keys below it.' }, uses: {}, defaults: { speed: 1 }, thumb: 'linear-gradient(90deg,#1e3a8a,#be185d,#f59e0b)' },
];

export const PALETTES: { name: L; colours: Rgb[] }[] = [
  { name: { tr: 'Spektrum', en: 'Spectrum' }, colours: [] },
  { name: { tr: 'Neon', en: 'Neon' }, colours: [[255, 61, 200], [61, 214, 255]] },
  { name: { tr: 'Okyanus', en: 'Ocean' }, colours: [[20, 60, 255], [0, 220, 220], [120, 255, 220]] },
  { name: { tr: 'Gün batımı', en: 'Sunset' }, colours: [[255, 120, 40], [255, 50, 110], [140, 40, 255]] },
  { name: { tr: 'Orman', en: 'Forest' }, colours: [[30, 255, 90], [160, 255, 40], [0, 140, 60]] },
  { name: { tr: 'Ateş', en: 'Fire' }, colours: [[255, 30, 0], [255, 140, 0], [255, 230, 80]] },
  { name: { tr: 'Buz', en: 'Ice' }, colours: [[255, 255, 255], [120, 230, 255], [40, 100, 255]] },
];

export const DIRECTIONS: Record<string, L> = {
  right: { tr: 'Sağa', en: 'Right' },
  left: { tr: 'Sola', en: 'Left' },
  up: { tr: 'Yukarı', en: 'Up' },
  down: { tr: 'Aşağı', en: 'Down' },
  diag: { tr: 'Çapraz', en: 'Diagonal' },
  out: { tr: 'Dışa', en: 'Outwards' },
  in: { tr: 'İçe', en: 'Inwards' },
};

export function makeEffect(info: SwInfo, prev?: SwEffect): SwEffect {
  return {
    kind: info.kind,
    speed: info.defaults.speed ?? prev?.speed ?? 1,
    brightness: prev?.brightness ?? 1,
    colours: info.defaults.colours ?? (info.uses.colours ? prev?.colours ?? [] : []),
    direction: info.defaults.direction ?? (info.uses.direction?.includes(prev?.direction ?? '') ? prev!.direction : info.uses.direction?.[0] ?? 'right'),
    scale: prev?.scale ?? 1,
  };
}

/** Reactive and music layers usually sit on top and only add light. */
export function defaultBlend(kind: string): Blend {
  const g = SW.find((s) => s.kind === kind)?.group;
  return g === 'react' || g === 'audio' ? 'add' : 'normal';
}

export const BLENDS: { id: Blend; name: L; hint: L }[] = [
  { id: 'normal', name: { tr: 'Üstüne', en: 'Cover' }, hint: { tr: 'Alttaki katmanları örter.', en: 'Covers the layers below.' } },
  { id: 'add', name: { tr: 'Parlat', en: 'Add' }, hint: { tr: 'Işığını alttakinin üstüne ekler; karanlık yerleri alttakini göstermeye devam eder.', en: 'Adds its light on top; where it is dark, the layer below still shows.' } },
  { id: 'max', name: { tr: 'Aydınlat', en: 'Lighten' }, hint: { tr: 'Her tuşta hangisi daha parlaksa o görünür.', en: 'On each key, whichever is brighter wins.' } },
];

export function makeLayer(info: SwInfo, prev?: SwEffect): Layer {
  return { effect: makeEffect(info, prev), opacity: 1, blend: defaultBlend(info.kind), keys: [], visible: true };
}

/** Quick key groups, by layout key id (LayoutKey.ansi; Turkish letters sit on the same ids). */
export const KEY_GROUPS: { id: string; name: L; keys: string[] }[] = [
  { id: 'wasd', name: { tr: 'WASD', en: 'WASD' }, keys: ['W', 'A', 'S', 'D'] },
  { id: 'num', name: { tr: 'Rakamlar', en: 'Numbers' }, keys: ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '='] },
  {
    id: 'letters',
    name: { tr: 'Harfler', en: 'Letters' },
    keys: ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '[', ']', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ';', "'", 'ISO#', 'ISO\\', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', ',', '.', '/', '\\'],
  },
  { id: 'mods', name: { tr: 'Kenar tuşları', en: 'Modifiers' }, keys: ['Esc', 'Backspace', 'Tab', 'CapsLock', 'Enter', 'LShift', 'RShift', 'LCtrl', 'LWin', 'LAlt', 'RAlt', 'Fn', 'App', 'RCtrl'] },
  { id: 'space', name: { tr: 'Boşluk', en: 'Space bar' }, keys: ['Space'] },
];

/** One layer holding a single effect: what the Live view used before layers existed. */
export function sceneOf(effect: SwEffect): { layers: Layer[] } {
  return { layers: [{ effect, opacity: 1, blend: 'normal', keys: [], visible: true }] };
}

/** Names of the keyboard's own (firmware) effects, by catalogue key. */
export const HW_NAMES: Record<string, L> = {
  fixed: { tr: 'Sabit', en: 'Static' },
  respire: { tr: 'Nefes', en: 'Breathing' },
  rainbow: { tr: 'Gökkuşağı', en: 'Rainbow' },
  flash_away: { tr: 'Uçup giden', en: 'Flash away' },
  raindrops: { tr: 'Yağmur damlaları', en: 'Raindrops' },
  rainbow_wheel: { tr: 'Gökkuşağı çarkı', en: 'Rainbow wheel' },
  ripples: { tr: 'Parlayan halkalar', en: 'Shining ripples' },
  stars: { tr: 'Yıldız pırıltısı', en: 'Twinkling stars' },
  shadow: { tr: 'Kaybolan gölge', en: 'Fading shadow' },
  snake: { tr: 'Retro yılan', en: 'Retro snake' },
  neon: { tr: 'Neon akış', en: 'Neon stream' },
  reaction: { tr: 'Tepki', en: 'Reaction' },
  sine: { tr: 'Sinüs dalgası', en: 'Sine wave' },
  scan: { tr: 'Tarama', en: 'Scanning' },
  windmill: { tr: 'Yel değirmeni', en: 'Windmill' },
  waterfall: { tr: 'Renkli şelale', en: 'Colour waterfall' },
  blossom: { tr: 'Çiçek açma', en: 'Blossom' },
  storm: { tr: 'Dönen fırtına', en: 'Rotating storm' },
  collision: { tr: 'Çarpışma', en: 'Collision' },
  perfect: { tr: 'Mükemmel', en: 'Perfect' },
  off: { tr: 'Kapalı', en: 'Off' },
};

/** Display name of a firmware effect (the core's catalogue names are Turkish). */
export const hwName = (e: { key: string; name: string }) => (HW_NAMES[e.key] ? tx(HW_NAMES[e.key]) : e.name);

/** Layout names, by layout id. */
export const LAYOUT_NAMES: Record<string, L> = {
  tr_iso: { tr: 'Türkçe Q (ISO)', en: 'Turkish Q (ISO)' },
  ansi: { tr: 'ABD (ANSI)', en: 'US (ANSI)' },
};
