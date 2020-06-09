use std::io::Write;
use std::io::Result as Res;

use crate::{BigEndian, Endianness, LittleEndian, Serialize};

/**
	Only necessary for custom (de-)serializations.

	Interface for writing data with a specified endianness. Use this interface to make serializations automatically switch endianness without having to write the same code twice.

	In theory this would be the only write trait and BE-/LEWrite would be aliases to the BE/LE type parameter variants, but trait aliases aren't stable yet.

	## Examples

	```
	use endio::LEWrite;

	let mut writer = vec![];
	writer.write(42u8);
	writer.write(true);
	writer.write(754187983u32);

	assert_eq!(writer, b"\x2a\x01\xcf\xfe\xf3\x2c");
	```
*/
pub trait EWrite<E: Endianness>: Sized { // todo[supertrait item shadowing]: make Write a supertrait of this
	/**
		Writes a `Serialize` to the writer, in the writer's endianness.

		What's actually written is up to the implementation of the `Serialize`.
	*/
	fn write   <S: Serialize<E,            Self>>(&mut self, ser: S) -> Res<()> { ser.serialize(self) }
	/// Writes in forced big endian.
	fn write_be<S: Serialize<BigEndian,    Self>>(&mut self, ser: S) -> Res<()> { ser.serialize(self) }
	/// Writes in forced little endian.
	fn write_le<S: Serialize<LittleEndian, Self>>(&mut self, ser: S) -> Res<()> { ser.serialize(self) }
}

// todo[trait aliases]: make these aliases of EWrite

/**
	Use this to `write` in **big** endian.

	Wrapper for `EWrite<BigEndian>`.

	This exists solely to make `use` notation work. See `EWrite` for documentation.
*/
pub trait BEWrite: Sized {
	fn write   <S: Serialize<BigEndian,    Self>>(&mut self, ser: S) -> Res<()> { ser.serialize(self) }
	fn write_be<S: Serialize<BigEndian,    Self>>(&mut self, ser: S) -> Res<()> { ser.serialize(self) }
	fn write_le<S: Serialize<LittleEndian, Self>>(&mut self, ser: S) -> Res<()> { ser.serialize(self) }
}

/**
	Use this to `write` in **little** endian.

	Wrapper for `EWrite<LittleEndian>`.

	This exists solely to make `use` notation work. See `EWrite` for documentation.
*/
pub trait LEWrite: Sized {
	fn write   <S: Serialize<LittleEndian, Self>>(&mut self, ser: S) -> Res<()> { ser.serialize(self) }
	fn write_be<S: Serialize<BigEndian,    Self>>(&mut self, ser: S) -> Res<()> { ser.serialize(self) }
	fn write_le<S: Serialize<LittleEndian, Self>>(&mut self, ser: S) -> Res<()> { ser.serialize(self) }
}

impl<W: Write, E: Endianness> EWrite<E> for W {}
impl<W: Write> BEWrite for W {}
impl<W: Write> LEWrite for W {}

#[cfg(test)]
mod tests {
	const DATA: &[u8] = b"\xba\xad";

	#[test]
	fn write_be_forced() {
		use crate::LEWrite;
		let mut writer = vec![];
		writer.write_be(0xbaadu16).unwrap();
		assert_eq!(writer, DATA);
	}

	#[test]
	fn write_le_forced() {
		use crate::BEWrite;
		let mut writer = vec![];
		writer.write_le(0xadbau16).unwrap();
		assert_eq!(&writer[..], DATA);
	}
}
