use serde::{Deserialize, Serialize};
use std::{ffi::OsStr, path::Path};

use sndfile::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInfo {
    pub path: String,
    pub name: String,
    pub duration_sec: f64,
    pub image: String,
}

impl AudioInfo {
    pub fn default() -> Self {
        Self {
            path: String::default(),
            name: String::default(),
            image: String::default(),
            duration_sec: 0.0,
        }
    }

    pub fn update_file(&mut self, path: String) -> Result<(), AudioFileError> {
        let mut snd = sndfile::OpenOptions::ReadOnly(ReadOptions::Auto).from_path(path.clone())?;
        let sample_rate = snd.get_samplerate();
        let channels = snd.get_channels();
        let length = snd.len().unwrap();
        let duration_sec = length as f64 / sample_rate as f64 / channels as f64;

        let name = Path::new(&path)
            .file_name()
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap_or(&"")
            .to_string();

        let image = AudioInfo::get_image(snd)?;

        self.duration_sec = duration_sec;
        self.image = image;
        self.name = name;
        self.path = path;

        Ok(())
    }

    fn get_image(mut snd: SndFile) -> Result<String, ()> {
        let frames = 100;
        let samples_64: Vec<f64> = snd.read_all_to_vec()?;
        let frame_size = samples_64.len() / frames;

        let mut frame_values: Vec<f64> = vec![0.0; frames];

        // get 99th-ish percentile values
        for i in 0..frames {
            let start = frame_size * i;
            let end = frame_size * (i + 1);
            let mut values: Vec<f64> = vec![];
            for j in start..end {
                values.push(samples_64[j].abs());
            }
            values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            frame_values[i] = values[frame_size / 50 * 49];
        }

        // generate path
        // assume 16:9 aspect ratio.
        let mut path = String::default();
        let vertical_factor = 450.0; // 900 / 2
        let horizontal_step = 1600.0 / frames as f64;
        for i in 0..frame_values.len() {
            let pos_v = (vertical_factor + frame_values[i] * vertical_factor)
                .round()
                .to_string();
            if i == 0 {
                path += &format!("M{} {}", (i as f64 * horizontal_step) as usize, pos_v);
            } else {
                path += &format!("L{} {}", (i as f64 * horizontal_step) as usize, pos_v);
            }
        }
        // and backwards
        for i in (0..frame_values.len()).rev() {
            let neg_v = (vertical_factor - frame_values[i] * vertical_factor)
                .round()
                .to_string();
            if i == 0 {
                path += &format!("L{} {} ", (i as f64 * horizontal_step) as usize, neg_v);
                // loop back to origin
                let pos_v = (vertical_factor + frame_values[i] * vertical_factor)
                    .round()
                    .to_string();
                path += &format!("L{} {} ", (i as f64 * horizontal_step) as usize, pos_v);
            } else {
                let current = (vertical_factor - frame_values[i] * vertical_factor)
                    .round()
                    .to_string();
                path += &format!("L{} {} ", (i as f64 * horizontal_step) as usize, current);
            }
        }
        let svg = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" viewBox=\"0 0 1600 900\" preserveAspectRatio=\"none\" class=\"audio-svg\"><path class=\"audio-svg-path\" d=\"{}\"/></svg>", path);
        Ok(svg)
    }
}

#[derive(Debug)]
pub enum AudioFileError {
    Snd(SndFileError),
    Unknown(()),
}

impl From<SndFileError> for AudioFileError {
    fn from(error: SndFileError) -> Self {
        AudioFileError::Snd(error)
    }
}

impl From<()> for AudioFileError {
    fn from(error: ()) -> Self {
        AudioFileError::Unknown(error)
    }
}
