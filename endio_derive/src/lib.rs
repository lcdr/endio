mod deserialize;
mod serialize;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use syn::{Attribute, DeriveInput, Field, Lit, LitInt, Meta, NestedMeta};

#[proc_macro_derive(Deserialize, attributes(post_disc_padding, padding, trailing_padding))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
	deserialize::derive(input)
}

#[proc_macro_derive(Serialize, attributes(post_disc_padding, padding, trailing_padding))]
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

fn get_padding(attrs: &Vec<Attribute>, attr_name: &str) -> Option<LitInt> {
	for attr in attrs {
		if !attr.path.is_ident(attr_name) {
			continue;
		}
		let meta = match attr.parse_meta() {
			Err(_) => panic!("encountered unparseable {} attribute", attr_name),
			Ok(x) => x,
		};
		let lit = match meta {
			Meta::NameValue(x) => x.lit,
			_ => panic!("{} needs to be name=value", attr_name),
		};
		let int_lit = match lit {
			Lit::Int(x) => x,
			_ => panic!("{} needs to be an integer", attr_name),
		};
		return Some(int_lit);
	}
	None
}

fn get_field_padding(input: &Field) -> Option<LitInt> {
	get_padding(&input.attrs, "padding")
}

fn get_post_disc_padding(input: &DeriveInput) -> Option<LitInt> {
	get_padding(&input.attrs, "post_disc_padding")
}

fn get_trailing_padding(input: &DeriveInput) -> Option<LitInt> {
	get_padding(&input.attrs, "trailing_padding")
}
