# Prizma

Redragon K617 Fizz klavyesi için Windows ışık ve tuş stüdyosu.

Prizma, klavyenin kendi efektlerini (sabit, nefes, gökkuşağı, yılan ve diğerleri) renk, hız, parlaklık ve yön ayarlarıyla uygular. Ayarları klavyenin iki profiline birden yazar; böylece Fn kilidini açıp kapatmak ışığı değiştirmez. Uygulama sistem tepsisinde çalışır ve pencere kapatıldığında arka planda kalır.

## Durum

Aşama 1 (bu sürüm): klavyenin kendi efektleri, canlı klavye önizlemesi, Türkçe Q (ISO) ve ABD (ANSI) dizilim, Fn kilidi göstergesi, tepsi.

Sırada: bilgisayarda çalışan efektler (müzik, ekran rengi, tuşa basınca dalga, animasyon kütüphanesi), katmanlı efekt editörü, oyuna göre profil, sistem göstergeleri, tuş atama ve makro.

## Güvenlik

- Prizma klavyenin firmware'ine dokunmaz; yalnızca ayar bloklarını, resmi uygulamanın kullandığı yolla yazar.
- Her yazmadan önce klavyedeki güncel bloklar okunur ve yalnızca değişen baytlar değiştirilir.
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
