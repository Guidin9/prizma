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
  error: string | null;
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
};

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
