#[expect(clippy::cast_possible_truncation)]
pub const fn size_of_u32<T>() -> u32 {
    const {
        assert!(size_of::<T>() <= u32::MAX as usize);
    }
    size_of::<T>() as u32
}

#[expect(clippy::cast_possible_truncation)]
pub const fn size_of_u16<T>() -> u16 {
    const {
        assert!(size_of::<T>() <= u16::MAX as usize);
    }
    size_of::<T>() as u16
}
