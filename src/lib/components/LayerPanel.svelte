<script lang="ts">
  import type { Layer, Rgb, SavedScene, Scene, SwEffect } from '../api';
  import { BLENDS, DIRECTIONS, KEY_GROUPS, PALETTES, SW } from '../swcatalog';
  import { t, tx } from '../i18n.svelte';

  interface Props {
    scene: Scene;
    selected: number;
    sceneName: string | null;
    saved: SavedScene[];
    painting: boolean;
    live: boolean;
    connected: boolean;
    busy: boolean;
    message: string;
    failed: boolean;
    onscene: (s: Scene) => void;
    onselect: (i: number) => void;
    onpainting: (on: boolean) => void;
    onadd: () => void;
    onsave: (name: string) => void;
    onload: (name: string) => void;
    ondelete: (name: string) => void;
    onstart: () => void;
    onstop: () => void;
  }
  let {
    scene, selected, sceneName, saved, painting, live, connected, busy, message, failed,
    onscene, onselect, onpainting, onadd, onsave, onload, ondelete, onstart, onstop,
  }: Props = $props();

  let layer = $derived(scene.layers[selected] ?? scene.layers[0]);
  let effect = $derived(layer.effect);
  let info = $derived(SW.find((s) => s.kind === effect.kind) ?? SW[0]);
  let savedScene = $derived(saved.find((s) => s.name === sceneName));
  let changed = $derived(!!savedScene && JSON.stringify(savedScene.scene) !== JSON.stringify(scene));

  const hex = (c: Rgb) => '#' + c.map((v) => Math.round(v).toString(16).padStart(2, '0')).join('');
  const fromHex = (h: string): Rgb => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16)) as Rgb;
  const same = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);
  const swatch = (cols: Rgb[]) =>
    cols.length === 0
      ? 'linear-gradient(90deg,#ff3d6e,#ffe14d,#4dff91,#3dd6ff,#d24dff)'
      : `linear-gradient(90deg,${cols.map(hex).join(',')}${cols.length === 1 ? ',' + hex(cols[0]) : ''})`;

  function setLayerAt(i: number, patch: Partial<Layer>) {
    onscene({ layers: scene.layers.map((l, j) => (j === i ? { ...l, ...patch } : l)) });
  }
  const setLayer = (patch: Partial<Layer>) => setLayerAt(selected, patch);
  const set = (patch: Partial<SwEffect>) => setLayer({ effect: { ...effect, ...patch } });

  function move(i: number, by: number) {
    const j = i + by;
    if (j < 0 || j >= scene.layers.length) return;
    const layers = [...scene.layers];
    [layers[i], layers[j]] = [layers[j], layers[i]];
    onscene({ layers });
    if (selected === i) onselect(j);
    else if (selected === j) onselect(i);
  }
  function remove(i: number) {
    if (scene.layers.length < 2) return;
    onscene({ layers: scene.layers.filter((_, j) => j !== i) });
    onselect(Math.max(0, Math.min(selected > i ? selected - 1 : selected, scene.layers.length - 2)));
  }

  const nameOf = (l: Layer) => {
    const s = SW.find((x) => x.kind === l.effect.kind);
    return s ? tx(s.name) : l.effect.kind;
  };
  function keysLabel(keys: string[]) {
    if (keys.length === 0) return t('lp.allKeys');
    const g = KEY_GROUPS.find((g) => g.keys.length === keys.length && g.keys.every((k) => keys.includes(k)));
    return g ? tx(g.name) : t('lp.nKeys', { n: keys.length });
  }
  const isGroup = (keys: string[]) => (g: { keys: string[] }) => g.keys.length === keys.length && g.keys.every((k) => keys.includes(k));
  const blendName = (b: Layer['blend']) => {
    const x = BLENDS.find((y) => y.id === b);
    return x ? tx(x.name) : b;
  };

  // naming a scene before saving
  let naming = $state(false);
  let nameDraft = $state('');
  let confirmDelete = $state(false);
  function startNaming() {
    nameDraft = sceneName ?? '';
    naming = true;
  }
  function submitName(ev: SubmitEvent) {
    ev.preventDefault();
    if (!nameDraft.trim()) return;
    onsave(nameDraft.trim());
    naming = false;
  }
</script>

<aside class="panel">
  <section class="scenes" aria-label={t('lp.scene')}>
    <div class="row">
      <label class="select">
        <span class="lbl">{t('lp.scene')}</span>
        <select
          value={sceneName ?? ''}
          onchange={(e) => {
            const v = (e.target as HTMLSelectElement).value;
            confirmDelete = false;
            if (v) onload(v);
          }}
        >
          {#if !sceneName}<option value="">{t('lp.unsaved')}</option>{/if}
          {#each saved as s (s.name)}
            <option value={s.name}>{s.name}</option>
          {/each}
        </select>
      </label>
      {#if savedScene && changed}
        <button class="chip strong" onclick={() => onsave(savedScene.name)} title={t('lp.saveOver')}>{t('lp.save')}</button>
      {/if}
      <button class="chip" onclick={startNaming}>{savedScene ? t('lp.saveAs') : t('lp.save')}</button>
      {#if savedScene}
        {#if confirmDelete}
          <button class="chip danger" onclick={() => { ondelete(savedScene.name); confirmDelete = false; }}>{t('lp.confirmDelete')}</button>
        {:else}
          <button class="icon" aria-label={t('lp.deleteScene')} title={t('lp.deleteScene')} onclick={() => (confirmDelete = true)}>
            <svg viewBox="0 0 16 16"><path d="M3 4.5h10M6.5 4.5V3h3v1.5M4.5 4.5l.6 8.5h5.8l.6-8.5" /></svg>
          </button>
        {/if}
      {/if}
    </div>
    {#if changed}<p class="hint">{t('lp.changed')}</p>{/if}
    {#if naming}
      <form class="naming" onsubmit={submitName}>
        <!-- svelte-ignore a11y_autofocus -->
        <input bind:value={nameDraft} maxlength="40" placeholder={t('lp.namePlaceholder')} aria-label={t('lp.name')} autofocus />
        <button class="chip strong" type="submit" disabled={!nameDraft.trim()}>{t('lp.save')}</button>
        <button class="chip" type="button" onclick={() => (naming = false)}>{t('lp.cancel')}</button>
      </form>
    {/if}
  </section>

  <section class="layers" aria-label={t('lp.layers')}>
    <div class="head">
      <h3>{t('lp.layers')}</h3>
      <button class="chip" onclick={onadd}>{t('lp.addLayer')}</button>
    </div>
    <ol>
      {#each scene.layers.map((l, i) => ({ l, i })).reverse() as { l, i } (i)}
        <li class:on={i === selected} class:hidden={!l.visible}>
          <button class="eye" aria-label={l.visible ? t('lp.hide') : t('lp.show')} aria-pressed={l.visible} onclick={() => setLayerAt(i, { visible: !l.visible })}>
            {#if l.visible}
              <svg viewBox="0 0 16 16"><path d="M1.5 8s2.4-4.5 6.5-4.5S14.5 8 14.5 8 12.1 12.5 8 12.5 1.5 8 1.5 8Z" /><circle cx="8" cy="8" r="2" /></svg>
            {:else}
              <svg viewBox="0 0 16 16"><path d="M2 2l12 12M6.2 3.8A6.6 6.6 0 0 1 8 3.5c4.1 0 6.5 4.5 6.5 4.5a11 11 0 0 1-1.9 2.4M4 5.3A11.4 11.4 0 0 0 1.5 8s2.4 4.5 6.5 4.5c.9 0 1.7-.2 2.4-.5" /></svg>
            {/if}
          </button>
          <button class="pick" onclick={() => onselect(i)} aria-current={i === selected}>
            <span class="thumb" style:background={SW.find((s) => s.kind === l.effect.kind)?.kind === 'solid' && l.effect.colours[0] ? hex(l.effect.colours[0]) : (SW.find((s) => s.kind === l.effect.kind)?.thumb ?? '')}></span>
            <span class="txt">
              <span class="lname">{nameOf(l)}</span>
              <span class="meta">{keysLabel(l.keys)} · {blendName(l.blend)}{l.opacity < 1 ? ` · %${Math.round(l.opacity * 100)}` : ''}</span>
            </span>
          </button>
          <span class="tools">
            <button class="icon" aria-label={t('lp.up')} disabled={i === scene.layers.length - 1} onclick={() => move(i, 1)}>
              <svg viewBox="0 0 16 16"><path d="M8 12.5v-9M4.5 7 8 3.5 11.5 7" /></svg>
            </button>
            <button class="icon" aria-label={t('lp.down')} disabled={i === 0} onclick={() => move(i, -1)}>
              <svg viewBox="0 0 16 16"><path d="M8 3.5v9M4.5 9 8 12.5 11.5 9" /></svg>
            </button>
            <button class="icon" aria-label={t('lp.remove')} disabled={scene.layers.length < 2} onclick={() => remove(i)}>
              <svg viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" /></svg>
            </button>
          </span>
        </li>
      {/each}
    </ol>
    <p class="hint">{t('lp.layersHint')}</p>
  </section>

  <section>
    <h3>{t('lp.keys')} <span class="val">{keysLabel(layer.keys)}</span></h3>
    <div class="chips">
      <button class:on={layer.keys.length === 0} onclick={() => setLayer({ keys: [] })}>{t('lp.allKeys')}</button>
      {#each KEY_GROUPS as g (g.id)}
        <button class:on={isGroup(layer.keys)(g)} onclick={() => setLayer({ keys: [...g.keys] })}>{tx(g.name)}</button>
      {/each}
    </div>
    <button class="paint" class:on={painting} aria-pressed={painting} onclick={() => onpainting(!painting)}>
      <svg viewBox="0 0 16 16"><path d="M10.5 2.5l3 3-7.5 7.5H3v-3z" /><path d="M9 4l3 3" /></svg>
      {painting ? t('lp.paintOn') : t('lp.paintOff')}
    </button>
  </section>

  <section>
    <h3>{t('lp.blend')}</h3>
    <div class="seg" role="radiogroup" aria-label={t('lp.blend')}>
      {#each BLENDS as b (b.id)}
        <button role="radio" aria-checked={layer.blend === b.id} class:on={layer.blend === b.id} title={tx(b.hint)} onclick={() => setLayer({ blend: b.id })}>{tx(b.name)}</button>
      {/each}
    </div>
    <p class="hint">{tx(BLENDS.find((b) => b.id === layer.blend)?.hint ?? BLENDS[0].hint)}</p>
  </section>

  <section>
    <h3>{t('lp.opacity')} <span class="val">{Math.round(layer.opacity * 100)}%</span></h3>
    <input type="range" min="0" max="1" step="0.05" value={layer.opacity} oninput={(e) => setLayer({ opacity: +(e.target as HTMLInputElement).value })} aria-label={t('lp.opacity')} />
  </section>

  <header class="fxhead">
    <h2>{tx(info.name)}</h2>
    <p class="desc">{tx(info.desc)}</p>
  </header>

  {#if info.uses.colours}
    <section>
      <h3>{t('lp.colours')}</h3>
      {#if info.kind !== 'solid'}
        <div class="palettes">
          {#each PALETTES as p}
            <button class="pal" class:on={same(p.colours, effect.colours)} onclick={() => set({ colours: p.colours })} title={tx(p.name)}>
              <span class="bar" style:background={swatch(p.colours)}></span>
              <span class="pname">{tx(p.name)}</span>
            </button>
          {/each}
        </div>
      {/if}
      {#if effect.colours.length}
        <div class="custom">
          {#each effect.colours as c, i}
            <label class="sw" style:background={hex(c)} title={t('lp.changeColour')}>
              <input
                type="color"
                value={hex(c)}
                oninput={(e) => {
                  const next = [...effect.colours];
                  next[i] = fromHex((e.target as HTMLInputElement).value);
                  set({ colours: next });
                }}
              />
            </label>
          {/each}
          {#if info.kind !== 'solid' && effect.colours.length < 5}
            <button class="add" aria-label={t('lp.addColour')} onclick={() => set({ colours: [...effect.colours, [255, 255, 255]] })}>+</button>
          {/if}
          {#if info.kind !== 'solid' && effect.colours.length > 1}
            <button class="add" aria-label={t('lp.removeColour')} onclick={() => set({ colours: effect.colours.slice(0, -1) })}>−</button>
          {/if}
        </div>
      {/if}
    </section>
  {/if}

  {#if info.uses.direction}
    <section>
      <h3>{t('lp.direction')}</h3>
      <div class="chips">
        {#each info.uses.direction as d}
          <button class:on={effect.direction === d} onclick={() => set({ direction: d })}>{tx(DIRECTIONS[d])}</button>
        {/each}
      </div>
    </section>
  {/if}

  {#if info.kind !== 'solid'}
    <section>
      <h3>{t('lp.speed')} <span class="val">{effect.speed.toFixed(1)}×</span></h3>
      <input type="range" min="0.2" max="3" step="0.1" value={effect.speed} oninput={(e) => set({ speed: +(e.target as HTMLInputElement).value })} aria-label={t('lp.speed')} />
    </section>
  {/if}

  {#if info.uses.scale}
    <section>
      <h3>{t('lp.scale')} <span class="val">{effect.scale.toFixed(1)}×</span></h3>
      <input type="range" min="0.4" max="3" step="0.1" value={effect.scale} oninput={(e) => set({ scale: +(e.target as HTMLInputElement).value })} aria-label={t('lp.scale')} />
    </section>
  {/if}

  <section>
    <h3>{t('lp.brightness')} <span class="val">{Math.round(effect.brightness * 100)}%</span></h3>
    <input type="range" min="0.05" max="1" step="0.05" value={effect.brightness} oninput={(e) => set({ brightness: +(e.target as HTMLInputElement).value })} aria-label={t('lp.brightness')} />
  </section>

  <footer>
    <p class="note">{t('lp.note')}</p>
    {#if message}<p class="message" class:failed role="status">{message}</p>{/if}
    {#if live}
      <div class="livebadge"><span class="pulse"></span>{t('lp.running')}</div>
      <button class="ghost strong" disabled={busy} onclick={onstop}>{busy ? t('lp.stopping') : t('lp.stop')}</button>
    {:else}
      <button class="primary" disabled={!connected || busy} onclick={onstart}>{busy ? t('lp.starting') : t('lp.start')}</button>
    {/if}
  </footer>
</aside>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: var(--s5);
    padding: var(--s5) var(--s5) var(--s5);
    background: var(--surface);
    border-left: 1px solid var(--line);
    overflow-y: auto;
  }
  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-weight: 500;
    font-size: 20px;
    line-height: 1.15;
  }
  .desc {
    margin: var(--s2) 0 0;
    color: var(--muted);
  }
  .fxhead {
    padding-top: var(--s4);
    border-top: 1px solid var(--line);
  }
  h3 {
    display: flex;
    justify-content: space-between;
    margin: 0 0 var(--s3);
    font-size: 13px;
    font-weight: 700;
    color: var(--muted);
  }
  .val {
    font-family: var(--font-display);
    font-weight: 400;
    color: var(--text);
  }
  .hint {
    margin: var(--s2) 0 0;
    font-size: 12px;
    color: var(--faint);
  }
  button {
    font-family: inherit;
  }
  .row {
    display: flex;
    align-items: flex-end;
    gap: 6px;
  }
  .select {
    display: grid;
    gap: 4px;
    flex: 1;
    min-width: 0;
  }
  .lbl {
    font-size: 12px;
    font-weight: 700;
    color: var(--muted);
  }
  select,
  .naming input {
    height: 34px;
    min-width: 0;
    padding: 0 10px;
    border-radius: 9px;
    border: 1px solid var(--line-strong);
    background: var(--raised);
    color: var(--text);
    font: inherit;
    font-size: 13px;
  }
  .naming {
    display: flex;
    gap: 6px;
    margin-top: var(--s2);
  }
  .naming input {
    flex: 1;
  }
  .chip {
    height: 34px;
    padding: 0 12px;
    border-radius: 9px;
    border: 1px solid var(--line-strong);
    background: var(--raised);
    color: var(--text);
    font-size: 12.5px;
    font-weight: 700;
    cursor: pointer;
    white-space: nowrap;
  }
  .chip.strong {
    background: var(--text);
    color: var(--ink);
    border-color: var(--text);
  }
  .chip.danger {
    background: #ff6b8b22;
    border-color: var(--danger);
    color: var(--danger);
  }
  .chip:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .icon {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    flex: none;
    border-radius: 8px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
  }
  .icon:hover:not(:disabled) {
    color: var(--text);
    background: var(--raised);
  }
  .icon:disabled {
    opacity: 0.3;
    cursor: default;
  }
  .row > .icon {
    height: 34px;
    width: 34px;
    border-color: var(--line-strong);
  }
  svg {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--s2);
  }
  .head h3 {
    margin: 0;
  }
  ol {
    display: grid;
    gap: 4px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 4px;
    border-radius: 10px;
    border: 1px solid var(--line);
    background: var(--ink);
  }
  li.on {
    border-color: var(--text);
    background: var(--raised);
  }
  li.hidden .pick {
    opacity: 0.45;
  }
  .eye {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    flex: none;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
  }
  .eye:hover {
    color: var(--text);
  }
  .pick {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
    padding: 2px 4px;
    border: none;
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }
  .thumb {
    width: 26px;
    height: 26px;
    flex: none;
    border-radius: 7px;
    box-shadow: inset 0 0 0 1px #ffffff26;
  }
  .txt {
    display: grid;
    min-width: 0;
  }
  .lname {
    font-size: 13px;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .meta {
    font-size: 11.5px;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .tools {
    display: flex;
    flex: none;
  }
  .tools .icon {
    width: 26px;
    height: 28px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chips button,
  .seg button {
    padding: 7px 12px;
    border-radius: 999px;
    border: 1px solid var(--line);
    background: var(--raised);
    cursor: pointer;
    font-weight: 600;
    font-size: 12.5px;
    color: var(--muted);
  }
  .chips button.on,
  .seg button.on {
    color: var(--ink);
    background: var(--text);
    border-color: var(--text);
  }
  .seg {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
  }
  .seg button {
    border-radius: 9px;
  }
  .paint {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    margin-top: var(--s3);
    padding: 9px 12px;
    border-radius: 10px;
    border: 1px dashed var(--line-strong);
    background: transparent;
    color: var(--text);
    font-size: 12.5px;
    font-weight: 700;
    text-align: left;
    cursor: pointer;
  }
  .paint.on {
    border-style: solid;
    border-color: var(--ok);
    background: #7dffc114;
    color: var(--ok);
  }
  .palettes {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 6px;
  }
  .pal {
    display: grid;
    gap: 6px;
    padding: 8px;
    border-radius: 10px;
    border: 1px solid var(--line);
    background: var(--raised);
    cursor: pointer;
    text-align: left;
    color: var(--text);
  }
  .pal.on {
    border-color: var(--text);
  }
  .bar {
    height: 8px;
    border-radius: 4px;
  }
  .pname {
    font-size: 12px;
    font-weight: 600;
  }
  .custom {
    display: flex;
    gap: 6px;
    margin-top: var(--s3);
  }
  .sw {
    position: relative;
    width: 34px;
    height: 34px;
    border-radius: 9px;
    border: 1px solid #ffffff30;
    overflow: hidden;
    cursor: pointer;
  }
  .sw input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
  }
  .add {
    width: 34px;
    height: 34px;
    border-radius: 9px;
    border: 1px dashed var(--line-strong);
    background: transparent;
    color: var(--muted);
    font-size: 18px;
    cursor: pointer;
  }
  input[type='range'] {
    width: 100%;
    accent-color: #8f7bff;
  }
  footer {
    margin-top: auto;
    display: grid;
    gap: var(--s3);
  }
  .note {
    margin: 0;
    font-size: 12.5px;
    color: var(--faint);
  }
  .message {
    margin: 0;
    font-size: 13px;
    color: var(--ok);
  }
  .message.failed {
    color: var(--danger);
  }
  .livebadge {
    display: flex;
    align-items: center;
    gap: var(--s2);
    font-size: 13px;
    color: var(--text);
  }
  .pulse {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--ok);
    box-shadow: 0 0 0 0 var(--ok);
    animation: ping 1.6s infinite;
  }
  @keyframes ping {
    70% {
      box-shadow: 0 0 0 8px #7dffc100;
    }
  }
  .primary {
    height: 46px;
    border-radius: 12px;
    border: none;
    background: var(--text);
    color: var(--ink);
    font-weight: 800;
    font-size: 14.5px;
    cursor: pointer;
  }
  .primary:disabled {
    background: var(--raised);
    color: var(--faint);
  }
  .ghost {
    height: 44px;
    border-radius: 12px;
    border: 1px solid var(--line-strong);
    background: transparent;
    color: var(--text);
    font-weight: 700;
    cursor: pointer;
  }
</style>
