//! Firmware ("hardware") lighting effects and how they are stored.
//!
//! Settings sector 0xB600 = EXEC block body (frame offset = body offset + 8):
//!   body 0x00..0x07  prefix (byte 0 = active keymap profile, 5A A5 at 6..7)
//!   body 0x08..0x82  profile 0 record (0x7B bytes, 5A A5 at record+0x79)
//!   body 0x83..0xFD  profile 1 record
//! Inside a record (firmware RAM 0x0356+i):
//!   [4]  "custom layout" flag (1 = show a Define slot)    [5] effect id (1..22)
//!   [20 + 2*id]  bit7 = reverse direction, low nibble = colour index (0..6 palette, 7 = multicolour)
//!   [21 + 2*id]  high nibble = speed-1 (0..4), low nibble = brightness (0..4)
//! MODE block body: palette of effect `id` for profile p = 7 RGB colours at p*0x1F8 + id*21.
//! The firmware keeps all of this per profile; Prizma always writes BOTH profiles so the
//! Fn-lock profile switch never changes the lighting.

use serde::{Deserialize, Serialize};

pub const HDR: usize = 8;
pub const REC: [usize; 2] = [0x08, 0x83];
pub const REC_LEN: usize = 0x7B;
pub const PAL_PROFILE_STRIDE: usize = 0x1F8;
pub const PAL_ENTRY: usize = 21;
pub const SELF_DEFINE: u8 = 0x15;
pub const OFF: u8 = 0x16;

#[derive(Serialize, Clone, Copy, Debug)]
pub struct EffectInfo {
    pub id: u8,
    pub key: &'static str,
    pub name: &'static str,
    pub animated: bool,
    pub colour: bool,
}

pub const EFFECTS: &[EffectInfo] = &[
    EffectInfo { id: 0x01, key: "fixed", name: "Sabit", animated: false, colour: true },
    EffectInfo { id: 0x02, key: "respire", name: "Nefes", animated: true, colour: true },
    EffectInfo { id: 0x03, key: "rainbow", name: "Gökkuşağı", animated: true, colour: true },
    EffectInfo { id: 0x04, key: "flash_away", name: "Uçup giden", animated: true, colour: true },
    EffectInfo { id: 0x05, key: "raindrops", name: "Yağmur damlaları", animated: true, colour: true },
    EffectInfo { id: 0x06, key: "rainbow_wheel", name: "Gökkuşağı çarkı", animated: true, colour: true },
    EffectInfo { id: 0x07, key: "ripples", name: "Parlayan halkalar", animated: true, colour: true },
    EffectInfo { id: 0x08, key: "stars", name: "Yıldız pırıltısı", animated: true, colour: true },
    EffectInfo { id: 0x09, key: "shadow", name: "Kaybolan gölge", animated: true, colour: true },
    EffectInfo { id: 0x0A, key: "snake", name: "Retro yılan", animated: true, colour: true },
    EffectInfo { id: 0x0B, key: "neon", name: "Neon akış", animated: true, colour: true },
    EffectInfo { id: 0x0C, key: "reaction", name: "Tepki", animated: true, colour: true },
    EffectInfo { id: 0x0D, key: "sine", name: "Sinüs dalgası", animated: true, colour: true },
    EffectInfo { id: 0x0E, key: "scan", name: "Tarama", animated: true, colour: true },
    EffectInfo { id: 0x0F, key: "windmill", name: "Yel değirmeni", animated: true, colour: true },
    EffectInfo { id: 0x10, key: "waterfall", name: "Renkli şelale", animated: true, colour: true },
    EffectInfo { id: 0x11, key: "blossom", name: "Çiçek açma", animated: true, colour: true },
    EffectInfo { id: 0x12, key: "storm", name: "Dönen fırtına", animated: true, colour: true },
    EffectInfo { id: 0x13, key: "collision", name: "Çarpışma", animated: true, colour: true },
    EffectInfo { id: 0x14, key: "perfect", name: "Mükemmel", animated: true, colour: true },
    EffectInfo { id: OFF, key: "off", name: "Kapalı", animated: false, colour: false },
];

pub fn info(id: u8) -> Option<&'static EffectInfo> {
    EFFECTS.iter().find(|e| e.id == id)
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HwEffect {
    pub id: u8,
    /// None = multicolour, Some(rgb) = single colour
    pub colour: Option<[u8; 3]>,
    /// 1..=5
    pub speed: u8,
    /// 0..=4
    pub brightness: u8,
    pub reverse: bool,
}

fn rec_at(profile: usize, i: usize) -> usize {
    HDR + REC[profile] + i
}

/// Read the effect of profile 0 from an EXEC frame (and its palette from MODE).
pub fn decode(exec: &[u8], mode: &[u8]) -> HwEffect {
    decode_id(exec, mode, exec[rec_at(0, 5)])
}

/// The settings the keyboard keeps for effect `id` (profile 0), whether or not it is active.
/// The firmware stores colour, speed, brightness and direction separately for every effect.
pub fn decode_id(exec: &[u8], mode: &[u8], id: u8) -> HwEffect {
    let pair = 20 + 2 * id as usize;
    let a = exec[rec_at(0, pair)];
    let b = exec[rec_at(0, pair + 1)];
    let ci = (a & 0x0F) as usize;
    let colour = if ci < 7 {
        let o = HDR + id as usize * PAL_ENTRY + ci * 3;
        Some(crate::colour::from_led([mode[o], mode[o + 1], mode[o + 2]]))
    } else {
        None
    };
    HwEffect { id, colour, speed: ((b >> 4) & 0x07) + 1, brightness: b & 0x0F, reverse: a & 0x80 != 0 }
}

/// Patch EXEC and MODE frames so BOTH profiles show `e`. Single colours go into palette slot 0.
pub fn encode(exec: &mut [u8], mode: &mut [u8], e: &HwEffect) {
    let id = e.id as usize;
    let speed = e.speed.clamp(1, 5) - 1;
    let bright = e.brightness.min(4);
    let ci: u8 = if e.colour.is_some() { 0 } else { 7 };
    for p in 0..2 {
        exec[rec_at(p, 4)] = 0; // show the effect, not a custom Define layout
        exec[rec_at(p, 5)] = e.id;
        let pair = 20 + 2 * id;
        exec[rec_at(p, pair)] = (if e.reverse { 0x80 } else { 0 }) | ci;
        exec[rec_at(p, pair + 1)] = (speed << 4) | bright;
        if let Some(rgb) = e.colour {
            let o = HDR + p * PAL_PROFILE_STRIDE + id * PAL_ENTRY;
            // the firmware drives the LEDs with these bytes as they are
            mode[o..o + 3].copy_from_slice(&crate::colour::to_led_u8(rgb));
        }
    }
}

/// Stored settings of every effect in the catalogue, in catalogue order.
pub fn decode_all(exec: &[u8], mode: &[u8]) -> Vec<HwEffect> {
    EFFECTS.iter().map(|e| decode_id(exec, mode, e.id)).collect()
}

/// True when both profile records carry a valid signature (an invalid record makes the
/// firmware reset every setting to factory defaults on the next profile switch).
pub fn records_valid(exec: &[u8]) -> bool {
    (0..2).all(|p| exec[rec_at(p, 0x79)] == 0x5A && exec[rec_at(p, 0x7A)] == 0xA5)
}

/// Copy profile 0's record over profile 1 (keeps both profiles' lighting identical).
pub fn mirror_profiles(exec: &mut [u8], mode: &mut [u8]) {
    let (a, b) = (HDR + REC[0], HDR + REC[1]);
    let r0: Vec<u8> = exec[a..a + REC_LEN].to_vec();
    exec[b..b + REC_LEN].copy_from_slice(&r0);
    let p0: Vec<u8> = mode[HDR..HDR + PAL_PROFILE_STRIDE].to_vec();
    mode[HDR + PAL_PROFILE_STRIDE..HDR + 2 * PAL_PROFILE_STRIDE].copy_from_slice(&p0);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank() -> (Vec<u8>, Vec<u8>) {
        let mut exec = vec![0u8; 1032];
        let mode = vec![0u8; 1032];
        for p in 0..2 {
            exec[rec_at(p, 0x79)] = 0x5A;
            exec[rec_at(p, 0x7A)] = 0xA5;
        }
        (exec, mode)
    }

    #[test]
    fn roundtrip_and_both_profiles() {
        let (mut exec, mut mode) = blank();
        let e = HwEffect { id: 0x0B, colour: Some([61, 214, 255]), speed: 4, brightness: 3, reverse: true };
        encode(&mut exec, &mut mode, &e);
        assert_eq!(decode(&exec, &mode), e);
        for p in 0..2 {
            assert_eq!(exec[rec_at(p, 5)], 0x0B);
            assert_eq!(exec[rec_at(p, 20 + 22)], 0x80);
            assert_eq!(exec[rec_at(p, 21 + 22)], 0x33);
            let o = HDR + p * PAL_PROFILE_STRIDE + 11 * PAL_ENTRY;
            assert_eq!(&mode[o..o + 3], &crate::colour::to_led_u8([61, 214, 255]));
        }
        assert!(records_valid(&exec));
    }

    #[test]
    fn multicolour_and_fixed_on_matches_fizzctl_offsets() {
        let (mut exec, mut mode) = blank();
        let e = HwEffect { id: 1, colour: None, speed: 1, brightness: 4, reverse: false };
        encode(&mut exec, &mut mode, &e);
        // fizzctl: EXEC[21] effect id, EXEC[38 + 2*(id-1)] colour toggle, EXEC[39 + 2*(id-1)] speed/bright
        assert_eq!(exec[21], 1);
        assert_eq!(exec[38], 0x07);
        assert_eq!(exec[39], 0x04);
        assert_eq!(decode(&exec, &mode), e);
    }

    #[test]
    fn each_effect_keeps_its_own_settings() {
        let (mut exec, mut mode) = blank();
        let a = HwEffect { id: 0x02, colour: Some([0, 255, 0]), speed: 2, brightness: 1, reverse: false };
        let b = HwEffect { id: 0x03, colour: None, speed: 5, brightness: 4, reverse: true };
        encode(&mut exec, &mut mode, &a);
        encode(&mut exec, &mut mode, &b);
        assert_eq!(decode(&exec, &mode), b);
        assert_eq!(decode_id(&exec, &mode, 0x02), a);
        let all = decode_all(&exec, &mode);
        assert_eq!(all.len(), EFFECTS.len());
        assert_eq!(all.iter().find(|e| e.id == 0x02), Some(&a));
    }

    #[test]
    fn mirror_copies_record_and_palette() {
        let (mut exec, mut mode) = blank();
        exec[HDR + REC[0] + 5] = 3;
        mode[HDR + 3 * PAL_ENTRY] = 99;
        mirror_profiles(&mut exec, &mut mode);
        assert_eq!(exec[HDR + REC[1] + 5], 3);
        assert_eq!(mode[HDR + PAL_PROFILE_STRIDE + 3 * PAL_ENTRY], 99);
        assert!(records_valid(&exec));
    }
}
