//! Low-level HID access to the Redragon K617 Fizz (USB 258A:0049).
//!
//! On Windows interface 1 is split into HID collections (verified on the device):
//!   Col03  input   report 03 (3 bytes)    key events: 03 20 00 01/00 Fn down/up, 03 01 03 00 profile toggle
//!   Col05  feature report 05 (5 bytes)    block selector / INIT
//!   Col06  feature report 06 (1031 bytes) 1032-byte block bodies (8-byte header + 1024)
//!   Col08  feature report 08 (381 bytes)  volatile per-key RGB frame
//!
//! Block read: send `05 8x <page> 00 00 00`, wait ~80 ms, GET_FEATURE(06, 1032); `<page>` is the
//! flash page high byte. Protocol research credit: fizzctl (MIT) and our own firmware analysis.

use anyhow::{anyhow, Context, Result};
use hidapi::{HidApi, HidDevice};
use std::ffi::CString;
use std::thread::sleep;
use std::time::Duration;

pub const VID: u16 = 0x258A;
pub const PID: u16 = 0x0049;
pub const BLOCK_LEN: usize = 1032;
pub const PERKEY_LEN: usize = 382;
pub const PERKEY_HEADER: [u8; 4] = [0x08, 0x0A, 0x7A, 0x01];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Block {
    Mode,
    Canvas,
    Routing,
    Exec,
    Macro,
    KeymapD4,
    KeymapD8,
}

impl Block {
    pub fn selector(self) -> [u8; 6] {
        match self {
            Block::Mode => [0x05, 0x88, 0xB8, 0, 0, 0],
            Block::Canvas => [0x05, 0x89, 0xBC, 0, 0, 0],
            Block::Routing => [0x05, 0x89, 0xC0, 0, 0, 0],
            Block::Exec => [0x05, 0x83, 0xB6, 0, 0, 0],
            Block::Macro => [0x05, 0x85, 0xDC, 0, 0, 0],
            Block::KeymapD4 => [0x05, 0x84, 0xD4, 0, 0, 0],
            Block::KeymapD8 => [0x05, 0x84, 0xD8, 0, 0, 0],
        }
    }

    /// Write header (first 5 bytes); the read comes back with a "read-flipped" header.
    pub fn header(self) -> [u8; 5] {
        match self {
            Block::Mode => [0x06, 0x08, 0xB8, 0x00, 0x40],
            Block::Canvas => [0x06, 0x09, 0xBC, 0x00, 0x40],
            Block::Routing => [0x06, 0x09, 0xC0, 0x00, 0x40],
            Block::Exec => [0x06, 0x03, 0xB6, 0x00, 0x00],
            Block::Macro => [0x06, 0x05, 0xDC, 0x00, 0x40],
            Block::KeymapD4 => [0x06, 0x04, 0xD4, 0x00, 0x40],
            Block::KeymapD8 => [0x06, 0x04, 0xD8, 0x00, 0x40],
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Paths {
    pub events: Option<CString>,
    pub cmd: Option<CString>,
    pub xfer: Option<CString>,
    pub perkey: Option<CString>,
}

pub fn find_paths(api: &HidApi) -> Paths {
    let mut p = Paths::default();
    for d in api.device_list() {
        if d.vendor_id() != VID || d.product_id() != PID || d.interface_number() != 1 {
            continue;
        }
        let path = d.path().to_string_lossy().to_ascii_lowercase();
        let owned = d.path().to_owned();
        if path.contains("&col03") {
            p.events = Some(owned);
        } else if path.contains("&col05") {
            p.cmd = Some(owned);
        } else if path.contains("&col06") {
            p.xfer = Some(owned);
        } else if path.contains("&col08") {
            p.perkey = Some(owned);
        }
    }
    p
}

/// Command/transfer/per-key handles. Not used concurrently: callers keep it behind a mutex.
pub struct K617 {
    cmd: HidDevice,
    xfer: HidDevice,
    perkey: HidDevice,
}

impl K617 {
    pub fn open(api: &HidApi) -> Result<Self> {
        let p = find_paths(api);
        let open = |c: &Option<CString>, what: &str| -> Result<HidDevice> {
            let path = c.as_ref().ok_or_else(|| anyhow!("K617 {what} collection not found"))?;
            api.open_path(path).with_context(|| format!("open {what}"))
        };
        Ok(Self {
            cmd: open(&p.cmd, "Col05")?,
            xfer: open(&p.xfer, "Col06")?,
            perkey: open(&p.perkey, "Col08")?,
        })
    }

    pub fn send(&self, frame: &[u8]) -> Result<()> {
        let dev = match frame.first() {
            Some(0x05) => &self.cmd,
            Some(0x06) => &self.xfer,
            Some(0x08) => &self.perkey,
            other => return Err(anyhow!("unknown report id {other:?}")),
        };
        dev.send_feature_report(frame).context("send_feature_report")?;
        Ok(())
    }

    pub fn read_block(&self, b: Block) -> Result<Vec<u8>> {
        self.cmd.send_feature_report(&b.selector()).context("select block")?;
        sleep(Duration::from_millis(80));
        let mut buf = vec![0u8; BLOCK_LEN];
        buf[0] = 0x06;
        self.xfer.get_feature_report(&mut buf).context("read block")?;
        buf[..5].copy_from_slice(&b.header());
        Ok(buf)
    }

    /// GET_REPORT(06) poll the firmware needs after INIT in lighting bursts.
    pub fn handshake(&self) -> Result<()> {
        let mut buf = vec![0u8; BLOCK_LEN];
        buf[0] = 0x06;
        self.xfer.get_feature_report(&mut buf).context("handshake")?;
        Ok(())
    }

    /// One volatile per-key frame. NOTE: the firmware switches to effect 21 (host mode) on the
    /// first frame and freezes on the last frame; leave it by applying a firmware effect.
    pub fn send_perkey(&self, frame: &[u8; PERKEY_LEN]) -> Result<()> {
        self.perkey.send_feature_report(frame).context("per-key frame")?;
        Ok(())
    }
}

pub fn open_events(api: &HidApi) -> Result<HidDevice> {
    let p = find_paths(api);
    let path = p.events.ok_or_else(|| anyhow!("K617 event collection not found"))?;
    api.open_path(&path).context("open Col03")
}
