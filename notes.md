# Interior Mutabilitiy

From [Stack Overflow:](https://stackoverflow.com/questions/45674479/need-holistic-explanation-about-rusts-cell-and-reference-counted-types)

There are two essential concepts in Rust:

- Ownership,
- Mutability.

The various pointer types (`Box`, `Rc`, `Arc`) are concerned with Ownership: they allow controlling whether there is a single or multiple owners for a single object.

On the other hand, the various cells (`Cell`, `RefCell`, `Mutex`, `RwLock`, `AtomicXXX)` are concerned with Mutability.

The founding rule of Rust's safety is *Aliasing XOR Mutability*. That is, an object can only be safely mutated if there is no outstanding reference to its interior.

This rule is generally enforced at compile time by the borrow checker:

if you have a `&T`, you cannot also have a `&mut T` to the same object in scope,
if you have a `&mut T`, you cannot also have any reference to the same object in scope.
However, sometimes, this is not flexible enough. Sometimes you DO need (or want) the ability to have multiple references to the same object and yet mutate it. Enter the cells.

The idea of `Cell` and `RefCell` is to permit mutability in the presence of aliasing in a controlled manner:

`Cell` prevents the formation of reference to its interior, avoiding dangling references,
`RefCell` shifts the enforcement of *Aliasing XOR Mutability* from compile time to runtime.
This functionality is sometimes described as providing interior mutability, that is where an object which otherwise looks immutable from the outside (&T) can actually be mutated.

When this mutability extends across multiple threads, you will instead use `Mutex`, `RwLock` or `AtomicXXX`; they provide the same functionality:

`AtomicXXX` are just `Cell`: no reference to the interior, just moving in/out,
`RwLock` is just `RefCell`: can obtain references to the interior through guards,
`Mutex` is a simplified version of `RwLock` which does not distinguish between a read-only guard and write guard; so conceptually similar to a RefCell with only a borrow_mut method.
If you come from a C++ background:

> Following part is a bit questionable in its comparisons

`Box` is `unique_ptr`,
`Arc` is `shared_ptr`,
`Rc` is a non thread-safe version of `shared_ptr`.
And the cells provide a similar functionality as mutable, except with additional guarantees to avoid aliasing issues; think of `Cell` as `std::atomic` and `RefCell` as a non thread-safe version of `std::shared_mutex` (which throws instead of blocking if the lock is taken).