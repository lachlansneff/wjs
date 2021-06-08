
mod gc_ptr;
mod trace;
mod type_id;

use std::{cell::Cell, marker::PhantomData, mem::{self, MaybeUninit}, ops::Range, ptr::{self, NonNull, addr_of_mut}};


pub use gc_ptr::{GcHeader, GcPtr};
pub use trace::Trace;
pub use type_id::{CompactTypeId, CheckTypeId};
pub use wjs_derive::{Trace, CompactTypeId};


pub const PAGE_SIZE: usize = 65_536;
pub const PAGE_USABLE_SIZE: usize = PAGE_SIZE - mem::size_of::<PageMetadata>();

#[repr(C)]
struct PageMetadata {
    next: Option<*mut Page>,
    bump: usize,
}

#[repr(C, align(65_536))]
#[repr(C)]
pub struct Page {
    metadata: PageMetadata,
    data: [MaybeUninit<u8>; PAGE_USABLE_SIZE],
}

/// The Gc "context".
pub struct Gc {
    head: Cell<*mut Page>,
}

fn new_page() -> Box<Page> {
    let mut b: Box<MaybeUninit<Page>> = Box::new_uninit();
    let ptr = (*b).as_mut_ptr();
    
    unsafe {
        addr_of_mut!((*ptr).metadata).write(PageMetadata {
            next: None,
            bump: 0,
        });
        b.assume_init()
    }
}

impl Gc {
    pub fn new() -> Self {
        Self {
            head: Cell::new(Box::leak(new_page())),
        }
    }

    pub fn alloc<'gc, T: Trace + CompactTypeId>(&'gc self, value: T) -> GcPtr<'gc, T> {
        assert!(mem::size_of::<T>() <= PAGE_USABLE_SIZE);

        let page = unsafe { &mut *self.head.get() };

        let bumped = page.metadata.bump + mem::size_of::<GcHeader<T>>();

        if bumped < PAGE_USABLE_SIZE {
            unsafe {
                let ptr = page.data.as_mut_ptr().add(page.metadata.bump);
                page.metadata.bump = bumped;
                GcPtr::new_place(ptr as *mut _, value)
            }
        } else {
            let mut new_page = Box::leak(new_page());
            new_page.metadata.next = Some(page);
            self.head.set(new_page);
            self.alloc(value)
        }
    }

    pub fn page_iter(&self) -> PageIter {
        PageIter {
            current: Some(self.head.get()),
            _marker: PhantomData,
        }
    }
}

impl Drop for Gc {
    fn drop(&mut self) {
        for page in self.page_iter() {
            for mut ptr in unsafe { &mut *page }.gc_iter() {
                unsafe { ptr.finalize() };
            }

            drop(unsafe { Box::from_raw(page) });
        }
    }
}

impl Page {
    pub fn gc_iter(&mut self) -> GcIter {
        let Range { start, end } = self.data[..self.metadata.bump].as_mut_ptr_range();
        GcIter {
            start: start as _,
            end: end as _,
            _marker: PhantomData,
        }
    }
}

pub struct GcIter<'a> {
    start: *mut u8,
    end: *mut u8,
    _marker: PhantomData<&'a mut ()>,
}

impl<'a> Iterator for GcIter<'a> {
    type Item = GcPtr<'a, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            debug_assert_eq!(self.start, self.end);
            return None;
        }

        let header = self.start as *mut GcHeader<()>;
        
        let ptr = unsafe {
            self.start = self.start.add((*header).size_of());
            GcPtr::from_non_null(NonNull::new_unchecked(header))
        };

        Some(ptr)
    }
}

pub struct PageIter<'a> {
    current: Option<*mut Page>,
    _marker: PhantomData<&'a mut ()>,
}

impl<'a> Iterator for PageIter<'a> {
    type Item = *mut Page;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(page) = self.current.take() {
            self.current = unsafe { (&*page).metadata.next };

            Some(page)
        } else {
            None
        }
    }
}
