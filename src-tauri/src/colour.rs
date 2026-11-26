//! Screen colours -> K617 LED values.
//!
//! The keyboard drives its LEDs with the bytes it gets (no gamma table on the host path; the only
//! curve in the firmware is the breathing effect's). Screen colours are sRGB, so sent unchanged a
//! warm yellow like (255, 210, 60) comes out almost white. Its blue LEDs are also much stronger
//! than red and green: full white looks violet.
//!
//! Tuned on the device on 2026-11-26 by eye (the user picked between side-by-side test patches;
//! a desk camera agreed on the direction but overrated blue): gamma 2.2 per channel, then blue
//! at 45 %. Red and green stay as they are; lowering red turned yellow green.

pub const GAMMA: f32 = 2.2;
pub const GAIN: [f32; 3] = [1.0, 1.0, 0.45];

fn encode(v: f32, ch: usize) -> u8 {
    let lin = (v.clamp(0.0, 255.0) / 255.0).powf(GAMMA);
    (lin * GAIN[ch] * 255.0).round() as u8
}

fn decode(v: u8, ch: usize) -> u8 {
    let lin = (v as f32 / 255.0 / GAIN[ch]).min(1.0);
    (lin.powf(1.0 / GAMMA) * 255.0).round() as u8
}

/// sRGB (0..=255, as float) to what the LEDs need.
pub fn to_led(c: [f32; 3]) -> [u8; 3] {
    [encode(c[0], 0), encode(c[1], 1), encode(c[2], 2)]
}

pub fn to_led_u8(c: [u8; 3]) -> [u8; 3] {
    to_led([c[0] as f32, c[1] as f32, c[2] as f32])
}

/// Inverse of [`to_led_u8`], for colours read back from the keyboard. Values the keyboard got
/// from elsewhere (its own presets) may be brighter than Prizma would ever send; they clamp.
pub fn from_led(c: [u8; 3]) -> [u8; 3] {
    [decode(c[0], 0), decode(c[1], 1), decode(c[2], 2)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_and_black() {
        assert_eq!(to_led_u8([255, 255, 255]), [255, 255, 115]);
        assert_eq!(to_led_u8([0, 0, 0]), [0, 0, 0]);
    }

    #[test]
    fn warm_yellow_loses_its_blue() {
        // the fire effect's old hottest colour: looked white on the keyboard without correction
        assert_eq!(to_led_u8([255, 210, 60]), [255, 166, 5]);
    }

    #[test]
    fn round_trip_keeps_bright_colours() {
        for c in [[61u8, 214, 255], [255, 61, 110], [255, 225, 77], [120, 120, 120]] {
            let back = from_led(to_led_u8(c));
            for ch in 0..3 {
                assert!((back[ch] as i32 - c[ch] as i32).abs() <= 3, "{c:?} -> {back:?}");
            }
        }
    }
}
