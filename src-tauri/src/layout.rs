//! Physical key layouts (key rectangles + LED indices) used by the software effect engine.

use serde::Deserialize;

#[derive(Deserialize)]
struct RawKey {
    name: String,
    ansi: String,
    leds: Vec<usize>,
    rects: Vec<[f32; 4]>,
}

#[derive(Deserialize)]
struct RawLayout {
    id: String,
    keys: Vec<RawKey>,
}

#[derive(Deserialize)]
struct RawFile {
    layouts: Vec<RawLayout>,
}

#[derive(Clone, Debug)]
pub struct Key {
    pub name: String,
    pub ansi: String,
    pub leds: Vec<usize>,
    /// normalised 0..1 rectangles (x0, y0, x1, y1)
    pub rects: Vec<[f32; 4]>,
    pub cx: f32,
    pub cy: f32,
    pub seed: f32,
}

#[derive(Clone, Debug)]
pub struct Layout {
    pub id: String,
    pub keys: Vec<Key>,
    /// height / width of the key area, for round effects
    pub aspect: f32,
}

fn hash(n: u32) -> f32 {
    let mut x = n.wrapping_mul(0x9e37_79b9) ^ 0x85eb_ca6b;
    x ^= x >> 13;
    x = x.wrapping_mul(0xc2b2_ae35);
    x ^= x >> 16;
    x as f32 / u32::MAX as f32
}

pub fn load(json: &str, id: &str) -> Option<Layout> {
    let f: RawFile = serde_json::from_str(json).ok()?;
    let raw = f.layouts.into_iter().find(|l| l.id == id)?;
    let all: Vec<[f32; 4]> = raw.keys.iter().flat_map(|k| k.rects.clone()).collect();
    let x0 = all.iter().map(|r| r[0]).fold(f32::MAX, f32::min);
    let y0 = all.iter().map(|r| r[1]).fold(f32::MAX, f32::min);
    let x1 = all.iter().map(|r| r[2]).fold(f32::MIN, f32::max);
    let y1 = all.iter().map(|r| r[3]).fold(f32::MIN, f32::max);
    let (w, h) = (x1 - x0, y1 - y0);
    let keys = raw
        .keys
        .into_iter()
        .enumerate()
        .map(|(i, k)| {
            let rects: Vec<[f32; 4]> =
                k.rects.iter().map(|r| [(r[0] - x0) / w, (r[1] - y0) / h, (r[2] - x0) / w, (r[3] - y0) / h]).collect();
            let rx0 = rects.iter().map(|r| r[0]).fold(f32::MAX, f32::min);
            let rx1 = rects.iter().map(|r| r[2]).fold(f32::MIN, f32::max);
            let ry0 = rects.iter().map(|r| r[1]).fold(f32::MAX, f32::min);
            let ry1 = rects.iter().map(|r| r[3]).fold(f32::MIN, f32::max);
            Key {
                name: k.name,
                ansi: k.ansi,
                leds: k.leds,
                rects,
                cx: (rx0 + rx1) / 2.0,
                cy: (ry0 + ry1) / 2.0,
                seed: hash(i as u32 * 7919 + 17),
            }
        })
        .collect();
    Some(Layout { id: raw.id, keys, aspect: h / w })
}

/// Windows set-1 scan code (with the extended flag in bit 8) for a vendor ANSI key name, plus
/// the alternates the Fn layer sends from the same physical key.
pub fn scancodes(ansi: &str) -> &'static [u16] {
    const E: u16 = 0x100;
    match ansi {
        "Esc" => &[0x01],
        "1" => &[0x02, 0x3B],
        "2" => &[0x03, 0x3C],
        "3" => &[0x04, 0x3D],
        "4" => &[0x05, 0x3E],
        "5" => &[0x06, 0x3F],
        "6" => &[0x07, 0x40],
        "7" => &[0x08, 0x41],
        "8" => &[0x09, 0x42],
        "9" => &[0x0A, 0x43],
        "0" => &[0x0B, 0x44],
        "-" => &[0x0C, 0x57],
        "=" => &[0x0D, 0x58],
        "Backspace" => &[0x0E],
        "Tab" => &[0x0F],
        "Q" => &[0x10],
        "W" => &[0x11, E | 0x48],
        "E" => &[0x12],
        "R" => &[0x13],
        "T" => &[0x14],
        "Y" => &[0x15],
        "U" => &[0x16],
        "I" => &[0x17],
        "O" => &[0x18],
        "P" => &[0x19],
        "[" => &[0x1A, E | 0x52],
        "]" => &[0x1B, E | 0x53],
        "Enter" => &[0x1C],
        "LCtrl" => &[0x1D],
        "A" => &[0x1E, E | 0x4B],
        "S" => &[0x1F, E | 0x50],
        "D" => &[0x20, E | 0x4D],
        "F" => &[0x21],
        "G" => &[0x22],
        "H" => &[0x23],
        "J" => &[0x24],
        "K" => &[0x25],
        "L" => &[0x26],
        ";" => &[0x27, E | 0x47],
        "'" => &[0x28, E | 0x4F],
        "`" => &[0x29],
        "LShift" => &[0x2A],
        "ISO#" | "\\" => &[0x2B],
        "Z" => &[0x2C],
        "X" => &[0x2D],
        "C" => &[0x2E],
        "V" => &[0x2F],
        "B" => &[0x30],
        "N" => &[0x31],
        "M" => &[0x32],
        "," => &[0x33],
        "." => &[0x34, E | 0x49],
        "/" => &[0x35, E | 0x51],
        "RShift" => &[0x36],
        "LAlt" => &[0x38],
        "Space" => &[0x39],
        "CapsLock" => &[0x3A],
        "ISO\\" => &[0x56],
        "RCtrl" => &[E | 0x1D],
        "RAlt" => &[E | 0x38],
        "LWin" => &[E | 0x5B],
        "App" => &[E | 0x5D],
        _ => &[],
    }
}
