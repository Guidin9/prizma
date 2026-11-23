pub mod effects;
pub mod hid;

use anyhow::{anyhow, Result};
use effects::HwEffect;
use hid::{Block, K617};
use std::thread::sleep;
use std::time::Duration;

pub const LAYOUTS_JSON: &str = include_str!("../../layouts/k617.json");

const BURST_GAP: Duration = Duration::from_millis(60);

/// Live lighting blocks, read fresh before every write so nothing else changes.
pub struct Lighting {
    pub mode: Vec<u8>,
    pub canvas: Vec<u8>,
    pub routing: Vec<u8>,
    pub exec: Vec<u8>,
}

pub fn read_lighting(dev: &K617) -> Result<Lighting> {
    Ok(Lighting {
        mode: dev.read_block(Block::Mode)?,
        canvas: dev.read_block(Block::Canvas)?,
        routing: dev.read_block(Block::Routing)?,
        exec: dev.read_block(Block::Exec)?,
    })
}

/// fizzctl's lighting burst: INIT 05 83 b6 (selects the EXEC block), GET_REPORT(06) handshake
/// (reads that block back), then MODE, CANVAS, ROUTING, EXEC. On Windows a GET after
/// `05 05 81` fails, so the keymap-style INIT must not be used here.
pub fn write_lighting(dev: &K617, l: &Lighting) -> Result<()> {
    dev.send(&Block::Exec.selector())?;
    sleep(BURST_GAP);
    dev.handshake()?;
    sleep(BURST_GAP);
    for frame in [&l.mode, &l.canvas, &l.routing, &l.exec] {
        dev.send(frame)?;
        sleep(BURST_GAP);
    }
    Ok(())
}

pub fn current_effect(dev: &K617) -> Result<HwEffect> {
    let exec = dev.read_block(Block::Exec)?;
    let mode = dev.read_block(Block::Mode)?;
    Ok(effects::decode(&exec, &mode))
}

/// Apply a firmware effect to both profiles (one flash write).
pub fn apply_effect(dev: &K617, e: &HwEffect) -> Result<()> {
    if effects::info(e.id).is_none() {
        return Err(anyhow!("unknown effect id {}", e.id));
    }
    let mut l = read_lighting(dev)?;
    if !effects::records_valid(&l.exec) {
        // Profile 1 record never written: mirror profile 0 first, otherwise the firmware
        // would reset all settings to defaults when the Fn lock switches profiles.
        let (exec, mode) = (&mut l.exec, &mut l.mode);
        effects::mirror_profiles(exec, mode);
        if !effects::records_valid(&l.exec) {
            return Err(anyhow!("settings sector has no valid profile record; not writing"));
        }
    }
    effects::encode(&mut l.exec, &mut l.mode, e);
    write_lighting(dev, &l)?;
    // the flash commit takes a moment; then confirm the keyboard really stored the effect
    for _ in 0..10 {
        sleep(Duration::from_millis(150));
        let exec = dev.read_block(Block::Exec)?;
        if exec[effects::HDR + effects::REC[0] + 5] == e.id && exec[effects::HDR + effects::REC[1] + 5] == e.id {
            return Ok(());
        }
    }
    Err(anyhow!("klavye ayarı kabul etmedi (geri okumada efekt değişmemiş)"))
}

/// Active keymap profile (0 = normal, 1 = Fn lock) as last saved by the firmware.
pub fn saved_profile(dev: &K617) -> Result<u8> {
    let exec = dev.read_block(Block::Exec)?;
    Ok(exec[effects::HDR])
}
