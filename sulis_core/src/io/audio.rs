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

use std::thread::{self, JoinHandle};
use std::fmt;
use std::time::Duration;
use std::collections::VecDeque;
use std::cell::{RefCell};
use std::io::{BufReader, Error, ErrorKind};
use std::fs::File;

use rodio::{
    Sink, Device, DeviceTrait, Source, Decoder, OutputStream, OutputStreamHandle,
    source::Buffered,
    cpal::traits::HostTrait,
};

use crate::config::{AudioConfig, Config};
use crate::resource::{sound_set::EntryBuilder, ResourceSet};

thread_local! {
    static AUDIO_QUEUE: RefCell<Vec<QueueEntry>> = RefCell::new(Vec::new());
}

#[derive(PartialEq, Eq)]
enum QueueKind {
    Ambient,
    StopAmbient,
    Music,
    StopMusic,
    Sfx,
}

#[derive(Eq, PartialEq)]
struct QueueEntry {
    sound: Option<SoundSource>,
    kind: QueueKind,
}

pub struct Audio {}

impl Audio {
    pub fn stop_ambient() {
        Audio::enqueue(None, QueueKind::StopAmbient);
    }

    pub fn change_ambient(sound: Option<SoundSource>) {
        let kind = if sound.is_some() {
            QueueKind::Ambient
        } else {
            QueueKind::StopAmbient
        };
        Audio::enqueue(sound, kind);
    }

    pub fn change_music(sound: Option<SoundSource>) {
        let kind = if sound.is_some() {
            QueueKind::Music
        } else {
            QueueKind::StopMusic
        };
        Audio::enqueue(sound, kind);
    }

    pub fn play_ambient(source_id: &str, volume: f32) {
        Audio::enqueue_id(source_id, QueueKind::Ambient, volume);
    }

    pub fn stop_music() {
        Audio::enqueue(None, QueueKind::StopMusic);
    }

    pub fn play_music(source_id: &str, volume: f32) {
        Audio::enqueue_id(source_id, QueueKind::Music, volume);
    }

    pub fn play_sfx(source_id: &str, volume: f32) {
        Audio::enqueue_id(source_id, QueueKind::Sfx, volume);
    }

    fn enqueue(sound: Option<SoundSource>, kind: QueueKind) {
        AUDIO_QUEUE.with(|q| {
            q.borrow_mut().push(QueueEntry { sound, kind });
        });
    }

    fn enqueue_id(source_id: &str, kind: QueueKind, volume: f32) {
        let sound = match ResourceSet::sound(source_id) {
            Err(e) => {
                warn!("Unable to locate sound '{}': {}", source_id, e);
                return;
            },
            Ok(mut sound) => {
                sound.mult_volume(volume);
                Some(sound)
            }
        };

        Audio::enqueue(sound, kind);
    }

    pub(crate) fn update(device: Option<&mut AudioDevice>, elapsed_millis: u32) {
        match device {
            None => AUDIO_QUEUE.with(|q| q.borrow_mut().clear()),
            Some(device) => {
                let entries: Vec<_> = AUDIO_QUEUE.with(|q| q.borrow_mut().drain(..).collect());
                for entry in entries {
                    device.play(entry);
                }

                device.update(elapsed_millis);
            }
        }
    }
}

#[derive(Clone)]
pub struct SoundSource {
    id: String,
    sound: Buffered<Decoder<BufReader<File>>>,
    loops: bool,
    volume: f32,
    delay: Duration,
}

impl PartialEq for SoundSource {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SoundSource {}

impl SoundSource {
    pub fn new(
        id: String,
        file: File,
        entry: &EntryBuilder
    ) -> Result<SoundSource, Error> {
        let sound = match Decoder::new(BufReader::new(file)) {
            Ok(sound) => sound,
            Err(e) => {
                warn!("Error reading sound from file: {}", e);
                return Err(Error::new(ErrorKind::InvalidData, e));
            }
        };

        Ok(SoundSource {
            id,
            sound: sound.buffered(),
            loops: entry.loops,
            volume: entry.volume,
            delay: Duration::from_secs_f32(entry.delay),
        })
    }

    pub fn mult_volume(&mut self, volume: f32) {
        self.volume *= volume;
    }
}

const FADE_TIME: i32 = 1000;

enum SinkQueueEntry {
    FadeIn(i32),
    FadeOut(i32),
    Stop,
    Start(SoundSource),
}

impl fmt::Debug for SinkQueueEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SinkQueueEntry::*;
        match self {
            FadeIn(time) => write!(f, "FadeIn {time}"),
            FadeOut(time) => write!(f, "FadeOut {time}"),
            Stop => write!(f, "Stop"),
            Start(_) => write!(f, "Start"),
        }
    }
}

struct AudioSink {
    sink: Sink,
    cur_id: String,
    queue: VecDeque<SinkQueueEntry>,
    base_volume: f32,
}

impl AudioSink {
    fn new(handle: &OutputStreamHandle, base_volume: f32) -> Result<AudioSink, String> {
        // TODO PlayError doesn't implement std::error::Error yet
        let sink = match Sink::try_new(handle) {
            Ok(sink) => sink,
            Err(_) => {
                return Err("Error decoding audio or output device was lost".to_string());
            }
        };

        Ok(AudioSink {
            sink,
            cur_id: String::new(),
            queue: VecDeque::new(),
            base_volume
        })
    }

    fn update(&mut self, handle: &OutputStreamHandle, elapsed_millis: u32) {
        let millis = elapsed_millis as i32;

        loop {
            let entry = match self.queue.pop_front() {
                None => break,
                Some(entry) => entry,
            };

            use SinkQueueEntry::*;
            match entry {
                FadeIn(time) => {
                    let time = time - millis;
                    if time < 0 {
                        self.sink.set_volume(self.base_volume);
                        continue;
                    } else {
                        self.sink.set_volume(
                            self.base_volume * (1.0 - (time as f32 / FADE_TIME as f32))
                        );
                        self.queue.push_front(FadeIn(time));
                        break;
                    }
                },
                FadeOut(time) => {
                    let time = time - millis;
                    if time < 0 {
                        self.sink.set_volume(0.0);
                        continue;
                    } else {
                        self.sink.set_volume(
                            self.base_volume * (time as f32 / FADE_TIME as f32)
                        );
                        self.queue.push_front(FadeOut(time));
                        break;
                    }
                },
                Stop => {
                    self.cur_id.clear();
                    self.sink.stop();
                    self.sink = Sink::try_new(handle).expect("Failed to create new sink");
                    self.sink.set_volume(self.base_volume);
                },
                Start(sound) => {
                    self.play_immediate(sound);
                }
            }
        }
    }

    fn stop_play(&mut self) {
        self.queue.push_back(SinkQueueEntry::FadeOut(FADE_TIME));
        self.queue.push_back(SinkQueueEntry::Stop);
    }

    fn switch_to_source(&mut self, source: SoundSource) {
        if self.cur_id == source.id { return; }

        if !self.cur_id.is_empty() {
            self.queue.push_back(SinkQueueEntry::FadeOut(FADE_TIME));
            self.queue.push_back(SinkQueueEntry::Stop);
        }

        self.cur_id = source.id.to_string();
        self.queue.push_back(SinkQueueEntry::Start(source));
        self.queue.push_back(SinkQueueEntry::FadeIn(FADE_TIME));
    }

    fn play_immediate(&mut self, source: SoundSource) {
        self.cur_id = source.id;

        let sound = source.sound.amplify(source.volume).delay(source.delay);

        if source.loops {
            self.sink.append(sound.repeat_infinite());
        } else {
            self.sink.append(sound);
        }
    }

    fn detach(self) {
        self.sink.detach();
    }
}

pub struct AudioDevice {
    stream_handle: OutputStreamHandle,
    name: String,
    config: AudioConfig,
    music: AudioSink,
    ambient: AudioSink,
}

fn new_device(device: Device, name: String, mut config: AudioConfig) -> Result<AudioDevice, String> {
    // precompute output volumes
    config.music_volume *= config.master_volume;
    config.effects_volume *= config.master_volume;
    config.ambient_volume *= config.master_volume;

    let (stream, stream_handle) = match OutputStream::try_from_device(&device) {
        Ok(data) => data,
        Err(e) => {
            warn!("Error creating audio output stream: {}", e);
            return Err("Unable to create audio output stream".to_string())
        }
    };

    // TODO to handle this properly we probably need to move all audio processing
    // into its own thread.  we can't send stream and dropping it stops all playback
    std::mem::forget(stream);

    let music = AudioSink::new(&stream_handle, config.music_volume)?;
    let ambient = AudioSink::new(&stream_handle, config.ambient_volume)?;

    Ok(AudioDevice {
        stream_handle,
        name,
        config,
        music,
        ambient,
    })
}

impl AudioDevice {
    pub fn name(&self) -> &str {
        &self.name
    }

    fn new(device: Device, name: String, config: AudioConfig) -> Result<AudioDevice, String> {
        // Run Rodio init in its own thread to avoid problems with winit:
        // this is really nasty now that you can't send cpal stuff over threads easily
        // https://github.com/rust-windowing/winit/issues/1185
        let device_init: JoinHandle<Result<AudioDevice, String>> = thread::spawn(move || {
            new_device(device, name, config)
        });

        let result = match device_init.join() {
            Ok(result) => result,
            Err(_) => {
                return Err("Thread panic initializing audio system".to_string());
            }
        };

        result
    }

    fn update(&mut self, elapsed_millis: u32) {
        self.music.update(&self.stream_handle, elapsed_millis);
        self.ambient.update(&self.stream_handle, elapsed_millis);
    }

    fn play(&mut self, entry: QueueEntry) {
        match entry.kind {
            QueueKind::Music => self.play_music(entry.sound.unwrap()),
            QueueKind::StopMusic => self.stop_music(),
            QueueKind::Sfx => self.play_sfx(entry.sound.unwrap()),
            QueueKind::Ambient => self.play_ambient(entry.sound.unwrap()),
            QueueKind::StopAmbient => self.stop_ambient(),
        }
    }

    fn stop_music(&mut self) {
        self.music.stop_play();
    }

    fn play_music(&mut self, sound: SoundSource) {
        self.music.switch_to_source(sound);
    }

    fn stop_ambient(&mut self) {
        self.ambient.stop_play();
    }

    fn play_ambient(&mut self, sound: SoundSource) {
        self.ambient.switch_to_source(sound);
    }

    fn play_sfx(&mut self, sound: SoundSource) {
        let mut sink = match AudioSink::new(&self.stream_handle, self.config.effects_volume) {
            Err(_) => return,
            Ok(sink) => sink,
        };
        sink.play_immediate(sound);
        sink.detach();
    }
}

fn device_name(device: &Device, index: usize) -> String {
    device.name().unwrap_or_else(|_e| format!("Audio Device {index}"))
}

pub struct AudioDeviceInfo {
    pub name: String,
    device: Device,
    config: AudioConfig,
}

pub fn get_audio_devices() -> Vec<AudioDeviceInfo> {
    let audio_config = Config::audio_config();

    info!("Querying audio devices");

    let host = rodio::cpal::default_host();

    // Run Rodio init in its own thread to avoid problems with winit:
    // https://github.com/rust-windowing/winit/issues/1185
    let audio_thread = thread::spawn(move || {
        let devices = match host.output_devices() {
            Err(e) => {
                warn!("Error querying audio devices: {}", e);
                return Vec::new();
            },
            Ok(devices) => devices,
        };

        let mut output = Vec::new();
        for (index, device) in devices.enumerate() {
            let name = device_name(&device, index);

            if name.contains("CARD=") {
                // this device will most likely crash rodio
                continue;
            }

            let mut formats = match device.supported_output_configs() {
                Err(e) => {
                    info!("Error getting supported configs for audio device {}: {}", name, e);
                    continue;
                }, Ok(formats) => formats,
            };

            if formats.next().is_none() {
                info!("Audio device {} did not have any supported output configs.", name);
                continue;
            }

            if let Err(e) = device.default_output_config() {
                info!("Error getting an output config for audio device: {}: {}", name, e);
                continue;
            }

            let config = audio_config.clone();
            let audio_device = AudioDeviceInfo {
                device,
                name: name.to_string(),
                config
            };

            info!("Found supported audio device: {}", name);
            output.push(audio_device);
        }

        output
    });

    match audio_thread.join() {
        Err(e) => {
            warn!("Error in audio setup: {:?}", e);
            Vec::new()
        }, Ok(output) => output,
    }
}

pub fn init_device(info: AudioDeviceInfo) -> Option<AudioDevice> {
    match AudioDevice::new(info.device, info.name.to_string(), info.config) {
        Err(_) => {
            warn!("Unable to create audio device for {}", info.name);
            None
        }, Ok(device) => Some(device),
    }
}

pub fn create_audio_device() -> Option<AudioDevice> {
    let mut devices = get_audio_devices();

    if devices.is_empty() {
        warn!("No available audio devices.  Audio disabled.");
        return None;
    }

    let audio_config = Config::audio_config();

    if audio_config.device < devices.len() {
        return init_device(devices.remove(audio_config.device));
    }

    warn!("Configured audio device with index {} not found", audio_config.device);
    warn!("Using default audio device");

    init_device(devices.remove(0))
}
