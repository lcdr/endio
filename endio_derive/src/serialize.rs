use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DataEnum, DeriveInput, Fields, LitInt, Generics, WhereClause};

use crate::{get_field_padding, get_pre_disc_padding, get_post_disc_padding, get_trailing_padding};

pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let mut input = parse_macro_input!(input as DeriveInput);
	let where_generics = &mut input.generics.clone();
	let mut where_clause = where_generics.make_where_clause();
	let ser_code;

	let name = &input.ident;

	match &input.data {
		Data::Struct(data) => {
			add_where_clauses_fields(&mut where_clause, &data.fields);
			ser_code = gen_ser_code_struct(&data.fields, &name);
		}
		Data::Enum(data) => {
			let ty = crate::get_enum_type(&input);
			add_where_clauses_enum(&mut where_clause, data, &ty);
			let pre_disc_padding = get_pre_disc_padding(&input);
			let post_disc_padding = get_post_disc_padding(&input);
			ser_code = gen_ser_code_enum(data, &name, &ty, &pre_disc_padding, &post_disc_padding, &input.generics);
		}
		Data::Union(_) => unimplemented!(),
	};

	let trailing_padding = get_trailing_padding(&input);
	let write_padding = gen_write_padding(&trailing_padding);

	let (_, ty_generics, where_clause) = where_generics.split_for_impl();

	// todo[hygiene]: replace __ENDIO_LIFETIME, __ENDIO_ENDIANNESS, __ENDIO_WRITER with unique ident
	input.generics.params.push(parse_quote!('__ENDIO_LIFETIME));
	input.generics.params.push(parse_quote!(__ENDIO_ENDIANNESS: ::endio::Endianness));
	input.generics.params.push(parse_quote!(__ENDIO_WRITER: ::std::io::Write + ::endio::EWrite<__ENDIO_ENDIANNESS>));
	let (impl_generics,	_, _) = input.generics.split_for_impl();

	let expanded = quote! {
		impl #impl_generics ::endio::Serialize<__ENDIO_ENDIANNESS, __ENDIO_WRITER> for &'__ENDIO_LIFETIME #name #ty_generics #where_clause {
			fn serialize(self, writer: &mut __ENDIO_WRITER) -> ::std::io::Result<()> {
				#ser_code
				#write_padding
				Ok(())
			}
		}
	};

	expanded.into()
}

fn add_where_clauses_fields(where_clause: &mut WhereClause, fields: &Fields) {
	match fields {
		Fields::Named(fields) => {
			for f in &fields.named {
				let ty = &f.ty;
				where_clause.predicates.push(
					parse_quote!(&'__ENDIO_LIFETIME #ty: ::endio::Serialize<__ENDIO_ENDIANNESS, __ENDIO_WRITER>)
				);
			}
		}
		Fields::Unnamed(fields) => {
			for f in &fields.unnamed {
				let ty = &f.ty;
				where_clause.predicates.push(
					parse_quote!(&'__ENDIO_LIFETIME #ty: ::endio::Serialize<__ENDIO_ENDIANNESS, __ENDIO_WRITER>)
				);
			}
		}
		Fields::Unit => {}
	}
}

fn gen_ser_code_fields(fields: &Fields) -> TokenStream {
	match fields {
		Fields::Named(fields) => {
			let mut pat = vec![];
			let mut ser = vec![];
			for f in &fields.named {
				let ident = &f.ident;
				let padding = get_field_padding(f);
				let write_padding = gen_write_padding(&padding);
				pat.push(quote! { #ident, });
				ser.push(quote! {
					#write_padding
					::endio::EWrite::write(writer, #ident)?;
				});
			}
			quote! { { #(#pat)* } => { #(#ser)* } }
		}
		Fields::Unnamed(fields) => {
			let mut index = String::from("a");
			let mut pat = vec![];
			let mut ser = vec![];
			for f in &fields.unnamed {
				let ident = Ident::new(&index, Span::call_site());
				let padding = get_field_padding(f);
				let write_padding = gen_write_padding(&padding);
				pat.push(quote! { #ident, });
				ser.push(quote! {
					#write_padding
					::endio::EWrite::write(writer, #ident)?;
				});
				index += "a";
			}
			quote! { ( #(#pat)* ) => { #(#ser)* } }
		}
		Fields::Unit => {
			quote! { => {} }
		}
	}
}

fn gen_ser_code_struct(fields: &Fields, name: &Ident) -> TokenStream {
	let ser_code = gen_ser_code_fields(fields);
	quote! {
		match self {
			#name #ser_code
		}
	}
}

fn add_where_clauses_enum(where_clause: &mut WhereClause, data: &DataEnum, ty: &Ident) {
	where_clause.predicates.push(
		parse_quote!(#ty: ::endio::Serialize<__ENDIO_ENDIANNESS, __ENDIO_WRITER>)
	);
	for var in &data.variants {
		add_where_clauses_fields(where_clause, &var.fields);
	}
}

fn gen_ser_code_enum(data: &DataEnum, name: &Ident, ty: &Ident, pre_disc_padding: &Option<LitInt>, post_disc_padding: &Option<LitInt>, generics: &Generics) -> TokenStream {
	let mut arms = vec![];
	for f in &data.variants {
		let ident = &f.ident;
		let ser_fields = gen_ser_code_fields(&f.fields);
		let expanded = quote! { #name::#ident #ser_fields };
		arms.push(expanded);
	}
	let write_pre_padding = gen_write_padding(pre_disc_padding);
	let write_post_padding = gen_write_padding(post_disc_padding);
	quote! {
		#write_pre_padding
		let disc = unsafe { *(self as *const #name #generics as *const #ty) };
		::endio::EWrite::write(writer, disc)?;
		#write_post_padding
		match self {
			#(#arms)*
		}
	}
}

fn gen_write_padding(padding: &Option<LitInt>) -> TokenStream {
	match padding {
		Some(x) => quote! {
			let mut padding = [0; #x];
			::std::io::Write::write_all(writer, &padding)?;
		},
		None => quote! { },
	}
}
