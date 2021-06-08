// use std::{cell::{Cell, UnsafeCell}, ops::{Deref, DerefMut}};
// use crate::gc::GcPtr;

// pub struct ShadowStack {
//     head: Cell<*mut ShadowStackEntry<'a, ()>>,
// }

// impl ShadowStack {
//     pub fn head(&self) -> Option<&'a ShadowStackEntry<'a, ()>> {
//         self.head.get()
//     }

//     pub unsafe fn set_head(&self, entry: &'a ShadowStackEntry<'a, ()>) {
//         self.head.set(Some(entry))
//     }

//     pub fn iter(&self) -> StackIter<'a> {
//         StackIter {
//             current: self.head.get(),
//         }
//     }
// }

// pub struct StackIter<'a> {
//     current: Option<&'a ShadowStackEntry<'a, ()>>,
// }

// impl<'a> Iterator for StackIter<'a> {
//     type Item = &'a ShadowStackEntry<'a, ()>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let out = self.current;

//         if let Some(entry) = out {
//             self.current = entry.prev.get();
//         }

//         out
//     }
// }

// pub struct ShadowStackEntry<'a, T> {
//     stack: &'a ShadowStack<'a>,
//     prev: Cell<Option<&'a ShadowStackEntry<'a, ()>>>,
//     ptr: GcPtr<T>,
// }

// impl<'a, T> ShadowStackEntry<'a, T> {
//     pub fn create(stack: &'a ShadowStack<'a>, prev: Option<&'a ShadowStackEntry<'a, ()>>, ptr: GcPtr<T>) -> Self {
//         Self {
//             stack,
//             prev: Cell::new(prev),
//             ptr,
//         }
//     }

//     pub fn untype(&'a self) -> &'a ShadowStackEntry<'a, ()> {
//         unsafe {
//             &*(self as *const _ as *const ShadowStackEntry<()>)
//         }
//     }
// }

// impl<'a, T> Drop for ShadowStackEntry<'a, T> {
//     fn drop(&mut self) {
//         self.stack.head.set(self.prev.get());
//     }
// }

// impl<'a> ShadowStack<'a> {
//     pub fn new() -> Self {
//         Self {
//             head: Cell::new(None),
//         }
//     }
// }

// pub struct Rooted<'a, T> {
//     entry: &'a ShadowStackEntry<'a, T>,
// }

// impl<T> Deref for Rooted<'_, T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         unsafe {
//             &*self.entry.ptr.as_value_ptr().as_ptr()
//         }
//     }
// }

// impl<T> DerefMut for Rooted<'_, T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         unsafe {
//             &mut *self.entry.ptr.as_value_ptr().as_ptr()
//         }
//     }
// }

// macro_rules! root {
//     ($stack:expr => let $name:ident = $ptr:expr) => {
//         let stack: &ShadowStack = &$stack;
//         let ptr: GcPtr<_> = $ptr;
//         let $name = ShadowStackEntry::create(stack, stack.head(), ptr);
//         let $name = &$name;

//         unsafe {
//             stack.set_head($name.untype());
//         }

//         let $name = Rooted { entry: $name };
//     };
//     ($stack:expr => let mut $name:ident = $ptr:expr) => {
//         let stack: &ShadowStack = &$stack;
//         let ptr: GcPtr<_> = $ptr;
//         let $name = ShadowStackEntry::create(stack, stack.head(), ptr);
//         let $name = &$name;

//         unsafe {
//             stack.set_head($name.untype());
//         }

//         let mut $name = Rooted { entry: $name };
//     };
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_rooting() {
//         let shadow_stack = ShadowStack::new();

//         root!(shadow_stack => let a = GcPtr::new(5.0f64));
//         assert_eq!(*a, 5.0);
//     }

//     #[test]
//     fn test_stack_iter() {
//         let stack = ShadowStack::new();

//         root!(stack => let a = GcPtr::new(5.0f64));
//         root!(stack => let b = GcPtr::new(-1.0f64));

//         assert_eq!(*a, 5.0);
//         assert_eq!(*b, -1.0);

//         assert_eq!(stack.iter().count(), 2);

//         // drop(a);
//     }
// }