/**
	Only necessary for custom (de-)serializations.

	You can use this as a blanket impl trait bound to write code that is not endian-specific.

	You can't implement this trait, it only exists as a trait bound.
*/
pub trait Endianness: private::Sealed {}

/**
	Only necessary for custom (de-)serializations.

	You can use this as a type parameter in your implementation to write code specific to big endian.
*/
pub struct BigEndian;
/**
	Only necessary for custom (de-)serializations.

	You can use this as a type parameter in your implementation to write code specific to little endian.
*/
pub struct LittleEndian;

impl Endianness for BigEndian {}
impl Endianness for LittleEndian {}

// ensures no one else implements the trait
mod private {
	pub trait Sealed {}

	impl Sealed for super::BigEndian {}
	impl Sealed for super::LittleEndian {}
}
