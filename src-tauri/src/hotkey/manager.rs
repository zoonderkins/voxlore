#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Input modes for voice recording.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum InputMode {
    /// Hold to record, release to transcribe.
    PushToTalk,
    /// Press to start, press again to stop.
    Toggle,
}

/// Recording state machine states.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RecordingState {
    Idle,
    Recording,
    Processing,
}

/// Manages the recording state machine based on hotkey events.
pub struct HotkeyManager {
    pub mode: InputMode,
    pub state: RecordingState,
}

impl HotkeyManager {
    pub fn new(mode: InputMode) -> Self {
        Self {
            mode,
            state: RecordingState::Idle,
        }
    }

    /// Handle a key down event. Returns the new state.
    pub fn on_key_down(&mut self) -> RecordingState {
        match self.mode {
            InputMode::PushToTalk => {
                if self.state == RecordingState::Idle {
                    self.state = RecordingState::Recording;
                }
            }
            InputMode::Toggle => {
                self.state = match self.state {
                    RecordingState::Idle => RecordingState::Recording,
                    RecordingState::Recording => RecordingState::Processing,
                    RecordingState::Processing => RecordingState::Processing, // no-op while processing
                };
            }
        }
        self.state
    }

    /// Handle a key up event. Returns the new state.
    pub fn on_key_up(&mut self) -> RecordingState {
        match self.mode {
            InputMode::PushToTalk => {
                if self.state == RecordingState::Recording {
                    self.state = RecordingState::Processing;
                }
            }
            InputMode::Toggle => {
                // Toggle mode doesn't react to key up
            }
        }
        self.state
    }

    /// Mark processing as complete, return to idle.
    pub fn on_processing_complete(&mut self) {
        self.state = RecordingState::Idle;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_to_talk_flow() {
        let mut mgr = HotkeyManager::new(InputMode::PushToTalk);
        assert_eq!(mgr.state, RecordingState::Idle);

        assert_eq!(mgr.on_key_down(), RecordingState::Recording);
        assert_eq!(mgr.on_key_up(), RecordingState::Processing);

        mgr.on_processing_complete();
        assert_eq!(mgr.state, RecordingState::Idle);
    }

    #[test]
    fn test_toggle_flow() {
        let mut mgr = HotkeyManager::new(InputMode::Toggle);
        assert_eq!(mgr.state, RecordingState::Idle);

        // First press -> recording
        assert_eq!(mgr.on_key_down(), RecordingState::Recording);
        mgr.on_key_up(); // no-op for toggle

        // Second press -> processing
        assert_eq!(mgr.on_key_down(), RecordingState::Processing);

        mgr.on_processing_complete();
        assert_eq!(mgr.state, RecordingState::Idle);
    }
}
