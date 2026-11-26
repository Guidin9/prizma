<script lang="ts">
  import type { Rgb, SwEffect } from '../api';
  import { DIRECTIONS, PALETTES, type SwInfo } from '../swcatalog';

  interface Props {
    info: SwInfo;
    effect: SwEffect;
    live: boolean;
    connected: boolean;
    busy: boolean;
    message: string;
    failed: boolean;
    onchange: (e: SwEffect) => void;
    onstart: () => void;
    onstop: () => void;
  }
  let { info, effect, live, connected, busy, message, failed, onchange, onstart, onstop }: Props = $props();

  const hex = (c: Rgb) => '#' + c.map((v) => Math.round(v).toString(16).padStart(2, '0')).join('');
  const fromHex = (h: string): Rgb => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16)) as Rgb;
  const set = (patch: Partial<SwEffect>) => onchange({ ...effect, ...patch });
  const same = (a: Rgb[], b: Rgb[]) => JSON.stringify(a) === JSON.stringify(b);
  const swatch = (cols: Rgb[]) =>
    cols.length === 0
      ? 'linear-gradient(90deg,#ff3d6e,#ffe14d,#4dff91,#3dd6ff,#d24dff)'
      : `linear-gradient(90deg,${cols.map(hex).join(',')}${cols.length === 1 ? ',' + hex(cols[0]) : ''})`;
</script>

<aside class="inspector">
  <header>
    <h2>{info.name}</h2>
    <p class="desc">{info.desc}</p>
  </header>

  {#if info.uses.colours}
    <section>
      <h3>Renkler</h3>
      <div class="palettes">
        {#each PALETTES as p}
          <button class="pal" class:on={same(p.colours, effect.colours)} onclick={() => set({ colours: p.colours })} title={p.name}>
            <span class="bar" style:background={swatch(p.colours)}></span>
            <span class="pname">{p.name}</span>
          </button>
        {/each}
      </div>
      {#if effect.colours.length}
        <div class="custom">
          {#each effect.colours as c, i}
            <label class="sw" style:background={hex(c)} title="Rengi değiştir">
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
          {#if effect.colours.length < 5}
            <button class="add" aria-label="Renk ekle" onclick={() => set({ colours: [...effect.colours, [255, 255, 255]] })}>+</button>
          {/if}
          {#if effect.colours.length > 1}
            <button class="add" aria-label="Son rengi çıkar" onclick={() => set({ colours: effect.colours.slice(0, -1) })}>−</button>
          {/if}
        </div>
      {/if}
    </section>
  {/if}

  {#if info.uses.direction}
    <section>
      <h3>Yön</h3>
      <div class="dirs">
        {#each info.uses.direction as d}
          <button class:on={effect.direction === d} onclick={() => set({ direction: d })}>{DIRECTIONS[d]}</button>
        {/each}
      </div>
    </section>
  {/if}

  <section>
    <h3>Hız <span class="val">{effect.speed.toFixed(1)}×</span></h3>
    <input type="range" min="0.2" max="3" step="0.1" value={effect.speed} oninput={(e) => set({ speed: +(e.target as HTMLInputElement).value })} />
  </section>

  {#if info.uses.scale}
    <section>
      <h3>Desen boyu <span class="val">{effect.scale.toFixed(1)}×</span></h3>
      <input type="range" min="0.4" max="3" step="0.1" value={effect.scale} oninput={(e) => set({ scale: +(e.target as HTMLInputElement).value })} />
    </section>
  {/if}

  <section>
    <h3>Parlaklık <span class="val">%{Math.round(effect.brightness * 100)}</span></h3>
    <input type="range" min="0.05" max="1" step="0.05" value={effect.brightness} oninput={(e) => set({ brightness: +(e.target as HTMLInputElement).value })} />
  </section>

  <footer>
    <p class="note">Bu efektler Prizma açıkken çalışır. Durdurduğunda ya da Prizma kapandığında klavye kendi efektine döner.</p>
    {#if message}<p class="message" class:failed role="status">{message}</p>{/if}
    {#if live}
      <div class="livebadge"><span class="pulse"></span>Klavyede çalışıyor; değişiklikler anında yansır.</div>
      <button class="ghost strong" disabled={busy} onclick={onstop}>{busy ? 'Durduruluyor…' : 'Durdur ve klavyenin efektine dön'}</button>
    {:else}
      <button class="primary" disabled={!connected || busy} onclick={onstart}>{busy ? 'Başlatılıyor…' : 'Klavyede başlat'}</button>
    {/if}
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
  }
  .desc {
    margin: var(--s2) 0 0;
    color: var(--muted);
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
  .dirs {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .dirs button {
    padding: 7px 12px;
    border-radius: 999px;
    border: 1px solid var(--line);
    background: var(--raised);
    cursor: pointer;
    font-weight: 600;
    font-size: 12.5px;
    color: var(--muted);
  }
  .dirs button.on {
    color: var(--ink);
    background: var(--text);
    border-color: var(--text);
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
