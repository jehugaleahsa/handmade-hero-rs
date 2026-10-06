use handmade_hero_interface::monitor::Monitor;
use handmade_hero_interface::units::si::length::{Length, pixel};
use handmade_hero_interface::{narrow_unsigned, units::si::frequency::Frequency};
use uom::si::frequency::hertz;
use windows::Win32::Foundation::{LPARAM, RECT, TRUE};
use windows::Win32::Graphics::Gdi::{
    DEVMODEW, ENUM_CURRENT_SETTINGS, EnumDisplayMonitors, EnumDisplaySettingsW, GetMonitorInfoW,
    HDC, HMONITOR, MONITORINFOEXW,
};
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::WindowsAndMessaging::MONITORINFOF_PRIMARY;
use windows::core::{BOOL, PCWSTR, Result};

pub fn find_monitors() -> Vec<Monitor> {
    let mut monitors = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(add_monitor),
            LPARAM(&raw mut monitors as isize),
        );
    }
    monitors
}

extern "system" fn add_monitor(next: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
    let mut monitor_info = MONITORINFOEXW::default();
    monitor_info.monitorInfo.cbSize = narrow_unsigned!(size_of::<MONITORINFOEXW>() => u32);
    let monitor_info_result = unsafe { GetMonitorInfoW(next, &raw mut monitor_info.monitorInfo) };
    if !monitor_info_result.as_bool() {
        return TRUE;
    }
    let device_name = PCWSTR(monitor_info.szDevice.as_ptr());
    let Some(refresh_rate) = find_monitor_refresh_rate(device_name) else {
        return TRUE;
    };

    let dimensions = &monitor_info.monitorInfo.rcMonitor;
    let width = dimensions.right.abs_diff(dimensions.left);
    #[expect(clippy::cast_precision_loss)]
    let width = Length::new::<pixel>(width as f32);
    let height = dimensions.top.abs_diff(dimensions.bottom);
    #[expect(clippy::cast_precision_loss)]
    let height = Length::new::<pixel>(height as f32);
    let primary = (monitor_info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0;
    let monitor = Monitor::new(width, height, refresh_rate, primary);
    let monitors = unsafe { &mut *(data.0 as *mut Vec<Monitor>) };
    monitors.push(monitor);
    TRUE
}

fn find_monitor_refresh_rate(device_name: PCWSTR) -> Option<Frequency> {
    let size = narrow_unsigned!(size_of::<DEVMODEW>() => u16);
    let mut mode = DEVMODEW {
        dmSize: size,
        ..DEVMODEW::default()
    };
    let success =
        unsafe { EnumDisplaySettingsW(device_name, ENUM_CURRENT_SETTINGS, &raw mut mode) };
    if !success.as_bool() {
        return None;
    }
    let frequency = mode.dmDisplayFrequency;
    if frequency == 0 || frequency == 1 {
        return None;
    }
    let frequency = Frequency::new::<hertz>(frequency);
    Some(frequency)
}

pub fn set_dpi_awareness() -> Result<()> {
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) }
}
