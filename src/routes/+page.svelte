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
    type SwEffect,
  } from '$lib/api';
  import { REACTIVE, geometry, type KeyGeo } from '$lib/preview';
  import KeyPressIcon from '$lib/components/KeyPressIcon.svelte';
  import { GROUPS, SW, makeEffect } from '$lib/swcatalog';
  import Titlebar from '$lib/components/Titlebar.svelte';
  import KeyboardStage from '$lib/components/KeyboardStage.svelte';
  import EffectTile from '$lib/components/EffectTile.svelte';
  import Inspector from '$lib/components/Inspector.svelte';
  import SwInspector from '$lib/components/SwInspector.svelte';

  const LABELS: Record<string, string> = {
    LShift: 'Shift', RShift: 'Shift', CapsLock: 'Caps', Backspace: '⌫', LCtrl: 'Ctrl', RCtrl: 'Ctrl',
    LWin: 'Win', LAlt: 'Alt', RAlt: 'AltGr', App: 'Menü', Space: '', Enter: 'Enter', Esc: 'Esc', Tab: 'Tab',
  };
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
  let sw: SwEffect = $state(load('prizma.sw', makeEffect(SW[0])));
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
    g.keys = g.keys.map((k: KeyGeo) => ({ ...k, name: LABELS[k.name] ?? k.name }));
    return g;
  });
  let info = $derived(catalog.find((e) => e.id === draft.id) ?? catalog[0]);
  let swInfo = $derived(SW.find((s) => s.kind === sw.kind) ?? SW[0]);

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
      if (!known && s.engine !== 'live') say('Klavye bilgisayar kontrolü modunda kalmış. Bir efekt seçip uygula.');
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
      say('Klavyeye uygulandı, iki profile de yazıldı.');
    } catch (e) {
      say(`Yazılamadı: ${e}`, true);
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
      await api.engineStart($state.snapshot(sw));
      engine = 'live';
    } catch (e) {
      say(`Başlatılamadı: ${e}`, true);
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
      say('Klavye kendi efektine döndü.');
    } catch (e) {
      say(`Durdurulamadı: ${e}`, true);
    } finally {
      busy = false;
    }
  }

  function goto(next: typeof view) {
    say('');
    view = next;
    if (next === 'live') api.enginePreview($state.snapshot(sw));
    else if (engine !== 'live') api.enginePreview(null);
    if (next !== 'live' && engine !== 'live') liveColours = null;
  }

  onMount(() => {
    const un: Promise<() => void>[] = [];
    (async () => {
      layouts = (await api.layouts()).layouts;
      catalog = await api.catalog();
      await refresh(true);
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
        say(msg, true);
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
    localStorage.setItem('prizma.layout', JSON.stringify(layoutId));
    api.setLayout(layoutId);
  });
  $effect(() => {
    const snap = $state.snapshot(sw);
    localStorage.setItem('prizma.sw', JSON.stringify(snap));
    if (view === 'live' || engine === 'live') api.enginePreview(snap);
  });
</script>

<div class="app">
  <Titlebar {connected} {lock} />
  <div class="body">
    <nav class="rail" aria-label="Bölümler">
      <button class:on={view === 'light'} aria-current={view === 'light'} onclick={() => goto('light')}>
        <svg viewBox="0 0 24 24"><path d="M3 18 12 4l9 14H3Z" /><path d="M12 4v14" /></svg>
        <span>Işık</span>
      </button>
      <button class:on={view === 'live'} aria-current={view === 'live'} onclick={() => goto('live')}>
        <svg viewBox="0 0 24 24"><path d="M3 12h3l2-6 4 12 3-9 2 3h4" /></svg>
        <span>Canlı</span>
        {#if engine === 'live'}<i class="livedot" aria-label="Canlı efekt çalışıyor"></i>{/if}
      </button>
      <button class:on={view === 'settings'} aria-current={view === 'settings'} onclick={() => goto('settings')}>
        <svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="3" /><path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9 7 7M17 17l2.1 2.1M4.9 19.1 7 17M17 7l2.1-2.1" /></svg>
        <span>Ayarlar</span>
      </button>
    </nav>

    {#if view === 'light'}
      <main class="main">
        <div class="stage">
          {#if geo}
            <KeyboardStage keys={geo.keys} bounds={geo.bounds} fx={draft} lock={!!lock} {pulse} />
          {/if}
          {#if REACTIVE.has(draft.id)}
            <p class="pressnote"><KeyPressIcon />Tuşa basınca çalışır · ön izlemede sanal tuş basışları</p>
          {/if}
          {#if !connected}
            <p class="empty">Klavyeyi tak; Prizma onu kendisi bulur.</p>
          {:else if engine === 'live'}
            <p class="banner">Klavyede şu an canlı bir efekt çalışıyor. Buradan bir efekt uygularsan canlı efekt durur.</p>
          {/if}
        </div>
        <section class="strip" aria-label="Klavyenin kendi efektleri">
          <h2>Klavyenin efektleri</h2>
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
            <KeyboardStage keys={geo.keys} bounds={geo.bounds} fx={draft} lock={!!lock} pulse={0} colours={liveColours} />
          {/if}
          {#if swInfo.group === 'audio'}
            <p class="banner soft">Müzik ya da video oynat; efekt bilgisayardan çıkan sese tepki verir.</p>
          {:else if swInfo.group === 'react'}
            <p class="banner soft">Klavyede yazmaya başla; her basış efekti tetikler.</p>
          {/if}
        </div>
        <section class="strip" aria-label="Canlı efektler">
          <div class="groups">
            {#each GROUPS as g}
              <div class="group">
                <h2>{g.name}</h2>
                <div class="swgrid">
                  {#each SW.filter((s) => s.group === g.id) as s (s.kind)}
                    <button class="swtile" class:selected={s.kind === sw.kind} aria-pressed={s.kind === sw.kind} onclick={() => (sw = makeEffect(s, sw))}>
                      <span class="thumb" style:background={s.thumb}></span>
                      <span class="name">{s.name}</span>
                    </button>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        </section>
      </main>
      <SwInspector info={swInfo} effect={sw} live={engine === 'live'} {connected} {busy} {message} {failed} onchange={(e) => (sw = e)} onstart={startLive} onstop={stopLive} />
    {:else}
      <main class="settings">
        <h2>Ayarlar</h2>
        <section>
          <h3>Klavye düzeni</h3>
          <p class="hint">Efektler tuşların gerçek yerine göre hesaplanır; klavyendeki dizilimi seç.</p>
          <div class="choices">
            {#each layouts as l (l.id)}
              <button class:on={layoutId === l.id} onclick={() => (layoutId = l.id)}>{l.name}</button>
            {/each}
          </div>
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
  .choices button.on { border-color: var(--line-strong); background: var(--raised); }
</style>
