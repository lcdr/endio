
# Comparison to other crates

## `byteorder`

`byteorder` is a widely used crate for binary I/O.

`byteorder` and `endio` differ in the following:

| Feature                  | `byteorder`                             | `endio`                                           |
|--------------------------|-----------------------------------------|---------------------------------------------------|
| Endianness specification | Can only be specified per I/O operation | Can be specified per I/O operation and per object |
| Type specification       | Hardcoded functions with suffixes       | Inferred through return type                      |
| Extensibility            | None                                    | Can be extended to support any type               |

The impact of these differences is best shown in an example:

### Reading multiple types with the same endianness

#### `byteorder`

```rust
use byteorder::{ReadBytesExt, LittleEndian};

let mut reader = &b"<binary content>"[..];

let a = reader.read_u16::<LittleEndian>()?;
let b = reader.read_u32::<LittleEndian>()?;
let c = reader.read_f64::<LittleEndian>()?;
```

### `endio`

```rust
use endio::LERead;

let mut reader = &b"<binary content>"[..];

let a: u16 = reader.read()?;
let b: u32 = reader.read()?;
let c: f64 = reader.read()?;
```

In most I/O scenarios, the endianness is fixed for the entirety of I/O, so being able to specify the endianness for all I/O operations can save a lot of typing. Often the type annotations above can also be left out, as the types can be inferred through other code.

`endio`'s extensibility can also further help to simplify code:

### Reading a custom struct

#### `byteorder`

```rust
use byteorder::{ReadBytesExt, LittleEndian};

struct Vector3(f32, f32, f32);

let mut reader = &b"<binary content>"[..];

let x = reader.read_f32::<LittleEndian>()?;
let y = reader.read_f32::<LittleEndian>()?;
let z = reader.read_f32::<LittleEndian>()?;

let vector = Vector3(x, y, z);
```

#### `endio`

```rust
use endio::{Deserialize, LERead};

#[derive(Deserialize)]
struct Vector3(f32, f32, f32);

let mut reader = &b"<binary content>"[..];

let vector: Vector3 = reader.read()?;
```

Being able to directly read and write complex types can save a lot of repetition and prevent bugs.

Custom (de-)serializations can also easily support both endiannesses by delegating it to sub-item (de-)serializations. They can also call type-specific methods and use the full functionality of the type; they aren't limited to just reading/writing. See the documentation for `endio::Deserialize` for details.

## `byteordered`

`byteordered` is a wrapper on top of `byteorder`, adding per-object endianness specification. It also adds some alternate endianness markers with conditional constructors, which it claims make runtime endianness selection possible. However runtime selection is already possible in raw `byteorder` through the use of generics, so it's unclear how much utility `byteordered` actually provides.

`byteordered`'s way of adding per-object endianness specification is through a wrapper type, which is inherently cumbersome as any access to type-specific methods needs to acquire a reference from the wrapper first. In contrast, `endio`'s way of per-object endianness specification is through the use of a trait, which allows direct use of the I/O object without any wrapper. The difference can be seen below.

### Reading from a `TcpStream`

In this example we want to read from a `TcpStream` and also get the address it is connected to:

#### `byteordered`

```rust
use std::net::TcpStream;
use byteordered::ByteOrdered;

let mut reader = ByteOrdered::le(TcpStream::connect("example.com")?);

let a = reader.read_u32()?;

// `inner_mut` call necessary
let tcp = reader.inner_mut();
// can't use `reader` while reference is alive
// reader.read_u32()?; // error
let addr = tcp.peer_addr()?;

let b = reader.read_f64()?;
```

#### `endio`

```rust
use std::net::TcpStream;
use endio::LERead;

let mut reader = TcpStream::connect("example.com")?;

let a: u32 = reader.read()?;

// direct access possible, no restrictions
let addr = reader.peer_addr();

let b: f64 = reader.read()?;
```

# Detailed comparison tables with various crates

Here are some detailed comparisons with various related crates I found on crates.io. If a crate isn't mentioned, tell me and I'll add it.

Not included are:
- `endian_type` - not documented, unclear
- `byteorder-pod` - documentation link broken
- `scroll` - overly complex, unclear, some documentation links broken

## Core functionality

| crate           | big endian | little endian | reading | writing |
|-----------------|------------|---------------|---------|---------|
| `endio`         | ✅          | ✅             | ✅       | ✅       |
| `byteorder`     | ✅          | ✅             | ✅       | ✅       |
| `endianrw`      | ✅          | ✅             | ✅       | ✅       |
| `little-endian` | ❌          | ✅             | ✅       | ✅       |
| `endian`        | ✅          | ✅             | ✅       | ✅       |
| `endianness`    | ✅          | ✅             | ✅       | ❌       |

## I/O object type

| crate           | I/O object type |
|-----------------|-----------------|
| `endio`         | `Read`/`Write`  |
| `byteorder`     | `Read`/`Write`  |
| `endianrw`      | `Read`/`Write`  |
| `little-endian` | slice           |
| `endian`        | slice           |
| `endianness`    | slice           |

## Supported types

| crate           | {u, i}{16, 32, 64} | {u, i}8 | {u, i}128 | f32, f64 | bool | user types |
|-----------------|--------------------|---------|-----------|----------|------|------------|
| `endio`         | ✅                  | ✅       | ✅         | ✅        | ✅    | ✅          |
| `byteorder`     | ✅                  | ❌       | ✅         | ✅        | ❌    | ❌          |
| `endianrw`      | ✅                  | ✅       | ❌         | ✅        | ❌    | (✅)        |
| `little-endian` | ✅                  | ✅       | ✅         | ❌        | ❌    | (✅)        |
| `endian`        | ✅                  | ❌       | ❌         | ❌        | ❌    | (✅)        |
| `endianness`    | ✅                  | ❌       | ❌         | ✅        | ❌    | ❌          |

(✅) = Not intended by the author, but the interface could be used to implement user types to some extent

## Parameter specification

| crate           | endianness specification  | type specification |
|-----------------|---------------------------|--------------------|
| `endio`         | per operation, per object | return type        |
| `byteorder`     | per operation             | suffix             |
| `endianrw`      | per operation             | type parameter     |
| `little-endian` | ❌                         | trait on type      |
| `endian`        | per operation             | trait on type      |
| `endianness`    | per operation             | suffix             |
