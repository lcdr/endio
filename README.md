
# endio

## Simple and ergonomic reading and writing of binary data.

`std::io::{Read, Write}` only allow reading and writing of raw bytes. This can be cumbersome when trying to read/write data, e.g. integers, as they have to be converted from/to bytes first.
This crate allows direct reading/writing of data types, making this as easy as

`let value: u8 = reader.read()?;`

and

`writer.write(42u8);`.

### Per-object endianness specification

When working with integers consisting of multiple bytes, two kinds of byte ordering are possible, big and little endian. This crate can automatically convert integers from/to the desired endianness when they're read/written, without you having to specify the endianness for every read/write call. This works by making the endianness part of the type of the read/write implementor through different traits. In most scenarios the endianness stays the same for the entirety of the I/O, so this can save a lot of typing.

You aren't bound to the specified endianness for the entirety of the I/O. If you need to work with data where the endianness changes midway through, you can explicitly override the endianness with the `_be`-/`_le` -suffixed methods.

### Entirely trait-based I/O

This crate is entirely trait-based, which means it adds zero state at runtime. All the information needed is encoded at the type level. All functionality is delegated to (de-)serialization code.

Compared to the alternative way of per-object endianness specification via a wrapper type, traits also have the advantage that they don't limit access to the specific type at all, whereas with wrappers you need to explicitly get a reference the underlying object to use it.

### Zero-cost abstractions

As this crate is entirely trait-based, the compiler will typically aggressively inline and optimize out all calls to this crate. The only thing remaining will be the (de-)serialization code, which is also necessary without the use of this crate. Therefore using this crate will not result in any penalty to your code, neither in speed nor memory use.

When I compared code using byte-level I/O from this crate to code using the equivalent raw `std::io` functionality in a disassembler, compiled in release mode, I only found found a difference of a few opcodes, and all function calls to this crate had been completely optimized away.

I haven't yet done any serious benchmarks, or checked the disassembly of all types and features, but these initial results look promising.

### Extensibility: Reading & writing your own types

You can take advantage of this crate's features and the `read`/`write` methods by implementing this crate's traits for your own types. No distinction is made between user types and types for which (de-)serialization is already provided by this crate, and you will be able to use `read` and `write` for your own types just like for primitive types.

You can write (de-)serializations that differ depending on endianness, or (de-)serializations that don't differentiate and instead use the automatic endian inference of this crate to delegate endian differentiation to sub-structs.

You're not limited to `std::io::{Read, Write}` or this crate's abstractions when writing (de-)serializations, you can also use other functionality through appropriate trait bounds. For example, it's also possible to write a (de-)serialization that uses `std::io::Seek` in its code.

### Simple migration

If you were using raw `std::io::{Read, Write}` with manual (de-)serialization code before, or a crate providing abstractions on top of `{Read, Write}`, the migration towards using this crate is extremely straightforward. Simply swap out the std::io traits with the endian-specific traits from this crate, and you'll have access to the direct read/write methods. You don't need to write any extra code, everything that implements `std::io::Read` or `std::io::Write` will automatically implement the endian-specific traits.

(De-)serializations for Rust's primitive types are already implemented, if it makes sense to implement them. For example, `isize` and `usize` aren't implemented, since they are inherently variable-sized and therefore can't have a portable byte representation. However, the other integral and floating point types are implemented, and if you just want to read/write some simple primitive types, using the `read`/`write` methods will Just Work™.

## Comparison to other crates

Binary I/O and endianness conversion are common problems, and a number of crates providing solutions already exist. [See here for how they compare to this crate](https://bitbucket.org/lcdr/endio/src/tip/Comparison.md).

## Examples

Here are two typical ways to use this crate:

### Read data from bytes in memory, in little endian:

```rust
// This will make the read calls use little endian.
use endio::LERead;

// Works with any object implementing std::io::Read.
let mut reader = &b"\x2a\x01\x2c\xf3\xfe\xcf"[..];

let a: u8   = reader.read().unwrap();     // Reads in little endian (specified by trait).
let b: bool = reader.read().unwrap();     // Deserialization code is automatically inferred.
let c: u32  = reader.read_be().unwrap();  // Reads in forced big endian.

// The results are already converted into the appropriate types and ready for use.
assert_eq!(a, 42);
assert_eq!(b, true);
assert_eq!(c, 754187983);
```

### Write data to a vector of bytes, in big endian:

```rust
// This time we'll use big endian.
use endio::BEWrite;

// Vec implements std::io::Write.
let mut writer = vec![];

writer.write(42u8);          // Directly specifying values works fine.
writer.write(true);          // Everything is automatically converted to bytes.
writer.write_le(754187983);  // The trait endianness can be overwritten if necessary.

// Done!
assert_eq!(writer, b"\x2a\x01\xcf\xfe\xf3\x2c");
```

More examples and explanations on how this crate works are included in the documentation of the interfaces.
There are also examples on how to implement your own types.

## Getting started

To conduct I/O you `use` the traits `BERead` & `BEWrite`, or `LERead` & `LEWrite`. Choose `BERead` & `BEWrite` for big endian I/O, and `LERead` & `LEWrite` for little endian I/O. This will give you the `read`/`write` methods on your structs. `read` returns values of your desired type, and `write` accepts values as a parameter. The deserialization to be used and the type to be returned are handled through type inference, so most of the time you won't even need to annotate the type explicitly.

You can read and write your own types by implementing `Serialize`/`Deserialize`. See their documentation for details.
