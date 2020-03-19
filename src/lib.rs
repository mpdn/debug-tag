
//! A library for creating debug-only tags to track and check where values are used.
//! 
//! Useful for asserting that two values stem from the same origin. For example, an operation on two
//! nodes may only be valid if the nodes are in the same graph. `DebugTag` can be used to check for
//! this in tests and when debugging.
//! 
//! See the `DebugTag` type for more details.
//! 
//! # Motivation
//! 
//! When developing Rust, we often use *indices* to refer to other values instead of using
//! *pointers*. Using indices allows us to "point" to a value without borrowing it and can basically
//! be seen as bypassing the borrow checker in a restricted way.
//! 
//! One of the issues of using indicies, is that it is not connected at compile time to the
//! container storing the value. An index used with an incorrect container might panic, or worse,
//! return garbage values.
//! 
//! We need a way to "tag" values such that we can determine if an index is used with the correct
//! container.
//! 
//! # Example
//! 
//! ```
//! use debug_tag::DebugTag;
//! 
//! /// An example vec-like structure that allows pushing values and getting values by index.
//! #[derive(Default)]
//! struct Slab<T>{
//!     items: Vec<T>,
//!     tag: DebugTag,
//! }
//! 
//! /// An index into a value on a slab.
//! #[derive(Copy, Clone)]
//! struct Index {
//!     index: usize,
//!     tag: DebugTag,
//! }
//! 
//! impl<T> Slab<T> {
//!     /// Pushes a new value onto the slab.
//!     fn push(&mut self, item: T) -> Index {
//!         let index = self.items.len();
//!         self.items.push(item);
//!         Index {
//!             index,
//!             tag: self.tag
//!         }
//!     }
//! 
//!     /// Gets a value in this `Slab`. If the index is not from this slab, either panics or
//!     /// returns an arbitrary value.
//!     fn get(&self, index: Index) -> &T {
//!         assert_eq!(self.tag, index.tag, "Index must stem from this slab");
//!         &self.items[index.index]
//!     }
//! }
//! 
//! let mut slab_a = Slab::default();
//! let ix = slab_a.push(42);
//! 
//! assert_eq!(slab_a.get(ix), &42);
//! 
//! let mut slab_b = Slab::default();
//! slab_b.push(1337);
//! 
//! // Panics due to the tags being checked:
//! //   assert_eq!(slab_b.get(ix), &1337);
//! ``` 
//! 
//! Without `DebugTag`s, the last line above would just return 1337 - a "garbage value" since `ix`
//! stems from a different `Slab`.

#[cfg(debug_assertions)]
mod checked {
    use std::cell::Cell;
    use std::sync::atomic::{AtomicU32, Ordering};

    // The increment to global every time we fetch a local tag offset. This is equal to
    // 2**32 * (1 - 1/(golden ratio)), which ends up distributing offsets well for an arbitrary
    // number of local threads.
    const INCREMENT: u32 = 1_640_531_527;

    static GLOBAL: AtomicU32 = AtomicU32::new(INCREMENT);

    thread_local! {
        static LOCAL: Cell<u32> = Cell::new(GLOBAL.fetch_add(INCREMENT, Ordering::SeqCst));
    }

    pub fn next() -> u32 {
        LOCAL.with(|local| {
            let old = local.get();
            local.set(old.wrapping_add(1));
            old
        })
    }
}

/// A value that guarentees that if two `DebugTag`s are not equal, then they are *not* clones.
/// 
/// This can be used to tag information during debug, such that the use of a value can be tracked
/// and checked. For example, you can use this to ensure (in debug) that a value returned by a data
/// structure is only used with the instance that returned the value.
/// 
/// This tagging is only done if `debug_assertions` is set. If `debug_assertions` is not set, then
/// all `DebugTags` are equal. Even if `debug_assertions` is set, two `DebugTag`s that are not
/// clones can still be equal. This is unlikely, however.
/// 
/// Therefore, functionality should not directly depend on the equality these tags but only use them
/// for additional sanity checks.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct DebugTag(
    #[cfg(debug_assertions)]
    u32,
);

impl Default for DebugTag {
    fn default() -> DebugTag {
        #[cfg(debug_assertions)]
        let tag = DebugTag(checked::next());

        #[cfg(not(debug_assertions))]
        let tag = DebugTag();

        tag
    }
}

impl DebugTag {
    /// Creates a new `DebugTag`
    pub fn new() -> DebugTag {
        DebugTag::default()
    }

    /// Create a new tag with the specified value.
    /// 
    /// Prefer using `new` instead, which will generate a value. Use this only in cases where that 
    /// is not possible, like when creating a const debug tag.
    /// 
    /// The tag value should be a randomly chosen constant.
    pub const fn from(_tag: u32) -> DebugTag {
        #[cfg(debug_assertions)]
        let tag = DebugTag(_tag);

        #[cfg(not(debug_assertions))]
        let tag = DebugTag();

        tag
    }
}

#[cfg(test)]
mod tests {
    use super::DebugTag;

    #[test]
    #[cfg(debug_assertions)]
    fn not_equal() {
        let a = DebugTag::new();
        let b = DebugTag::new();
        assert!(a != b); 
    }

    #[test]
    fn equal() {
        let a = DebugTag::new();
        let b = a.clone();
        assert!(a == b);
    }

    #[test]
    fn reflexive() {
        let a = DebugTag::new();
        assert!(a == a);
    }
}