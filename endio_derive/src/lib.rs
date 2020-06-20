mod deserialize;
mod serialize;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use syn::{DeriveInput, Lit, LitInt, Meta, NestedMeta};

#[proc_macro_derive(Deserialize, attributes(disc_padding))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
	deserialize::derive(input)
}

#[proc_macro_derive(Serialize, attributes(disc_padding))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
	serialize::derive(input)
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

fn get_disc_padding(input: &DeriveInput) -> Option<LitInt> {
	for attr in &input.attrs {
		if !attr.path.is_ident("disc_padding") {
			continue;
		}
		let meta = match attr.parse_meta() {
			Err(_) => panic!("encountered unparseable disc_padding attribute"),
			Ok(x) => x,
		};
		let lit = match meta {
			Meta::NameValue(x) => x.lit,
			_ => panic!("disc_padding needs to be name=value"),
		};
		let int_lit = match lit {
			Lit::Int(x) => x,
			_ => panic!("disc_padding needs to be an integer"),
		};
		return Some(int_lit);
	}
	None
}
