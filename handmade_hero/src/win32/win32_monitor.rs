use handmade_hero_interface::{narrow_unsigned, units::si::frequency::Frequency};
use uom::si::frequency::hertz;
use windows::Win32::Graphics::Gdi::{DEVMODEW, ENUM_CURRENT_SETTINGS, EnumDisplaySettingsW};

const DEFAULT_REFRESH_RATE: u32 = 60;

pub fn find_monitor_refresh_rate() -> Frequency {
    let size = narrow_unsigned!(size_of::<DEVMODEW>() => u16);
    let mut mode = DEVMODEW {
        dmSize: size,
        ..DEVMODEW::default()
    };
    let success = unsafe { EnumDisplaySettingsW(None, ENUM_CURRENT_SETTINGS, &raw mut mode) };
    if !success.as_bool() {
        return Frequency::new::<hertz>(DEFAULT_REFRESH_RATE);
    }
    let frequency = mode.dmDisplayFrequency;
    if frequency == 0 || frequency == 1 {
        return Frequency::new::<hertz>(DEFAULT_REFRESH_RATE);
    }
    Frequency::new::<hertz>(frequency)
}
