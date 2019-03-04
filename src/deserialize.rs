use std::io;
use std::io::Read;
use std::io::Result as Res;
use std::mem::size_of;

use crate::{BigEndian, ERead, Endianness, LittleEndian};

/**
	Implement this for your types to be able to `read` them.

	## Examples

	### Deserialize a struct:

	Note how the trait bound for `R` is `ERead<E>`, as we want to use the functionality of this crate to delegate deserialization to the struct's fields.

	Note: As you can see below, you may need to write `where` clauses when delegating functionality to other `read` operations. There are two reasons for this:

	- Rust currently can't recognize sealed traits. Even though there are only two endiannesses, and the primitive types are implemented for them, the compiler can't recognize that. If/When the compiler gets smarter about sealed traits this will be resolved. Alternatively, once Rust gets support for specialization, I will be able to add a dummy blanket `impl` to primitives which will work around this issue.

	- The underlying `R` type needs to implement `std::io::Read` to be able to read into primitive types. You can work around this by explicitly specifying `Read` as trait bound, but since both `Read` and `ERead` have a `read` method, Rust will force you to use UFCS syntax to disambiguate between them. This makes using `read` less ergonomic, and I personally think that `where` clauses are the better alternative here, since they avoid this issue.

		Ideally I'd like to make `std::io::Read` a supertrait of `ERead`, since the deserialization will normally depend on `Read` anyway. Unfortunately, supertraits' methods automatically get brought into scope, so this would mean that you would be forced to use UFCS every time, without being able to work around them with `where` clauses. ([Rust issue #17151](https://github.com/rust-lang/rust/issues/17151)).
	```
	# #[derive(Debug, PartialEq)]
	struct Example {
		a: u8,
		b: bool,
		c: u32,
	}
	{
		use std::io::Result;
		use endio::{Deserialize, Endianness, ERead};

		impl<E: Endianness, R: ERead<E>> Deserialize<E, R> for Example
			where bool: Deserialize<E, R>,
			      u8  : Deserialize<E, R>,
			      u32 : Deserialize<E, R> {
			fn deserialize(reader: &mut R) -> Result<Self> {
				let a = reader.read()?;
				let b = reader.read()?;
				let c = reader.read()?;
				Ok(Example { a, b, c })
			}
		}
	}
	// will then allow you to directly write:
	{
		use endio::LERead;
		let mut reader = &b"\x2a\x01\xcf\xfe\xf3\x2c"[..];
		let e: Example = reader.read().unwrap();

		assert_eq!(e, Example { a: 42, b: true, c: 754187983 });
	}
	# {
	# 	use endio::BERead;
	# 	let mut reader = &b"\x2a\x01\x2c\xf3\xfe\xcf"[..];
	# 	let e: Example = reader.read().unwrap();
	#
	# 	assert_eq!(e, Example { a: 42, b: true, c: 754187983 });
	# }
	```

	### Deserialize a primitive / something where you need the bare `std::io::Read` functionality:

	Note how the trait bound for `R` is `Read`.

	```
	use std::io::{Read, Result};
	use endio::{Deserialize, Endianness, ERead};

	struct new_u8(u8);

	impl<E: Endianness, R: Read> Deserialize<E, R> for new_u8 {
		fn deserialize(reader: &mut R) -> Result<Self> {
			let mut buf = [0; 1];
			reader.read_exact(&mut buf);
			Ok(new_u8(buf[0]))
		}
	}
	```

	### Deserialize with endian-specific code:

	Note how instead of using a trait bound on Endianness, we implement Deserialize twice, once for `BigEndian` and once for `LittleEndian`.
	```
	use std::io::{Read, Result};
	use std::mem::size_of;
	use endio::{BigEndian, Deserialize, LittleEndian};

	struct new_u16(u16);

	impl<R: Read> Deserialize<BigEndian, R> for new_u16 {
		fn deserialize(reader: &mut R) -> Result<Self> {
			let mut buf = [0; size_of::<u16>()];
			reader.read_exact(&mut buf)?;
			Ok(new_u16(u16::from_be_bytes(buf)))
		}
	}

	impl<R: Read> Deserialize<LittleEndian, R> for new_u16 {
		fn deserialize(reader: &mut R) -> Result<Self> {
			let mut buf = [0; size_of::<u16>()];
			reader.read_exact(&mut buf)?;
			Ok(new_u16(u16::from_le_bytes(buf)))
		}
	}
	```
*/
pub trait Deserialize<E: Endianness, R>: Sized {
	/// Deserializes the type by reading from the reader.
	fn deserialize(reader: &mut R) -> Res<Self>;
}

/// Reads a bool by reading a byte, returning false for 0, true for 1, and an `InvalidData` error for any other value.
impl<E: Endianness, R: Read> Deserialize<E, R> for bool {
	fn deserialize(reader: &mut R) -> Res<Self> {
		let mut buf = [0; size_of::<Self>()];
		reader.read_exact(&mut buf)?;
		match buf[0] {
			0 => Ok(false),
			1 => Ok(true),
			_ => Err(io::Error::new(io::ErrorKind::InvalidData, "bool had value other than 0 or 1")),
		}
	}
}

impl<E: Endianness, R: Read> Deserialize<E, R> for i8 {
	fn deserialize(reader: &mut R) -> Res<Self> {
		let mut buf = [0; size_of::<Self>()];
		reader.read_exact(&mut buf)?;
		Ok(Self::from_ne_bytes(buf))
	}
}

impl<E: Endianness, R: Read> Deserialize<E, R> for u8 {
	fn deserialize(reader: &mut R) -> Res<Self> {
		let mut buf = [0; size_of::<Self>()];
		reader.read_exact(&mut buf)?;
		Ok(Self::from_ne_bytes(buf))
	}
}

macro_rules! impl_int {
	($t:ident) => {
		impl<R: Read> Deserialize<BigEndian, R> for $t {
			fn deserialize(reader: &mut R) -> Res<Self> {
				let mut buf = [0; size_of::<Self>()];
				reader.read_exact(&mut buf)?;
				Ok(Self::from_be_bytes(buf))
			}
		}

		impl<R: Read> Deserialize<LittleEndian, R> for $t {
			fn deserialize(reader: &mut R) -> Res<Self> {
				let mut buf = [0; size_of::<Self>()];
				reader.read_exact(&mut buf)?;
				Ok(Self::from_le_bytes(buf))
			}
		}

		#[cfg(test)]
		mod $t {
			#[test]
			fn test() {
				let integer: u128 = 0xbaadf00dbaadf00dbaadf00dbaadf00d;
				let bytes = b"\x0d\xf0\xad\xba\x0d\xf0\xad\xba\x0d\xf0\xad\xba\x0d\xf0\xad\xba";
				let mut val: $t;
				{
					use crate::BERead;
					let mut reader = &bytes[..];
					val = reader.read().unwrap();
					assert_eq!(val, (integer as $t).to_be());
				}
				{
					use crate::LERead;
					let mut reader = &bytes[..];
					val = reader.read().unwrap();
					assert_eq!(val, (integer as $t).to_le());
				}
			}
		}
	}
}

impl_int!(u16);
impl_int!(u32);
impl_int!(u64);
impl_int!(u128);
impl_int!(i16);
impl_int!(i32);
impl_int!(i64);
impl_int!(i128);

impl<E: Endianness, R: ERead<E>> Deserialize<E, R> for f32 where u32: Deserialize<E, R> {
	fn deserialize(reader: &mut R) -> Res<Self> {
		let ival: u32 = reader.read()?;
		Ok(Self::from_bits(ival))
	}
}

impl<E: Endianness, R: ERead<E>> Deserialize<E, R> for f64 where u64: Deserialize<E, R> {
	fn deserialize(reader: &mut R) -> Res<Self> {
		let ival: u64 = reader.read()?;
		Ok(Self::from_bits(ival))
	}
}

#[cfg(test)]
mod tests {
	use std::io;
	use std::io::Result as Res;

	#[test]
	fn read_bool_false() {
		let data = b"\x00";
		let mut val: bool;
		{
			use crate::BERead;
			let mut reader = &data[..];
			val = reader.read().unwrap();
			assert_eq!(val, false);
		}
		{
			use crate::LERead;
			let mut reader = &data[..];
			val = reader.read().unwrap();
			assert_eq!(val, false);
		}
	}

	#[test]
	fn read_bool_true() {
		let data = b"\x01";
		let mut val: bool;
		{
			use crate::BERead;
			let mut reader = &data[..];
			val = reader.read().unwrap();
			assert_eq!(val, true);
		}
		{
			use crate::LERead;
			let mut reader = &data[..];
			val = reader.read().unwrap();
			assert_eq!(val, true);
		}
	}

	#[test]
	fn read_bool_invalid() {
		let data = b"\x2a";
		let mut val;
		{
			use crate::BERead;
			let mut reader = &data[..];
			val = reader.read::<bool>().unwrap_err();
			assert_eq!(val.kind(), io::ErrorKind::InvalidData);
		}
		{
			use crate::LERead;
			let mut reader = &data[..];
			val = reader.read::<bool>().unwrap_err();
			assert_eq!(val.kind(), io::ErrorKind::InvalidData);
		}
	}

	#[test]
	fn read_i8() {
		let data = b"\x80";
		let mut val: i8;
		{
			use crate::BERead;
			let mut reader = &data[..];
			val = reader.read().unwrap();
			assert_eq!(val, i8::min_value());
		}
		{
			use crate::LERead;
			let mut reader = &data[..];
			val = reader.read().unwrap();
			assert_eq!(val, i8::min_value());
		}
	}

	#[test]
	fn read_u8() {
		let data = b"\xff";
		let mut val: u8;
		{
			use crate::BERead;
			let mut reader = &data[..];
			val = reader.read().unwrap();
			assert_eq!(val, u8::max_value());
		}
		{
			use crate::LERead;
			let mut reader = &data[..];
			val = reader.read().unwrap();
			assert_eq!(val, u8::max_value());
		}
	}

	#[test]
	fn read_f32() {
		let data = b"\x44\x20\xa7\x44";
		let mut val: f32;
		{
			use crate::BERead;
			let mut reader = &data[..];
			val = reader.read().unwrap();
			assert_eq!(val, 642.613525390625);
		}
		{
			use crate::LERead;
			let mut reader = &data[..];
			val = reader.read().unwrap();
			assert_eq!(val, 1337.0083007812);
		}
	}

	#[test]
	fn read_f64() {
		let data = b"\x40\x94\x7a\x14\xae\xe5\x94\x40";
		let mut val: f64;
		{
			use crate::BERead;
			let mut reader = &data[..];
			val = reader.read().unwrap();
			assert_eq!(val, 1310.5201984283194);
		}
		{
			use crate::LERead;
			let mut reader = &data[..];
			val = reader.read().unwrap();
			assert_eq!(val, 1337.4199999955163);
		}
	}

	#[test]
	fn read_struct_forced() {
		struct Test {
			a: u16,
		}
		{
			use crate::{Deserialize, Endianness, ERead};

			impl<E: Endianness, R: ERead<E>> Deserialize<E, R> for Test where u16: Deserialize<E, R> {
				fn deserialize(reader: &mut R) -> Res<Self> {
					let a = reader.read()?;
					Ok(Test { a })
				}
			}
		}

		use crate::LERead;
		let data = b"\xba\xad";
		let mut reader = &data[..];
		let val: Test = reader.read_be().unwrap();
		assert_eq!(val.a, 0xbaad);
	}
}
