mod deserialize;
mod serialize;

use proc_macro2::Ident;
use syn::{DeriveInput, Meta, NestedMeta};

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	deserialize::derive_deserialize(input)
}

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	serialize::derive_serialize(input)
}

fn get_enum_type(input: &DeriveInput) -> Ident {
	for attr in &input.attrs {
		if !attr.path.is_ident("repr") {
			continue;
		}
		let meta = match attr.parse_meta() {
			Err(_) => panic!("encountered unparseable repr attribute"),
			Ok(x) => x,
		};
		let list = match meta {
			Meta::List(x) => x,
			_ => continue,
		};
		if list.nested.is_empty() {
			panic!("encountered repr attribute with no arguments");
		}
		for nested_meta in list.nested {
			let meta = match nested_meta {
				NestedMeta::Meta(x) => x,
				NestedMeta::Lit(_) => continue,
			};
			let path = match meta {
				Meta::Path(x) => x,
				_ => continue,
			};
			if path.is_ident("C") || path.is_ident("transparent") {
				continue;
			}
			return (*path.get_ident().expect("invalid repr attribute argument")).clone();
		}
	}
	panic!("You need to add a repr attribute to specify the discriminant type, e.g. #[repr(u16)]");
}
