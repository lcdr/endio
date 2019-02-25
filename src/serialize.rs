use std::io::Result as Res;
use std::io::Write;

use crate::{BigEndian, Endianness, EWrite, LittleEndian};

/**
	Implement this for your types to be able to `write` them.

	## Examples

	### Serialize a struct:

	Note how the trait bound for `W` is `EWrite<E>`, as we want to use the functionality of this crate to delegate serialization to the struct's fields.

	Note: Rust currently can't recognize sealed traits, so even though the primitive types are implemented, you may need to write `where` clauses like below for this to work. If/When the compiler gets smarter about sealed traits this won't be necessary.
	```
	struct Example {
		a: u8,
		b: bool,
		c: u32,
	}
	{
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
	}
	// will then allow you to directly write:
	{
		use endio::LEWrite;

		let mut writer = vec![];
		let e = Example { a: 42, b: true, c: 754187983 };
		writer.write(&e);

		assert_eq!(writer, b"\x2a\x01\xcf\xfe\xf3\x2c");
	}
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

/// Writes the entire contents of the byte slice. Equivalent to `std::io::Write::write_all`.
impl<E: Endianness, W: Write> Serialize<E, W> for &[u8] {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write_all(self)
	}
}

/// Writes the entire contents of the Vec<u8>.
impl<E: Endianness, W: Write> Serialize<E, W> for &Vec<u8> {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write_all(&self[..])
	}
}

/// Writes a bool by writing a byte.
impl<E: Endianness, W: Write> Serialize<E, W> for bool {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write_all(&(self as u8).to_ne_bytes())
	}
}

impl<E: Endianness, W: Write> Serialize<E, W> for u8 {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write_all(&self.to_ne_bytes())
	}
}

impl<E: Endianness, W: Write> Serialize<E, W> for i8 {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write_all(&self.to_ne_bytes())
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

impl_int!(u16);
impl_int!(u32);
impl_int!(u64);
impl_int!(u128);
impl_int!(i16);
impl_int!(i32);
impl_int!(i64);
impl_int!(i128);

impl<E: Endianness, W: EWrite<E>> Serialize<E, W> for f32 where u32: Serialize<E, W> {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write(self.to_bits())
	}
}

impl<E: Endianness, W: EWrite<E>> Serialize<E, W> for f64 where u64: Serialize<E, W> {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write(self.to_bits())
	}
}

#[cfg(test)]
mod tests {
	use std::io::Result as Res;

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
