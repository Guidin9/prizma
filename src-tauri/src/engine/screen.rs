//! Samples the primary screen into a small colour grid for the ambient effect (GDI, ~30 fps).
//! Runs only while the ambient effect is active.

use super::effects::ScreenGrid;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SelectObject, SetStretchBltMode,
    StretchBlt, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, COLORONCOLOR, DIB_RGB_COLORS, SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

const GW: usize = 16;
const GH: usize = 6;
const SUB: usize = 6; // samples per grid cell side

pub fn spawn(grid: Arc<Mutex<ScreenGrid>>, on: Arc<AtomicBool>) {
    thread::spawn(move || loop {
        if !on.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(200));
            continue;
        }
        if let Some(px) = unsafe { capture() } {
            *grid.lock() = ScreenGrid { w: GW, h: GH, px };
        }
        thread::sleep(Duration::from_millis(33));
    });
}

unsafe fn capture() -> Option<Vec<[f32; 3]>> {
    let (w, h) = ((GW * SUB) as i32, (GH * SUB) as i32);
    let screen = GetDC(None);
    let mem = CreateCompatibleDC(Some(screen));
    let bmp = CreateCompatibleBitmap(screen, w, h);
    let old = SelectObject(mem, bmp.into());
    SetStretchBltMode(mem, COLORONCOLOR);
    let (sw, sh) = (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN));
    let ok = StretchBlt(mem, 0, 0, w, h, Some(screen), 0, 0, sw, sh, SRCCOPY).as_bool();
    let mut info = BITMAPINFO::default();
    info.bmiHeader = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: w,
        biHeight: -h,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
    };
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let lines = GetDIBits(mem, bmp, 0, h as u32, Some(buf.as_mut_ptr() as *mut _), &mut info, DIB_RGB_COLORS);
    SelectObject(mem, old);
    let _ = DeleteObject(bmp.into());
    let _ = DeleteDC(mem);
    ReleaseDC(None, screen);
    if !ok || lines == 0 {
        return None;
    }
    let mut px = vec![[0f32; 3]; GW * GH];
    for gy in 0..GH {
        for gx in 0..GW {
            let mut acc = [0f32; 3];
            for sy in 0..SUB {
                for sx in 0..SUB {
                    let o = (((gy * SUB + sy) * GW * SUB) + gx * SUB + sx) * 4;
                    acc[0] += buf[o + 2] as f32;
                    acc[1] += buf[o + 1] as f32;
                    acc[2] += buf[o] as f32;
                }
            }
            let k = 1.0 / (SUB * SUB) as f32;
            let c = [acc[0] * k, acc[1] * k, acc[2] * k];
            // boost saturation so the keyboard reads the screen's colour, not its grey average
            let avg = (c[0] + c[1] + c[2]) / 3.0;
            px[gy * GW + gx] = [
                (avg + (c[0] - avg) * 1.6).clamp(0.0, 255.0),
                (avg + (c[1] - avg) * 1.6).clamp(0.0, 255.0),
                (avg + (c[2] - avg) * 1.6).clamp(0.0, 255.0),
            ];
        }
    }
    Some(px)
}
