import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type Rgb = [number, number, number];

export interface EffectInfo {
  id: number;
  key: string;
  name: string;
  animated: boolean;
  colour: boolean;
}

export interface HwEffect {
  id: number;
  colour: Rgb | null;
  speed: number; // 1..5
  brightness: number; // 0..4
  reverse: boolean;
}

export interface Status {
  connected: boolean;
  lock: boolean | null;
  effect: HwEffect | null;
  /** settings the keyboard keeps for every effect; each effect remembers its own */
  stored: HwEffect[];
  engine: 'off' | 'preview' | 'live';
  error: string | null;
}

export interface SwEffect {
  kind: string;
  speed: number; // 0.25..3
  brightness: number; // 0..1
  colours: Rgb[]; // empty = full spectrum
  direction: string;
  scale: number;
}

/** How a layer combines with the layers below it. */
export type Blend = 'normal' | 'add' | 'max';

export interface Layer {
  effect: SwEffect;
  opacity: number; // 0..1
  blend: Blend;
  /** key ids (LayoutKey.ansi) the layer covers; empty = whole keyboard */
  keys: string[];
  visible: boolean;
}

/** Layers bottom first. */
export interface Scene {
  layers: Layer[];
}

export interface SavedScene {
  name: string;
  scene: Scene;
}

export interface KbdEvent {
  kind: 'fn_down' | 'fn_up' | 'lock_toggle' | 'effect_key' | 'brightness_key' | 'speed_key' | 'other';
  raw: number[];
}

export const api = {
  status: () => invoke<Status>('status'),
  catalog: () => invoke<EffectInfo[]>('effect_catalog'),
  layouts: async () => JSON.parse(await invoke<string>('layouts')) as { layouts: Layout[] },
  applyEffect: (effect: HwEffect) => invoke<HwEffect>('apply_effect', { effect }),
  setLayout: (id: string) => invoke<void>('set_layout', { id }),
  setVisible: (visible: boolean) => invoke<void>('set_visible', { visible }),
  enginePreview: (scene: Scene | null) => invoke<void>('engine_preview', { scene }),
  engineStart: (scene: Scene) => invoke<void>('engine_start', { scene }),
  engineStop: () => invoke<HwEffect>('engine_stop'),
  scenes: () => invoke<SavedScene[]>('scenes_list'),
  setLanguage: (lang: 'tr' | 'en') => invoke<void>('set_language', { lang }),
  saveScene: (name: string, scene: Scene) => invoke<SavedScene[]>('scene_save', { name, scene }),
  deleteScene: (name: string) => invoke<SavedScene[]>('scene_delete', { name }),
};

export function onEngineFrame(cb: (keys: Rgb[]) => void): Promise<UnlistenFn> {
  return listen<Rgb[]>('engine-frame', (e) => cb(e.payload));
}

export function onEngineError(cb: (msg: string) => void): Promise<UnlistenFn> {
  return listen<string>('engine-error', (e) => cb(e.payload));
}

export function onLock(cb: (lock: boolean | null) => void): Promise<UnlistenFn> {
  return listen<boolean | null>('kbd-lock', (e) => cb(e.payload));
}

export function onConnection(cb: (connected: boolean) => void): Promise<UnlistenFn> {
  return listen<boolean>('kbd-connection', (e) => cb(e.payload));
}

export function onKbdEvent(cb: (ev: KbdEvent) => void): Promise<UnlistenFn> {
  return listen<KbdEvent>('kbd-event', (e) => cb(e.payload));
}

export interface LayoutKey {
  name: string;
  ansi: string;
  leds: number[];
  rects: [number, number, number, number][];
}

export interface Layout {
  id: string;
  name: string;
  keys: LayoutKey[];
}

export const sameEffect = (a: HwEffect | null, b: HwEffect | null) =>
  !!a &&
  !!b &&
  a.id === b.id &&
  a.speed === b.speed &&
  a.brightness === b.brightness &&
  a.reverse === b.reverse &&
  JSON.stringify(a.colour) === JSON.stringify(b.colour);
