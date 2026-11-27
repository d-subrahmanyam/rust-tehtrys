//! Sound. (§13.2 sound effects; §13.3 music; flows/12 render->audio seam)
//!
//! `core` stays pure; audio is presentation. We implement **procedural synthesis**
//! original sound effects = procedural synthesis; `Korobeiniki` music slot ships separate third-party PCM16 `korobeiniki.wav` (classicals.de; CC-BY-NC-4.0; see assets/audio/CREDITS; can't verify; not The Tetris Company); no external audio lib; SDL2 built-in audio.
//! (The Phase-5 plan named `sdl2_mixer`, but that system lib isn't available; this
//! satisfies the same `Audio`-trait requirement with zero extra dependency.)
//!
//! Pure, testable parts: [`SoundCue`], [`cues_from_events`], [`synth_cue`],
//! [`SoundSettings`], [`MusicSlot`], [`parse_wav`]. SDL backend [`SdlAudio`] +
//! [`NullAudio`] implement [`Audio`].

use crate::core::types::EngineEvent;
use sdl2::AudioSubsystem;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

/// One sound effect. (§13.2 min + extra; original synthesized, no protected audio)
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SoundCue {
    // input / navigation
    MenuMove,
    MenuSelect,
    MenuBack,
    Pause,
    Resume,
    // gameplay (core events)
    Spawn,
    Move,
    MoveFailed,
    Rotate,
    RotateFailed,
    Hold,
    SoftDrop,
    HardDrop,
    Lock,
    ClearSingle,
    ClearDouble,
    ClearTriple,
    ClearTetris,
    TSpin,
    TSpinMini,
    BackToBack,
    LevelUp,
    GameOver,
    VariantEnd,
}

/// BGM slot. **No licensed asset shipped.** (§13.3) `Korobeiniki` is a *slot*:
/// BGM slot. `Korobeiniki` ships a converted PCM16 `korobeiniki.wav`
/// (`assets/audio/`), **third-party** source `https://www.classicals.de/tetris-theme`
/// **"Tetris Theme - Korobeiniki - Arranged for Music Box"**; **recorded,
/// produced, published by Gregor Quendel**; **arrangement based on notes by
/// Maru Piano**; **CC BY-NC 4.0** (non-commercial; attribution + link + changes);
/// **separate**; not The Tetris Company; **not** original; **can't verify**;
/// **not legal advice**; `<data>/music/` override; PCM16 WAV only; otherwise silent.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MusicSlot {
    /// Korobeiniki music slot. (§13.3; converted PCM16 `korobeiniki.wav`; see assets/audio/CREDITS; separate license)
    Korobeiniki,
    /// original synthesized / silent slot. no asset.
    Silent,
}

impl MusicSlot {
    /// filename; `<data>/music/` override, then bundled `assets/audio`. `None` => synthesized/silent.
    pub fn asset_filename(self) -> Option<&'static str> {
        match self {
            MusicSlot::Korobeiniki => Some("korobeiniki.wav"),
            MusicSlot::Silent => None,
        }
    }
}

/// Sound + music settings. (§2.5; persisted) volumes 0..=100.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SoundSettings {
    pub sfx_enabled: bool,
    pub sfx_volume: u8, // 0..=100
    pub music_enabled: bool,
    pub music_volume: u8, // 0..=100
    pub muted: bool,      // master mute (§13.2 toggle)
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            sfx_enabled: true,
            sfx_volume: 80,
            music_enabled: true,
            music_volume: 50,
            muted: false,
        }
    }
}

impl SoundSettings {
    pub fn sfx_gain(&self) -> f32 {
        if self.muted || !self.sfx_enabled {
            0.0
        } else {
            self.sfx_volume as f32 / 100.0
        }
    }
    pub fn music_gain(&self) -> f32 {
        if self.muted || !self.music_enabled {
            0.0
        } else {
            self.music_volume as f32 / 100.0
        }
    }
    pub fn clamp_volume(v: i32) -> u8 {
        v.clamp(0, 100) as u8
    }
}

// ============ pure event → cue mapping ============

/// Map core events to sound cues. Pure; testable. (§13.2 effects driven by events)
pub fn cues_from_events(events: &[EngineEvent]) -> Vec<SoundCue> {
    let mut out = Vec::new();
    let mut had_clear = false;
    let mut b2b = false;
    let mut tspin_mini = false;
    let mut tspin_full = false;
    for ev in events {
        match ev {
            EngineEvent::PieceGenerated { .. } => out.push(SoundCue::Spawn),
            EngineEvent::Moved { .. } => out.push(SoundCue::Move),
            EngineEvent::MoveFailed => out.push(SoundCue::MoveFailed),
            EngineEvent::Rotated { .. } => out.push(SoundCue::Rotate),
            EngineEvent::RotationFailed => out.push(SoundCue::RotateFailed),
            EngineEvent::HoldUsed { .. } => out.push(SoundCue::Hold),
            EngineEvent::SoftDropped { .. } => out.push(SoundCue::SoftDrop),
            EngineEvent::HardDropped { .. } => out.push(SoundCue::HardDrop),
            EngineEvent::LockedDown { .. } => out.push(SoundCue::Lock),
            EngineEvent::TSpin { kind } => match kind {
                crate::core::types::TSpinKindRepr::Full => tspin_full = true,
                crate::core::types::TSpinKindRepr::Mini => tspin_mini = true,
            },
            EngineEvent::LinesCleared { count, .. } => {
                if *count > 0 {
                    had_clear = true;
                    out.push(match count {
                        1 => SoundCue::ClearSingle,
                        2 => SoundCue::ClearDouble,
                        3 => SoundCue::ClearTriple,
                        _ => SoundCue::ClearTetris,
                    });
                }
            }
            EngineEvent::Scored { b2b: applied, .. } => {
                if *applied {
                    b2b = true;
                }
            }
            EngineEvent::LevelUp { .. } => out.push(SoundCue::LevelUp),
            EngineEvent::GameOver { .. } => out.push(SoundCue::GameOver),
            EngineEvent::VariantEnd { .. } => out.push(SoundCue::VariantEnd),
            _ => {}
        }
    }
    // T-Spin + B2B chimes layered on line clears. (§13.2; §8.1)
    if tspin_full {
        out.push(SoundCue::TSpin);
    } else if tspin_mini {
        out.push(SoundCue::TSpinMini);
    }
    if b2b && had_clear {
        out.push(SoundCue::BackToBack);
    }
    out
}

// ============ procedural synthesis (no assets) ============

/// Output sample rate. mono i16.
pub const SAMPLE_RATE: u32 = 44_100;

/// Synthesize PCM16 mono samples for a cue. Pure; deterministic. (no protected audio)
pub fn synth_cue(cue: SoundCue, sample_rate: u32) -> Vec<i16> {
    let sr = sample_rate as f32;
    match cue {
        SoundCue::MenuMove => blip(440.0, 0.03, sr, Wave::Square, 0.35),
        SoundCue::MenuSelect => arpeggio(&[440.0, 660.0], 0.05, sr, Wave::Square, 0.4),
        SoundCue::MenuBack => arpeggio(&[660.0, 440.0], 0.05, sr, Wave::Square, 0.4),
        SoundCue::Pause => arpeggio(&[520.0, 390.0], 0.06, sr, Wave::Sine, 0.45),
        SoundCue::Resume => arpeggio(&[390.0, 520.0], 0.06, sr, Wave::Sine, 0.45),
        SoundCue::Spawn => blip(520.0, 0.02, sr, Wave::Sine, 0.22),
        SoundCue::Move => blip(300.0, 0.02, sr, Wave::Square, 0.25),
        SoundCue::MoveFailed => blip(150.0, 0.04, sr, Wave::Square, 0.3),
        SoundCue::Rotate => blip(560.0, 0.03, sr, Wave::Square, 0.3),
        SoundCue::RotateFailed => blip(180.0, 0.05, sr, Wave::Square, 0.3),
        SoundCue::Hold => arpeggio(&[480.0, 600.0], 0.04, sr, Wave::Triangle, 0.35),
        SoundCue::SoftDrop => blip(260.0, 0.015, sr, Wave::Sine, 0.18),
        SoundCue::HardDrop => thump(120.0, 0.07, sr, 0.5),
        SoundCue::Lock => thump(200.0, 0.05, sr, 0.4),
        SoundCue::ClearSingle => arpeggio(&[523.25, 659.25], 0.06, sr, Wave::Square, 0.4),
        SoundCue::ClearDouble => arpeggio(&[523.25, 659.25, 783.99], 0.05, sr, Wave::Square, 0.42),
        SoundCue::ClearTriple => arpeggio(
            &[523.25, 659.25, 783.99, 1046.5],
            0.05,
            sr,
            Wave::Square,
            0.45,
        ),
        SoundCue::ClearTetris => arpeggio(
            &[523.25, 659.25, 783.99, 1046.5, 1318.5],
            0.055,
            sr,
            Wave::Square,
            0.5,
        ),
        SoundCue::TSpin => arpeggio(&[783.99, 987.77, 1174.66], 0.05, sr, Wave::Triangle, 0.45),
        SoundCue::TSpinMini => arpeggio(&[783.99, 987.77], 0.05, sr, Wave::Triangle, 0.4),
        SoundCue::BackToBack => arpeggio(&[880.0, 1108.73, 1318.5], 0.045, sr, Wave::Sine, 0.45),
        SoundCue::LevelUp => arpeggio(
            &[523.25, 659.25, 783.99, 1046.5],
            0.05,
            sr,
            Wave::Square,
            0.45,
        ),
        SoundCue::GameOver => arpeggio(
            &[392.0, 349.23, 311.13, 261.63],
            0.12,
            sr,
            Wave::Square,
            0.45,
        ),
        SoundCue::VariantEnd => arpeggio(
            &[523.25, 659.25, 783.99, 1046.5, 1318.5],
            0.07,
            sr,
            Wave::Sine,
            0.5,
        ),
    }
}

#[derive(Clone, Copy)]
enum Wave {
    Sine,
    Square,
    Triangle,
}

fn osc(wave: Wave, phase: f32) -> f32 {
    match wave {
        Wave::Sine => (phase * std::f32::consts::TAU).sin(),
        Wave::Square => {
            if phase.fract() < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        Wave::Triangle => 4.0 * (phase - (phase + 0.5).floor()).abs() - 1.0,
    }
}

fn amp(sample: f32, vol: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * vol * i16::MAX as f32) as i16
}

/// short tonal blip with attack/decay envelope.
fn blip(freq: f32, dur_s: f32, sr: f32, wave: Wave, vol: f32) -> Vec<i16> {
    let n = (dur_s * sr).max(1.0) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / sr;
            let env = envelope(i, n);
            amp(osc(wave, t * freq) * env, vol)
        })
        .collect()
}

/// low thump (hard drop / lock) with downward pitch + decay + a little noise.
fn thump(freq: f32, dur_s: f32, sr: f32, vol: f32) -> Vec<i16> {
    let n = (dur_s * sr).max(1.0) as usize;
    let mut rng = 0x1234_5678u32;
    (0..n)
        .map(|i| {
            let t = i as f32 / sr;
            let p = freq * (1.0 - (i as f32 / n as f32) * 0.5);
            let body = (t * p * std::f32::consts::TAU).sin();
            // tiny deterministic noise
            rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
            let noise = ((rng >> 8) as f32 / (1u32 << 24) as f32) * 2.0 - 1.0;
            let env = envelope(i, n);
            amp((body * 0.8 + noise * 0.2) * env, vol)
        })
        .collect()
}

/// ascending/descending arpeggio across segments.
fn arpeggio(freqs: &[f32], seg_s: f32, sr: f32, wave: Wave, vol: f32) -> Vec<i16> {
    let seg = (seg_s * sr).max(1.0) as usize;
    let mut out = Vec::with_capacity(seg * freqs.len());
    for &f in freqs {
        for i in 0..seg {
            let t = i as f32 / sr;
            let env = envelope(i, seg);
            out.push(amp(osc(wave, t * f) * env, vol));
        }
    }
    out
}

fn envelope(i: usize, n: usize) -> f32 {
    let attack = (n / 10).max(1);
    let release = (n / 4).max(1);
    if i < attack {
        i as f32 / attack as f32
    } else if i > n.saturating_sub(release) {
        (n - i) as f32 / release as f32
    } else {
        1.0
    }
}

// ============ WAV (PCM16) parser — own, no external lib; testable ============

/// Decoded PCM WAV. mono/stereo; we mix down to mono on use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WavData {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<i16>, // interleaved
}

impl WavData {
    /// Mono f32 samples at `target_rate` (naive nearest resample + mixdown). pure.
    pub fn to_mono_f32(&self, target_rate: u32) -> Vec<f32> {
        if self.samples.is_empty() || self.channels == 0 {
            return Vec::new();
        }
        let ch = self.channels as usize;
        let frames = self.samples.len() / ch;
        // mix to mono
        let mut mono = Vec::with_capacity(frames);
        for f in 0..frames {
            let mut acc = 0.0f32;
            for c in 0..ch {
                acc += self.samples[f * ch + c] as f32 / i16::MAX as f32;
            }
            mono.push(acc / ch as f32);
        }
        if self.sample_rate == target_rate || mono.is_empty() {
            return mono;
        }
        // naive linear resample
        let ratio = self.sample_rate as f32 / target_rate as f32;
        let out_len = ((mono.len() as f32) / ratio).max(1.0) as usize;
        let mut out = Vec::with_capacity(out_len);
        for i in 0..out_len {
            let src = i as f32 * ratio;
            let i0 = src.floor() as usize;
            let i1 = (i0 + 1).min(mono.len() - 1);
            let frac = src - i0 as f32;
            out.push(mono[i0] * (1.0 - frac) + mono[i1.min(mono.len() - 1)] * frac);
        }
        out
    }
}

/// Parse a RIFF/WAVE PCM16 file. Returns `None` on any malformed/unsupported data.
/// (no external lib; robust; pure)
pub fn parse_wav(data: &[u8]) -> Option<WavData> {
    if data.len() < 44 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return None;
    }
    let mut pos = 12;
    let mut sample_rate = 0u32;
    let mut channels = 0u16;
    let mut bits = 0u16;
    let mut format_tag = 0u16;
    let mut samples = Vec::new();
    let mut got_fmt = false;
    let mut got_data = false;
    while pos + 8 <= data.len() {
        let id = &data[pos..pos + 4];
        let size = u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
            as usize;
        let body_start = pos + 8;
        let body_end = body_start.checked_add(size)?.min(data.len());
        let body = &data[body_start..body_end];
        match id {
            b"fmt " => {
                if body.len() < 16 {
                    return None;
                }
                format_tag = u16::from_le_bytes([body[0], body[1]]);
                channels = u16::from_le_bytes([body[2], body[3]]);
                sample_rate = u32::from_le_bytes([body[4], body[5], body[6], body[7]]);
                bits = u16::from_le_bytes([body[14], body[15]]);
                got_fmt = true;
            }
            b"data" => {
                // PCM16 little-endian
                let mut i = 0;
                while i + 1 < body.len() {
                    samples.push(i16::from_le_bytes([body[i], body[i + 1]]));
                    i += 2;
                }
                got_data = true;
            }
            _ => {}
        }
        pos = body_start + size + (size & 1); // chunks are word-aligned
    }
    if !got_fmt || !got_data || format_tag != 1 || bits != 16 || channels == 0 {
        return None;
    }
    Some(WavData {
        sample_rate,
        channels,
        samples,
    })
}

// ============ Audio trait + backends ============

/// Audio sink. (`Audio` trait; §13.2/§13.3) Object-safe.
pub trait Audio {
    fn play(&mut self, cue: SoundCue);
    fn set_sfx_enabled(&mut self, on: bool);
    fn set_music_enabled(&mut self, on: bool);
    fn set_sfx_volume(&mut self, volume: u8); // 0..=100
    fn set_music_volume(&mut self, volume: u8); // 0..=100
    fn set_muted(&mut self, muted: bool);
    fn start_music(&mut self, slot: MusicSlot);
    fn stop_music(&mut self);
    /// service audio each frame (music loop etc). dt ms.
    fn update(&mut self, dt_ms: u32);
    /// apply a full `SoundSettings`.
    fn apply_settings(&mut self, s: SoundSettings) {
        self.set_sfx_enabled(s.sfx_enabled);
        self.set_music_enabled(s.music_enabled);
        self.set_sfx_volume(s.sfx_volume);
        self.set_music_volume(s.music_volume);
        self.set_muted(s.muted);
    }
    fn sound_settings(&self) -> SoundSettings;
}

/// Silent sink (headless / disabled / no device). Always available.
#[derive(Clone, Debug, Default)]
pub struct NullAudio {
    pub settings: SoundSettings,
    pub last_cue: Option<SoundCue>,
    pub music: Option<MusicSlot>,
}

impl NullAudio {
    pub fn new() -> Self {
        Self {
            settings: SoundSettings::default(),
            last_cue: None,
            music: None,
        }
    }
}

impl Audio for NullAudio {
    fn play(&mut self, cue: SoundCue) {
        self.last_cue = Some(cue);
    }
    fn set_sfx_enabled(&mut self, on: bool) {
        self.settings.sfx_enabled = on;
    }
    fn set_music_enabled(&mut self, on: bool) {
        self.settings.music_enabled = on;
    }
    fn set_sfx_volume(&mut self, volume: u8) {
        self.settings.sfx_volume = SoundSettings::clamp_volume(volume as i32);
    }
    fn set_music_volume(&mut self, volume: u8) {
        self.settings.music_volume = SoundSettings::clamp_volume(volume as i32);
    }
    fn set_muted(&mut self, muted: bool) {
        self.settings.muted = muted;
    }
    fn start_music(&mut self, slot: MusicSlot) {
        self.music = Some(slot);
    }
    fn stop_music(&mut self) {
        self.music = None;
    }
    fn update(&mut self, _dt_ms: u32) {}
    fn sound_settings(&self) -> SoundSettings {
        self.settings
    }
}

// ---------- SDL2 backend ----------

/// One playing voice in the mixer.
struct Voice {
    samples: Vec<i16>,
    pos: usize,
    gain: f32,
    looping: bool,
    is_music: bool,
}

/// SDL2 audio callback mixer. (no sdl2_mixer; own tiny mixer)
struct Mixer {
    voices: Vec<Voice>,
    sfx_volume: u8,
    music_volume: u8,
    sfx_enabled: bool,
    music_enabled: bool,
    muted: bool,
}

impl Mixer {
    fn sfx_gain(&self) -> f32 {
        if self.muted || !self.sfx_enabled {
            0.0
        } else {
            self.sfx_volume as f32 / 100.0
        }
    }
    fn music_gain(&self) -> f32 {
        if self.muted || !self.music_enabled {
            0.0
        } else {
            self.music_volume as f32 / 100.0
        }
    }
}

impl AudioCallback for Mixer {
    type Channel = i16;
    fn callback(&mut self, out: &mut [i16]) {
        for sample in out.iter_mut() {
            let mut acc = 0.0f32;
            let sfx_gain = self.sfx_gain();
            let music_gain = self.music_gain();
            let mut i = 0;
            while i < self.voices.len() {
                let v = &mut self.voices[i];
                let gain = if v.is_music { music_gain } else { sfx_gain } * v.gain;
                if v.pos < v.samples.len() {
                    acc += v.samples[v.pos] as f32 / i16::MAX as f32 * gain;
                    v.pos += 1;
                }
                if v.pos >= v.samples.len() {
                    if v.looping && !v.samples.is_empty() {
                        v.pos = 0;
                    } else {
                        self.voices.swap_remove(i);
                        continue;
                    }
                }
                i += 1;
            }
            *sample = (acc.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        }
    }
}

/// SDL2 built-in audio backend. Procedural SFX + optional WAV music loop.
pub struct SdlAudio {
    device: AudioDevice<Mixer>,
    sample_rate: u32,
    settings: SoundSettings,
    /// directory searched for music assets. `<data>/music`.
    pub music_dir: std::path::PathBuf,
}

impl SdlAudio {
    /// Open an audio device. `Err` if SDL audio unavailable (caller falls back to NullAudio).
    pub fn open(audio: &AudioSubsystem, music_dir: std::path::PathBuf) -> Result<Self, String> {
        let desired = AudioSpecDesired {
            freq: Some(SAMPLE_RATE as i32),
            channels: Some(1),
            samples: Some(512),
        };
        let device = audio.open_playback(None, &desired, |_spec| Mixer {
            voices: Vec::new(),
            sfx_volume: 80,
            music_volume: 50,
            sfx_enabled: true,
            music_enabled: true,
            muted: false,
        })?;
        let sample_rate = device.spec().freq.max(1) as u32;
        device.resume();
        Ok(Self {
            device,
            sample_rate,
            settings: SoundSettings::default(),
            music_dir,
        })
    }
}

impl Audio for SdlAudio {
    fn play(&mut self, cue: SoundCue) {
        if !self.settings.sfx_enabled || self.settings.muted {
            return;
        }
        let samples = synth_cue(cue, self.sample_rate);
        let mut dev = self.device.lock();
        dev.voices.push(Voice {
            samples,
            pos: 0,
            gain: 1.0,
            looping: false,
            is_music: false,
        });
    }

    fn set_sfx_enabled(&mut self, on: bool) {
        self.settings.sfx_enabled = on;
        self.device.lock().sfx_enabled = on;
    }
    fn set_music_enabled(&mut self, on: bool) {
        self.settings.music_enabled = on;
        self.device.lock().music_enabled = on;
        if !on {
            self.device.lock().voices.retain(|v| !v.is_music);
        }
    }
    fn set_sfx_volume(&mut self, volume: u8) {
        let v = SoundSettings::clamp_volume(volume as i32);
        self.settings.sfx_volume = v;
        self.device.lock().sfx_volume = v;
    }
    fn set_music_volume(&mut self, volume: u8) {
        let v = SoundSettings::clamp_volume(volume as i32);
        self.settings.music_volume = v;
        self.device.lock().music_volume = v;
    }
    fn set_muted(&mut self, muted: bool) {
        self.settings.muted = muted;
        self.device.lock().muted = muted;
    }
    fn start_music(&mut self, slot: MusicSlot) {
        self.stop_music();
        if !self.settings.music_enabled {
            return;
        }
        let Some(fname) = slot.asset_filename() else {
            return;
        };
        // search order: user `<data>/music` override, then bundled `assets/audio`, then repo assets.
        // `korobeiniki.wav` shipped converted PCM16 (classicals.de CC-BY-NC; see assets/audio/CREDITS;
        // original MP3 NOT used). only PCM16 WAV honored (own parser; no sdl2_mixer; MP3 unsupported).
        let candidates = [
            self.music_dir.join(fname),
            std::path::PathBuf::from("assets").join("audio").join(fname),
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join("audio")
                .join(fname),
        ];
        let Some(bytes) = candidates.iter().find_map(|p| std::fs::read(p).ok()) else {
            return;
        };
        let Some(wav) = parse_wav(&bytes) else { return };
        let samples: Vec<i16> = wav
            .to_mono_f32(self.sample_rate)
            .into_iter()
            .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            .collect();
        if samples.is_empty() {
            return;
        }
        let mut dev = self.device.lock();
        dev.voices.push(Voice {
            samples,
            pos: 0,
            gain: 1.0,
            looping: true,
            is_music: true,
        });
    }
    fn stop_music(&mut self) {
        self.device.lock().voices.retain(|v| !v.is_music);
    }
    fn update(&mut self, _dt_ms: u32) {}
    fn sound_settings(&self) -> SoundSettings {
        self.settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::*;

    #[test]
    fn event_to_cue_mapping() {
        let evs = vec![
            EngineEvent::Moved {
                dir: Direction::Left,
            },
            EngineEvent::Rotated {
                from: Facing::North,
                to: Facing::East,
                kick: 1,
            },
            EngineEvent::HardDropped {
                cells: 3,
                kind: Tetrimino::T,
                facing: Facing::North,
                from: Point::new(0, 0),
                to: Point::new(0, 3),
            },
            EngineEvent::LockedDown {
                kind: Tetrimino::T,
                cells: vec![],
            },
            EngineEvent::LinesCleared {
                count: 4,
                rows: vec![],
            },
            EngineEvent::Scored {
                action: ScoreAction::Tetris,
                points: 800,
                b2b: true,
                total: 800,
                level: 1,
            },
        ];
        let cues = cues_from_events(&evs);
        assert!(cues.contains(&SoundCue::Move));
        assert!(cues.contains(&SoundCue::Rotate));
        assert!(cues.contains(&SoundCue::HardDrop));
        assert!(cues.contains(&SoundCue::Lock));
        assert!(cues.contains(&SoundCue::ClearTetris));
        assert!(cues.contains(&SoundCue::BackToBack)); // b2b + clear
    }

    #[test]
    fn tspin_mini_and_full() {
        let mini = cues_from_events(&[EngineEvent::TSpin {
            kind: TSpinKindRepr::Mini,
        }]);
        assert_eq!(mini, vec![SoundCue::TSpinMini]);
        let full = cues_from_events(&[EngineEvent::TSpin {
            kind: TSpinKindRepr::Full,
        }]);
        assert_eq!(full, vec![SoundCue::TSpin]);
    }

    #[test]
    fn synth_is_deterministic_and_bounded() {
        for cue in [
            SoundCue::Move,
            SoundCue::ClearTetris,
            SoundCue::HardDrop,
            SoundCue::GameOver,
        ] {
            let a = synth_cue(cue, SAMPLE_RATE);
            let b = synth_cue(cue, SAMPLE_RATE);
            assert_eq!(a, b); // deterministic
            assert!(!a.is_empty());
            // i16 samples are inherently bounded; just sanity that some nonzero exists
            assert!(a.iter().any(|&s| s != 0));
        }
    }

    #[test]
    fn volume_clamped_and_gains() {
        assert_eq!(SoundSettings::clamp_volume(-5), 0);
        assert_eq!(SoundSettings::clamp_volume(150), 100);
        let mut s = SoundSettings {
            sfx_volume: 50,
            music_volume: 25,
            ..SoundSettings::default()
        };
        assert!((s.sfx_gain() - 0.5).abs() < 1e-6);
        assert!((s.music_gain() - 0.25).abs() < 1e-6);
        s.muted = true;
        assert_eq!(s.sfx_gain(), 0.0);
        assert_eq!(s.music_gain(), 0.0);
    }

    #[test]
    fn korobeiniki_slot_pcm16_wav() {
        assert_eq!(
            MusicSlot::Korobeiniki.asset_filename(),
            Some("korobeiniki.wav")
        );
        assert_eq!(MusicSlot::Silent.asset_filename(), None);
    }

    #[test]
    fn parse_wav_pcm16_roundtrip() {
        // build a minimal PCM16 mono WAV in-memory
        let samples: Vec<i16> = vec![0, 1000, -1000, 2000, -2000, 0];
        let data_len = (samples.len() * 2) as u32;
        let mut w = Vec::new();
        w.extend_from_slice(b"RIFF");
        w.extend_from_slice(&(36 + data_len).to_le_bytes());
        w.extend_from_slice(b"WAVE");
        w.extend_from_slice(b"fmt ");
        w.extend_from_slice(&16u32.to_le_bytes()); // fmt size
        w.extend_from_slice(&1u16.to_le_bytes()); // PCM
        w.extend_from_slice(&1u16.to_le_bytes()); // mono
        w.extend_from_slice(&44100u32.to_le_bytes()); // rate
        w.extend_from_slice(&(44100u32 * 2).to_le_bytes()); // byte rate
        w.extend_from_slice(&2u16.to_le_bytes()); // block align
        w.extend_from_slice(&16u16.to_le_bytes()); // bits
        w.extend_from_slice(b"data");
        w.extend_from_slice(&data_len.to_le_bytes());
        for s in &samples {
            w.extend_from_slice(&s.to_le_bytes());
        }
        let parsed = parse_wav(&w).expect("valid wav");
        assert_eq!(parsed.channels, 1);
        assert_eq!(parsed.sample_rate, 44100);
        assert_eq!(parsed.samples, samples);
        // garbage rejected
        assert!(parse_wav(b"not a wav").is_none());
        assert!(parse_wav(&w[..20]).is_none());
    }

    #[test]
    fn null_audio_records_and_tracks_settings() {
        let mut a = NullAudio::new();
        a.play(SoundCue::ClearTetris);
        assert_eq!(a.last_cue, Some(SoundCue::ClearTetris));
        a.apply_settings(SoundSettings {
            sfx_volume: 30,
            music_volume: 70,
            muted: true,
            sfx_enabled: false,
            music_enabled: false,
        });
        let s = a.sound_settings();
        assert_eq!(s.sfx_volume, 30);
        assert_eq!(s.music_volume, 70);
        assert!(s.muted);
        a.start_music(MusicSlot::Korobeiniki);
        assert_eq!(a.music, Some(MusicSlot::Korobeiniki));
        a.stop_music();
        assert_eq!(a.music, None);
    }
}
