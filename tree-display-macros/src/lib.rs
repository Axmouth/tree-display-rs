#![allow(unused_imports)]
extern crate proc_macro;
use ::proc_macro::TokenStream;
use ::proc_macro2::{Span, TokenStream as TokenStream2};
use ::quote::{quote, quote_spanned, ToTokens};
use ::syn::{
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
    Result, *,
};

#[proc_macro_derive(TreeDisplay)]
pub fn rule_system_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as _);

    let out = TokenStream::from(match impl_my_trait(ast) {
        Ok(it) => it,
        Err(err) => err.to_compile_error(),
    });
    // println!("{}", out);
    out
}

fn impl_my_trait(ast: DeriveInput) -> Result<TokenStream2> {
    Ok({
        let name = ast.ident;
        let where_clause = ast.generics.where_clause.clone();
        let generics = ast.generics;

        match ast.data {
            Data::Enum(DataEnum {
                enum_token: token::Enum { span: _ },
                brace_token: _,
                variants,
            }) => {
                for variant in variants {
                    println!(
                        "{} {:?}",
                        variant.ident,
                        variant
                            .fields
                            .iter()
                            .map(|f| f.ident.as_ref())
                            .collect::<Vec<_>>()
                    );
                }

                let name_span = name.span();
                let name_stringified = LitStr::new(&name.to_string(), name_span);
                quote! {
                    impl TreeDisplay for #name {
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, indent: &str, show_types: bool) -> ::std::fmt::Result {
                            writeln!(f, "({})", #name_stringified)?;
                            writeln!(f, "{}|", indent)?;
                            Ok(())
                        }
                    }
                }
            }
            Data::Union(DataUnion {
                union_token: token::Union { span: _ },
                fields: _,
            }) => {
                let name_span = name.span();
                let name_stringified = LitStr::new(&name.to_string(), name_span);

                quote! {
                    impl TreeDisplay for #name {
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, indent: &str, show_types: bool) -> ::std::fmt::Result {
                            writeln!(f, "({})", #name_stringified)?;
                            writeln!(f, "{}|", indent)?;
                            Ok(())
                        }
                    }
                }
            }

            Data::Struct(DataStruct {
                fields: Fields::Named(fields),
                ..
            }) => {
                let field_count = fields.named.len();
                let data_expanded_members =
                    fields.named.into_iter().enumerate().map(|(i, field)| {
                        let field_name = field.ident.expect("Unreachable field name");
                        let name_span = field_name.span();
                        let field_name_stringified =
                            LitStr::new(&field_name.to_string(), name_span);
                        if field_count - 1 > i {
                            quote_spanned! { name_span =>
                                let mut indent_modified = indent.to_string();
                                indent_modified.push_str("|  ");
                                write!(f, "{}├──{} ", indent, #field_name_stringified)?;

                                TreeDisplay::tree_fmt(&self.#field_name, f, &indent_modified, show_types)?;
                            }
                        } else {
                            quote_spanned! { name_span =>
                                let mut indent_modified = indent.to_string();
                                indent_modified.push_str("   ");
                                write!(f, "{}└──{} ", indent, #field_name_stringified)?;

                                TreeDisplay::tree_fmt(&self.#field_name, f, &indent_modified, show_types)?;
                            }
                        }
                    });

                let span = name.span();
                let name_stringified = LitStr::new(&name.to_string(), span);
                quote! {
                    impl #generics TreeDisplay for #name #generics #where_clause {
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, indent: &str, show_types: bool) -> ::std::fmt::Result {
                            if show_types {
                                writeln!(f, "({})", #name_stringified)?;
                            } else {
                                writeln!(f, "")?;
                            }

                            writeln!(f, "{}|", indent)?;
                            #(#data_expanded_members)*
                            Ok(())
                        }
                    }
                }
            }

            Data::Struct(DataStruct {
                fields: Fields::Unnamed(fields),
                ..
            }) => {
                let field_count = fields.unnamed.len();
                let data_expanded_members =
                    fields.unnamed.into_iter().enumerate().map(|(i, field)| {
                        let field_name = field.ty;
                        let name_span = field_name.span();
                        let field_accessor = syn::Index::from(i);
                        if field_count - 1 > i {
                            quote_spanned! { name_span =>
                                let mut indent_modified = indent.to_string();
                                indent_modified.push_str("|  ");
                                write!(f, "{}├──{}", indent, #field_accessor)?;
                                TreeDisplay::tree_fmt(&self.#field_accessor, f, &indent_modified, show_types)?;
                            }
                        } else {
                            quote_spanned! { name_span =>
                                let mut indent_modified = indent.to_string();
                                indent_modified.push_str("   ");
                                write!(f, "{}└──{}", indent, #field_accessor)?;
                                TreeDisplay::tree_fmt(&self.#field_accessor, f, &indent_modified, show_types)?;
                            }
                        }
                    });

                let span = name.span();
                let name_stringified = LitStr::new(&name.to_string(), span);
                quote! {
                    impl #generics TreeDisplay for #name #generics #where_clause {
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, indent: &str, show_types: bool) -> ::std::fmt::Result {
                            writeln!(f, "({})", #name_stringified)?;
                            writeln!(f, "{}|", indent)?;
                            #(#data_expanded_members)*
                            Ok(())
                        }
                    }
                }
            }

            Data::Struct(DataStruct {
                fields: Fields::Unit,
                ..
            }) => {
                let span = name.span();
                let name_stringified = LitStr::new(&name.to_string(), span);
                quote! {
                    impl #generics TreeDisplay for #name #generics #where_clause {
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, indent: &str, show_types: bool) -> ::std::fmt::Result {
                            writeln!(f, "({})", #name_stringified)?;
                            Ok(())
                        }
                    }
                }
            }
        }
    })
}
