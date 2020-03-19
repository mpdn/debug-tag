# debug-tag

A library for creating debug-only tags to track and check where values are used.

Useful for asserting that two values stem from the same origin. For example, an operation on two
nodes may only be valid if the nodes are in the same graph. `DebugTag` can be used to check for
this in tests and when debugging.

See the `DebugTag` type for more details.

## Motivation

When developing Rust, we often use *indices* to refer to other values instead of using
*pointers*. Using indices allows us to "point" to a value without borrowing it and can basically
be seen as bypassing the borrow checker in a restricted way.

One of the issues of using indicies, is that it is not connected at compile time to the
container storing the value. An index used with an incorrect container might panic, or worse,
return garbage values.

We need a way to "tag" values such that we can determine if an index is used with the correct
container.

## Example

```rust
use debug_tag::DebugTag;

/// An example vec-like structure that allows pushing values and getting values by index.
#[derive(Default)]
struct Slab<T>{
    items: Vec<T>,
    tag: DebugTag,
}

/// An index into a value on a slab.
#[derive(Copy, Clone)]
struct Index {
    index: usize,
    tag: DebugTag,
}

impl<T> Slab<T> {
    /// Pushes a new value onto the slab.
    fn push(&mut self, item: T) -> Index {
        let index = self.items.len();
        self.items.push(item);
        Index {
            index,
            tag: self.tag
        }
    }

    /// Gets a value in this `Slab`. If the index is not from this slab, either panics or
    /// returns an arbitrary value.
    fn get(&self, index: Index) -> &T {
        assert_eq!(self.tag, index.tag, "Index must stem from this slab");
        &self.items[index.index]
    }
}

let mut slab_a = Slab::default();
let ix = slab_a.push(42);

assert_eq!(slab_a.get(ix), &42);

let mut slab_b = Slab::default();
slab_b.push(1337);

// Panics due to the tags being checked:
//   assert_eq!(slab_b.get(ix), &1337);
```

Without `DebugTag`s, the last line above would just return 1337 - a "garbage value" since `ix`
stems from a different `Slab`.

License: MIT
