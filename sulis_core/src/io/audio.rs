//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::cell::RefCell;
use std::io::{BufReader, Error, ErrorKind};
use std::fs::File;

use rodio::{Sink, Device, DeviceTrait, Source, Decoder, source::Buffered};

use crate::config::{AudioConfig, Config};
use crate::resource::ResourceSet;

thread_local! {
    static AUDIO_QUEUE: RefCell<Vec<QueueEntry>> = RefCell::new(Vec::new());
}

enum QueueKind {
    Music,
    Sfx,
    StopMusic,
}

struct QueueEntry {
    sound: Option<SoundSource>,
    kind: QueueKind,
}

pub struct Audio {}

impl Audio {
    pub fn stop_music() {
        AUDIO_QUEUE.with(|q| q.borrow_mut().push(
            QueueEntry { sound: None, kind: QueueKind::StopMusic }
        ));
    }

    pub fn play_music(source_id: &str) {
        Audio::enqueue(source_id, QueueKind::Music);
    }

    pub fn play_sfx(source_id: &str) {
        Audio::enqueue(source_id, QueueKind::Sfx);
    }

    fn enqueue(source_id: &str, kind: QueueKind) {
        let sound = match ResourceSet::sound(source_id) {
            Err(e) => {
                warn!("Unable to locate sound '{}': {}", source_id, e);
                return;
            },
            Ok(sound) => Some(sound),
        };

        let entry = QueueEntry { sound, kind };

        AUDIO_QUEUE.with(|q| q.borrow_mut().push(entry));
    }

    pub(crate) fn update(device: Option<&mut AudioDevice>, _elapsed_millis: u32) {
        match device {
            None => AUDIO_QUEUE.with(|q| q.borrow_mut().clear()),
            Some(device) => {
                let entries: Vec<_> = AUDIO_QUEUE.with(|q| q.borrow_mut().drain(..).collect());
                for entry in entries {
                    device.play(entry);
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct SoundSource {
    id: String,
    sound: Buffered<Decoder<BufReader<File>>>,
    loops: bool,
    volume: Option<f32>,
}

impl SoundSource {
    pub fn new(
        id: String,
        file: File,
        loops: bool,
        volume: Option<f32>
    ) -> Result<SoundSource, Error> {
        let sound = match Decoder::new(BufReader::new(file)) {
            Ok(sound) => sound,
            Err(e) => {
                warn!("Error reading sound from file: {}", e);
                return Err(Error::new(ErrorKind::InvalidData, e));
            }
        };

        Ok(SoundSource { id, sound: sound.buffered(), loops, volume })
    }
}

struct AudioSink {
    sink: Sink,
}

impl AudioSink {
    fn play(&self, source: SoundSource) {
        let sound = match source.volume {
            None => source.sound.amplify(1.0),
            Some(vol) => source.sound.amplify(vol),
        };
        if source.loops {
            self.sink.append(sound.repeat_infinite());
        } else {
            self.sink.append(sound);
        }
    }

    fn stop(&self) {
        self.sink.stop();
    }

    fn detach(self) {
        self.sink.detach();
    }

    fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }
}

pub struct AudioDevice {
    device: Device,
    name: String,
    config: AudioConfig,
    music: AudioSink,
    last_music_id: Option<String>,
}

impl AudioDevice {
    pub fn name(&self) -> &str {
        &self.name
    }

    fn new(device: Device, name: String, mut config: AudioConfig) -> AudioDevice {
        // precompute output volumes
        config.music_volume *= config.master_volume;
        config.effects_volume *= config.master_volume;
        config.ambient_volume *= config.master_volume;

        // this sink will never actually be used since each music play
        // creates a new one
        let sink = Sink::new(&device);
        let music = AudioSink { sink };

        AudioDevice {
            device,
            name,
            config,
            music,
            last_music_id: None,
        }
    }

    fn play(&mut self, entry: QueueEntry) {
        match entry.kind {
            QueueKind::Music => self.play_music(entry.sound.unwrap()),
            QueueKind::Sfx => self.play_sfx(entry.sound.unwrap()),
            QueueKind::StopMusic => self.stop_music(),
        }
    }

    fn stop_music(&mut self) {
        self.last_music_id = None;
        self.music.stop();
    }

    fn play_music(&mut self, sound: SoundSource) {
        if self.last_music_id.as_ref() == Some(&sound.id) { return; }

        self.last_music_id = Some(sound.id.to_string());
        self.music.stop();
        self.music = self.create_sink();
        self.music.set_volume(self.config.music_volume);
        self.music.play(sound);
    }

    fn play_sfx(&mut self, sound: SoundSource) {
        let sink = self.create_sink();
        sink.set_volume(self.config.effects_volume);
        sink.play(sound);
        sink.detach();
    }

    fn create_sink(&self) -> AudioSink {
        let sink = Sink::new(&self.device);
        AudioSink { sink }
    }
}

fn device_name(device: &Device, index: usize) -> String {
    device.name().unwrap_or_else(|_e| format!("Audio Device {}", index))
}

#[derive(Clone)]
pub struct AudioDeviceInfo {
    pub name: String,
}

pub fn get_audio_device_info() -> Vec<AudioDeviceInfo> {
    let devices = get_audio_devices();

    devices.into_iter().map(|device| AudioDeviceInfo { name: device.name }).collect()
}

fn get_audio_devices() -> Vec<AudioDevice> {
    let audio_config = Config::audio_config();

    let devices = match rodio::output_devices() {
        Err(e) => {
            warn!("Error querying audio devices: {}", e);
            return Vec::new();
        },
        Ok(devices) => devices,
    };

    let mut output = Vec::new();
    for (index, device) in devices.enumerate() {
        let name = device_name(&device, index);
        let config = audio_config.clone();
        output.push(AudioDevice::new(device, name, config));
    }

    output
}

pub fn create_audio_device() -> Option<AudioDevice> {
    let mut devices = get_audio_devices();

    if devices.is_empty() {
        warn!("No available audio devices.  Audio disabled.");
        return None;
    }

    let audio_config = Config::audio_config();

    if audio_config.device < devices.len() {
        return Some(devices.remove(audio_config.device));
    }

    warn!("Configured audio device with index {} not found", audio_config.device);
    warn!("Using default audio device");

    Some(devices.remove(0))
}
