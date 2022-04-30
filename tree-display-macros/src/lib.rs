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
    println!("{}", out);
    out
}

fn gen_named_fields(fields: FieldsNamed) -> impl Iterator<Item = TokenStream2> {
    let field_count = fields.named.len();
    fields.named.into_iter().enumerate().map(move |(i, field)| {
        let field_name = field.ident.expect("Unreachable field name");
        let name_span = field_name.span();
        let field_name_stringified = LitStr::new(&field_name.to_string(), name_span);
        if field_count - 1 > i {
            quote_spanned! { name_span =>
                let mut indent_modified = indent.to_string();
                indent_modified.push_str("|  ");
                write!(f, "{}├──{}", indent, #field_name_stringified)?;
                if show_types {
                    self.#field_name.type_name_fmt(f)?;
                }
                writeln!(f)?;

                tree_display::TreeDisplay::tree_fmt(&self.#field_name, f, &indent_modified, show_types, dense)?;
            }
        } else {
            quote_spanned! { name_span =>
                let mut indent_modified = indent.to_string();
                indent_modified.push_str("   ");
                write!(f, "{}└──{}", indent, #field_name_stringified)?;
                if show_types {
                    self.#field_name.type_name_fmt(f)?;
                }
                writeln!(f)?;

                tree_display::TreeDisplay::tree_fmt(&self.#field_name, f, &indent_modified, show_types, dense)?;
            }
        }
    })
}

fn gen_unnamed_fields(fields: FieldsUnnamed) -> impl Iterator<Item = TokenStream2> {
    let field_count = fields.unnamed.len();
    fields
        .unnamed
        .into_iter()
        .enumerate()
        .map(move |(i, _)| {
            let field_accessor = syn::Index::from(i);
            if field_count - 1 > i {
                quote! {
                    let mut indent_modified = indent.to_string();
                    indent_modified.push_str("|  ");
                    write!(f, "{}├──{}", indent, #field_accessor)?;
                    if show_types {
                        self.#field_accessor.type_name_fmt(f)?;
                    }
                    writeln!(f)?;
                    tree_display::TreeDisplay::tree_fmt(&self.#field_accessor, f, &indent_modified, show_types, dense)?;
                }
            } else {
                quote! {
                    let mut indent_modified = indent.to_string();
                    indent_modified.push_str("   ");
                    write!(f, "{}└──{}", indent, #field_accessor)?;
                    if show_types {
                        self.#field_accessor.type_name_fmt(f)?;
                    }
                    writeln!(f)?;
                    tree_display::TreeDisplay::tree_fmt(&self.#field_accessor, f, &indent_modified, show_types, dense)?;
                }
            }
        })
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
                let name_span = name.span();
                let name_stringified = LitStr::new(&name.to_string(), name_span);

                let variants_code = variants.into_iter().map(|v| {
                    let variant_name_stringified =
                        LitStr::new(&v.ident.to_string(), v.ident.span());
                    let variant_name = v.ident;

                    let variant_name_code = quote! {
                        writeln!(f, "{}└──{}", indent, #variant_name_stringified)?;
                        if !dense {
                            writeln!(f, "{}   |", indent)?;
                        }
                        let mut indent_modified = indent.to_string();
                        indent_modified.push_str("   ");
                    };

                    match v.fields {
                        Fields::Named(named) => {
                            let mut i = 0;
                            let fields = named
                                .named
                                .iter()
                                .map(|f| f.ident.clone())
                                .map(|f| {
                                    f.unwrap_or_else(|| {
                                        i += 1;
                                        Ident::new(&format!("__field_{}", i), Span::call_site())
                                    })
                                })
                                .collect::<Vec<_>>();
                            let destructure_code = quote! {
                                #(#fields ,)*
                            };
                            let fields_fmt = fields.iter().enumerate().map(|(pos, ident)| {
                                let field_stringified =
                                    LitStr::new(&ident.to_string(), ident.span());
                                if pos >= fields.len() - 1 {
                                    quote! {
                                        write!(f, "{}└──{}", indent_modified, #field_stringified)?;
                                        if show_types {
                                            #ident.type_name_fmt(f)?;
                                        }
                                        writeln!(f)?;
                                        let mut indent_modified2 = indent_modified.to_string();
                                        indent_modified2.push_str("   ");
                                        #ident.tree_fmt(f, &indent_modified2, show_types, dense)?;
                                    }
                                } else {
                                    quote! {
                                        write!(f, "{}├──{}", indent_modified, #field_stringified)?;
                                        if show_types {
                                            #ident.type_name_fmt(f)?;
                                        }
                                        writeln!(f)?;
                                        let mut indent_modified2 = indent_modified.to_string();
                                        indent_modified2.push_str("|  ");
                                        #ident.tree_fmt(f, &indent_modified2, show_types, dense)?;
                                    }
                                }
                            });

                            quote! {
                                #name::#variant_name { #destructure_code } => {
                                    #variant_name_code
                                    #(#fields_fmt)*
                                }
                            }
                        }
                        Fields::Unnamed(unnamed) => {
                            let mut i = 0;
                            let fields = unnamed
                                .unnamed
                                .iter()
                                .map(|f| f.ident.clone())
                                .map(|f| {
                                    f.unwrap_or_else(|| {
                                        i += 1;
                                        Ident::new(&format!("__field_{}", i), Span::call_site())
                                    })
                                })
                                .collect::<Vec<_>>();
                            let destructure_code = quote! {
                                #(#fields ,)*
                            };
                            let fields_fmt = fields.iter().map(|ident| {
                                quote! {
                                    #ident.tree_fmt(f, &indent_modified, show_types, dense)?;
                                }
                            });

                            quote! {
                                #name::#variant_name(#destructure_code) => {
                                    #variant_name_code
                                    #(#fields_fmt)*
                                }
                            }
                        }
                        Fields::Unit => quote! {
                                #name::#variant_name => {
                                    writeln!(f, "{}└──{}", indent, #variant_name_stringified)?;
                                    if !dense {
                                        writeln!(f, "{}    ", indent)?;
                                    }
                                    let mut indent_modified = indent.to_string();
                                    indent_modified.push_str("   ");
                            }
                        },
                    }
                });

                quote! {
                    impl tree_display::TreeDisplay for #name {
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, indent: &str, show_types: bool, dense: bool) -> ::std::fmt::Result {
                            if !dense {
                                writeln!(f, "{}|", indent)?;
                            }
                            match self {
                                #(#variants_code)*
                            }
                            Ok(())
                        }

                        fn type_name_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            write!(f, " ({})", #name_stringified)
                        }
                    }
                }
            }
            Data::Union(DataUnion {
                union_token: token::Union { span: _ },
                fields,
            }) => {
                let name_span = name.span();
                let name_stringified = LitStr::new(&name.to_string(), name_span);

                let named_fields_code = gen_named_fields(fields);

                quote! {
                    impl tree_display::TreeDisplay for #name {
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, indent: &str, show_types: bool, dense: bool) -> ::std::fmt::Result {
                            if !dense {
                                writeln!(f, "{}|", indent)?;
                            }
                            #(#named_fields_code)*
                            Ok(())
                        }

                        fn type_name_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            write!(f, " ({})", #name_stringified)
                        }
                    }
                }
            }

            Data::Struct(DataStruct {
                fields: Fields::Named(fields),
                ..
            }) => {
                let span = name.span();
                let name_stringified = LitStr::new(&name.to_string(), span);
                let named_fields_code = gen_named_fields(fields);
                quote! {
                    impl #generics tree_display::TreeDisplay for #name #generics #where_clause {
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, indent: &str, show_types: bool, dense: bool) -> ::std::fmt::Result {
                            if !dense {
                                writeln!(f, "{}|", indent)?;
                            }
                            #(#named_fields_code)*
                            Ok(())
                        }

                        fn type_name_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            write!(f, " ({})", #name_stringified)
                        }
                    }
                }
            }

            Data::Struct(DataStruct {
                fields: Fields::Unnamed(fields),
                ..
            }) => {
                let span = name.span();
                let name_stringified = LitStr::new(&name.to_string(), span);
                let unnamed_fields_code = gen_unnamed_fields(fields);
                quote! {
                    impl #generics tree_display::TreeDisplay for #name #generics #where_clause {
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, indent: &str, show_types: bool, dense: bool) -> ::std::fmt::Result {
                            if !dense {
                                writeln!(f, "{}|", indent)?;
                            }
                            #(#unnamed_fields_code)*
                            Ok(())
                        }

                        fn type_name_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            write!(f, " ({})", #name_stringified)
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
                    impl #generics tree_display::TreeDisplay for #name #generics #where_clause {
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, indent: &str, show_types: bool, dense: bool) -> ::std::fmt::Result {
                            if !dense {
                                writeln!(f, "{}", indent)?;
                            }
                            Ok(())
                        }

                        fn type_name_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            write!(f, " ({})", #name_stringified)
                        }
                    }
                }
            }
        }
    })
}
