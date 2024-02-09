# VBox

VBox is a type erased Box of trait object that stores the vtable pointer.

`VBox` is just like a `Box<dyn Trait>` but erases type `Trait` so that to use it, there is no need to have `Trait` as one of its type parameters. Only the creator and the consumer needs to agree on the type parameters.

Internally, it stores the trait object’s data pointer in a `Box<dyn Any + Send>`, so that the `Drop::drop()` will be called when the wrapper is dropped. And it stores the vtable pointer in another `usize` to make sure it is `Send`.

## [Example](#example)

```rust
// Pack a u64 into a `Debug` trait object and erase the type `Debug`.
let vbox: VBox = into_vbox!(dyn Debug, 10u64);

// Unpack to different trait object will panic:
// let _panic = from_vbox!(dyn Display, vbox);

// Unpack the `Debug` trait object.
let unpacked: Box<dyn Debug> = from_vbox!(dyn Debug, vbox);

assert_eq!("10", format!("{:?}", unpacked));
```
