# Prizma

Redragon K617 Fizz klavyesi için Windows ışık ve tuş stüdyosu.

Prizma, klavyenin kendi efektlerini (sabit, nefes, gökkuşağı, yılan ve diğerleri) renk, hız, parlaklık ve yön ayarlarıyla uygular. Ayarları klavyenin iki profiline birden yazar; böylece Fn kilidini açıp kapatmak ışığı değiştirmez. Uygulama sistem tepsisinde çalışır ve pencere kapatıldığında arka planda kalır.

## Durum

- **Klavyenin kendi efektleri:** 20 efekt; her efekt kendi rengini, hızını ve parlaklığını hatırlar (klavye bunları efekt başına ayrı saklar). Ön izlemeler klavyenin gerçek görüntüsüne göre modellenmiştir; tuşa basınca çalışan efektler ayrıca işaretlidir.
- **Canlı efektler (Prizma açıkken):** animasyonlar (gökkuşağı dalgası, sarmal, aurora, plazma, ateş, matrix ve diğerleri), tuşa basınca dalga/iz/ısı haritası, müziğe göre spektrum/bas/ses seviyesi, ekran rengi. Durdurunca ya da Prizma kapanınca klavye kendi efektine döner.
- **Renk düzeltmesi:** K617'nin LED'leri ekran renklerini düz değerle sürer ve mavisi çok güçlüdür; Prizma renkleri göndermeden önce gama (2,2) ve mavi dengesiyle düzeltir, böylece sarı beyaza, beyaz mora kaçmaz.
- Türkçe Q (ISO) ve ABD (ANSI) dizilim, Fn kilidi göstergesi, sistem tepsisi.

Sırada: katmanlı efekt editörü, oyuna göre profil, sistem göstergeleri, tuş atama ve makro.

## Güvenlik

- Prizma klavyenin firmware'ine dokunmaz; yalnızca ayar bloklarını, resmi uygulamanın kullandığı yolla yazar.
- Her yazmadan önce klavyedeki güncel bloklar okunur ve yalnızca değişen baytlar değiştirilir. Renk değişmediyse yalnızca ayar bloğu yazılır (daha az flaş yazması).
- Canlı efektler flaşa hiç yazmaz; klavyeye geçici kareler gönderir.
- Profil kaydı bozuksa (ikinci profil hiç kullanılmamışsa) önce birinci profil kopyalanır; aksi halde firmware tüm ayarları fabrika değerlerine döndürür.

## Geliştirme

```
npm install
npm run tauri dev
```

Gerekenler: Node.js, Rust (MSVC), Visual Studio Build Tools, WebView2.

## Teşekkür

- Protokol araştırması için [fizzctl](https://github.com/ayoubdya/fizzctl) (MIT).
- Flash okuma için [sinowisp](https://github.com/karstx/sinowisp).

Redragon ve K617 Fizz, sahiplerinin markalarıdır. Bu proje Redragon ile bağlantılı değildir.

## Lisans

GPL-3.0-or-later. Ayrıntılar için `LICENSE` dosyasına bakın.
