use std::io::Result as Res;
use std::io::Write;
use std::net::Ipv4Addr;

use crate::{BEWrite, BigEndian, Endianness, EWrite, LEWrite, LittleEndian};

/**
	Implement this for your types to be able to `write` them.

	## Derive macro for common serializations

	A common case is the following: You want to make your struct serializable. Your struct's fields are all serializable themselves. You want to simply serialize everything in order, and then construct a struct instance from that.

	If that's all you want, simply `#[derive(Serialize)]` and you're good to go.

	## Examples

	### Serialize a struct:
	```
	# #[cfg(feature="derive")] {
	# use endio::Serialize;
	#[derive(Serialize)]
	struct Example {
		a: u16,
		b: bool,
		c: u32,
	}
	# {
	use endio::LEWrite;
	let mut writer = vec![];
	writer.write(&Example { a: 0xadba, b: true, c: 0x0df0adba }).unwrap();
	assert_eq!(writer, b"\xba\xad\x01\xba\xad\xf0\x0d");
	# }
	# {
	# use endio::BEWrite;
	# let mut writer = vec![];
	# writer.write(&Example { a: 0xbaad, b: true, c: 0xbaadf00d }).unwrap();
	# assert_eq!(writer, b"\xba\xad\x01\xba\xad\xf0\x0d");
	# }
	# }
	```

	This also works with tuple structs:

	```
	# #[cfg(feature="derive")] {
	# use endio::Serialize;
	#[derive(Serialize)]
	struct Example(u16, bool, u32);
	# {
	# use endio::LEWrite;
	# let mut writer = vec![];
	# writer.write(&Example(0xadba, true, 0x0df0adba)).unwrap();
	# assert_eq!(writer, b"\xba\xad\x01\xba\xad\xf0\x0d");
	# }
	# {
	# use endio::BEWrite;
	# let mut writer = vec![];
	# writer.write(&Example(0xbaad, true, 0xbaadf00d)).unwrap();
	# assert_eq!(writer, b"\xba\xad\x01\xba\xad\xf0\x0d");
	# }
	# }
	```

	and unit structs:

	```
	# #[cfg(feature="derive")] {
	# use endio::Serialize;
	#[derive(Serialize)]
	struct Example;

	# {
	# use endio::LEWrite;
	# let mut writer = vec![];
	# writer.write(&Example).unwrap();
	# assert_eq!(writer, b"");
	# }
	# {
	# use endio::BEWrite;
	# let mut writer = vec![];
	# writer.write(&Example).unwrap();
	# assert_eq!(writer, b"");
	# }
	# }
	```

	### Serialize an enum:

	The derive macro also works with enums, however you will have to explicitly specify the type of the discriminant by adding a repr attribute with an int type argument to the enum.

	The derive macro works even without explicitly specified discriminant values and with variants carrying data. Nightly Rust also supports the combination of both under [`#![feature(arbitrary_enum_discriminant)]`](https://github.com/rust-lang/rust/issues/60553).

	```
	# #[cfg(feature="derive")] {
	# use endio::Serialize;
	#[derive(Serialize)]
	#[repr(u16)]
	enum Example {
		A,
		B,
		C = 42,
		D,
	}
	# {
	use endio::LEWrite;
	let mut writer = vec![];
	writer.write(&Example::A).unwrap();
	assert_eq!(writer, b"\x00\x00");
	# writer.write(&Example::B).unwrap();
	# writer.write(&Example::C).unwrap();
	# writer.write(&Example::D).unwrap();
	# assert_eq!(writer, b"\x00\x00\x01\x00\x2a\x00\x2b\x00");
	# }
	# {
	# use endio::BEWrite;
	# let mut writer = vec![];
	# writer.write(&Example::A).unwrap();
	# assert_eq!(writer, b"\x00\x00");
	# writer.write(&Example::B).unwrap();
	# writer.write(&Example::C).unwrap();
	# writer.write(&Example::D).unwrap();
	# assert_eq!(writer, b"\x00\x00\x00\x01\x00\x2a\x00\x2b");
	# }
	# }
	```

	```
	# #[cfg(feature="derive")] {
	# use endio::Serialize;
	#[derive(Serialize)]
	#[repr(u16)]
	enum Example {
		A,
		B(u8),
		C { a: u16, b: bool, c: u32 },
		D(u32),
	}
	# {
	use endio::LEWrite;
	let mut writer = vec![];
	writer.write(&Example::C { a: 0xadba, b: true, c: 0x0df0adba }).unwrap();
	assert_eq!(writer, b"\x02\x00\xba\xad\x01\xba\xad\xf0\x0d");
	# }
	# {
	# use endio::BEWrite;
	# let mut writer = vec![];
	# writer.write(&Example::C { a: 0xbaad, b: true, c: 0xbaadf00d }).unwrap();
	# assert_eq!(writer, b"\x00\x02\xba\xad\x01\xba\xad\xf0\x0d");
	# }
	# }
	```

	### Padding

	Sometimes you'll have to work with formats containing padding bytes of useless data, or you know that the recipient will ignore some parts. This derive macro provides some attributes to support these cases:

	- Add the `#[padding=n]` attribute to fields to specify `n` bytes of padding before the field.
	- Add the `#[post_disc_padding=n]` attribute on an enum to specify `n` bytes of padding after the discriminant.
	- Add the `#[trailing_padding=n]` attribute on a struct or enum to specify `n` bytes of padding after the struct/enum.

	The derive macro will then automatically write zeros for these bytes.

	```
	# #[cfg(feature="derive")] {
	# use endio::Serialize;
	#[derive(Serialize)]
	#[repr(u16)]
	#[post_disc_padding=3]
	#[trailing_padding=1]
	enum Example {
		A {
			a: u16,
			#[padding=1]
			b: bool,
			#[padding=1]
			c: u32,
		},
	}
	use endio::LEWrite;
	let mut writer = vec![];
	writer.write(&Example::A { a: 0xadba, b: true, c: 0x0df0adba }).unwrap();
	assert_eq!(writer, b"\x00\x00\x00\x00\x00\xba\xad\x00\x01\x00\xba\xad\xf0\x0d\x00");
	# }
	```

	## Custom serializations

	If your serialization is complex or has special cases, you'll need to implement `Serialize` manually.

	### Examples

	### Serialize a struct:

	Note how the trait bound for `W` is `EWrite<E>`, as we want to use the functionality of this crate to delegate serialization to the struct's fields.

	Note: As you can see below, you may need to write `where` clauses when delegating functionality to other `write` operations. There are two reasons for this:

	- Rust currently can't recognize sealed traits. Even though there are only two endiannesses, and the primitive types are implemented for them, the compiler can't recognize that. If/When the compiler gets smarter about sealed traits this will be resolved. Alternatively, once Rust gets support for specialization, I will be able to add a dummy blanket `impl` to primitives which will work around this issue.

	- The underlying `W` type needs to implement `std::io::Write` to be able to write primitive types. You can work around this by explicitly specifying `Write` as trait bound, but since both `Write` and `EWrite` have a `write` method, Rust will force you to use UFCS syntax to disambiguate between them. This makes using `write` less ergonomic, and I personally think that `where` clauses are the better alternative here, since they avoid this issue.

		Ideally I'd like to make `std::io::Write` a supertrait of `EWrite`, since the serialization will normally depend on `Write` anyway. Unfortunately, supertraits' methods automatically get brought into scope, so this would mean that you would be forced to use UFCS every time, without being able to work around them with `where` clauses. ([Rust issue #17151](https://github.com/rust-lang/rust/issues/17151)).
	```
	struct Example {
		a: u8,
		b: bool,
		c: u32,
	}
	# {
	use std::io::Result;
	use endio::{Endianness, EWrite, Serialize};

	impl<E: Endianness, W: EWrite<E>> Serialize<E, W> for &Example
		where u8  : Serialize<E, W>,
		      bool: Serialize<E, W>,
		      u32 : Serialize<E, W> {
		fn serialize(self, writer: &mut W) -> Result<()> {
			writer.write(self.a)?;
			writer.write(self.b)?;
			writer.write(self.c)
		}
	}
	# }
	// will then allow you to directly write:
	# {
	use endio::LEWrite;
	let mut writer = vec![];
	let e = Example { a: 42, b: true, c: 754187983 };
	writer.write(&e);
	assert_eq!(writer, b"\x2a\x01\xcf\xfe\xf3\x2c");
	# }
	# {
	# 	use endio::BEWrite;
	# 	let mut writer = vec![];
	# 	let e = Example { a: 42, b: true, c: 754187983 };
	# 	writer.write(&e);
	# 	assert_eq!(writer, b"\x2a\x01\x2c\xf3\xfe\xcf");
	# }
	```

	### Serialize a primitive / something where you need to use the bare `std::io::Write` functionality:

	Note how the trait bound for `W` is `Write`.
	```
	use std::io::{Result, Write};
	use endio::{Endianness, EWrite, Serialize};

	struct new_u8(u8);

	impl<E: Endianness, W: Write> Serialize<E, W> for &new_u8 {
		fn serialize(self, writer: &mut W) -> Result<()> {
			let mut buf = [0; 1];
			buf[0] = self.0;
			writer.write_all(&buf);
			Ok(())
		}
	}
	```

	### Serialize with endian-specific code:

	Note how instead of using a trait bound on Endianness, we implement Serialize twice, once for `BigEndian` and once for `LittleEndian`.
	```
	use std::io::{Result, Write};
	use std::mem::size_of;
	use endio::{BigEndian, Serialize, LittleEndian};

	struct new_u16(u16);

	impl<W: Write> Serialize<BigEndian, W> for new_u16 {
		fn serialize(self, writer: &mut W) -> Result<()> {
			let mut buf = [0; size_of::<u16>()];
			writer.write_all(&self.0.to_be_bytes())?;
			Ok(())
		}
	}

	impl<W: Write> Serialize<LittleEndian, W> for new_u16 {
		fn serialize(self, writer: &mut W) -> Result<()> {
			writer.write_all(&self.0.to_le_bytes())?;
			Ok(())
		}
	}
	```
*/
pub trait Serialize<E: Endianness, W> {
	/// Serializes the type by writing to the writer.
	fn serialize(self, writer: &mut W) -> Res<()>;
}

// todo[specialization]: specialize for &[u8] (std::io::Write::write_all)
/// Writes the entire contents of the byte slice.
impl<E: Endianness, W: EWrite<E>, S> Serialize<E, W> for &[S] where for<'a> &'a S: Serialize<E, W> {
	fn serialize(self, writer: &mut W) -> Res<()> {
		for elem in self {
			writer.write(elem)?;
		}
		Ok(())
	}
}

/// Writes the entire contents of the Vec.
impl<E: Endianness, W: EWrite<E>, S> Serialize<E, W> for &Vec<S> where for<'a> &'a S: Serialize<E, W> {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write(self.as_slice())
	}
}

macro_rules! impl_ref {
	($t:ident) => {
		impl<W: Write+BEWrite> Serialize<BigEndian, W> for &$t {
			fn serialize(self, writer: &mut W) -> Res<()> {
				BEWrite::write(writer, *self)
			}
		}
		impl<W: Write+LEWrite> Serialize<LittleEndian, W> for &$t {
			fn serialize(self, writer: &mut W) -> Res<()> {
				LEWrite::write(writer, *self)
			}
		}
	}
}

macro_rules! impl_int {
	($t:ident) => {
		impl<W: Write> Serialize<BigEndian, W> for $t {
			fn serialize(self, writer: &mut W) -> Res<()> {
				writer.write_all(&self.to_be_bytes())
			}
		}

		impl<W: Write> Serialize<LittleEndian, W> for $t {
			fn serialize(self, writer: &mut W) -> Res<()> {
				writer.write_all(&self.to_le_bytes())
			}
		}

		impl_ref!($t);

		#[cfg(test)]
		mod $t {
			use std::mem::size_of;

			#[test]
			fn test() {
				let integer: u128 = 0xbaadf00dbaadf00dbaadf00dbaadf00d;
				let bytes = b"\x0d\xf0\xad\xba\x0d\xf0\xad\xba\x0d\xf0\xad\xba\x0d\xf0\xad\xba";

				{
					use crate::BEWrite;
					let mut writer = vec![];
					writer.write((integer as $t).to_be()).unwrap();
					assert_eq!(&writer[..], &bytes[..size_of::<$t>()]);
				}
				{
					use crate::LEWrite;
					let mut writer = vec![];
					writer.write((integer as $t).to_le()).unwrap();
					assert_eq!(&writer[..], &bytes[..size_of::<$t>()]);
				}
			}
		}
	}
}

impl_int!(u8);
impl_int!(u16);
impl_int!(u32);
impl_int!(u64);
impl_int!(u128);
impl_int!(i8);
impl_int!(i16);
impl_int!(i32);
impl_int!(i64);
impl_int!(i128);

impl<E: Endianness, W: EWrite<E>> Serialize<E, W> for f32 where u32: Serialize<E, W> {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write(self.to_bits())
	}
}
impl_ref!(f32);

impl<E: Endianness, W: EWrite<E>> Serialize<E, W> for f64 where u64: Serialize<E, W> {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write(self.to_bits())
	}
}
impl_ref!(f64);

/// Writes a bool by writing a byte.
impl<E: Endianness, W: Write> Serialize<E, W> for bool {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write_all(&(self as u8).to_ne_bytes())
	}
}
impl_ref!(bool);

impl<E: Endianness, W: Write> Serialize<E, W> for Ipv4Addr {
	fn serialize(self, writer: &mut W) -> Res<()>	{
		writer.write_all(&self.octets()[..])
	}
}
impl_ref!(Ipv4Addr);

#[cfg(test)]
mod tests {
	use std::io::Result as Res;

	#[test]
	fn write_slice() {
		let data = b"\xba\xad\xba\xad";
		use crate::LEWrite;
		let mut writer = vec![];
		writer.write(&[0xadbau16, 0xadbau16][..]).unwrap();
		assert_eq!(writer, data);
	}

	#[test]
	fn write_vec() {
		let data = b"\xba\xad\xba\xad";
		use crate::LEWrite;
		let mut writer = vec![];
		writer.write(&vec![0xadbau16, 0xadbau16]).unwrap();
		assert_eq!(writer, data);
	}

	#[test]
	fn write_bool_false() {
		let data = b"\x00";
		let val = false;
		{
			use crate::BEWrite;
			let mut writer = vec![];
			writer.write(val).unwrap();
			assert_eq!(writer, data);
		}
		{
			use crate::LEWrite;
			let mut writer = vec![];
			writer.write(val).unwrap();
			assert_eq!(writer, data);
		}
	}

	#[test]
	fn write_bool_true() {
		let data = b"\x01";
		let val = true;
		{
			use crate::BEWrite;
			let mut writer = vec![];
			writer.write(val).unwrap();
			assert_eq!(writer, data);
		}
		{
			use crate::LEWrite;
			let mut writer = vec![];
			writer.write(val).unwrap();
			assert_eq!(writer, data);
		}
	}

	#[test]
	fn write_i8() {
		let data = b"\x80";
		let val = i8::min_value();
		{
			use crate::BEWrite;
			let mut writer = vec![];
			writer.write(val).unwrap();
			assert_eq!(writer, data);
		}
		{
			use crate::LEWrite;
			let mut writer = vec![];
			writer.write(val).unwrap();
			assert_eq!(writer, data);
		}
	}

	#[test]
	fn write_u8() {
		let data = b"\xff";
		let val = u8::max_value();
		{
			use crate::BEWrite;
			let mut writer = vec![];
			writer.write(val).unwrap();
			assert_eq!(writer, data);
		}
		{
			use crate::LEWrite;
			let mut writer = vec![];
			writer.write(val).unwrap();
			assert_eq!(writer, data);
		}
	}

	#[test]
	fn write_f32() {
		let data = b"\x44\x20\xa7\x44";
		{
			use crate::BEWrite;
			let mut writer = vec![];
			writer.write(642.613525390625f32).unwrap();
			assert_eq!(writer, data);
		}
		{
			use crate::LEWrite;
			let mut writer = vec![];
			writer.write(1337.0083007812f32).unwrap();
			assert_eq!(writer, data);
		}
	}

	#[test]
	fn write_f64() {
		let data = b"\x40\x94\x7a\x14\xae\xe5\x94\x40";
		{
			use crate::BEWrite;
			let mut writer = vec![];
			writer.write(1310.5201984283194f64).unwrap();
			assert_eq!(writer, data);
		}
		{
			use crate::LEWrite;
			let mut writer = vec![];
			writer.write(1337.4199999955163f64).unwrap();
			assert_eq!(writer, data);
		}
	}

	#[test]
	fn write_ipv4_addr() {
		use std::net::Ipv4Addr;

		let data = b"\x7f\x00\x00\x01";
		{
			use crate::BEWrite;
			let mut writer = vec![];
			writer.write(Ipv4Addr::LOCALHOST).unwrap();
			assert_eq!(writer, data);
		}
		{
			use crate::LEWrite;
			let mut writer = vec![];
			writer.write(Ipv4Addr::LOCALHOST).unwrap();
			assert_eq!(writer, data);
		}
	}

	#[test]
	fn write_struct_forced() {
		struct Test {
			a: u16,
		}
		{
			use crate::{Endianness, EWrite, Serialize};

			impl<E: Endianness, W: EWrite<E>> Serialize<E, W> for Test where u16: Serialize<E, W> {
				fn serialize(self, writer: &mut W) -> Res<()> {
					writer.write(self.a)?;
					Ok(())
				}
			}
		}

		use crate::LEWrite;
		let data = b"\xba\xad";
		let mut writer = vec![];
		writer.write_be(Test { a: 0xbaad }).unwrap();
		assert_eq!(&writer[..], data);
	}
}
