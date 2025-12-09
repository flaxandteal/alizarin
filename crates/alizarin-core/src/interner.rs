//! String interner for efficient UUID handling
//!
//! UUIDs are 36-character strings. Comparing and hashing them repeatedly
//! is expensive. This module wraps the `lasso` crate to provide efficient
//! string interning for use throughout Alizarin.
//!
//! The `Spur` key type from lasso is a 32-bit integer that can be used
//! as a HashMap key instead of the full UUID string.

pub use lasso::{Rodeo, Spur, RodeoReader, RodeoResolver};

#[cfg(feature = "multi-threaded")]
pub use lasso::ThreadedRodeo;

/// Type alias for the interned ID type
pub type InternedId = Spur;

/// Type alias for the single-threaded interner (for WASM)
pub type Interner = Rodeo;

/// Type alias for the thread-safe interner (for NAPI/Python)
/// Only available with the `multi-threaded` feature
#[cfg(feature = "multi-threaded")]
pub type ThreadSafeInterner = ThreadedRodeo;

/// Extension trait for convenient interning of Option<String>
pub trait InternerExt {
    /// Intern a string reference
    fn intern_str(&mut self, s: &str) -> InternedId;

    /// Intern an Option<&String>, returning a sentinel for None/empty
    fn intern_option(&mut self, s: Option<&String>) -> Option<InternedId>;

    /// Intern an Option<String>, returning a sentinel for None/empty
    fn intern_option_owned(&mut self, s: Option<String>) -> Option<InternedId>;
}

impl InternerExt for Rodeo {
    #[inline]
    fn intern_str(&mut self, s: &str) -> InternedId {
        self.get_or_intern(s)
    }

    #[inline]
    fn intern_option(&mut self, s: Option<&String>) -> Option<InternedId> {
        match s {
            Some(s) if !s.is_empty() => Some(self.get_or_intern(s)),
            _ => None,
        }
    }

    #[inline]
    fn intern_option_owned(&mut self, s: Option<String>) -> Option<InternedId> {
        match s {
            Some(s) if !s.is_empty() => Some(self.get_or_intern(s)),
            _ => None,
        }
    }
}

#[cfg(feature = "multi-threaded")]
impl InternerExt for ThreadedRodeo {
    #[inline]
    fn intern_str(&mut self, s: &str) -> InternedId {
        self.get_or_intern(s)
    }

    #[inline]
    fn intern_option(&mut self, s: Option<&String>) -> Option<InternedId> {
        match s {
            Some(s) if !s.is_empty() => Some(self.get_or_intern(s)),
            _ => None,
        }
    }

    #[inline]
    fn intern_option_owned(&mut self, s: Option<String>) -> Option<InternedId> {
        match s {
            Some(s) if !s.is_empty() => Some(self.get_or_intern(s)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_basic() {
        let mut interner = Rodeo::default();

        let id1 = interner.get_or_intern("hello");
        let id2 = interner.get_or_intern("world");
        let id3 = interner.get_or_intern("hello"); // Same as id1

        assert_eq!(id1, id3);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_resolve() {
        let mut interner = Rodeo::default();

        let id = interner.get_or_intern("test-uuid-1234");
        assert_eq!(interner.resolve(&id), "test-uuid-1234");
    }

    #[test]
    fn test_intern_option() {
        let mut interner = Rodeo::default();

        let some_str = Some(String::from("hello"));
        let none_str: Option<&String> = None;
        let empty_str = Some(String::from(""));

        assert!(interner.intern_option(some_str.as_ref()).is_some());
        assert!(interner.intern_option(none_str).is_none());
        assert!(interner.intern_option(empty_str.as_ref()).is_none());
    }
}
