use crate::units::si::information::Information;

/// A sample type the game can view a [`SoundBufferWindow`] as.
///
/// [`SoundBufferWindow`]: crate::sound_buffer::SoundBufferWindow
///
/// # Safety
///
/// The window reinterprets raw bytes as `Self`, and the platform later reads what the game wrote
/// back out as bytes. An implementor promises the layout makes both directions sound:
///
/// - Every bit pattern of `size_of::<Self>()` bytes is a valid `Self`. Integers and floats
///   qualify. `bool`, enums, references, and the `NonZero` types don't.
/// - `Self` has no padding bytes. Writing a padded value leaves its padding uninitialized, and
///   reading those bytes back for the device would be undefined behavior. `repr(C)` with field
///   sizes that sum to the struct size is the usual way to get this; assert it at compile time.
///
/// `Copy` is required because samples are plain data the platform copies to the device.
pub unsafe trait Sample: Copy {
    #[must_use]
    fn channel_size(&self) -> Information;
    #[must_use]
    fn channel_count(&self) -> u16;
}
