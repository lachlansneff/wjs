use std::ptr;

pub unsafe trait Trace {
    unsafe fn trace(&mut self);
    unsafe fn finalize(&mut self);
}

macro_rules! empty_trace_impl {
    ($($t:ty),*) => {
        $(
            unsafe impl Trace for $t {
                unsafe fn trace(&mut self) {}
                unsafe fn finalize(&mut self) {}
            } 
        )*
    };
}

empty_trace_impl!(f64, u8, i8, u16, i16, u32, i32, u64, i64, usize, isize, &'static str);

unsafe impl<T: Trace> Trace for Option<T> {
    unsafe fn trace(&mut self) {
        if let Some(x) = self {
            x.trace();
        }
    }
    unsafe fn finalize(&mut self) {
        if let Some(x) = self {
            x.finalize();
        }
    }
}

unsafe impl<T: Trace, E: Trace> Trace for Result<T, E> {
    unsafe fn trace(&mut self) {
        match self {
            Ok(ok) => ok.trace(),
            Err(err) => err.trace(),
        }
    }
    unsafe fn finalize(&mut self) {
        match self {
            Ok(ok) => ok.finalize(),
            Err(err) => err.finalize(),
        }
    }
}

unsafe impl<T: Trace + ?Sized> Trace for Box<T> {
    unsafe fn trace(&mut self) {
        T::trace(&mut *self);
    }
    unsafe fn finalize(&mut self) {
        T::finalize(&mut *self);
        unsafe { ptr::drop_in_place(self); }
    }
}

unsafe impl<T: Trace> Trace for Vec<T> {
    unsafe fn trace(&mut self) {
        for element in self {
            T::trace(element);
        }
    }
    unsafe fn finalize(&mut self) {
        for element in self.iter_mut() {
            T::finalize(element);
        }
        ptr::drop_in_place(self);
    }
}

unsafe impl<T: Trace> Trace for [T] {
    unsafe fn trace(&mut self) {
        for element in self {
            T::trace(element);
        }
    }
    unsafe fn finalize(&mut self) {
        for element in self {
            T::finalize(element);
        }
    }
}
