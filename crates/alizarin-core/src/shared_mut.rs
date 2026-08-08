// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(target_arch = "wasm32")]
mod imp {
    use std::cell::{Ref, RefCell, RefMut};
    use std::convert::Infallible;
    use std::rc::Rc;

    #[derive(Debug, Default, Clone)]
    pub struct SharedMut<T>(Rc<RefCell<T>>);

    impl<T> SharedMut<T> {
        pub fn new(value: T) -> Self {
            SharedMut(Rc::new(RefCell::new(value)))
        }

        pub fn read(&self) -> Result<Ref<'_, T>, Infallible> {
            Ok(self.0.borrow())
        }

        pub fn write(&self) -> Result<RefMut<'_, T>, Infallible> {
            Ok(self.0.borrow_mut())
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

    #[derive(Debug, Default, Clone)]
    pub struct SharedMut<T>(Arc<Mutex<T>>);

    impl<T> SharedMut<T> {
        pub fn new(value: T) -> Self {
            SharedMut(Arc::new(Mutex::new(value)))
        }

        pub fn read(&self) -> Result<MutexGuard<'_, T>, PoisonError<MutexGuard<'_, T>>> {
            self.0.lock()
        }

        pub fn write(&self) -> Result<MutexGuard<'_, T>, PoisonError<MutexGuard<'_, T>>> {
            self.0.lock()
        }
    }
}

pub use imp::SharedMut;
