use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DataEnum, DeriveInput, Fields, LitInt, WhereClause};

use crate::{get_field_padding, get_pre_disc_padding, get_post_disc_padding, get_trailing_padding};

pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let mut input = parse_macro_input!(input as DeriveInput);
	let where_generics = &mut input.generics.clone();
	let mut where_clause = where_generics.make_where_clause();

	let name = &input.ident;

	let deser_code = match &input.data {
		Data::Struct(data) => {
			add_where_clauses_fields(&mut where_clause, &data.fields);
			gen_deser_code_struct(&data.fields)
		}
		Data::Enum(data) => {
			let ty = crate::get_enum_type(&input);
			add_where_clauses_enum(&mut where_clause, data, &ty);
			let pre_disc_padding = get_pre_disc_padding(&input);
			let post_disc_padding = get_post_disc_padding(&input);
			gen_deser_code_enum(data, &name, &ty, &pre_disc_padding, &post_disc_padding)
		}
		Data::Union(_) => unimplemented!(),
	};

	let trailing_padding = get_trailing_padding(&input);
	let read_padding = gen_read_padding(&trailing_padding);

	let (_, ty_generics, where_clause) = where_generics.split_for_impl();

	// todo[hygiene]: replace __ENDIO_ENDIANNESS, __ENDIO_READER with unique ident
	input.generics.params.push(parse_quote!(__ENDIO_ENDIANNESS: ::endio::Endianness));
	input.generics.params.push(parse_quote!(__ENDIO_READER: ::std::io::Read + ::endio::ERead<__ENDIO_ENDIANNESS>));
	let (impl_generics,	_, _) = input.generics.split_for_impl();

	let expanded = quote! {
		impl #impl_generics ::endio::Deserialize<__ENDIO_ENDIANNESS, __ENDIO_READER> for #name #ty_generics #where_clause {
			fn deserialize(reader: &mut __ENDIO_READER) -> ::std::io::Result<Self> {
				#deser_code
				#read_padding
				Ok(ret)
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
					parse_quote!(#ty: ::endio::Deserialize<__ENDIO_ENDIANNESS, __ENDIO_READER>)
				);
			}
		}
		Fields::Unnamed(fields) => {
			for f in &fields.unnamed {
				let ty = &f.ty;
				where_clause.predicates.push(
					parse_quote!(#ty: ::endio::Deserialize<__ENDIO_ENDIANNESS, __ENDIO_READER>)
				);
			}
		}
		Fields::Unit => {}
	}
}

fn gen_deser_code_fields(fields: &Fields) -> TokenStream {
	match fields {
		Fields::Named(fields) => {
			let mut deser = vec![];
			for f in &fields.named {
				let ident = &f.ident;
				let padding = get_field_padding(f);
				let read_padding = gen_read_padding(&padding);
				deser.push(quote! { #ident: {
					#read_padding
					::endio::ERead::read(reader)?
				}, });
			}
			quote! { { #(#deser)* } }
		}
		Fields::Unnamed(fields) => {
			let mut deser = vec![];
			for f in &fields.unnamed {
				let padding = get_field_padding(f);
				let read_padding = gen_read_padding(&padding);
				deser.push(quote! { {
					#read_padding
					::endio::ERead::read(reader)?
				}, });
			}
			quote! { ( #(#deser)* ) }
		}
		Fields::Unit => {
			quote! { }
		}
	}
}

fn gen_deser_code_struct(fields: &Fields) -> TokenStream {
	let deser_code = gen_deser_code_fields(fields);
	quote! { let ret = Self #deser_code; }
}

fn add_where_clauses_enum(where_clause: &mut WhereClause, data: &DataEnum, ty: &Ident) {
	where_clause.predicates.push(
		parse_quote!(#ty: ::endio::Deserialize<__ENDIO_ENDIANNESS, __ENDIO_READER>)
	);
	for var in &data.variants {
		add_where_clauses_fields(where_clause, &var.fields);
	}
}

fn gen_deser_code_enum(data: &DataEnum, name: &Ident, ty: &Ident, pre_disc_padding: &Option<LitInt>, post_disc_padding: &Option<LitInt>) -> TokenStream {
	let last_disc: syn::ExprLit = parse_quote! { 0 };
	let mut last_disc = &last_disc.into();
	let mut disc_offset = 0;
	let mut arms = vec![];
	for f in &data.variants {
		let ident = &f.ident;
		if let Some((_, x)) = &f.discriminant {
			last_disc = x;
			disc_offset = 0;
		}
		let deser_fields = gen_deser_code_fields(&f.fields);
		let arm = quote! { disc if disc == (#last_disc + (#disc_offset as #ty)) => Self::#ident #deser_fields, };
		disc_offset += 1;
		arms.push(arm);
	}
	let read_pre_padding = gen_read_padding(pre_disc_padding);
	let read_post_padding = gen_read_padding(post_disc_padding);
	quote! {
		#read_pre_padding
		let disc: #ty = ::endio::ERead::read(reader)?;
		#read_post_padding
		let ret = match disc {
			#(#arms)*
			_ => return ::std::result::Result::Err(::std::io::Error::new(::std::io::ErrorKind::InvalidData, format!("invalid discriminant value for {}: {}", stringify!(#name), disc)))
		};
	}
}

fn gen_read_padding(padding: &Option<LitInt>) -> TokenStream {
	match padding {
		Some(x) => quote! {
			let mut padding = [0; #x];
			::std::io::Read::read_exact(reader, &mut padding)?;
		},
		None => quote! { },
	}
}
