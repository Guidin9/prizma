//! Listens to what Windows is playing (WASAPI loopback on the default output) and turns it into
//! 16 log-spaced frequency bands. Runs only while an audio effect is active.

use super::effects::AudioLevels;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use windows::Win32::Media::Audio::{
    eConsole, eRender, IAudioCaptureClient, IAudioClient, IMMDeviceEnumerator, MMDeviceEnumerator, AUDCLNT_BUFFERFLAGS_SILENT,
    AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK,
};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED};

const N: usize = 1024;

pub fn spawn(levels: Arc<Mutex<AudioLevels>>, on: Arc<AtomicBool>) {
    thread::spawn(move || loop {
        if !on.load(Ordering::Relaxed) {
            *levels.lock() = AudioLevels::default();
            thread::sleep(Duration::from_millis(200));
            continue;
        }
        if let Err(e) = unsafe { capture(&levels, &on) } {
            log::warn!("audio capture: {e}");
            thread::sleep(Duration::from_secs(1));
        }
    });
}

unsafe fn capture(levels: &Arc<Mutex<AudioLevels>>, on: &AtomicBool) -> windows::core::Result<()> {
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    let en: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let dev = en.GetDefaultAudioEndpoint(eRender, eConsole)?;
    let client: IAudioClient = dev.Activate(CLSCTX_ALL, None)?;
    let fmt = client.GetMixFormat()?;
    let channels = (*fmt).nChannels.max(1) as usize;
    let rate = (*fmt).nSamplesPerSec as f32;
    let bits = (*fmt).wBitsPerSample;
    client.Initialize(AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK, 1_000_000, 0, fmt, None)?;
    let cap: IAudioCaptureClient = client.GetService()?;
    client.Start()?;
    let mut ring = vec![0f32; N];
    let mut pos = 0usize;
    let mut peak = 1e-3f32;
    let window: Vec<f32> = (0..N).map(|i| 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / N as f32).cos()).collect();
    let mut smooth = [0f32; 16];
    while on.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(12));
        loop {
            let packet = cap.GetNextPacketSize()?;
            if packet == 0 {
                break;
            }
            let mut data: *mut u8 = std::ptr::null_mut();
            let mut frames = 0u32;
            let mut flags = 0u32;
            cap.GetBuffer(&mut data, &mut frames, &mut flags, None, None)?;
            let silent = flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0;
            for f in 0..frames as usize {
                let v = if silent || bits != 32 {
                    0.0
                } else {
                    let s = std::slice::from_raw_parts(data as *const f32, frames as usize * channels);
                    s[f * channels..f * channels + channels].iter().sum::<f32>() / channels as f32
                };
                ring[pos] = v;
                pos = (pos + 1) % N;
            }
            cap.ReleaseBuffer(frames)?;
        }
        // FFT of the latest N samples
        let mut re: Vec<f32> = (0..N).map(|i| ring[(pos + i) % N] * window[i]).collect();
        let mut im = vec![0f32; N];
        fft(&mut re, &mut im);
        let mut bands = [0f32; 16];
        for (b, band) in bands.iter_mut().enumerate() {
            let lo = 40.0 * (16000.0f32 / 40.0).powf(b as f32 / 16.0);
            let hi = 40.0 * (16000.0f32 / 40.0).powf((b + 1) as f32 / 16.0);
            let (i0, i1) = (((lo / rate) * N as f32) as usize, (((hi / rate) * N as f32) as usize).max(1));
            let mut sum = 0.0;
            for i in i0.max(1)..=i1.min(N / 2 - 1).max(i0.max(1)) {
                sum += (re[i] * re[i] + im[i] * im[i]).sqrt();
            }
            *band = sum;
        }
        let maxb = bands.iter().cloned().fold(0.0, f32::max);
        peak = (peak * 0.995).max(maxb).max(1e-3);
        let mut out = AudioLevels::default();
        for b in 0..16 {
            let v = (bands[b] / peak).sqrt().clamp(0.0, 1.0);
            smooth[b] = if v > smooth[b] { v } else { smooth[b] * 0.82 + v * 0.18 };
            out.bands[b] = smooth[b];
        }
        out.bass = (smooth[0] + smooth[1] + smooth[2]) / 3.0;
        out.level = smooth.iter().sum::<f32>() / 16.0 * 1.6;
        *levels.lock() = out;
    }
    let _ = client.Stop();
    CoTaskMemFree(Some(fmt as *const _));
    Ok(())
}

/// In-place iterative radix-2 FFT (length must be a power of two).
fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let ang = -std::f32::consts::TAU / len as f32;
        for start in (0..n).step_by(len) {
            for k in 0..len / 2 {
                let (wr, wi) = ((ang * k as f32).cos(), (ang * k as f32).sin());
                let (a, b) = (start + k, start + k + len / 2);
                let (xr, xi) = (re[b] * wr - im[b] * wi, re[b] * wi + im[b] * wr);
                re[b] = re[a] - xr;
                im[b] = im[a] - xi;
                re[a] += xr;
                im[a] += xi;
            }
        }
        len <<= 1;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fft_finds_a_sine() {
        let n = 1024;
        let mut re: Vec<f32> = (0..n).map(|i| (std::f32::consts::TAU * 64.0 * i as f32 / n as f32).sin()).collect();
        let mut im = vec![0f32; n];
        super::fft(&mut re, &mut im);
        let mags: Vec<f32> = (0..n / 2).map(|i| (re[i] * re[i] + im[i] * im[i]).sqrt()).collect();
        let peak = mags.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap().0;
        assert_eq!(peak, 64);
    }
}
