<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { t } from '../i18n.svelte';

  interface Props {
    connected: boolean;
    lock: boolean | null;
  }
  let { connected, lock }: Props = $props();
  const win = getCurrentWindow();
</script>

<header class="bar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <span class="word" data-tauri-drag-region>Prizma</span>
    <span class="beam" aria-hidden="true"></span>
  </div>

  <div class="device" data-tauri-drag-region>
    <span class="chip" class:off={!connected}>
      <span class="led"></span>{connected ? t('bar.connected') : t('bar.notFound')}
    </span>
    {#if connected && lock !== null}
      <span class="chip lock" class:on={lock} title={t('bar.lockHint')}>
        <span class="key">Fn</span>{lock ? t('bar.lockOn') : t('bar.lockOff')}
      </span>
    {/if}
  </div>

  <div class="controls">
    <button aria-label={t('bar.minimize')} onclick={() => win.minimize()}>
      <svg viewBox="0 0 10 10"><path d="M1 5h8" /></svg>
    </button>
    <button aria-label={t('bar.maximize')} onclick={() => win.toggleMaximize()}>
      <svg viewBox="0 0 10 10"><rect x="1.5" y="1.5" width="7" height="7" rx="1" /></svg>
    </button>
    <button class="close" aria-label={t('bar.toTray')} title={t('bar.toTrayHint')} onclick={() => win.close()}>
      <svg viewBox="0 0 10 10"><path d="M2 2l6 6M8 2l-6 6" /></svg>
    </button>
  </div>
</header>

<style>
  .bar {
    height: 52px;
    display: flex;
    align-items: center;
    gap: var(--s5);
    padding-left: var(--s5);
    border-bottom: 1px solid var(--line);
    background: var(--ink);
  }
  .brand {
    position: relative;
    display: flex;
    align-items: center;
    height: 100%;
  }
  .word {
    font-family: var(--font-display);
    font-weight: 600;
    font-size: 17px;
    letter-spacing: 0.02em;
  }
  /* the refracted beam: white light in, spectrum out */
  .beam {
    position: absolute;
    left: 100%;
    margin-left: 10px;
    width: 120px;
    height: 2px;
    background: linear-gradient(90deg, #ffffff 0 18%, transparent 18% 22%, #ff3d6e 22%, #ffe14d 40%, #4dff91 58%, #3dd6ff 76%, #d24dff 100%);
    border-radius: 2px;
    opacity: 0.9;
    filter: drop-shadow(0 0 6px #6b5cff);
  }
  .device {
    flex: 1;
    display: flex;
    justify-content: center;
    gap: var(--s2);
    height: 100%;
    align-items: center;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: var(--s2);
    padding: 5px 12px 5px 10px;
    border-radius: var(--r-pill);
    background: var(--surface);
    border: 1px solid var(--line);
    font-size: 12.5px;
    font-weight: 600;
    color: var(--muted);
  }
  .led {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--ok);
    box-shadow: 0 0 8px var(--ok);
  }
  .chip.off .led {
    background: var(--danger);
    box-shadow: 0 0 8px var(--danger);
  }
  .key {
    display: inline-grid;
    place-items: center;
    min-width: 24px;
    height: 18px;
    padding: 0 4px;
    border-radius: 5px;
    background: var(--raised);
    border: 1px solid var(--line-strong);
    font-size: 10.5px;
    color: var(--muted);
  }
  .lock.on {
    color: var(--text);
    border-color: #ffffff55;
  }
  .lock.on .key {
    background: #ffffff;
    color: var(--ink);
    box-shadow: 0 0 12px #ffffffaa;
  }
  .controls {
    display: flex;
    height: 100%;
  }
  .controls button {
    width: 48px;
    height: 100%;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
  }
  .controls button:hover {
    background: var(--surface);
    color: var(--text);
  }
  .controls .close:hover {
    background: #c42b4c;
  }
  svg {
    width: 10px;
    height: 10px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.2;
  }
</style>
