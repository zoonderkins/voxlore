use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, Stream, StreamConfig};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::audio::resampler::Resampler;
use crate::error::AppError;

/// Target sample rate for STT engines (Vosk requires 16kHz).
const TARGET_SAMPLE_RATE: u32 = 16000;

/// Audio capture manages microphone recording and sample delivery.
pub struct AudioCapture {
    stream: Option<Stream>,
    receiver: Option<mpsc::Receiver<Vec<i16>>>,
    is_recording: Arc<Mutex<bool>>,
}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            stream: None,
            receiver: None,
            is_recording: Arc::new(Mutex::new(false)),
        }
    }

    /// Start recording from the default input device.
    /// Returns a receiver that delivers PCM i16 chunks at 16kHz mono.
    pub fn start(&mut self) -> Result<(), AppError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| AppError::Audio("No input device available".into()))?;

        let config = device
            .default_input_config()
            .map_err(|e| AppError::Audio(format!("Failed to get input config: {e}")))?;

        let source_rate = config.sample_rate().0;
        let channels = config.channels() as usize;

        let (tx, rx) = mpsc::channel::<Vec<i16>>();
        let is_recording = self.is_recording.clone();
        *is_recording.lock().unwrap() = true;

        let resampler = Arc::new(Mutex::new(Resampler::new(source_rate, TARGET_SAMPLE_RATE)));

        let stream_config = StreamConfig {
            channels: config.channels(),
            sample_rate: SampleRate(source_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let err_fn = |err| crate::app_log!("Audio stream error: {err}");

        let stream = match config.sample_format() {
            SampleFormat::I16 => {
                let resampler = resampler.clone();
                let is_recording = is_recording.clone();
                device
                    .build_input_stream(
                        &stream_config,
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            if !*is_recording.lock().unwrap() {
                                return;
                            }
                            // Mix to mono if needed
                            let mono: Vec<i16> = if channels > 1 {
                                data.chunks(channels)
                                    .map(|frame| {
                                        let sum: i32 =
                                            frame.iter().map(|&s| s as i32).sum();
                                        (sum / channels as i32) as i16
                                    })
                                    .collect()
                            } else {
                                data.to_vec()
                            };

                            let resampled = resampler.lock().unwrap().resample(&mono);
                            let _ = tx.send(resampled);
                        },
                        err_fn,
                        None,
                    )
                    .map_err(|e| AppError::Audio(format!("Failed to build stream: {e}")))?
            }
            SampleFormat::F32 => {
                let resampler = resampler.clone();
                let is_recording = is_recording.clone();
                device
                    .build_input_stream(
                        &stream_config,
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            if !*is_recording.lock().unwrap() {
                                return;
                            }
                            // Convert f32 -> i16 and mix to mono
                            let mono: Vec<i16> = if channels > 1 {
                                data.chunks(channels)
                                    .map(|frame| {
                                        let sum: f32 = frame.iter().sum();
                                        let avg = sum / channels as f32;
                                        (avg * i16::MAX as f32) as i16
                                    })
                                    .collect()
                            } else {
                                data.iter()
                                    .map(|&s| (s * i16::MAX as f32) as i16)
                                    .collect()
                            };

                            let resampled = resampler.lock().unwrap().resample(&mono);
                            let _ = tx.send(resampled);
                        },
                        err_fn,
                        None,
                    )
                    .map_err(|e| AppError::Audio(format!("Failed to build stream: {e}")))?
            }
            format => {
                return Err(AppError::Audio(format!(
                    "Unsupported sample format: {format:?}"
                )));
            }
        };

        stream
            .play()
            .map_err(|e| AppError::Audio(format!("Failed to start stream: {e}")))?;

        self.stream = Some(stream);
        self.receiver = Some(rx);
        Ok(())
    }

    /// Stop recording and release the audio stream.
    pub fn stop(&mut self) {
        *self.is_recording.lock().unwrap() = false;
        self.stream = None;
        self.receiver = None;
    }

    /// Take the audio sample receiver (can only be taken once per recording session).
    pub fn take_receiver(&mut self) -> Option<mpsc::Receiver<Vec<i16>>> {
        self.receiver.take()
    }
}
