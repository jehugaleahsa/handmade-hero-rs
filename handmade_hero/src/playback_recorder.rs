use handmade_hero_interface::application::Application;
use handmade_hero_interface::application_error::{ApplicationError, Result};
use handmade_hero_interface::game_state::GameState;
use handmade_hero_interface::input_state::InputState;
use handmade_hero_interface::plugin_state::PluginState;
use handmade_hero_interface::plugin_state_seed::PluginStateSeed;
use serde::Deserialize;
use serde::de::DeserializeSeed;
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

/// One recorded frame: the input the game saw and the state it produced.
#[derive(Debug)]
pub struct PlaybackState {
    pub input: InputState,
    pub state: GameState,
    pub plugin: Box<dyn PluginState>,
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

    /// Writes one frame as two consecutive values: the platform-owned part, then the plugin
    /// state. bincode is a bare byte stream with no framing, so consecutive encodes into the same
    /// writer simply concatenate, and [`Self::playback`] reads them back in the same order.
    pub fn record(
        &mut self,
        input: &InputState,
        state: &GameState,
        plugin: &dyn PluginState,
    ) -> Result<()> {
        let writer = self.get_recording_file()?;
        let config = bincode::config::standard();
        bincode::serde::encode_into_std_write((input, state), writer, config).map_err(|e| {
            ApplicationError::wrap("Could not write the game state to the recording file", e)
        })?;
        bincode::serde::encode_into_std_write(plugin, writer, config).map_err(|e| {
            ApplicationError::wrap("Could not write the plugin state to the recording file", e)
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

    /// Reads the next recorded frame. The plugin is needed because a frame ends with state only
    /// the plugin knows how to rebuild.
    pub fn playback(&mut self, application: &dyn Application) -> Result<Option<PlaybackState>> {
        let Some(reader) = self.get_playback_file()? else {
            return Ok(None);
        };
        let mut decoder =
            bincode::serde::OwnedSerdeDecoder::from_reader(reader, bincode::config::standard());
        let platform = <(InputState, GameState)>::deserialize(decoder.as_deserializer());
        let seed = PluginStateSeed::new(application);
        let plugin = seed.deserialize(decoder.as_deserializer());
        if let (Ok((input, state)), Ok(plugin)) = (platform, plugin) {
            self.remaining_recordings -= 1;
            let playback_state = PlaybackState {
                input,
                state,
                plugin,
            };
            Ok(Some(playback_state))
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
