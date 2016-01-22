use std::thread::{self};
use sdl2::{Sdl};
use sdl2::audio::{self, AudioSpecDesired, AudioSpecWAV, AudioCallback, AudioDevice};

struct CopiedData {
    bytes: Vec<u8>,
    position: usize,
}

impl AudioCallback for CopiedData {
    type Channel = u8;

    fn callback(&mut self, data: &mut [u8]) {
        let (start, end) = (self.position, self.position + data.len());
        self.position += data.len();

        let audio_data = &self.bytes[start..end];
        for (src, dst) in audio_data.iter().zip(data.iter_mut()) {
            *dst = *src;
        }
    }
}


struct WrappedData {
    audio: AudioSpecWAV,
    position: usize,
}

impl AudioCallback for WrappedData {
    type Channel = u8;

    fn callback(&mut self, data: &mut [u8]) {
        let (start, end) = (self.position, self.position + data.len());
        self.position += data.len();

        let audio_data = &self.audio.buffer()[start..end];
        for (src, dst) in audio_data.iter().zip(data.iter_mut()) {
            *dst = *src;
        }
    }
}

unsafe impl Send for WrappedData { }

pub fn playback_for(track_path: &str) {
    let sdl_context = ::sdl2::init().unwrap();
    let audio_system = sdl_context.audio().unwrap();

    let audio_spec = AudioSpecDesired{ freq: None, channels: None, samples: None };
    let audio_wav = AudioSpecWAV::load_wav(track_path).unwrap();

    let copied_data = CopiedData{ bytes: audio_wav.buffer().to_vec(), position: 0 };

    let audio_device = audio_system.open_playback(None, audio_spec, move |spec| {
        copied_data
    }).unwrap();

    audio_device.resume();

    thread::sleep_ms(2000);
}
