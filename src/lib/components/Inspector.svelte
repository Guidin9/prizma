<script lang="ts">
  import type { EffectInfo, HwEffect, Rgb } from '../api';
  import { sameEffect } from '../api';
  import { DESCRIPTIONS } from '../preview';

  interface Props {
    info: EffectInfo;
    draft: HwEffect;
    applied: HwEffect | null;
    connected: boolean;
    busy: boolean;
    message: string;
    failed: boolean;
    onchange: (e: HwEffect) => void;
    onapply: () => void;
    onrevert: () => void;
  }
  let { info, draft, applied, connected, busy, message, failed, onchange, onapply, onrevert }: Props = $props();

  const SWATCHES: Rgb[] = [
    [255, 61, 110],
    [255, 157, 46],
    [255, 225, 77],
    [77, 255, 145],
    [61, 214, 255],
    [107, 92, 255],
    [210, 77, 255],
    [255, 255, 255],
  ];
  const hex = (c: Rgb) => '#' + c.map((v) => v.toString(16).padStart(2, '0')).join('');
  const fromHex = (h: string): Rgb => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16)) as Rgb;
  const set = (patch: Partial<HwEffect>) => onchange({ ...draft, ...patch });
  let dirty = $derived(!sameEffect(draft, applied));
</script>

<aside class="inspector">
  <header>
    <h2>{info.name}</h2>
    <p class="desc">{DESCRIPTIONS[info.id] ?? ''}</p>
  </header>

  {#if info.colour}
    <section>
      <h3>Renk</h3>
      <div class="mode" role="radiogroup" aria-label="Renk modu">
        <button role="radio" aria-checked={draft.colour === null} class:on={draft.colour === null} onclick={() => set({ colour: null })}>
          <span class="rainbow"></span>Çok renkli
        </button>
        <button role="radio" aria-checked={draft.colour !== null} class:on={draft.colour !== null} onclick={() => set({ colour: draft.colour ?? [61, 214, 255] })}>
          <span class="solid" style:background={draft.colour ? hex(draft.colour) : '#3dd6ff'}></span>Tek renk
        </button>
      </div>
      {#if draft.colour}
        <div class="swatches">
          {#each SWATCHES as c}
            <button
              class="sw"
              aria-label={hex(c)}
              class:on={hex(c) === hex(draft.colour)}
              style:--c={hex(c)}
              onclick={() => set({ colour: c })}
            ></button>
          {/each}
          <label class="sw custom" title="Kendi rengini seç">
            <input type="color" value={hex(draft.colour)} oninput={(e) => set({ colour: fromHex((e.target as HTMLInputElement).value) })} />
          </label>
        </div>
      {/if}
    </section>
  {/if}

  {#if info.animated}
    <section>
      <h3>Hız</h3>
      <div class="steps" role="radiogroup" aria-label="Hız">
        {#each [1, 2, 3, 4, 5] as v}
          <button role="radio" aria-checked={draft.speed === v} class:on={draft.speed >= v} onclick={() => set({ speed: v })}>{v}</button>
        {/each}
      </div>
    </section>
  {/if}

  {#if info.id !== 0x16}
    <section>
      <h3>Parlaklık</h3>
      <div class="steps" role="radiogroup" aria-label="Parlaklık">
        {#each [0, 1, 2, 3, 4] as v}
          <button role="radio" aria-checked={draft.brightness === v} class:on={draft.brightness >= v} onclick={() => set({ brightness: v })}>{v + 1}</button>
        {/each}
      </div>
    </section>
  {/if}

  {#if info.animated}
    <section class="row">
      <h3>Ters yön</h3>
      <button class="switch" role="switch" aria-checked={draft.reverse} aria-label="Ters yön" class:on={draft.reverse} onclick={() => set({ reverse: !draft.reverse })}>
        <span></span>
      </button>
    </section>
  {/if}

  <footer>
    <p class="note">Ayar iki profile birden yazılır; Fn kilidini açıp kapatmak ışığı değiştirmez.</p>
    {#if message}<p class="message" class:failed role="status">{message}</p>{/if}
    <div class="actions">
      <button class="primary" disabled={!connected || busy || !dirty} onclick={onapply}>
        {busy ? 'Uygulanıyor…' : dirty ? 'Klavyeye uygula' : 'Klavyede bu var'}
      </button>
      {#if dirty && applied}
        <button class="ghost" onclick={onrevert} disabled={busy}>Değişiklikleri geri al</button>
      {/if}
    </div>
  </footer>
</aside>

<style>
  .inspector {
    display: flex;
    flex-direction: column;
    gap: var(--s5);
    padding: var(--s6) var(--s5) var(--s5);
    background: var(--surface);
    border-left: 1px solid var(--line);
    overflow-y: auto;
  }
  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-weight: 500;
    font-size: 24px;
    line-height: 1.15;
    letter-spacing: -0.01em;
  }
  .desc {
    margin: var(--s2) 0 0;
    color: var(--muted);
  }
  h3 {
    margin: 0 0 var(--s3);
    font-size: 13px;
    font-weight: 700;
    color: var(--muted);
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .row h3 {
    margin: 0;
  }
  .mode {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--s2);
  }
  .mode button {
    display: flex;
    align-items: center;
    gap: var(--s2);
    padding: 10px 12px;
    background: var(--raised);
    border: 1px solid var(--line);
    border-radius: 10px;
    cursor: pointer;
    font-weight: 600;
  }
  .mode button.on {
    border-color: var(--line-strong);
    background: #2a2545;
  }
  .rainbow,
  .solid {
    width: 16px;
    height: 16px;
    border-radius: 5px;
    flex: none;
  }
  .rainbow {
    background: conic-gradient(#ff3d6e, #ff9d2e, #ffe14d, #4dff91, #3dd6ff, #6b5cff, #d24dff, #ff3d6e);
  }
  .swatches {
    display: grid;
    grid-template-columns: repeat(9, 1fr);
    gap: 6px;
    margin-top: var(--s3);
  }
  .sw {
    aspect-ratio: 1;
    border-radius: 8px;
    border: 2px solid transparent;
    background: var(--c);
    cursor: pointer;
    padding: 0;
    box-shadow: 0 0 12px -2px var(--c);
  }
  .sw.on {
    border-color: var(--text);
  }
  .custom {
    position: relative;
    background: conic-gradient(#ff3d6e, #ffe14d, #4dff91, #3dd6ff, #d24dff, #ff3d6e);
    box-shadow: none;
    overflow: hidden;
  }
  .custom input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
  }
  .steps {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 6px;
  }
  .steps button {
    height: 36px;
    border-radius: 8px;
    border: 1px solid var(--line);
    background: var(--raised);
    color: var(--faint);
    font-family: var(--font-display);
    font-size: 13px;
    cursor: pointer;
    transition: background 120ms, color 120ms;
  }
  .steps button.on {
    background: #3a3266;
    border-color: #5a4fa0;
    color: var(--text);
  }
  .switch {
    width: 44px;
    height: 24px;
    border-radius: var(--r-pill);
    border: 1px solid var(--line-strong);
    background: var(--raised);
    position: relative;
    cursor: pointer;
    padding: 0;
  }
  .switch span {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--muted);
    transition: transform 160ms, background 160ms;
  }
  .switch.on span {
    transform: translateX(20px);
    background: var(--text);
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
  .actions {
    display: grid;
    gap: var(--s2);
  }
  .primary {
    position: relative;
    height: 46px;
    border-radius: 12px;
    border: none;
    background: var(--text);
    color: var(--ink);
    font-weight: 800;
    font-size: 14.5px;
    cursor: pointer;
  }
  .primary:not(:disabled)::after {
    content: '';
    position: absolute;
    inset: auto 14px -6px;
    height: 10px;
    background: var(--spectrum);
    filter: blur(10px);
    opacity: 0.8;
    z-index: -1;
  }
  .primary:disabled {
    background: var(--raised);
    color: var(--faint);
    cursor: default;
  }
  .ghost {
    height: 36px;
    border-radius: 10px;
    border: 1px solid var(--line);
    background: transparent;
    color: var(--muted);
    cursor: pointer;
  }
</style>
