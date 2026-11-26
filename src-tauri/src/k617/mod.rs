pub mod effects;
pub mod hid;

use anyhow::{anyhow, Result};
use effects::HwEffect;
use hid::{Block, K617};
use std::thread::sleep;
use std::time::Duration;

pub const LAYOUTS_JSON: &str = include_str!("../../layouts/k617.json");

const BURST_GAP: Duration = Duration::from_millis(60);

/// Lighting write: INIT `05 83 b6` (selects the settings block), GET_REPORT(06) handshake, then
/// the blocks, EXEC last. On Windows a GET after `05 05 81` fails, so the keymap-style INIT must
/// not be used here.
///
/// Firmware (verified in the disassembly and on the device): when the EXEC (settings) block
/// arrives, the USB interrupt writes it to flash at once and flags the main loop, which reloads
/// the active profile's record and picks up the new effect. So EXEC alone is enough when no
/// colour changed, which saves three flash-sector erases per apply.
fn write_blocks(dev: &K617, frames: &[&[u8]]) -> Result<()> {
    dev.send(&Block::Exec.selector())?;
    sleep(BURST_GAP);
    dev.handshake()?;
    sleep(BURST_GAP);
    for frame in frames {
        dev.send(frame)?;
        sleep(BURST_GAP);
    }
    Ok(())
}

/// What the keyboard shows now, plus the stored settings of every effect (catalogue order).
pub struct Settings {
    pub current: HwEffect,
    pub stored: Vec<HwEffect>,
}

pub fn read_settings(dev: &K617) -> Result<Settings> {
    let exec = dev.read_block(Block::Exec)?;
    let mode = dev.read_block(Block::Mode)?;
    Ok(Settings { current: effects::decode(&exec, &mode), stored: effects::decode_all(&exec, &mode) })
}

pub fn current_effect(dev: &K617) -> Result<HwEffect> {
    Ok(read_settings(dev)?.current)
}

/// Apply a firmware effect to both profiles.
pub fn apply_effect(dev: &K617, e: &HwEffect) -> Result<()> {
    if effects::info(e.id).is_none() {
        return Err(anyhow!("unknown effect id {}", e.id));
    }
    let mut exec = dev.read_block(Block::Exec)?;
    let mut mode = dev.read_block(Block::Mode)?;
    let mode_before = mode.clone();
    if !effects::records_valid(&exec) {
        // Profile 1 record never written: mirror profile 0 first, otherwise the firmware
        // would reset all settings to defaults when the Fn lock switches profiles.
        effects::mirror_profiles(&mut exec, &mut mode);
        if !effects::records_valid(&exec) {
            return Err(anyhow!("settings sector has no valid profile record; not writing"));
        }
    }
    effects::encode(&mut exec, &mut mode, e);
    if mode != mode_before {
        // A new colour: send the full verified burst. CANVAS/ROUTING arriving makes the firmware
        // restart the effect, which re-reads the palette even when the effect id is unchanged.
        let canvas = dev.read_block(Block::Canvas)?;
        let routing = dev.read_block(Block::Routing)?;
        write_blocks(dev, &[&mode, &canvas, &routing, &exec])?;
    } else {
        write_blocks(dev, &[&exec])?;
    }
    // confirm the keyboard really stored the effect
    for _ in 0..10 {
        sleep(Duration::from_millis(150));
        let back = dev.read_block(Block::Exec)?;
        if back[effects::HDR + effects::REC[0] + 5] == e.id && back[effects::HDR + effects::REC[1] + 5] == e.id {
            return Ok(());
        }
    }
    Err(anyhow!("the keyboard did not accept the setting (unchanged when read back)"))
}

/// Active keymap profile (0 = normal, 1 = Fn lock) as last saved by the firmware.
pub fn saved_profile(dev: &K617) -> Result<u8> {
    let exec = dev.read_block(Block::Exec)?;
    Ok(exec[effects::HDR])
}
