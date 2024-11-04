use parking_lot::Mutex;
use std::ops::DerefMut;

pub mod matcher;

pub struct LazyMutex<T> {
    inner: Mutex<Option<T>>,
    init: fn() -> T,
}

impl<T> LazyMutex<T> {
    pub const fn new(init: fn() -> T) -> Self {
        Self {
            inner: Mutex::new(None),
            init,
        }
    }

    pub fn lock(&self) -> impl DerefMut<Target = T> + '_ {
        parking_lot::MutexGuard::map(self.inner.lock(), |val| {
            val.get_or_insert_with(self.init)
        })
    }
}

pub static MATCHER: LazyMutex<nucleo::Matcher> =
    LazyMutex::new(nucleo::Matcher::default);
