use std::io::Read;
use std::io::Result as Res;

use crate::{BigEndian, Deserialize, Endianness, LittleEndian};

/**
	Only necessary for custom (de-)serializations.

	Interface for reading data with a specified endianness. Use this interface to make deserializations automatically switch endianness without having to write the same code twice.

	In theory this would be the only write trait and BE-/LERead would be aliases to the BE/LE type parameter variants, but for some reason that doesn't import methods in `use` notation.

	## Examples

	```
	use endio::LERead;

	let mut reader = &b"\x2a\x01\xcf\xfe\xf3\x2c"[..];
	let a: u8 = reader.read().unwrap();
	let b: bool = reader.read().unwrap();
	let c: u32 = reader.read().unwrap();
	assert_eq!(a, 42);
	assert_eq!(b, true);
	assert_eq!(c, 754187983);
	```
*/
pub trait ERead<E: Endianness>: Sized {
	/**
		Reads a `Deserialize` from the reader, in the reader's endianness.

		What's actually read is up to the implementation of the `Deserialize`.
	*/
	fn read   <D: Deserialize<E,            Self>>(&mut self) -> Res<D> { D::deserialize(self) }
	/// Reads in forced big endian.
	fn read_be<D: Deserialize<BigEndian,    Self>>(&mut self) -> Res<D> { D::deserialize(self) }
	/// Reads in forced little endian.
	fn read_le<D: Deserialize<LittleEndian, Self>>(&mut self) -> Res<D> { D::deserialize(self) }
}

/**
	Use this to `read` in **big** endian.

	Wrapper for `ERead<BigEndian>`.

	This exists solely to make `use` notation work. See `ERead` for documentation.
*/
pub trait BERead: Sized {
	fn read   <D: Deserialize<BigEndian,    Self>>(&mut self) -> Res<D> { D::deserialize(self) }
	fn read_be<D: Deserialize<BigEndian,    Self>>(&mut self) -> Res<D> { D::deserialize(self) }
	fn read_le<D: Deserialize<LittleEndian, Self>>(&mut self) -> Res<D> { D::deserialize(self) }
}

/**
	Use this to `read` in **little** endian.

	Wrapper for `ERead<LittleEndian>`.

	This exists solely to make `use` notation work. See `ERead` for documentation.
*/
pub trait LERead: Sized {
	fn read   <D: Deserialize<LittleEndian, Self>>(&mut self) -> Res<D> { D::deserialize(self) }
	fn read_be<D: Deserialize<BigEndian,    Self>>(&mut self) -> Res<D> { D::deserialize(self) }
	fn read_le<D: Deserialize<LittleEndian, Self>>(&mut self) -> Res<D> { D::deserialize(self) }
}

impl<R: Read, E: Endianness> ERead<E> for R {}
impl<R: Read> BERead for R {}
impl<R: Read> LERead for R {}

#[cfg(test)]
mod tests {
	const DATA: &[u8] = b"\xba\xad";

	#[test]
	fn read_be_forced() {
		use super::LERead;
		let mut reader = &DATA[..];
		let val: u16 = reader.read_be().unwrap();
		assert_eq!(val, 0xbaad);
	}

	#[test]
	fn read_le_forced() {
		use crate::BERead;
		let mut reader = &DATA[..];
		let val: u16 = reader.read_le().unwrap();
		assert_eq!(val, 0xadba);
	}
}
