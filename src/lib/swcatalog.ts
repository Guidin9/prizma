import type { Rgb, SwEffect } from './api';

export type Group = 'anim' | 'react' | 'audio' | 'screen';

export interface SwInfo {
  kind: string;
  name: string;
  group: Group;
  desc: string;
  /** controls this effect uses */
  uses: { colours?: boolean; direction?: string[]; scale?: boolean };
  defaults: Partial<SwEffect>;
  /** static thumbnail */
  thumb: string;
}

export const GROUPS: { id: Group; name: string; hint: string }[] = [
  { id: 'anim', name: 'Animasyonlar', hint: '' },
  { id: 'react', name: 'Tuşa basınca', hint: 'Bastığın tuşa tepki verir.' },
  { id: 'audio', name: 'Müzik', hint: 'Bilgisayardan çalan sese göre hareket eder.' },
  { id: 'screen', name: 'Ekran', hint: 'Ekrandaki renkleri klavyeye yansıtır.' },
];

const SPECTRUM = 'linear-gradient(90deg,#ff3d6e,#ff9d2e,#ffe14d,#4dff91,#3dd6ff,#6b5cff,#d24dff)';

export const SW: SwInfo[] = [
  { kind: 'wave', name: 'Gökkuşağı dalgası', group: 'anim', desc: 'Renkler seçtiğin yönde akar.', uses: { colours: true, direction: ['right', 'left', 'up', 'down', 'diag', 'out', 'in'], scale: true }, defaults: { direction: 'right' }, thumb: SPECTRUM },
  { kind: 'spiral', name: 'Sarmal', group: 'anim', desc: 'Renkler ortadan sarmal çizerek döner.', uses: { colours: true, direction: ['right', 'left'], scale: true }, defaults: {}, thumb: 'conic-gradient(from 90deg,#ff3d6e,#ffe14d,#4dff91,#3dd6ff,#d24dff,#ff3d6e)' },
  { kind: 'aurora', name: 'Aurora', group: 'anim', desc: 'Kuzey ışıkları gibi yumuşak renk perdeleri.', uses: { colours: true, scale: true }, defaults: { speed: 0.8 }, thumb: 'linear-gradient(120deg,#0b3,#14ffa0 30%,#2878ff 60%,#aa3cff)' },
  { kind: 'plasma', name: 'Plazma', group: 'anim', desc: 'Kıvrılan, iç içe geçen renk dalgaları.', uses: { colours: true, scale: true }, defaults: {}, thumb: 'radial-gradient(circle at 30% 40%,#ff3dcb,transparent 45%),radial-gradient(circle at 70% 60%,#3dd6ff,transparent 45%),#2a1a5a' },
  { kind: 'breathe', name: 'Nefes gradyanı', group: 'anim', desc: 'Renkler arasında nefes alır gibi geçiş yapar.', uses: { colours: true }, defaults: { colours: [[255, 61, 110], [61, 214, 255]] }, thumb: 'linear-gradient(90deg,#ff3d6e,#1a1630 50%,#3dd6ff)' },
  { kind: 'stars', name: 'Yıldız tozu', group: 'anim', desc: 'Tuşlar rastgele yıldız gibi parlar.', uses: { colours: true }, defaults: {}, thumb: 'radial-gradient(circle at 20% 30%,#fff 0 2px,transparent 3px),radial-gradient(circle at 70% 60%,#ffe14d 0 2px,transparent 3px),radial-gradient(circle at 45% 80%,#3dd6ff 0 2px,transparent 3px),#120f20' },
  { kind: 'matrix', name: 'Matrix yağmuru', group: 'anim', desc: 'Yeşil karakter yağmuru gibi düşen ışık.', uses: { colours: true }, defaults: {}, thumb: 'repeating-linear-gradient(90deg,#062,#062 6px,#28ff5a 7px,#062 9px)' },
  { kind: 'fire', name: 'Ateş', group: 'anim', desc: 'Alt sıradan yükselen alevler.', uses: { colours: true, scale: true }, defaults: {}, thumb: 'linear-gradient(0deg,#ffd23c,#ff2800 45%,#300)' },
  { kind: 'ripple', name: 'Dalga', group: 'react', desc: 'Bastığın tuştan halka yayılır.', uses: { colours: true, scale: true }, defaults: {}, thumb: 'radial-gradient(circle,transparent 30%,#3dd6ff 36%,transparent 42%,transparent 60%,#d24dff 66%,transparent 72%),#120f20' },
  { kind: 'trail', name: 'İz', group: 'react', desc: 'Bastığın tuş yanar, yavaşça söner.', uses: { colours: true }, defaults: {}, thumb: 'linear-gradient(90deg,#120f20,#6b5cff,#fff)' },
  { kind: 'heatmap', name: 'Isı haritası', group: 'react', desc: 'En çok bastığın tuşlar kırmızıya döner.', uses: {}, defaults: {}, thumb: 'linear-gradient(90deg,#001e,#2878ff,#4dff91,#ffe14d,#ff2800)' },
  { kind: 'spectrum', name: 'Spektrum', group: 'audio', desc: 'Her sütun bir frekansı, yükseklik sesi gösterir.', uses: { colours: true }, defaults: {}, thumb: 'linear-gradient(0deg,#4dff91 0 30%,transparent 30%) 0 0/12% 100% no-repeat,linear-gradient(0deg,#ffe14d 0 70%,transparent 70%) 20% 0/12% 100% no-repeat,linear-gradient(0deg,#ff3d6e 0 50%,transparent 50%) 40% 0/12% 100% no-repeat,linear-gradient(0deg,#3dd6ff 0 85%,transparent 85%) 60% 0/12% 100% no-repeat,linear-gradient(0deg,#d24dff 0 40%,transparent 40%) 80% 0/12% 100% no-repeat,#120f20' },
  { kind: 'bass', name: 'Bas vuruşu', group: 'audio', desc: 'Bas vurdukça klavye ortadan patlar.', uses: { colours: true }, defaults: {}, thumb: 'radial-gradient(circle,#ff3d6e,#6b5cff 50%,#120f20 75%)' },
  { kind: 'vu', name: 'Ses seviyesi', group: 'audio', desc: 'Ses yükseldikçe ışık ortadan yanlara uzar.', uses: { colours: true }, defaults: {}, thumb: 'linear-gradient(90deg,#ff2800,#ffe14d 25%,#4dff91 45%,#4dff91 55%,#ffe14d 75%,#ff2800)' },
  { kind: 'ambient', name: 'Ekran rengi', group: 'screen', desc: 'Ekranın her bölgesinin rengi altındaki tuşlara geçer.', uses: {}, defaults: { speed: 1 }, thumb: 'linear-gradient(90deg,#1e3a8a,#be185d,#f59e0b)' },
];

export const PALETTES: { name: string; colours: Rgb[] }[] = [
  { name: 'Spektrum', colours: [] },
  { name: 'Neon', colours: [[255, 61, 200], [61, 214, 255]] },
  { name: 'Okyanus', colours: [[20, 60, 255], [0, 220, 220], [120, 255, 220]] },
  { name: 'Gün batımı', colours: [[255, 120, 40], [255, 50, 110], [140, 40, 255]] },
  { name: 'Orman', colours: [[30, 255, 90], [160, 255, 40], [0, 140, 60]] },
  { name: 'Ateş', colours: [[255, 30, 0], [255, 140, 0], [255, 230, 80]] },
  { name: 'Buz', colours: [[255, 255, 255], [120, 230, 255], [40, 100, 255]] },
];

export const DIRECTIONS: Record<string, string> = {
  right: 'Sağa',
  left: 'Sola',
  up: 'Yukarı',
  down: 'Aşağı',
  diag: 'Çapraz',
  out: 'Dışa',
  in: 'İçe',
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
