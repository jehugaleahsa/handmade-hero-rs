use handmade_hero_interface::application_error::{ApplicationError, Result};
use handmade_hero_interface::monitor::Monitor;
use handmade_hero_interface::monitor_mode::MonitorMode;
use handmade_hero_interface::narrow_unsigned;
use uom::si::frequency::hertz;
use uom::si::u32::Frequency;
use windows::Win32::Devices::Display::{
    DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_DEVICE_INFO_TYPE, DISPLAYCONFIG_MODE_INFO,
    DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME,
    DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QDC_ONLY_ACTIVE_PATHS,
    QueryDisplayConfig,
};
use windows::Win32::Foundation::{
    ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, HWND, LPARAM, LUID, RECT, TRUE,
};
use windows::Win32::Graphics::Gdi::{
    CDS_FULLSCREEN, CDS_TYPE, ChangeDisplaySettingsExW, DEVMODEW, DISP_CHANGE_SUCCESSFUL,
    DM_BITSPERPEL, DM_DISPLAYFREQUENCY, DM_INTERLACED, DM_PELSHEIGHT, DM_PELSWIDTH,
    ENUM_CURRENT_SETTINGS, ENUM_DISPLAY_SETTINGS_MODE, EnumDisplayMonitors, EnumDisplaySettingsW,
    GetMonitorInfoW, HDC, HMONITOR, MONITOR_DEFAULTTONEAREST, MONITORINFOEXW, MonitorFromWindow,
};
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::WindowsAndMessaging::MONITORINFOF_PRIMARY;
use windows::core::{BOOL, PCWSTR, Result as Win32Result};

/// A monitor as the game sees it, plus the handle Windows uses to refer to it.
///
/// The handle stays valid only until the monitors are reconfigured, so it never leaves the
/// platform layer.
#[derive(Debug)]
pub struct Win32Monitor {
    monitor: Monitor,
    handle: HMONITOR,
    /// The GDI device name, such as `\\.\DISPLAY1`, that display mode functions expect.
    device_name: [u16; 32],
    /// The monitor's full area on the desktop, taskbar included, in physical pixels.
    bounds: RECT,
}

impl Win32Monitor {
    #[inline]
    #[must_use]
    pub fn monitor(&self) -> &Monitor {
        &self.monitor
    }

    #[inline]
    #[must_use]
    pub fn bounds(&self) -> RECT {
        self.bounds
    }

    #[inline]
    #[must_use]
    pub fn handle(&self) -> HMONITOR {
        self.handle
    }
}

struct MonitorNames {
    device_name: String,
    display_name: String,
    identifier: String,
}

struct MonitorContext {
    current: HMONITOR,
    names: Vec<MonitorNames>,
    monitors: Vec<Win32Monitor>,
}

pub fn find_current_monitor(window_handle: HWND) -> HMONITOR {
    unsafe { MonitorFromWindow(window_handle, MONITOR_DEFAULTTONEAREST) }
}

pub fn find_monitors(window_handle: HWND) -> Vec<Win32Monitor> {
    let mut context = MonitorContext {
        current: find_current_monitor(window_handle),
        names: find_monitor_names(),
        monitors: Vec::new(),
    };
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(add_monitor),
            LPARAM(&raw mut context as isize),
        );
    }
    context.monitors
}

extern "system" fn add_monitor(next: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
    let mut monitor_info = MONITORINFOEXW::default();
    monitor_info.monitorInfo.cbSize = narrow_unsigned!(size_of::<MONITORINFOEXW>() => u32);
    let monitor_info_result = unsafe { GetMonitorInfoW(next, &raw mut monitor_info.monitorInfo) };
    if !monitor_info_result.as_bool() {
        return TRUE;
    }
    let device_name = PCWSTR(monitor_info.szDevice.as_ptr());
    let Some(current_mode) = find_current_display_mode(device_name) else {
        return TRUE;
    };
    let modes = find_display_modes(device_name, &current_mode)
        .iter()
        .map(|mode| {
            let current = is_same_mode(mode, &current_mode);
            MonitorMode::new(
                mode.dmPelsWidth,
                mode.dmPelsHeight,
                Frequency::new::<hertz>(mode.dmDisplayFrequency),
                current,
            )
        })
        .collect();

    let dimensions = &monitor_info.monitorInfo.rcMonitor;
    let width = dimensions.right.abs_diff(dimensions.left);
    let height = dimensions.top.abs_diff(dimensions.bottom);
    let primary = (monitor_info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0;
    let context = unsafe { &mut *(data.0 as *mut MonitorContext) };
    let current = next == context.current;

    let device_name = string_from_wide(&monitor_info.szDevice);
    let names = context
        .names
        .iter()
        .find(|names| names.device_name == device_name);
    let display_name = names
        .map(|names| names.display_name.clone())
        .filter(|display_name| !display_name.is_empty())
        .unwrap_or_else(|| device_name.clone());
    let identifier = names
        .map(|names| names.identifier.clone())
        .filter(|identifier| !identifier.is_empty())
        .unwrap_or(device_name);

    let monitor = Monitor::new(
        display_name,
        identifier,
        width,
        height,
        modes,
        primary,
        current,
    );
    let win32_monitor = Win32Monitor {
        monitor,
        handle: next,
        device_name: monitor_info.szDevice,
        bounds: monitor_info.monitorInfo.rcMonitor,
    };
    context.monitors.push(win32_monitor);
    TRUE
}

#[expect(dead_code, reason = "Called once the game can go fullscreen")]
pub fn set_display_mode(monitor: &Win32Monitor, mode: &MonitorMode) -> Result<()> {
    let device_name = PCWSTR(monitor.device_name.as_ptr());
    let current_mode = find_current_display_mode(device_name);
    let Some(current_mode) = current_mode else {
        return Err(ApplicationError::new(
            "Could not read the monitor's current display mode",
        ));
    };
    let display_modes = find_display_modes(device_name, &current_mode);
    let display_modes: Vec<DEVMODEW> = display_modes
        .into_iter()
        .filter(|display_mode| matches_monitor_mode(display_mode, mode))
        .collect();
    for mut display_mode in display_modes {
        // Only the listed fields are applied, so the scaling and orientation stay as they are.
        display_mode.dmFields = DM_PELSWIDTH | DM_PELSHEIGHT | DM_BITSPERPEL | DM_DISPLAYFREQUENCY;
        let result = unsafe {
            ChangeDisplaySettingsExW(
                device_name,
                Some(&raw const display_mode),
                None,
                CDS_FULLSCREEN,
                None,
            )
        };
        if result == DISP_CHANGE_SUCCESSFUL {
            return Ok(());
        }
    }
    Err(ApplicationError::new(
        "Could not set the requested display mode",
    ))
}

#[expect(dead_code, reason = "Called once the game can leave fullscreen")]
pub fn restore_display_mode(monitor: &Win32Monitor) -> Result<()> {
    let device_name = PCWSTR(monitor.device_name.as_ptr());
    // Passing no mode tells Windows to switch back to the mode saved in its settings.
    let result =
        unsafe { ChangeDisplaySettingsExW(device_name, None, None, CDS_TYPE::default(), None) };
    if result != DISP_CHANGE_SUCCESSFUL {
        return Err(ApplicationError::new(
            "Windows could not restore the display mode",
        ));
    }
    Ok(())
}

fn find_current_display_mode(device_name: PCWSTR) -> Option<DEVMODEW> {
    let mut mode = new_display_mode();
    let success =
        unsafe { EnumDisplaySettingsW(device_name, ENUM_CURRENT_SETTINGS, &raw mut mode) };
    success.as_bool().then_some(mode)
}

fn find_display_modes(device_name: PCWSTR, current_mode: &DEVMODEW) -> Vec<DEVMODEW> {
    let mut modes: Vec<DEVMODEW> = Vec::new();
    for index in 0.. {
        let mut mode = new_display_mode();
        let success = unsafe {
            EnumDisplaySettingsW(
                device_name,
                ENUM_DISPLAY_SETTINGS_MODE(index),
                &raw mut mode,
            )
        };
        if !success.as_bool() {
            break;
        }
        // Windows lists a mode once per scaling setting, among other things, so the same
        // resolution and refresh rate can appear more than once.
        let is_duplicate = modes.iter().any(|existing| is_same_mode(existing, &mode));
        if !is_duplicate && is_offered_mode(&mode, current_mode) {
            modes.push(mode);
        }
    }
    modes.sort_by_key(|mode| (mode.dmPelsWidth, mode.dmPelsHeight, mode.dmDisplayFrequency));
    modes
}

fn is_same_mode(mode: &DEVMODEW, other: &DEVMODEW) -> bool {
    mode.dmPelsWidth == other.dmPelsWidth
        && mode.dmPelsHeight == other.dmPelsHeight
        && mode.dmDisplayFrequency == other.dmDisplayFrequency
}

fn matches_monitor_mode(display_mode: &DEVMODEW, mode: &MonitorMode) -> bool {
    display_mode.dmPelsWidth == mode.width_in_pixels()
        && display_mode.dmPelsHeight == mode.height_in_pixels()
        && display_mode.dmDisplayFrequency == mode.refresh_rate().get::<hertz>()
}

fn is_offered_mode(mode: &DEVMODEW, current_mode: &DEVMODEW) -> bool {
    let is_interlaced = unsafe { mode.Anonymous2.dmDisplayFlags } & DM_INTERLACED.0 != 0;
    // Windows reports 0 or 1 for "the hardware's default rate", which isn't a real rate.
    let has_refresh_rate = mode.dmDisplayFrequency > 1;
    mode.dmBitsPerPel == current_mode.dmBitsPerPel && has_refresh_rate && !is_interlaced
}
fn new_display_mode() -> DEVMODEW {
    DEVMODEW {
        dmSize: narrow_unsigned!(size_of::<DEVMODEW>() => u16),
        ..DEVMODEW::default()
    }
}

fn find_monitor_names() -> Vec<MonitorNames> {
    loop {
        let mut path_count = 0u32;
        let mut mode_count = 0u32;
        let sizes_result = unsafe {
            GetDisplayConfigBufferSizes(
                QDC_ONLY_ACTIVE_PATHS,
                &raw mut path_count,
                &raw mut mode_count,
            )
        };
        if sizes_result != ERROR_SUCCESS {
            return Vec::new();
        }

        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];
        let query_result = unsafe {
            QueryDisplayConfig(
                QDC_ONLY_ACTIVE_PATHS,
                &raw mut path_count,
                paths.as_mut_ptr(),
                &raw mut mode_count,
                modes.as_mut_ptr(),
                None,
            )
        };
        if query_result == ERROR_INSUFFICIENT_BUFFER {
            // A monitor was connected between the two calls, so the sizes are stale. Ask again.
            continue;
        }
        if query_result != ERROR_SUCCESS {
            return Vec::new();
        }

        paths.truncate(path_count as usize);
        return paths.iter().filter_map(find_path_names).collect();
    }
}

fn find_path_names(path: &DISPLAYCONFIG_PATH_INFO) -> Option<MonitorNames> {
    let mut source = DISPLAYCONFIG_SOURCE_DEVICE_NAME {
        header: device_info_header::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>(
            DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
            path.sourceInfo.adapterId,
            path.sourceInfo.id,
        ),
        ..DISPLAYCONFIG_SOURCE_DEVICE_NAME::default()
    };
    if unsafe { DisplayConfigGetDeviceInfo(&raw mut source.header) } != 0 {
        return None;
    }

    let mut target = DISPLAYCONFIG_TARGET_DEVICE_NAME {
        header: device_info_header::<DISPLAYCONFIG_TARGET_DEVICE_NAME>(
            DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
            path.targetInfo.adapterId,
            path.targetInfo.id,
        ),
        ..DISPLAYCONFIG_TARGET_DEVICE_NAME::default()
    };
    if unsafe { DisplayConfigGetDeviceInfo(&raw mut target.header) } != 0 {
        return None;
    }

    let names = MonitorNames {
        device_name: string_from_wide(&source.viewGdiDeviceName),
        display_name: string_from_wide(&target.monitorFriendlyDeviceName),
        identifier: string_from_wide(&target.monitorDevicePath),
    };
    Some(names)
}

fn device_info_header<T>(
    request: DISPLAYCONFIG_DEVICE_INFO_TYPE,
    adapter_id: LUID,
    id: u32,
) -> DISPLAYCONFIG_DEVICE_INFO_HEADER {
    DISPLAYCONFIG_DEVICE_INFO_HEADER {
        r#type: request,
        size: narrow_unsigned!(size_of::<T>() => u32),
        adapterId: adapter_id,
        id,
    }
}

fn string_from_wide(wide: &[u16]) -> String {
    let length = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..length])
}

pub fn set_dpi_awareness() -> Win32Result<()> {
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) }
}
