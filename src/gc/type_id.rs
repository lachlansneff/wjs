
/// This ensures that the TypeIds never conflict, even when created by derive.
#[doc(hidden)]
pub trait CheckTypeId<const N: u32> {}

pub unsafe trait CompactTypeId {
    fn compact_type_id() -> u32;
}

macro_rules! impl_hardcoded_type_id {
    ($( $t:ty => $n:literal ),*) => {
        $(
            unsafe impl CompactTypeId for $t {
                fn compact_type_id() -> u32 {
                    $n
                }
            }
            impl CheckTypeId<$n> for () {}
        )*
    };
}

impl_hardcoded_type_id! {
    f64 => 0
}
