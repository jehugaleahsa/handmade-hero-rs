#[macro_export]
macro_rules! narrow_unsigned {
    ($v:expr => $t:ty) => {
        const {
            let v: u128 = $v as u128;
            assert!(
                v <= <$t>::MAX as u128,
                concat!("value does not fit in ", stringify!($t))
            );
            #[allow(clippy::cast_possible_truncation)]
            let result = v as $t;
            result
        }
    };
}
