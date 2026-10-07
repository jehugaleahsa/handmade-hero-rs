use crate::monitor_mode::MonitorMode;

#[derive(Debug, Clone)]
pub enum DisplaySettings {
    Windowed,
    Fullscreen {
        monitor_identifier: String,
        mode: MonitorMode,
    },
}
