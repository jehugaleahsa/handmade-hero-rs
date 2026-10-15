use std::fmt::Debug;

use handmade_hero_interface::narrow_unsigned;
use windows::Win32::UI::Accessibility::{
    FILTERKEYS, SKF_CONFIRMHOTKEY, SKF_HOTKEYACTIVE, SKF_STICKYKEYSON, STICKYKEYS, TOGGLEKEYS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    FKF_CONFIRMHOTKEY, FKF_FILTERKEYSON, FKF_HOTKEYACTIVE, SPI_GETFILTERKEYS, SPI_GETSTICKYKEYS,
    SPI_GETTOGGLEKEYS, SPI_SETFILTERKEYS, SPI_SETSTICKYKEYS, SPI_SETTOGGLEKEYS,
    SYSTEM_PARAMETERS_INFO_ACTION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
    TKF_CONFIRMHOTKEY, TKF_HOTKEYACTIVE, TKF_TOGGLEKEYSON,
};

/// Keeps the Windows accessibility shortcuts, such as pressing SHIFT five times to turn on
/// Sticky Keys, from interrupting the game while it has focus.
#[derive(Debug, Default)]
pub struct Win32AccessibilityShortcuts {
    saved: Option<SavedSettings>,
}

impl Win32AccessibilityShortcuts {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Turns the shortcuts off, remembering the player's settings so they can be put back.
    pub fn suppress(&mut self) {
        if self.saved.is_some() {
            return;
        }
        let saved = SavedSettings::read();
        saved.write_without_shortcuts();
        self.saved = Some(saved);
    }

    /// Puts back the settings the player had before the shortcuts were turned off.
    pub fn restore(&mut self) {
        if let Some(saved) = self.saved.take() {
            saved.write();
        }
    }
}

impl Drop for Win32AccessibilityShortcuts {
    fn drop(&mut self) {
        self.restore();
    }
}

/// Each feature is optional because a setting Windows wouldn't report is one we leave alone.
#[derive(Debug)]
struct SavedSettings {
    sticky: Option<STICKYKEYS>,
    toggle: Option<TOGGLEKEYS>,
    filter: Option<FILTERKEYS>,
}

impl SavedSettings {
    #[inline]
    #[must_use]
    fn read() -> Self {
        Self {
            sticky: read_setting(),
            toggle: read_setting(),
            filter: read_setting(),
        }
    }

    fn write(&self) {
        write_if_present(self.sticky);
        write_if_present(self.toggle);
        write_if_present(self.filter);
    }

    fn write_without_shortcuts(&self) {
        write_if_present(self.sticky.map(AccessibilitySetting::without_shortcut));
        write_if_present(self.toggle.map(AccessibilitySetting::without_shortcut));
        write_if_present(self.filter.map(AccessibilitySetting::without_shortcut));
    }
}

/// An accessibility feature whose keyboard shortcut can be turned off through
/// `SystemParametersInfoW`.
trait AccessibilitySetting: Copy + Debug {
    const GET: SYSTEM_PARAMETERS_INFO_ACTION;
    const SET: SYSTEM_PARAMETERS_INFO_ACTION;

    /// A blank setting with its size filled in, which Windows requires before it will fill in
    /// the rest.
    #[must_use]
    fn empty() -> Self;

    /// The same setting with its shortcut off, unless the feature itself is on.
    #[must_use]
    fn without_shortcut(self) -> Self;
}

impl AccessibilitySetting for STICKYKEYS {
    const GET: SYSTEM_PARAMETERS_INFO_ACTION = SPI_GETSTICKYKEYS;
    const SET: SYSTEM_PARAMETERS_INFO_ACTION = SPI_SETSTICKYKEYS;

    #[inline]
    fn empty() -> Self {
        Self {
            cbSize: narrow_unsigned!(size_of::<Self>() => u32),
            ..Default::default()
        }
    }

    fn without_shortcut(mut self) -> Self {
        if (self.dwFlags & SKF_STICKYKEYSON).0 == 0 {
            self.dwFlags &= !(SKF_HOTKEYACTIVE | SKF_CONFIRMHOTKEY);
        }
        self
    }
}

impl AccessibilitySetting for TOGGLEKEYS {
    const GET: SYSTEM_PARAMETERS_INFO_ACTION = SPI_GETTOGGLEKEYS;
    const SET: SYSTEM_PARAMETERS_INFO_ACTION = SPI_SETTOGGLEKEYS;

    #[inline]
    fn empty() -> Self {
        Self {
            cbSize: narrow_unsigned!(size_of::<Self>() => u32),
            ..Default::default()
        }
    }

    fn without_shortcut(mut self) -> Self {
        if self.dwFlags & TKF_TOGGLEKEYSON == 0 {
            self.dwFlags &= !(TKF_HOTKEYACTIVE | TKF_CONFIRMHOTKEY);
        }
        self
    }
}

impl AccessibilitySetting for FILTERKEYS {
    const GET: SYSTEM_PARAMETERS_INFO_ACTION = SPI_GETFILTERKEYS;
    const SET: SYSTEM_PARAMETERS_INFO_ACTION = SPI_SETFILTERKEYS;

    #[inline]
    fn empty() -> Self {
        Self {
            cbSize: narrow_unsigned!(size_of::<Self>() => u32),
            ..Default::default()
        }
    }

    fn without_shortcut(mut self) -> Self {
        if self.dwFlags & FKF_FILTERKEYSON == 0 {
            self.dwFlags &= !(FKF_HOTKEYACTIVE | FKF_CONFIRMHOTKEY);
        }
        self
    }
}

fn read_setting<T: AccessibilitySetting>() -> Option<T> {
    let mut setting = T::empty();
    let result = unsafe {
        SystemParametersInfoW(
            T::GET,
            narrow_unsigned!(size_of::<T>() => u32),
            Some(std::ptr::from_mut(&mut setting).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    result.ok()?;
    Some(setting)
}

/// Errors are ignored: the worst outcome is that a shortcut stays as it was.
fn write_if_present<T: AccessibilitySetting>(setting: Option<T>) {
    let Some(mut setting) = setting else {
        return;
    };
    // No update flags, so the change lasts for this login session and is never saved to the
    // player's profile.
    unsafe {
        let _ = SystemParametersInfoW(
            T::SET,
            narrow_unsigned!(size_of::<T>() => u32),
            Some(std::ptr::from_mut(&mut setting).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
    }
}
