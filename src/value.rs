use std::{convert::TryFrom, marker::PhantomData, ptr::NonNull};

use crate::gc::GcPtr;

macro_rules! cfg_bits {
    (32 => $e32:expr, 64 => $e64:expr $(,)?) => {{
        cfg_if::cfg_if! {
            if #[cfg(target_pointer_width = "32")] {
                { $e32 }
            } else if #[cfg(target_pointer_width = "64")] {
                { $e64 }
            } else {
                compile_error!("target must be 32 or 64 bits")
            }
        }
    }};
}

pub const SMI_BITS: u8 = cfg_bits! {
    32 => 31,
    64 => 32,
};

pub const SMI_MIN: i32 = (((-1i32) as u32) << (SMI_BITS - 1)) as i32;
pub const SMI_MAX: i32 = -(SMI_MIN + 1);
const SMI_SHIFT: u8 = cfg_bits! {
    32 => 1,
    64 => 32,
};

/// This relies on the bottom 2 bits of every
/// garbage collected pointer containing zeros.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct JsValue<'gc>(usize, PhantomData<&'gc mut ()>);

impl<'gc> JsValue<'gc> {
    #[inline]
    pub fn is_smi(self) -> bool {
        self.0 & 0b1 == 0
    }

    #[inline]
    pub fn is_pointer(self) -> bool {
        !self.is_smi()
    }

    #[inline]
    pub fn expect_smi(self) -> Option<i32> {
        self.is_smi().then(|| ((self.0 as isize) >> SMI_SHIFT) as i32)
    }

    #[inline]
    pub fn expect_gc_ptr(self) -> Option<GcPtr<'gc, ()>> {
        self.is_pointer().then(|| unsafe {
            GcPtr::from_non_null(NonNull::new_unchecked((self.0 & !0b1) as *mut _))
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidSmi;

impl<'gc> TryFrom<i32> for JsValue<'gc> {
    type Error = InvalidSmi;

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        if v >= SMI_MIN && v <= SMI_MAX {
            Ok(Self((v as usize) << SMI_SHIFT, PhantomData))
        } else {
            Err(InvalidSmi)
        }
    }
}

impl<'gc, T> From<GcPtr<'gc, T>> for JsValue<'gc> {
    fn from(ptr: GcPtr<'gc, T>) -> Self {
        Self(GcPtr::as_header_ptr(ptr).as_ptr() as usize | 0b1, PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use crate::gc::{Gc, GcPtr};
    use super::JsValue;

    #[test]
    fn test_smi() {
        let v: JsValue = (-1).try_into().unwrap();
        assert_eq!(v.expect_smi(), Some(-1));
    }

    #[test]
    fn test_gc_ptr() {
        let gc = Gc::new();

        let a = gc.alloc(5.0f64);
        let v: JsValue = a.into();
        let u: GcPtr<()> = v.expect_gc_ptr().expect("wasn't able to decode `GcPtr<()>`");
        let _: GcPtr<f64> = GcPtr::try_type(u).expect("failed to cast back to `GcPtr<f64>`");

        // assert_eq!(*b, 5.0);
    }
}

