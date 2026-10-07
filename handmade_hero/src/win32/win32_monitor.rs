use handmade_hero_interface::monitor::Monitor;
use handmade_hero_interface::units::si::length::{Length, pixel};
use handmade_hero_interface::{narrow_unsigned, units::si::frequency::Frequency};
use uom::si::frequency::hertz;
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
    DEVMODEW, ENUM_CURRENT_SETTINGS, EnumDisplayMonitors, EnumDisplaySettingsW, GetMonitorInfoW,
    HDC, HMONITOR, MONITOR_DEFAULTTONEAREST, MONITORINFOEXW, MonitorFromWindow,
};
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::WindowsAndMessaging::MONITORINFOF_PRIMARY;
use windows::core::{BOOL, PCWSTR, Result};

/// A monitor as the game sees it, plus the handle Windows uses to refer to it.
///
/// The handle stays valid only until the monitors are reconfigured, so it never leaves the
/// platform layer.
#[derive(Debug)]
pub struct Win32Monitor {
    monitor: Monitor,
    handle: HMONITOR,
}

impl Win32Monitor {
    #[inline]
    #[must_use]
    pub fn monitor(&self) -> &Monitor {
        &self.monitor
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
        refresh_rate,
        primary,
        current,
    );
    let win32_monitor = Win32Monitor {
        monitor,
        handle: next,
    };
    context.monitors.push(win32_monitor);
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

pub fn set_dpi_awareness() -> Result<()> {
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) }
}
