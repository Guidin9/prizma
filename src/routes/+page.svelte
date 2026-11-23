<script lang="ts">
  import { onMount } from 'svelte';
  import { api, onConnection, onKbdEvent, onLock, sameEffect, type EffectInfo, type HwEffect, type Layout } from '$lib/api';
  import { geometry, type KeyGeo } from '$lib/preview';
  import Titlebar from '$lib/components/Titlebar.svelte';
  import KeyboardStage from '$lib/components/KeyboardStage.svelte';
  import EffectTile from '$lib/components/EffectTile.svelte';
  import Inspector from '$lib/components/Inspector.svelte';

  const LABELS: Record<string, string> = {
    LShift: 'Shift', RShift: 'Shift', CapsLock: 'Caps', Backspace: '⌫', LCtrl: 'Ctrl', RCtrl: 'Ctrl',
    LWin: 'Win', LAlt: 'Alt', RAlt: 'AltGr', App: 'Menü', Space: '', Enter: 'Enter', Esc: 'Esc', Tab: 'Tab',
  };

  let layouts: Layout[] = $state([]);
  let layoutId = $state(localStorage.getItem('prizma.layout') ?? 'tr_iso');
  let catalog: EffectInfo[] = $state([]);
  let connected = $state(false);
  let lock: boolean | null = $state(null);
  let applied: HwEffect | null = $state(null);
  let draft: HwEffect = $state({ id: 0x03, colour: null, speed: 3, brightness: 4, reverse: false });
  let busy = $state(false);
  let message = $state('');
  let failed = $state(false);
  let pulse = $state(0);
  let view: 'light' | 'settings' = $state('light');

  let geo = $derived.by(() => {
    const l = layouts.find((x) => x.id === layoutId) ?? layouts[0];
    if (!l) return null;
    const g = geometry(l);
    g.keys = g.keys.map((k: KeyGeo) => ({ ...k, name: LABELS[k.name] ?? k.name }));
    return g;
  });
  let info = $derived(catalog.find((e) => e.id === draft.id) ?? catalog[0]);

  async function refresh(adoptDraft: boolean) {
    const s = await api.status();
    connected = s.connected;
    lock = s.lock;
    if (s.effect) {
      applied = s.effect;
      const known = catalog.some((e) => e.id === s.effect!.id);
      if (adoptDraft && known) draft = { ...s.effect };
      if (!known) {
        failed = false;
        message = 'Klavye bilgisayar kontrolü modunda kalmış. Bir efekt seçip uygula.';
      }
    }
  }

  async function apply() {
    busy = true;
    message = '';
    failed = false;
    try {
      applied = await api.applyEffect($state.snapshot(draft));
      draft = { ...applied };
      pulse += 1;
      message = 'Klavyeye uygulandı, iki profile de yazıldı.';
    } catch (e) {
      failed = true;
      message = `Yazılamadı: ${e}`;
    } finally {
      busy = false;
    }
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
          // the keyboard saved a change itself; pick it up once the flash write settles
          const wasClean = sameEffect(draft, applied);
          setTimeout(() => refresh(wasClean), 1500);
        }
      }),
    );
    return () => un.forEach((p) => p.then((f) => f()));
  });

  $effect(() => localStorage.setItem('prizma.layout', layoutId));
</script>

<div class="app">
  <Titlebar {connected} {lock} />
  <div class="body">
    <nav class="rail" aria-label="Bölümler">
      <button class:on={view === 'light'} aria-current={view === 'light'} onclick={() => (view = 'light')}>
        <svg viewBox="0 0 24 24"><path d="M3 18 12 4l9 14H3Z" /><path d="M12 4v14" /></svg>
        <span>Işık</span>
      </button>
      <button class:on={view === 'settings'} aria-current={view === 'settings'} onclick={() => (view = 'settings')}>
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
          {#if !connected}
            <p class="empty">Klavyeyi tak; Prizma onu kendisi bulur.</p>
          {/if}
        </div>
        <section class="strip" aria-label="Klavyenin kendi efektleri">
          <h2>Klavyenin efektleri</h2>
          <div class="grid">
            {#each catalog as e (e.id)}
              <EffectTile
                info={e}
                effect={draft}
                selected={e.id === draft.id}
                applied={applied?.id === e.id}
                onpick={() => (draft = { ...draft, id: e.id })}
              />
            {/each}
          </div>
        </section>
      </main>
      {#if info}
        <Inspector
          {info}
          {draft}
          {applied}
          {connected}
          {busy}
          {message}
          {failed}
          onchange={(e) => (draft = e)}
          onapply={apply}
          onrevert={() => applied && (draft = { ...applied })}
        />
      {/if}
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
  .app {
    display: grid;
    grid-template-rows: auto 1fr;
    height: 100vh;
  }
  .body {
    display: grid;
    grid-template-columns: 76px 1fr 360px;
    min-height: 0;
  }
  .rail {
    display: flex;
    flex-direction: column;
    gap: var(--s2);
    padding: var(--s4) var(--s2);
    border-right: 1px solid var(--line);
  }
  .rail button {
    display: grid;
    justify-items: center;
    gap: 4px;
    padding: 10px 0;
    border: none;
    border-radius: 12px;
    background: transparent;
    color: var(--faint);
    font-size: 11.5px;
    font-weight: 600;
    cursor: pointer;
  }
  .rail button:hover {
    color: var(--text);
    background: var(--surface);
  }
  .rail button.on {
    color: var(--text);
    background: var(--raised);
  }
  .rail svg {
    width: 22px;
    height: 22px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
    stroke-linejoin: round;
  }
  .main {
    display: grid;
    grid-template-rows: 1fr auto;
    min-height: 0;
    min-width: 0;
  }
  .stage {
    position: relative;
    min-height: 300px;
  }
  .empty {
    position: absolute;
    left: 0;
    right: 0;
    bottom: var(--s6);
    text-align: center;
    color: var(--muted);
  }
  .strip {
    padding: var(--s4) var(--s5) var(--s5);
    border-top: 1px solid var(--line);
    background: linear-gradient(180deg, #120f20, var(--ink));
  }
  .strip h2,
  .settings h2 {
    margin: 0 0 var(--s3);
    font-family: var(--font-display);
    font-weight: 500;
    font-size: 15px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(136px, 1fr));
    gap: var(--s2);
    max-height: 236px;
    overflow-y: auto;
    padding: 2px;
  }
  .settings {
    grid-column: 2 / 4;
    padding: var(--s6);
    max-width: 720px;
  }
  .settings h3 {
    margin: var(--s5) 0 var(--s1);
    font-size: 14px;
  }
  .hint {
    margin: 0 0 var(--s3);
    color: var(--muted);
  }
  .choices {
    display: flex;
    gap: var(--s2);
  }
  .choices button {
    padding: 10px 16px;
    border-radius: 10px;
    border: 1px solid var(--line);
    background: var(--surface);
    cursor: pointer;
    font-weight: 600;
  }
  .choices button.on {
    border-color: var(--line-strong);
    background: var(--raised);
  }
</style>
