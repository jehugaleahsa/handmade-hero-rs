use crate::application_error::{ApplicationError, Result};
use handmade_hero_interface::input_state::InputState;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Default)]
enum State {
    #[default]
    None,
    Recording(BufWriter<File>),
    Playing(BufReader<File>),
}

#[derive(Debug, Default)]
pub struct PlaybackRecorder {
    state: State,
    recordings: usize,
}

impl PlaybackRecorder {
    const RECORD_PATH: &'static str = "recording.hmr";

    #[inline]
    pub fn new() -> Self {
        Self {
            state: State::None,
            recordings: 0,
        }
    }

    pub fn record(&mut self, input: &InputState) -> Result<()> {
        let writer = self.get_recording_file()?;
        bincode::encode_into_std_write(input, writer, bincode::config::standard())
            .map_err(|e| ApplicationError::wrap("Could not write to the recording file", e))?;
        self.recordings += 1;
        Ok(())
    }

    fn get_recording_file(&mut self) -> Result<&mut BufWriter<File>> {
        let recording_file = if let State::Recording(ref mut recording_file) = self.state {
            recording_file
        } else {
            let file = File::options()
                .create(true)
                .write(true)
                .truncate(true)
                .open(Self::RECORD_PATH)
                .map_err(|e| ApplicationError::wrap("Could not create the recording file", e))?;
            let writer = BufWriter::new(file);
            self.state = State::Recording(writer);
            self.recordings = 0;
            let State::Recording(ref mut recording_file) = self.state else {
                unreachable!("We just assigned the state to recording but it's not assigned!");
            };
            recording_file
        };
        Ok(recording_file)
    }

    pub fn playback(&mut self) -> Result<Option<InputState>> {
        let Some(reader) = self.get_playback_file()? else {
            return Ok(None);
        };
        if let Ok(input) = bincode::decode_from_reader(reader, bincode::config::standard()) {
            self.recordings -= 1;
            Ok(Some(input))
        } else {
            self.state = State::None;
            Ok(None)
        }
    }

    fn get_playback_file(&mut self) -> Result<Option<&mut BufReader<File>>> {
        if self.recordings == 0 {
            return Ok(None);
        }
        match self.state {
            State::Playing(ref mut file) => Ok(Some(file)),
            State::None => Ok(None),
            State::Recording(_) => {
                let file = File::open(Self::RECORD_PATH)
                    .map_err(|e| ApplicationError::wrap("Could not open the recording file", e))?;
                let reader = BufReader::new(file);
                self.state = State::Playing(reader);
                let State::Playing(ref mut recording_file) = self.state else {
                    unreachable!("We just assigned the state to playback but it's not assigned!");
                };
                Ok(Some(recording_file))
            }
        }
    }
}
