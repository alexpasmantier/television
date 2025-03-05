/*
MIT License

Copyright (c) 2023 - sxyazi

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
use std::{
    cell::UnsafeCell,
    fmt::{self, Display},
    mem,
    ops::Deref,
};

// Read-only cell. It's safe to use this in a static variable, but it's not safe
// to mutate it. This is useful for storing static data that is expensive to
// initialize, but is immutable once.
pub struct RoCell<T>(UnsafeCell<Option<T>>);

unsafe impl<T> Sync for RoCell<T> {}

impl<T> RoCell<T> {
    #[inline]
    pub const fn new() -> Self {
        Self(UnsafeCell::new(None))
    }

    #[inline]
    pub const fn new_const(value: T) -> Self {
        Self(UnsafeCell::new(Some(value)))
    }

    #[inline]
    pub fn init(&self, value: T) {
        debug_assert!(!self.initialized());
        unsafe {
            *self.0.get() = Some(value);
        }
    }

    #[inline]
    pub fn with<F>(&self, f: F)
    where
        F: FnOnce() -> T,
    {
        self.init(f());
    }

    #[inline]
    pub fn drop(&self) -> T {
        debug_assert!(self.initialized());
        unsafe { mem::take(&mut *self.0.get()).unwrap_unchecked() }
    }

    #[inline]
    fn initialized(&self) -> bool {
        unsafe { (*self.0.get()).is_some() }
    }
}

impl<T> Default for RoCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for RoCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        debug_assert!(self.initialized());
        unsafe { (*self.0.get()).as_ref().unwrap_unchecked() }
    }
}

impl<T> Display for RoCell<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}
