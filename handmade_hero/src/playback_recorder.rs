use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::{ApplicationError, Result};
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::game_state_seed::GameStateSeed;
use handmade_hero_interface::input_state::InputState;
use serde::Serialize;
use serde::de::{DeserializeSeed, Deserializer, Error, SeqAccess, Visitor};
use std::fmt::{self, Debug, Formatter};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

#[derive(Debug, Default)]
enum State {
    #[default]
    None,
    Recording(BufWriter<File>),
    Playing(BufReader<File>),
}

/// One frame as it is written: the input the game saw and the state it produced.
#[derive(Debug, Serialize)]
struct PlaybackEncoding<'a>(&'a InputState, &'a GameState);

#[derive(Debug)]
pub struct PlaybackState {
    pub input: InputState,
    pub state: GameState,
}

/// Reads one frame back. `PlaybackEncoding` is written as a two-element tuple struct, so this
/// reads a two-element tuple struct, deserializing the second element with a [`GameStateSeed`]
/// because only the plugin can rebuild the plugin-owned parts of a [`GameState`].
#[derive(Clone, Copy)]
struct PlaybackSeed<'a> {
    application: &'a dyn Application,
}

impl<'de> DeserializeSeed<'de> for PlaybackSeed<'_> {
    type Value = PlaybackState;

    #[inline]
    fn deserialize<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        deserializer.deserialize_tuple_struct("PlaybackEncoding", 2, self)
    }
}

impl<'de> Visitor<'de> for PlaybackSeed<'_> {
    type Value = PlaybackState;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("a recorded frame of input and game state")
    }

    fn visit_seq<A: SeqAccess<'de>>(
        self,
        mut seq: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let input = seq
            .next_element()?
            .ok_or_else(|| Error::invalid_length(0, &self))?;
        let state = seq
            .next_element_seed(GameStateSeed::new(self.application))?
            .ok_or_else(|| Error::invalid_length(1, &self))?;
        Ok(PlaybackState { input, state })
    }
}

impl Debug for PlaybackSeed<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PlaybackSeed")
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Default)]
pub struct PlaybackRecorder {
    recording_directory: PathBuf,
    state: State,
    total_recordings: usize,
    remaining_recordings: usize,
}

impl PlaybackRecorder {
    const RECORD_FILE_NAME: &'static str = "recording.hmr";

    #[inline]
    #[must_use]
    pub fn new(recording_directory: impl Into<PathBuf>) -> Self {
        Self {
            recording_directory: recording_directory.into(),
            state: State::None,
            total_recordings: 0,
            remaining_recordings: 0,
        }
    }

    pub fn record(&mut self, input: &InputState, state: &GameState) -> Result<()> {
        let writer = self.get_recording_file()?;
        let recording = PlaybackEncoding(input, state);
        bincode::serde::encode_into_std_write(recording, writer, bincode::config::standard())
            .map_err(|e| {
                ApplicationError::wrap("Could not write the state to the recording file", e)
            })?;
        self.total_recordings += 1;
        Ok(())
    }

    fn get_recording_file(&mut self) -> Result<&mut BufWriter<File>> {
        let recording_file = if let State::Recording(ref mut recording_file) = self.state {
            recording_file
        } else {
            let file_path = self.recording_directory.join(Self::RECORD_FILE_NAME);
            let file = File::options()
                .create(true)
                .write(true)
                .truncate(true)
                .open(file_path)
                .map_err(|e| ApplicationError::wrap("Could not create the recording file", e))?;
            let writer = BufWriter::new(file);
            self.state = State::Recording(writer);
            let State::Recording(ref mut recording_file) = self.state else {
                unreachable!("We just assigned the state to recording but it's not assigned!");
            };
            self.total_recordings = 0;
            recording_file
        };
        Ok(recording_file)
    }

    /// Reads the next recorded frame. The plugin is needed because a frame contains state only
    /// the plugin knows how to rebuild.
    pub fn playback(&mut self, application: &dyn Application) -> Result<Option<PlaybackState>> {
        let Some(reader) = self.get_playback_file()? else {
            return Ok(None);
        };
        // The convenience functions on `bincode::serde` want a `DeserializeOwned`, which a seed
        // is not. Going one level down gives a plain `serde::Deserializer` a seed can drive.
        // `BufReader` implements bincode's own `Reader` trait, so it plugs in directly.
        let mut decoder =
            bincode::serde::OwnedSerdeDecoder::from_reader(reader, bincode::config::standard());
        let seed = PlaybackSeed { application };
        if let Ok(playback) = seed.deserialize(decoder.as_deserializer()) {
            self.remaining_recordings -= 1;
            Ok(Some(playback))
        } else {
            self.state = State::None;
            Ok(None)
        }
    }

    fn get_playback_file(&mut self) -> Result<Option<&mut BufReader<File>>> {
        match self.state {
            State::None => Ok(None),
            State::Playing(ref mut file) => {
                if self.remaining_recordings == 0 {
                    // Avoid trying to read an empty file.
                    Ok(None)
                } else {
                    Ok(Some(file))
                }
            }
            State::Recording(_) => {
                self.start_playing()?;
                let State::Playing(ref mut recording_file) = self.state else {
                    unreachable!("We just assigned the state to playback but it's not assigned!");
                };
                Ok(Some(recording_file))
            }
        }
    }

    pub fn reset_playback(&mut self) -> Result<()> {
        if let State::Playing(_) = self.state {
            self.start_playing()?;
        }
        Ok(())
    }

    fn start_playing(&mut self) -> Result<()> {
        let file_path = self.recording_directory.join(Self::RECORD_FILE_NAME);
        let file = File::open(file_path)
            .map_err(|e| ApplicationError::wrap("Could not open the recording file", e))?;
        let reader = BufReader::new(file);
        self.state = State::Playing(reader);
        self.remaining_recordings = self.total_recordings;
        Ok(())
    }
}
