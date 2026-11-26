// Interface language. Templates that call `t()` re-render when the language changes, because
// `t()` reads the reactive `i18n.lang`.

export type Lang = 'tr' | 'en';
/** A text in both languages (catalogue entries keep their translations next to their data). */
export type L = { tr: string; en: string };

function initial(): Lang {
  try {
    const v = localStorage.getItem('prizma.lang');
    if (v === 'tr' || v === 'en') return v;
  } catch {
    /* storage unavailable */
  }
  return (navigator.language || '').toLowerCase().startsWith('tr') ? 'tr' : 'en';
}

export const i18n = $state({ lang: initial() });

export function setLang(lang: Lang) {
  i18n.lang = lang;
  try {
    localStorage.setItem('prizma.lang', lang);
  } catch {
    /* storage unavailable */
  }
  document.documentElement.lang = lang;
}

/** Pick the current language from a two-language text. */
export const tx = (l: L) => l[i18n.lang];

const dict = {
  // navigation
  'nav.sections': { tr: 'Bölümler', en: 'Sections' },
  'nav.light': { tr: 'Işık', en: 'Lighting' },
  'nav.live': { tr: 'Canlı', en: 'Live' },
  'nav.liveRunning': { tr: 'Canlı efekt çalışıyor', en: 'Live effect running' },
  'nav.settings': { tr: 'Ayarlar', en: 'Settings' },

  // title bar
  'bar.connected': { tr: 'K617 bağlı', en: 'K617 connected' },
  'bar.notFound': { tr: 'Klavye bulunamadı', en: 'Keyboard not found' },
  'bar.lockOn': { tr: 'Kilit açık', en: 'Lock on' },
  'bar.lockOff': { tr: 'Kilit kapalı', en: 'Lock off' },
  'bar.lockHint': { tr: "Fn kilidi: klavyenin ikinci tuş profili", en: "Fn lock: the keyboard's second key profile" },
  'bar.minimize': { tr: 'Küçült', en: 'Minimize' },
  'bar.maximize': { tr: 'Büyüt', en: 'Maximize' },
  'bar.toTray': { tr: 'Tepsiye küçült', en: 'Close to tray' },
  'bar.toTrayHint': { tr: 'Prizma tepside çalışmaya devam eder', en: 'Prizma keeps running in the tray' },

  // lighting view
  'light.pressNote': { tr: 'Tuşa basınca çalışır · ön izlemede sanal tuş basışları', en: 'Reacts to key presses · the preview shows simulated presses' },
  'light.plugIn': { tr: 'Klavyeyi tak; Prizma onu kendisi bulur.', en: 'Plug in the keyboard; Prizma finds it on its own.' },
  'light.liveRunning': { tr: 'Klavyede şu an canlı bir efekt çalışıyor. Buradan bir efekt uygularsan canlı efekt durur.', en: 'A live effect is running on the keyboard. Applying an effect here stops it.' },
  'light.stripAria': { tr: 'Klavyenin kendi efektleri', en: "The keyboard's own effects" },
  'light.strip': { tr: 'Klavyenin efektleri', en: 'Keyboard effects' },
  'light.hostMode': { tr: 'Klavye bilgisayar kontrolü modunda kalmış. Bir efekt seçip uygula.', en: 'The keyboard is still in computer-controlled mode. Pick an effect and apply it.' },
  'light.applied': { tr: 'Klavyeye uygulandı, iki profile de yazıldı.', en: 'Applied to the keyboard and saved to both profiles.' },
  'light.writeFailed': { tr: 'Yazılamadı: {e}', en: 'Could not write: {e}' },

  // inspector (keyboard effects)
  'insp.reactiveTitle': { tr: 'Tuşa basınca çalışır', en: 'Reacts to key presses' },
  'insp.reactiveText': { tr: 'Klavye sen bir tuşa basana kadar karanlık kalır; bu bir arıza değil. Ön izlemede sanal tuş basışları gösteriliyor.', en: 'The keyboard stays dark until you press a key; that is not a fault. The preview shows simulated presses.' },
  'insp.approx': { tr: 'Ekrandaki ön izleme yaklaşık; asıl görünüm klavyede. Her efekt kendi rengini ve hızını hatırlar.', en: 'The on-screen preview is an approximation; the keyboard shows the real thing. Every effect remembers its own colour and speed.' },
  'insp.colour': { tr: 'Renk', en: 'Colour' },
  'insp.colourMode': { tr: 'Renk modu', en: 'Colour mode' },
  'insp.multi': { tr: 'Çok renkli', en: 'Multicolour' },
  'insp.single': { tr: 'Tek renk', en: 'Single colour' },
  'insp.ownColour': { tr: 'Kendi rengini seç', en: 'Pick your own colour' },
  'insp.speed': { tr: 'Hız', en: 'Speed' },
  'insp.brightness': { tr: 'Parlaklık', en: 'Brightness' },
  'insp.reverse': { tr: 'Ters yön', en: 'Reverse direction' },
  'insp.note': { tr: 'Ayar iki profile birden yazılır; Fn kilidini açıp kapatmak ışığı değiştirmez.', en: 'Saved to both key profiles, so toggling Fn lock never changes the lighting.' },
  'insp.applying': { tr: 'Uygulanıyor…', en: 'Applying…' },
  'insp.apply': { tr: 'Klavyeye uygula', en: 'Apply to keyboard' },
  'insp.onKeyboard': { tr: 'Klavyede bu var', en: 'On the keyboard now' },
  'insp.revert': { tr: 'Değişiklikleri geri al', en: 'Undo changes' },

  // effect tiles
  'tile.press': { tr: 'Tuşa basınca', en: 'On key press' },
  'tile.pressHint': { tr: 'Tuşa basınca çalışır; basmazsan klavye karanlık kalır.', en: 'Reacts to key presses; the keyboard stays dark until you press one.' },
  'tile.pressAria': { tr: '{name}, tuşa basınca çalışır', en: '{name}, reacts to key presses' },
  'tile.current': { tr: 'Klavyede şu an bu efekt var', en: 'This effect is on the keyboard now' },

  // live view
  'live.paintBanner': { tr: 'Tuşlara tıkla ya da üzerlerinde sürükle; seçili katman yalnızca bu tuşlarda görünür.', en: 'Click or drag over keys; the selected layer only shows on these keys.' },
  'live.done': { tr: 'Bitti', en: 'Done' },
  'live.audioBanner': { tr: 'Müzik ya da video oynat; efekt bilgisayardan çıkan sese tepki verir.', en: 'Play music or a video; the effect follows the sound your PC plays.' },
  'live.reactBanner': { tr: 'Klavyede yazmaya başla; her basış efekti tetikler.', en: 'Start typing; every key press triggers the effect.' },
  'live.stripAria': { tr: 'Canlı efektler', en: 'Live effects' },
  'live.startFailed': { tr: 'Başlatılamadı: {e}', en: 'Could not start: {e}' },
  'live.stopped': { tr: 'Klavye kendi efektine döndü.', en: "The keyboard is back on its own effect." },
  'live.stopFailed': { tr: 'Durdurulamadı: {e}', en: 'Could not stop: {e}' },
  'live.saved': { tr: '"{name}" sahnesi kaydedildi.', en: 'Saved scene "{name}".' },
  'live.saveFailed': { tr: 'Kaydedilemedi: {e}', en: 'Could not save: {e}' },
  'live.deleted': { tr: '"{name}" sahnesi silindi.', en: 'Deleted scene "{name}".' },
  'live.deleteFailed': { tr: 'Silinemedi: {e}', en: 'Could not delete: {e}' },

  // layer panel
  'lp.scene': { tr: 'Sahne', en: 'Scene' },
  'lp.unsaved': { tr: 'Kaydedilmemiş sahne', en: 'Unsaved scene' },
  'lp.save': { tr: 'Kaydet', en: 'Save' },
  'lp.saveOver': { tr: 'Değişiklikleri bu sahneye yaz', en: 'Write the changes to this scene' },
  'lp.saveAs': { tr: 'Farklı kaydet', en: 'Save as' },
  'lp.confirmDelete': { tr: 'Silinsin', en: 'Delete' },
  'lp.deleteScene': { tr: 'Sahneyi sil', en: 'Delete scene' },
  'lp.changed': { tr: 'Bu sahnede kaydedilmemiş değişiklik var.', en: 'This scene has unsaved changes.' },
  'lp.namePlaceholder': { tr: 'Sahne adı (ör. Oyun)', en: 'Scene name (e.g. Gaming)' },
  'lp.name': { tr: 'Sahne adı', en: 'Scene name' },
  'lp.cancel': { tr: 'Vazgeç', en: 'Cancel' },
  'lp.layers': { tr: 'Katmanlar', en: 'Layers' },
  'lp.addLayer': { tr: '+ Katman', en: '+ Layer' },
  'lp.hide': { tr: 'Katmanı gizle', en: 'Hide layer' },
  'lp.show': { tr: 'Katmanı göster', en: 'Show layer' },
  'lp.up': { tr: 'Yukarı taşı', en: 'Move up' },
  'lp.down': { tr: 'Aşağı taşı', en: 'Move down' },
  'lp.remove': { tr: 'Katmanı sil', en: 'Delete layer' },
  'lp.layersHint': { tr: 'Üstteki katman alttakilerin üstüne biner. Aşağıdaki efekt listesinden seçtiğin efekt, seçili katmana gelir.', en: 'Upper layers sit on top of lower ones. The effect you pick in the list below goes into the selected layer.' },
  'lp.keys': { tr: 'Tuşlar', en: 'Keys' },
  'lp.allKeys': { tr: 'Tüm klavye', en: 'Whole keyboard' },
  'lp.nKeys': { tr: '{n} tuş', en: '{n} keys' },
  'lp.paintOn': { tr: 'Seçim açık: klavyede tuşlara tıkla ya da sürükle', en: 'Picking: click or drag over keys on the keyboard' },
  'lp.paintOff': { tr: 'Klavyede tuş seç', en: 'Pick keys on the keyboard' },
  'lp.blend': { tr: 'Karışım', en: 'Blend' },
  'lp.opacity': { tr: 'Opaklık', en: 'Opacity' },
  'lp.colours': { tr: 'Renkler', en: 'Colours' },
  'lp.changeColour': { tr: 'Rengi değiştir', en: 'Change colour' },
  'lp.addColour': { tr: 'Renk ekle', en: 'Add colour' },
  'lp.removeColour': { tr: 'Son rengi çıkar', en: 'Remove last colour' },
  'lp.direction': { tr: 'Yön', en: 'Direction' },
  'lp.speed': { tr: 'Hız', en: 'Speed' },
  'lp.scale': { tr: 'Desen boyu', en: 'Pattern size' },
  'lp.brightness': { tr: 'Parlaklık', en: 'Brightness' },
  'lp.note': { tr: "Canlı efektler Prizma açıkken çalışır; pencereyi kapatınca tepside sürer. Durdurduğunda ya da Prizma'dan çıktığında klavye kendi efektine döner.", en: 'Live effects run while Prizma is open and keep going in the tray when you close the window. Stop them or quit Prizma and the keyboard goes back to its own effect.' },
  'lp.running': { tr: 'Klavyede çalışıyor; değişiklikler anında yansır.', en: 'Running on the keyboard; changes show up instantly.' },
  'lp.stopping': { tr: 'Durduruluyor…', en: 'Stopping…' },
  'lp.stop': { tr: 'Durdur ve klavyenin efektine dön', en: "Stop and return to the keyboard's effect" },
  'lp.starting': { tr: 'Başlatılıyor…', en: 'Starting…' },
  'lp.start': { tr: 'Klavyede başlat', en: 'Start on keyboard' },

  // settings
  'set.title': { tr: 'Ayarlar', en: 'Settings' },
  'set.layout': { tr: 'Klavye düzeni', en: 'Keyboard layout' },
  'set.layoutHint': { tr: 'Efektler tuşların gerçek yerine göre hesaplanır; klavyendeki dizilimi seç.', en: 'Effects follow where the keys really are; pick the layout of your keyboard.' },
  'set.language': { tr: 'Dil', en: 'Language' },
  'set.startup': { tr: 'Başlangıç', en: 'Startup' },
  'set.autostart': { tr: "Windows açılınca Prizma'yı tepside başlat", en: 'Start Prizma in the tray when Windows starts' },
  'set.autostartHint': { tr: 'Canlı efektlerin ve tepsi simgesinin oturum açınca hazır olması için.', en: 'So live effects and the tray icon are ready as soon as you sign in.' },

  // errors coming from the app core (mapped from its fixed English messages)
  'err.notAccepted': { tr: 'klavye ayarı kabul etmedi (geri okumada efekt değişmemiş)', en: 'the keyboard did not accept the setting (unchanged when read back)' },
  'err.sceneEmpty': { tr: 'sahnenin bir adı olmalı', en: 'the scene needs a name' },
  'err.sceneLong': { tr: 'sahne adı en fazla 40 karakter olabilir', en: 'scene names can be at most 40 characters' },
  'err.notFound': { tr: 'klavye bulunamadı', en: 'keyboard not found' },
  'err.frame': { tr: 'Klavyeye kare gönderilemedi: {e}', en: 'Could not send a frame to the keyboard: {e}' },
} satisfies Record<string, L>;

export type Key = keyof typeof dict;

export function t(key: Key, vars?: Record<string, string | number>): string {
  let s: string = dict[key][i18n.lang];
  if (vars) for (const [k, v] of Object.entries(vars)) s = s.replaceAll(`{${k}}`, String(v));
  return s;
}

/** Messages from the Rust core are fixed English strings; show the known ones in the UI language. */
export function errText(e: unknown): string {
  const s = String(e);
  if (s.includes('did not accept')) return t('err.notAccepted');
  if (s.includes('scene name is empty')) return t('err.sceneEmpty');
  if (s.includes('scene name is longer')) return t('err.sceneLong');
  if (/collection not found|K617 .* not found/i.test(s)) return t('err.notFound');
  const frame = s.match(/^could not send a frame to the keyboard: (.*)$/);
  if (frame) return t('err.frame', { e: frame[1] });
  return s;
}
