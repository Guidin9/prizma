<script lang="ts">
  import { onMount } from 'svelte';
  import {
    api,
    onConnection,
    onEngineError,
    onEngineFrame,
    onKbdEvent,
    onLock,
    sameEffect,
    type EffectInfo,
    type HwEffect,
    type Layout,
    type Rgb,
    type SavedScene,
    type Scene,
    type SwEffect,
  } from '$lib/api';
  import { REACTIVE, geometry, type KeyGeo } from '$lib/preview';
  import KeyPressIcon from '$lib/components/KeyPressIcon.svelte';
  import { GROUPS, KEY_GROUPS, LAYOUT_NAMES, SW, defaultBlend, makeEffect, makeLayer, sceneOf } from '$lib/swcatalog';
  import { enable as enableAutostart, disable as disableAutostart, isEnabled as autostartEnabled } from '@tauri-apps/plugin-autostart';
  import { errText, i18n, setLang, t, tx, type Lang } from '$lib/i18n.svelte';
  import Titlebar from '$lib/components/Titlebar.svelte';
  import KeyboardStage from '$lib/components/KeyboardStage.svelte';
  import EffectTile from '$lib/components/EffectTile.svelte';
  import Inspector from '$lib/components/Inspector.svelte';
  import LayerPanel from '$lib/components/LayerPanel.svelte';

  const LABELS: Record<string, string> = {
    LShift: 'Shift', RShift: 'Shift', CapsLock: 'Caps', Backspace: '⌫', LCtrl: 'Ctrl', RCtrl: 'Ctrl',
    LWin: 'Win', LAlt: 'Alt', RAlt: 'AltGr', Space: '', Enter: 'Enter', Esc: 'Esc', Tab: 'Tab',
  };
  const LANGS: { id: Lang; name: string }[] = [
    { id: 'tr', name: 'Türkçe' },
    { id: 'en', name: 'English' },
  ];
  let autostart = $state(false);
  const load = <T,>(key: string, fallback: T): T => {
    try {
      const v = localStorage.getItem(key);
      return v ? (JSON.parse(v) as T) : fallback;
    } catch {
      return fallback;
    }
  };

  let layouts: Layout[] = $state([]);
  let layoutId = $state(load('prizma.layout', 'tr_iso'));
  let catalog: EffectInfo[] = $state([]);
  let connected = $state(false);
  let lock: boolean | null = $state(null);
  let applied: HwEffect | null = $state(null);
  /** what the keyboard has stored for each effect id (colour, speed, brightness, direction) */
  let stored: Record<number, HwEffect> = $state({});
  let draft: HwEffect = $state({ id: 0x03, colour: null, speed: 3, brightness: 4, reverse: false });
  // the Live view edits a scene: layers bottom first. Older versions kept one effect ('prizma.sw').
  const firstScene = (): Scene => {
    const old = load<SwEffect | null>('prizma.sw', null);
    return sceneOf(old ?? makeEffect(SW.find((s) => s.kind === 'wave') ?? SW[0]));
  };
  const storedScene = load<Scene | null>('prizma.scene', null);
  const initialScene = storedScene?.layers?.length ? storedScene : firstScene();
  let scene: Scene = $state(initialScene);
  let sel = $state(initialScene.layers.length - 1);
  let sceneName: string | null = $state(load<string | null>('prizma.sceneName', null));
  let saved: SavedScene[] = $state([]);
  let painting = $state(false);
  let engine: 'off' | 'preview' | 'live' = $state('off');
  let liveColours: Rgb[] | null = $state(null);
  let busy = $state(false);
  let message = $state('');
  let failed = $state(false);
  let pulse = $state(0);
  let view: 'light' | 'live' | 'settings' = $state('light');

  let geo = $derived.by(() => {
    const l = layouts.find((x) => x.id === layoutId) ?? layouts[0];
    if (!l) return null;
    const g = geometry(l);
    const app = i18n.lang === 'tr' ? 'Menü' : 'Menu';
    g.keys = g.keys.map((k: KeyGeo) => ({ ...k, name: k.name === 'App' ? app : (LABELS[k.name] ?? k.name) }));
    return g;
  });
  let info = $derived(catalog.find((e) => e.id === draft.id) ?? catalog[0]);
  let layer = $derived(scene.layers[sel] ?? scene.layers[0]);
  let visibleGroups = $derived(new Set(scene.layers.filter((l) => l.visible).map((l) => SW.find((s) => s.kind === l.effect.kind)?.group)));

  const say = (text: string, bad = false) => {
    message = text;
    failed = bad;
  };

  async function refresh(adoptDraft: boolean) {
    const s = await api.status();
    connected = s.connected;
    lock = s.lock;
    engine = s.engine;
    if (s.stored.length) stored = Object.fromEntries(s.stored.map((e) => [e.id, e]));
    if (s.effect) {
      applied = s.effect;
      const known = catalog.some((e) => e.id === s.effect!.id);
      if (adoptDraft && known) draft = { ...s.effect };
      if (!known && s.engine !== 'live') say(t('light.hostMode'));
    }
  }

  async function apply() {
    busy = true;
    say('');
    try {
      applied = await api.applyEffect($state.snapshot(draft));
      stored[applied.id] = { ...applied };
      draft = { ...applied };
      if (engine === 'live') engine = 'off';
      pulse += 1;
      say(t('light.applied'));
    } catch (e) {
      say(t('light.writeFailed', { e: errText(e) }), true);
    } finally {
      busy = false;
    }
  }

  /** Switch the editor to another effect with the settings the keyboard keeps for it. */
  function pick(id: number) {
    if (id === draft.id) return;
    const own = stored[id];
    draft = own ? { ...own } : { ...draft, id };
  }

  async function startLive() {
    busy = true;
    say('');
    try {
      await api.engineStart($state.snapshot(scene));
      engine = 'live';
    } catch (e) {
      say(t('live.startFailed', { e: errText(e) }), true);
    } finally {
      busy = false;
    }
  }

  async function stopLive() {
    busy = true;
    say('');
    try {
      applied = await api.engineStop();
      stored[applied.id] = { ...applied };
      draft = { ...applied };
      engine = 'preview';
      say(t('live.stopped'));
    } catch (e) {
      say(t('live.stopFailed', { e: errText(e) }), true);
    } finally {
      busy = false;
    }
  }

  /** The catalogue below the stage puts an effect into the selected layer. */
  function pickEffect(kind: string) {
    const info = SW.find((s) => s.kind === kind);
    if (!info) return;
    const old = layer;
    const effect = makeEffect(info, old.effect);
    // keep a blend the user chose; follow the effect's natural blend otherwise
    const blend = old.blend === defaultBlend(old.effect.kind) ? defaultBlend(kind) : old.blend;
    setScene({ layers: scene.layers.map((l, i) => (i === sel ? { ...l, effect, blend } : l)) });
  }

  function setScene(next: Scene) {
    scene = next;
  }

  /** New layer above the selected one: red keys on WASD, ready for picking other keys. */
  function addLayer() {
    const solid = SW.find((s) => s.kind === 'solid')!;
    const wasd = KEY_GROUPS.find((g) => g.id === 'wasd')!.keys;
    const fresh = { ...makeLayer(solid), keys: [...wasd] };
    const layers = [...scene.layers];
    layers.splice(sel + 1, 0, fresh);
    scene = { layers };
    sel = sel + 1;
    painting = true;
  }

  function paintKey(id: string, add: boolean) {
    const keys = layer.keys.filter((k) => k !== id);
    if (add) keys.push(id);
    scene = { layers: scene.layers.map((l, i) => (i === sel ? { ...l, keys } : l)) };
  }

  async function saveScene(name: string) {
    try {
      saved = await api.saveScene(name, $state.snapshot(scene));
      sceneName = saved.find((s) => s.name.toLowerCase() === name.trim().toLowerCase())?.name ?? name.trim();
      say(t('live.saved', { name: sceneName }));
    } catch (e) {
      say(t('live.saveFailed', { e: errText(e) }), true);
    }
  }

  function loadScene(name: string) {
    const s = saved.find((x) => x.name === name);
    if (!s || !s.scene.layers.length) return;
    scene = structuredClone($state.snapshot(s.scene));
    sel = scene.layers.length - 1;
    sceneName = s.name;
    painting = false;
  }

  async function deleteScene(name: string) {
    try {
      saved = await api.deleteScene(name);
      if (sceneName === name) sceneName = null;
      say(t('live.deleted', { name }));
    } catch (e) {
      say(t('live.deleteFailed', { e: errText(e) }), true);
    }
  }

  function goto(next: typeof view) {
    say('');
    view = next;
    painting = false;
    if (next === 'live') api.enginePreview($state.snapshot(scene));
    else if (engine !== 'live') api.enginePreview(null);
    if (next !== 'live' && engine !== 'live') liveColours = null;
  }

  onMount(() => {
    const un: Promise<() => void>[] = [];
    (async () => {
      layouts = (await api.layouts()).layouts;
      catalog = await api.catalog();
      await refresh(true);
      saved = await api.scenes().catch(() => []);
      autostart = await autostartEnabled().catch(() => false);
      if (sceneName && !saved.some((x) => x.name === sceneName)) sceneName = null;
    })();
    un.push(onLock((v) => (lock = v)));
    un.push(
      onConnection((c) => {
        connected = c;
        if (c) refresh(false);
      }),
    );
    un.push(
      onKbdEvent((ev) => {
        if (ev.kind === 'effect_key' || ev.kind === 'brightness_key' || ev.kind === 'speed_key') {
          const wasClean = sameEffect(draft, applied);
          setTimeout(() => refresh(wasClean), 1500);
        }
      }),
    );
    un.push(onEngineFrame((keys) => (liveColours = keys)));
    un.push(
      onEngineError((msg) => {
        engine = 'preview';
        say(errText(msg), true);
      }),
    );
    const vis = () => api.setVisible(!document.hidden);
    document.addEventListener('visibilitychange', vis);
    return () => {
      un.forEach((p) => p.then((f) => f()));
      document.removeEventListener('visibilitychange', vis);
    };
  });

  $effect(() => {
    localStorage.setItem('prizma.sceneName', JSON.stringify(sceneName));
  });
  $effect(() => {
    // the tray menu follows the interface language
    document.documentElement.lang = i18n.lang;
    api.setLanguage(i18n.lang).catch(() => {});
  });

  async function toggleAutostart() {
    const next = !autostart;
    try {
      if (next) await enableAutostart();
      else await disableAutostart();
      autostart = await autostartEnabled();
    } catch (e) {
      say(errText(e), true);
    }
  }
  $effect(() => {
    localStorage.setItem('prizma.layout', JSON.stringify(layoutId));
    api.setLayout(layoutId);
  });
  $effect(() => {
    const snap = $state.snapshot(scene);
    localStorage.setItem('prizma.scene', JSON.stringify(snap));
    if (view === 'live' || engine === 'live') api.enginePreview(snap);
  });
</script>

<div class="app">
  <Titlebar {connected} {lock} />
  <div class="body">
    <nav class="rail" aria-label={t('nav.sections')}>
      <button class:on={view === 'light'} aria-current={view === 'light'} onclick={() => goto('light')}>
        <svg viewBox="0 0 24 24"><path d="M3 18 12 4l9 14H3Z" /><path d="M12 4v14" /></svg>
        <span>{t('nav.light')}</span>
      </button>
      <button class:on={view === 'live'} aria-current={view === 'live'} onclick={() => goto('live')}>
        <svg viewBox="0 0 24 24"><path d="M3 12h3l2-6 4 12 3-9 2 3h4" /></svg>
        <span>{t('nav.live')}</span>
        {#if engine === 'live'}<i class="livedot" aria-label={t('nav.liveRunning')}></i>{/if}
      </button>
      <button class:on={view === 'settings'} aria-current={view === 'settings'} onclick={() => goto('settings')}>
        <svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="3" /><path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9 7 7M17 17l2.1 2.1M4.9 19.1 7 17M17 7l2.1-2.1" /></svg>
        <span>{t('nav.settings')}</span>
      </button>
    </nav>

    {#if view === 'light'}
      <main class="main">
        <div class="stage">
          {#if geo}
            <KeyboardStage keys={geo.keys} bounds={geo.bounds} fx={draft} lock={!!lock} {pulse} />
          {/if}
          {#if REACTIVE.has(draft.id)}
            <p class="pressnote"><KeyPressIcon />{t('light.pressNote')}</p>
          {/if}
          {#if !connected}
            <p class="empty">{t('light.plugIn')}</p>
          {:else if engine === 'live'}
            <p class="banner">{t('light.liveRunning')}</p>
          {/if}
        </div>
        <section class="strip" aria-label={t('light.stripAria')}>
          <h2>{t('light.strip')}</h2>
          <div class="grid">
            {#each catalog as e (e.id)}
              <EffectTile info={e} effect={e.id === draft.id ? draft : (stored[e.id] ?? draft)} selected={e.id === draft.id} applied={applied?.id === e.id && engine !== 'live'} onpick={() => pick(e.id)} />
            {/each}
          </div>
        </section>
      </main>
      {#if info}
        <Inspector {info} {draft} {applied} {connected} {busy} {message} {failed} onchange={(e) => (draft = e)} onapply={apply} onrevert={() => applied && (draft = { ...applied })} />
      {/if}
    {:else if view === 'live'}
      <main class="main">
        <div class="stage">
          {#if geo}
            <KeyboardStage
              keys={geo.keys}
              bounds={geo.bounds}
              fx={draft}
              lock={!!lock}
              pulse={0}
              colours={liveColours}
              mask={painting || layer.keys.length ? layer.keys : null}
              onpaint={painting ? paintKey : null}
            />
          {/if}
          {#if painting}
            <p class="banner">{t('live.paintBanner')} <button class="link" onclick={() => (painting = false)}>{t('live.done')}</button></p>
          {:else if visibleGroups.has('audio')}
            <p class="banner soft">{t('live.audioBanner')}</p>
          {:else if visibleGroups.has('react')}
            <p class="banner soft">{t('live.reactBanner')}</p>
          {/if}
        </div>
        <section class="strip" aria-label={t('live.stripAria')}>
          <div class="groups">
            {#each GROUPS as g}
              <div class="group">
                <h2>{tx(g.name)}</h2>
                <div class="swgrid">
                  {#each SW.filter((s) => s.group === g.id) as s (s.kind)}
                    <button class="swtile" class:selected={s.kind === layer.effect.kind} aria-pressed={s.kind === layer.effect.kind} onclick={() => pickEffect(s.kind)}>
                      <span class="thumb" style:background={s.thumb}></span>
                      <span class="name">{tx(s.name)}</span>
                    </button>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        </section>
      </main>
      <LayerPanel
        {scene}
        selected={sel}
        {sceneName}
        {saved}
        {painting}
        live={engine === 'live'}
        {connected}
        {busy}
        {message}
        {failed}
        onscene={setScene}
        onselect={(i) => (sel = i)}
        onpainting={(on) => (painting = on)}
        onadd={addLayer}
        onsave={saveScene}
        onload={loadScene}
        ondelete={deleteScene}
        onstart={startLive}
        onstop={stopLive}
      />
    {:else}
      <main class="settings">
        <h2>{t('set.title')}</h2>
        <section>
          <h3>{t('set.language')}</h3>
          <div class="choices">
            {#each LANGS as l (l.id)}
              <button class:on={i18n.lang === l.id} lang={l.id} onclick={() => setLang(l.id)}>{l.name}</button>
            {/each}
          </div>
        </section>
        <section>
          <h3>{t('set.layout')}</h3>
          <p class="hint">{t('set.layoutHint')}</p>
          <div class="choices">
            {#each layouts as l (l.id)}
              <button class:on={layoutId === l.id} onclick={() => (layoutId = l.id)}>{LAYOUT_NAMES[l.id] ? tx(LAYOUT_NAMES[l.id]) : l.name}</button>
            {/each}
          </div>
        </section>
        <section>
          <h3>{t('set.startup')}</h3>
          <p class="hint">{t('set.autostartHint')}</p>
          <button class="toggle" class:on={autostart} role="switch" aria-checked={autostart} onclick={toggleAutostart}>
            <span class="track" aria-hidden="true"></span>{t('set.autostart')}
          </button>
        </section>
      </main>
    {/if}
  </div>
</div>

<style>
  .app { display: grid; grid-template-rows: auto 1fr; height: 100vh; }
  .body { display: grid; grid-template-columns: 76px 1fr 360px; min-height: 0; }
  .rail { display: flex; flex-direction: column; gap: var(--s2); padding: var(--s4) var(--s2); border-right: 1px solid var(--line); }
  .rail button { position: relative; display: grid; justify-items: center; gap: 4px; padding: 10px 0; border: none; border-radius: 12px; background: transparent; color: var(--faint); font-size: 11.5px; font-weight: 600; cursor: pointer; }
  .rail button:hover { color: var(--text); background: var(--surface); }
  .rail button.on { color: var(--text); background: var(--raised); }
  .rail svg { width: 22px; height: 22px; fill: none; stroke: currentColor; stroke-width: 1.6; stroke-linejoin: round; stroke-linecap: round; }
  .livedot { position: absolute; top: 8px; right: 14px; width: 8px; height: 8px; border-radius: 50%; background: var(--ok); box-shadow: 0 0 8px var(--ok); }
  .main { display: grid; grid-template-rows: 1fr auto; min-height: 0; min-width: 0; }
  .stage { position: relative; min-height: 300px; }
  .empty, .banner { position: absolute; left: 50%; transform: translateX(-50%); bottom: var(--s5); margin: 0; text-align: center; color: var(--muted); }
  .banner { padding: 8px 14px; border-radius: 999px; background: #2a2545cc; border: 1px solid var(--line-strong); color: var(--text); font-size: 13px; white-space: nowrap; }
  .pressnote { position: absolute; top: 10px; left: var(--s5); margin: 0; display: inline-flex; align-items: center; gap: 6px; padding: 6px 12px 6px 9px; border-radius: 999px; background: #2a2545cc; border: 1px solid var(--line-strong); color: var(--text); font-size: 12.5px; font-weight: 600; }
  .banner .link { margin-left: 8px; padding: 0; border: none; background: none; color: var(--ok); font: inherit; font-weight: 700; cursor: pointer; }
  .banner.soft { background: transparent; border-color: transparent; color: var(--muted); }
  .strip { padding: var(--s4) var(--s5) var(--s5); border-top: 1px solid var(--line); background: linear-gradient(180deg, #120f20, var(--ink)); }
  .strip h2, .settings h2 { margin: 0 0 var(--s3); font-family: var(--font-display); font-weight: 500; font-size: 15px; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(136px, 1fr)); gap: var(--s2); max-height: 236px; overflow-y: auto; padding: 2px; }
  .groups { display: grid; gap: var(--s4); max-height: 260px; overflow-y: auto; padding: 2px; }
  .group h2 { font-size: 13.5px; margin-bottom: var(--s2); }
  .swgrid { display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: var(--s2); }
  .swtile { position: relative; display: grid; gap: var(--s2); padding: 10px; border-radius: 12px; border: 1px solid var(--line); background: var(--surface); text-align: left; cursor: pointer; }
  .swtile:hover { background: var(--raised); border-color: var(--line-strong); }
  .swtile.selected { background: var(--raised); border-color: var(--text); }
  .thumb { height: 34px; border-radius: 7px; }
  .swtile .name { font-size: 12.5px; font-weight: 600; }
  .settings { grid-column: 2 / 4; padding: var(--s6); max-width: 720px; }
  .settings h3 { margin: var(--s5) 0 var(--s1); font-size: 14px; }
  .hint { margin: 0 0 var(--s3); color: var(--muted); }
  .choices { display: flex; gap: var(--s2); }
  .choices button { padding: 10px 16px; border-radius: 10px; border: 1px solid var(--line); background: var(--surface); cursor: pointer; font-weight: 600; }
  .choices button.on { border-color: var(--text); background: var(--text); color: var(--ink); }
  .toggle { display: flex; align-items: center; gap: var(--s3); padding: 0; border: none; background: none; color: var(--text); font: inherit; font-weight: 600; cursor: pointer; text-align: left; }
  .toggle .track { position: relative; width: 44px; height: 26px; flex: none; border-radius: 999px; background: var(--raised); box-shadow: inset 0 0 0 1px var(--line-strong); transition: background 160ms; }
  .toggle .track::after { content: ''; position: absolute; top: 4px; left: 4px; width: 18px; height: 18px; border-radius: 50%; background: var(--muted); transition: transform 160ms, background 160ms; }
  .toggle.on .track { background: var(--ok); box-shadow: none; }
  .toggle.on .track::after { transform: translateX(18px); background: var(--ink); }
</style>
