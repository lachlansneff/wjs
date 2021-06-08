// mod shadowstack;

use std::{cell::UnsafeCell, marker::PhantomData, mem, ptr::{self, DynMetadata, NonNull}};
use super::{Trace, CompactTypeId};

pub struct GcFlags {

}

#[repr(C)]
pub struct GcHeader<'gc, T> {
    vtable: DynMetadata<dyn Trace + 'gc>,
    type_id: u32,
    // _flags: u32,
    value: UnsafeCell<T>,
}

impl<'gc, T> GcHeader<'gc, T> {
    /// Size of header in bytes, including the value.
    pub fn size_of(&self) -> usize {
        mem::size_of::<GcHeader<()>>() + self.vtable.size_of()
    }
}

/// Always aligned to a word.
#[derive(Debug)]
#[repr(transparent)]
pub struct GcPtr<'gc, T> {
    inner: NonNull<GcHeader<'gc, T>>,
    _marker: PhantomData<&'gc mut T>,
}

impl<'gc, T> Clone for GcPtr<'gc, T> {
    fn clone(&self) -> Self { *self }
}
impl<'gc, T> Copy for GcPtr<'gc, T> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IncorrectGcType;

impl<'gc, T: Trace + CompactTypeId + 'gc> GcPtr<'gc, T> {
    pub(super) unsafe fn new_place(dest: *mut GcHeader<'gc, T>, value: T) -> Self {
        // addr_of_mut!((*dest).type_id).write(T::compact_type_id());
        // addr_of_mut!((*dest).value).write(UnsafeCell::new(value));
        // addr_of_mut!((*dest).vtable).write(ptr::metadata(addr_of!((*dest).value) as *const T as *const dyn Trace));
        let vtable = ptr::metadata(&value as &dyn Trace);
        dest.write(GcHeader {
            vtable,
            type_id: T::compact_type_id(),
            // _flags: 0,
            value: UnsafeCell::new(value),
        });

        Self {
            inner: NonNull::new_unchecked(dest),
            _marker: PhantomData,
        }
    }
}

impl<'gc, T> GcPtr<'gc, T> {
    pub fn untype(self) -> GcPtr<'gc, ()> {
        unsafe {
            mem::transmute(self)
        }
    }

    pub fn as_header_ptr(self) -> NonNull<GcHeader<'gc, ()>> {
        Self::untype(self).inner
    }

    pub fn as_value_ptr(self) -> NonNull<T> {
        unsafe {
            let value_ptr: *mut u8 = self.inner.as_ref().value.get() as _;
            let value_ptr = value_ptr.add(value_ptr.align_offset(self.inner.as_ref().vtable.align_of()));
            NonNull::new_unchecked(value_ptr as *mut T)
        }
    }

    pub unsafe fn as_mut_dyn_trace(self) -> &'gc mut dyn Trace {
        &mut *ptr::from_raw_parts_mut(
            self.as_value_ptr().as_ptr() as _,
            self.inner.as_ref().vtable,
        )
    }
}

impl<'gc> GcPtr<'gc, ()> {
    pub unsafe fn from_non_null(inner: NonNull<GcHeader<'gc, ()>>) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    pub fn try_type<T: CompactTypeId>(self) -> Result<GcPtr<'gc, T>, IncorrectGcType> {
        if unsafe { self.inner.as_ref().type_id } == T::compact_type_id() {
            Ok(unsafe {
                mem::transmute(self)
            })
        } else {
            Err(IncorrectGcType)
        }
    }
}

unsafe impl<'gc, T> Trace for GcPtr<'gc, T> {
    unsafe fn trace(&mut self) {
        self.as_mut_dyn_trace().trace();
    }
    unsafe fn finalize(&mut self) {
        let this = self.as_mut_dyn_trace();
        this.finalize();
        ptr::drop_in_place(this);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    use crate::gc::{Gc, GcPtr, CompactTypeId, Trace};

    #[derive(CompactTypeId, Trace)]
    pub struct TestGcItem {
        x: usize
    }

    #[test]
    fn test_alloc() {
        let gc = Gc::new();

        let _ptr = gc.alloc(TestGcItem {
            x: 42,
        });
    }

    #[test]
    fn test_alloc_many() {
        let gc = Gc::new();

        for x in 0..10_000 {
            let _ptr = gc.alloc(TestGcItem {
                x,
            });
        }
    }

    #[test]
    fn test_new_and_cast() {
        let gc = Gc::new();

        let a = gc.alloc(TestGcItem {
            x: 42,
        }).untype();

        let _b: GcPtr<TestGcItem> = a.try_type().unwrap();
    }

    #[test]
    fn test_page_iter() {
        let gc = Gc::new();

        for x in 0..10_000 {
            let _ptr = gc.alloc(TestGcItem {
                x,
            });
        }

        for page in gc.page_iter() {
            for ptr in unsafe { &mut *page }.gc_iter() {
                let _: GcPtr<TestGcItem> = ptr.try_type().unwrap();
            }
        }
    }

    #[test]
    fn test_gc_drop() {
        #[derive(CompactTypeId)]
        struct Foo(Arc<AtomicBool>);
        
        unsafe impl Trace for Foo {
            unsafe fn trace(&mut self) {}
            unsafe fn finalize(&mut self) {}
        }

        impl Drop for Foo {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let a = Arc::new(AtomicBool::new(false));
        {
            let gc = Gc::new();
            let _ptr = gc.alloc(Foo(a.clone()));
        }

        assert_eq!(a.load(Ordering::SeqCst), true);
    }

    #[test]
    fn test_nested_gc_ptr() {
        #[derive(Trace, CompactTypeId)]
        struct Foo<'gc>(Option<GcPtr<'gc, Self>>);

        let gc = Gc::new();
        let a = gc.alloc(Foo(None));
        let b = gc.alloc(Foo(Some(a)));

        drop(b);
    } 
}
