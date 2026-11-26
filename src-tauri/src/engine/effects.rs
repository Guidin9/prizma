//! Software effects rendered on the host and streamed as per-key frames.

use crate::layout::{Key, Layout};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

fn one() -> f32 {
    1.0
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct SwEffect {
    pub kind: String,
    #[serde(default = "one")]
    pub speed: f32,
    #[serde(default = "one")]
    pub brightness: f32,
    /// empty = full spectrum
    #[serde(default)]
    pub colours: Vec<[u8; 3]>,
    #[serde(default)]
    pub direction: String,
    #[serde(default = "one")]
    pub scale: f32,
}

pub const AUDIO: &[&str] = &["spectrum", "bass", "vu"];
pub const REACTIVE: &[&str] = &["ripple", "trail", "heatmap"];
pub const SCREEN: &[&str] = &["ambient"];

#[derive(Default, Clone)]
pub struct AudioLevels {
    /// 16 log-spaced bands, 0..1 (auto-gained)
    pub bands: [f32; 16],
    pub level: f32,
    pub bass: f32,
}

#[derive(Default, Clone)]
pub struct ScreenGrid {
    pub w: usize,
    pub h: usize,
    pub px: Vec<[f32; 3]>,
}

pub struct Inputs<'a> {
    /// keys pressed since the previous frame: (key index, engine time)
    pub presses: &'a [(usize, f32)],
    pub audio: &'a AudioLevels,
    pub screen: &'a ScreenGrid,
}

struct Ripple {
    x: f32,
    y: f32,
    t0: f32,
    hue: f32,
}

#[derive(Default)]
pub struct State {
    kind: String,
    per_key: Vec<f32>,
    aux: Vec<f32>,
    ripples: Vec<Ripple>,
    last_t: f32,
}

pub type Rgb = [f32; 3];

fn hsv(h: f32, s: f32, v: f32) -> Rgb {
    let h = h.rem_euclid(1.0) * 6.0;
    let i = h.floor() as i32;
    let f = h - i as f32;
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - f * s), v * (1.0 - (1.0 - f) * s));
    let (r, g, b) = match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    [r * 255.0, g * 255.0, b * 255.0]
}

fn lerp(a: Rgb, b: Rgb, t: f32) -> Rgb {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn scale(c: Rgb, k: f32) -> Rgb {
    [c[0] * k, c[1] * k, c[2] * k]
}

/// Cyclic palette lookup; an empty palette means the full spectrum.
fn palette(cols: &[[u8; 3]], u: f32) -> Rgb {
    if cols.is_empty() {
        return hsv(u, 1.0, 1.0);
    }
    if cols.len() == 1 {
        let c = cols[0];
        return [c[0] as f32, c[1] as f32, c[2] as f32];
    }
    let n = cols.len() as f32;
    let x = u.rem_euclid(1.0) * n;
    let i = x.floor() as usize % cols.len();
    let j = (i + 1) % cols.len();
    let f = x - x.floor();
    let f = f * f * (3.0 - 2.0 * f);
    let to = |c: [u8; 3]| [c[0] as f32, c[1] as f32, c[2] as f32];
    lerp(to(cols[i]), to(cols[j]), f)
}

fn h3(x: i32, y: i32, z: i32) -> f32 {
    let mut n = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841) ^ (z as u32).wrapping_mul(0xcb1a_b31f);
    n ^= n >> 13;
    n = n.wrapping_mul(0x85eb_ca6b);
    n ^= n >> 16;
    n as f32 / u32::MAX as f32
}

/// Smooth 3D value noise in 0..1.
fn noise(x: f32, y: f32, z: f32) -> f32 {
    let (xi, yi, zi) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let (fx, fy, fz) = (s(x - xi as f32), s(y - yi as f32), s(z - zi as f32));
    let mut v = 0.0;
    for (dx, wx) in [(0, 1.0 - fx), (1, fx)] {
        for (dy, wy) in [(0, 1.0 - fy), (1, fy)] {
            for (dz, wz) in [(0, 1.0 - fz), (1, fz)] {
                v += wx * wy * wz * h3(xi + dx, yi + dy, zi + dz);
            }
        }
    }
    v
}

/// Position along the configured direction, 0..1.
fn along(k: &Key, dir: &str) -> f32 {
    match dir {
        "left" => 1.0 - k.cx,
        "up" => 1.0 - k.cy,
        "down" => k.cy,
        "diag" => (k.cx + k.cy * 0.4) / 1.4,
        "out" | "in" => {
            let d = ((k.cx - 0.5).powi(2) + ((k.cy - 0.5) * 0.35).powi(2)).sqrt() / 0.53;
            if dir == "in" {
                1.0 - d
            } else {
                d
            }
        }
        _ => k.cx,
    }
}

/// Distance from a point to the nearest part of a key (normalised units, y squashed by aspect).
fn key_dist(k: &Key, x: f32, y: f32, aspect: f32) -> f32 {
    k.rects
        .iter()
        .map(|r| {
            let dx = (r[0] - x).max(0.0).max(x - r[2]);
            let dy = ((r[1] - y).max(0.0).max(y - r[3])) * aspect;
            (dx * dx + dy * dy).sqrt()
        })
        .fold(f32::MAX, f32::min)
}

pub fn render(e: &SwEffect, layout: &Layout, t: f32, inp: &Inputs, st: &mut State, out: &mut Vec<Rgb>) {
    let n = layout.keys.len();
    if st.kind != e.kind || st.per_key.len() != n {
        *st = State { kind: e.kind.clone(), per_key: vec![0.0; n], aux: vec![0.0; 32], ..Default::default() };
    }
    let dt = (t - st.last_t).clamp(0.0, 0.1);
    st.last_t = t;
    out.resize(n, [0.0; 3]);
    let sp = e.speed.clamp(0.1, 4.0);
    let tt = t * sp;
    let sc = e.scale.clamp(0.25, 4.0);
    let new_presses = inp.presses;

    match e.kind.as_str() {
        "solid" => {
            let c = e.colours.first().map(|c| [c[0] as f32, c[1] as f32, c[2] as f32]).unwrap_or([255.0; 3]);
            for o in out.iter_mut() {
                *o = c;
            }
        }
        "wave" => {
            for (i, k) in layout.keys.iter().enumerate() {
                out[i] = palette(&e.colours, along(k, &e.direction) / sc - tt * 0.25);
            }
        }
        "spiral" => {
            for (i, k) in layout.keys.iter().enumerate() {
                let (dx, dy) = (k.cx - 0.5, (k.cy - 0.5) * layout.aspect);
                let a = dy.atan2(dx) / TAU;
                let d = (dx * dx + dy * dy).sqrt();
                let dir = if e.direction == "left" { -1.0 } else { 1.0 };
                out[i] = palette(&e.colours, a * dir + d * 1.6 / sc - tt * 0.2);
            }
        }
        "aurora" => {
            let pal: Vec<[u8; 3]> = if e.colours.is_empty() { vec![[20, 255, 160], [40, 120, 255], [170, 60, 255], [20, 255, 200]] } else { e.colours.clone() };
            for (i, k) in layout.keys.iter().enumerate() {
                let v = noise(k.cx * 3.0 / sc, k.cy * 1.2 / sc, tt * 0.25);
                let b = noise(k.cx * 2.0 / sc + 7.0, k.cy * 2.0 / sc, tt * 0.18 + 3.0);
                out[i] = scale(palette(&pal, v * 1.5 + tt * 0.03), (0.25 + b).min(1.0));
            }
        }
        "plasma" => {
            for (i, k) in layout.keys.iter().enumerate() {
                let (x, y) = (k.cx * 6.0 / sc, k.cy * 2.5 / sc);
                let v = (x + tt).sin() + (y * 1.3 - tt * 0.7).sin() + ((x + y + tt * 0.5).sin()) + ((x * x + y * y).sqrt() - tt).sin();
                out[i] = palette(&e.colours, v * 0.125 + tt * 0.05);
            }
        }
        "breathe" => {
            let pal: Vec<[u8; 3]> = if e.colours.is_empty() { vec![[255, 61, 110], [61, 214, 255]] } else { e.colours.clone() };
            let cycle = tt * 0.25;
            let v = 0.08 + 0.92 * (0.5 - 0.5 * (cycle.fract() * TAU).cos());
            let c = palette(&pal, (cycle.floor()) / pal.len() as f32);
            for o in out.iter_mut() {
                *o = scale(c, v);
            }
        }
        "stars" => {
            for (i, k) in layout.keys.iter().enumerate() {
                let period = 1.6 + k.seed * 2.4;
                let ph = (tt / period + k.seed * 13.7).fract();
                let v = if ph < 0.15 { ph / 0.15 } else { (1.0 - (ph - 0.15) / 0.5).max(0.0) };
                let c = palette(&e.colours, k.seed + (tt / period).floor() * 0.37);
                out[i] = scale(c, v * v);
            }
        }
        "matrix" => {
            let pal: Vec<[u8; 3]> = if e.colours.is_empty() { vec![[40, 255, 90]] } else { e.colours.clone() };
            if st.aux.len() < 32 {
                st.aux.resize(32, 0.0);
            }
            for c in 0..16 {
                st.aux[c] += dt * sp * (0.6 + h3(c as i32, 7, 3) * 0.9);
                if st.aux[c] > 1.6 {
                    st.aux[c] = -h3(c as i32 + 40, 7, 3) * 0.8;
                }
            }
            for (i, k) in layout.keys.iter().enumerate() {
                let col = ((k.cx * 15.99) as usize).min(15);
                let d = st.aux[col] - k.cy;
                let v = if (0.0..0.55).contains(&d) { 1.0 - d / 0.55 } else { 0.0 };
                let head = if (0.0..0.12).contains(&d) { 1.0 } else { 0.0 };
                out[i] = lerp(scale(palette(&pal, 0.0), v), [235.0, 255.0, 235.0], head * 0.6);
            }
        }
        "fire" => {
            for (i, k) in layout.keys.iter().enumerate() {
                let rise = 1.0 - k.cy;
                let n1 = noise(k.cx * 5.0 / sc, rise * 3.0 - tt * 1.8, tt * 0.3);
                let heat = ((1.0 - rise * 1.25) + n1 * 0.7 - 0.25).clamp(0.0, 1.0);
                out[i] = if e.colours.is_empty() {
                    // ember red -> flame orange -> amber at the hottest (not lemon yellow)
                    const EMBER: [f32; 3] = [180.0, 10.0, 0.0];
                    const FLAME: [f32; 3] = [255.0, 90.0, 0.0];
                    const AMBER: [f32; 3] = [255.0, 175.0, 25.0];
                    if heat < 0.35 {
                        lerp([0.0, 0.0, 0.0], EMBER, heat / 0.35)
                    } else if heat < 0.7 {
                        lerp(EMBER, FLAME, (heat - 0.35) / 0.35)
                    } else {
                        lerp(FLAME, AMBER, (heat - 0.7) / 0.3)
                    }
                } else {
                    scale(palette(&e.colours, heat * 0.8), heat)
                };
            }
        }
        "ripple" => {
            for (ki, t0) in new_presses {
                if let Some(k) = layout.keys.get(*ki) {
                    st.ripples.push(Ripple { x: k.cx, y: k.cy, t0: *t0, hue: (t * 0.1).fract() });
                }
            }
            st.ripples.retain(|r| t - r.t0 < 1.6 / sp);
            for (i, k) in layout.keys.iter().enumerate() {
                let mut c = [0.0f32; 3];
                for r in &st.ripples {
                    let radius = (t - r.t0) * sp * 0.75;
                    let d = key_dist(k, r.x, r.y, layout.aspect);
                    let ring = (1.0 - (d - radius).abs() / (0.07 * sc)).max(0.0) * (1.0 - (t - r.t0) * sp / 1.6).max(0.0);
                    let col = if e.colours.is_empty() { hsv(r.hue + d, 1.0, 1.0) } else { palette(&e.colours, r.hue) };
                    c = [c[0].max(col[0] * ring), c[1].max(col[1] * ring), c[2].max(col[2] * ring)];
                }
                out[i] = c;
            }
        }
        "trail" => {
            for (ki, _) in new_presses {
                if let Some(v) = st.per_key.get_mut(*ki) {
                    *v = 1.0;
                }
            }
            for (i, k) in layout.keys.iter().enumerate() {
                let v = &mut st.per_key[i];
                *v = (*v - dt * sp * 0.9).max(0.0);
                out[i] = scale(palette(&e.colours, k.cx * 0.6 + t * 0.05), *v);
            }
        }
        "heatmap" => {
            for (ki, _) in new_presses {
                if let Some(v) = st.per_key.get_mut(*ki) {
                    *v = (*v + 0.25).min(1.0);
                }
            }
            for i in 0..n {
                let v = &mut st.per_key[i];
                *v = (*v - dt * 0.02 * sp).max(0.0);
                let h = *v;
                out[i] = if h <= 0.001 { [0.0, 0.0, 30.0] } else { hsv(0.66 - h * 0.66, 1.0, 0.35 + h * 0.65) };
            }
        }
        "spectrum" => {
            let a = inp.audio;
            for (i, k) in layout.keys.iter().enumerate() {
                let band = ((k.cx * 15.99) as usize).min(15);
                let level = a.bands[band];
                let row_from_bottom = 1.0 - k.cy;
                let lit = ((level * 1.15 - row_from_bottom) * 5.0).clamp(0.0, 1.0);
                out[i] = scale(palette(&e.colours, band as f32 / 16.0 * 0.8 + t * 0.02), 0.06 + lit * 0.94);
            }
        }
        "bass" => {
            let b = inp.audio.bass.clamp(0.0, 1.0);
            st.aux[0] = st.aux[0].max(b) - dt * 2.5;
            let pulse = st.aux[0].max(0.0);
            for (i, k) in layout.keys.iter().enumerate() {
                let d = ((k.cx - 0.5).powi(2) + ((k.cy - 0.5) * 0.35).powi(2)).sqrt() / 0.53;
                let v = (pulse * 1.3 - d * 0.6).clamp(0.05, 1.0);
                out[i] = scale(palette(&e.colours, d * 0.3 + t * 0.04), v);
            }
        }
        "vu" => {
            let lvl = inp.audio.level.clamp(0.0, 1.0);
            st.aux[1] = st.aux[1].max(lvl) - dt * 1.2;
            let peak = st.aux[1].max(0.0);
            for (i, k) in layout.keys.iter().enumerate() {
                let d = (k.cx - 0.5).abs() * 2.0;
                let lit = if d <= lvl { 1.0 } else if (d - peak).abs() < 0.06 { 0.8 } else { 0.05 };
                let c = if e.colours.is_empty() { hsv(0.33 - d * 0.33, 1.0, 1.0) } else { palette(&e.colours, d) };
                out[i] = scale(c, lit);
            }
        }
        "ambient" => {
            // smoothed colour per key lives in the effect's own state (not in `out`, which may be
            // a buffer shared with other layers)
            let g = inp.screen;
            if st.aux.len() != n * 3 {
                st.aux = vec![0.0; n * 3];
            }
            let a = (dt * 10.0 * sp).min(1.0);
            for (i, k) in layout.keys.iter().enumerate() {
                if g.w == 0 || g.h == 0 {
                    out[i] = [0.0; 3];
                    continue;
                }
                let gx = ((k.cx * g.w as f32) as usize).min(g.w - 1);
                let gy = ((k.cy * g.h as f32) as usize).min(g.h - 1);
                let target = g.px[gy * g.w + gx];
                let cur = &mut st.aux[i * 3..i * 3 + 3];
                for c in 0..3 {
                    cur[c] += (target[c] - cur[c]) * a;
                }
                out[i] = [cur[0], cur[1], cur[2]];
            }
        }
        _ => {
            for o in out.iter_mut() {
                *o = [0.0; 3];
            }
        }
    }
    let b = e.brightness.clamp(0.0, 1.0);
    for o in out.iter_mut() {
        *o = scale(*o, b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_renders_finite_colours() {
        let layout = crate::layout::load(crate::k617::LAYOUTS_JSON, "tr_iso").unwrap();
        let audio = AudioLevels { bands: [0.7; 16], level: 0.6, bass: 0.8 };
        let screen = ScreenGrid { w: 4, h: 2, px: vec![[10.0, 20.0, 30.0]; 8] };
        let presses = vec![(3usize, 0.1f32), (20, 0.2)];
        for kind in ["solid", "wave", "spiral", "aurora", "plasma", "breathe", "stars", "matrix", "fire", "ripple", "trail", "heatmap", "spectrum", "bass", "vu", "ambient"] {
            let e = SwEffect { kind: kind.into(), speed: 1.0, brightness: 1.0, colours: vec![], direction: "right".into(), scale: 1.0 };
            let mut st = State::default();
            let mut out = Vec::new();
            for f in 0..90 {
                let inp = Inputs { presses: &presses, audio: &audio, screen: &screen };
                render(&e, &layout, f as f32 / 60.0, &inp, &mut st, &mut out);
            }
            assert_eq!(out.len(), layout.keys.len(), "{kind}");
            assert!(out.iter().flatten().all(|v| v.is_finite() && (0.0..=255.5).contains(v)), "{kind}");
        }
    }
}
