extern crate proc_macro;
use ::proc_macro::TokenStream;
use ::proc_macro2::{Group, Span, TokenStream as TokenStream2, TokenTree};
use ::quote::quote;
use ::syn::{Result, *};
use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};

// TODO: implement attribute parsing
// > transparent
// makes content appear as if it was in the current object
// > skip
// does not render
// > skip if
// does not render if condition(pass function?)
// > rename (field)
// > rename_all (for all struct etc fields)
// Can rename to pattern or specific tag
// More serde like attributes
#[proc_macro_derive(TreeDisplay, attributes(tree_display))]
pub fn rule_system_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as _);

    let out = TokenStream::from(match impl_my_trait(ast) {
        Ok(it) => it,
        Err(err) => err.to_compile_error(),
    });
    // println!("{}", out);
    out
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum DisplayType {
    Transparent,
    Skip,
    SkipIf(TokenStream2),
    SkipIfFalse,
    SkipIfTrue,
    SkipIfNone,
    SkipIfEmpty,
    Rename(String),
    RenamePascal,
    RenameSnake,
    RenameKebab,
    RenameCamel,
    RenameAllPascal,
    RenameAllSnake,
    RenameAllKebab,
    RenameAllCamel,
}

#[derive(Debug, Clone)]
enum SkipType {
    Always,
    If(TokenStream2),
    IfEmpty,
    IfFalse,
    IfTrue,
    IfNone,
}

impl SkipType {
    fn from_display_type(display_type: &DisplayType) -> Option<SkipType> {
        match display_type {
            DisplayType::Skip => Some(SkipType::Always),
            DisplayType::SkipIf(stream) => Some(SkipType::If(stream.clone())),
            DisplayType::SkipIfFalse => Some(SkipType::IfFalse),
            DisplayType::SkipIfTrue => Some(SkipType::IfTrue),
            DisplayType::SkipIfNone => Some(SkipType::IfNone),
            DisplayType::SkipIfEmpty => Some(SkipType::IfEmpty),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum RenameType {
    Str(String),
    Pascal,
    Snake,
    Kebab,
    Camel,
}

impl RenameType {
    fn from_display_type(display_type: &DisplayType) -> Option<RenameType> {
        match display_type {
            DisplayType::Rename(s) => Some(RenameType::Str(s.clone())),
            DisplayType::RenamePascal => Some(RenameType::Pascal),
            DisplayType::RenameSnake => Some(RenameType::Snake),
            DisplayType::RenameKebab => Some(RenameType::Kebab),
            DisplayType::RenameCamel => Some(RenameType::Camel),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RenameAllType {
    Pascal,
    Snake,
    Kebab,
    Camel,
}

impl RenameAllType {
    fn from_display_type(display_type: &DisplayType) -> Option<RenameAllType> {
        match display_type {
            DisplayType::RenameAllPascal => Some(RenameAllType::Pascal),
            DisplayType::RenameAllSnake => Some(RenameAllType::Snake),
            DisplayType::RenameAllKebab => Some(RenameAllType::Kebab),
            DisplayType::RenameAllCamel => Some(RenameAllType::Camel),
            _ => None,
        }
    }
}

trait VecExt {
    fn try_get_rename(&self) -> Result<Option<RenameType>>;
    fn try_get_rename_all(&self) -> Result<Option<RenameAllType>>;
    fn try_get_transparent(&self) -> Result<Option<()>>;
    fn try_get_skip(&self) -> Result<Option<SkipType>>;
}

impl VecExt for Vec<DisplayType> {
    fn try_get_rename(&self) -> Result<Option<RenameType>> {
        let mut iter = self.iter().filter_map(|d| RenameType::from_display_type(d));
        let rename = iter.next();
        if iter.next().is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Only one rename attribute is allowed",
            ));
        }
        Ok(rename)
    }

    fn try_get_rename_all(&self) -> Result<Option<RenameAllType>> {
        let mut iter = self
            .iter()
            .filter_map(|d| RenameAllType::from_display_type(d));
        let rename_all = iter.next();
        if iter.next().is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Only one rename_all attribute is allowed",
            ));
        }
        Ok(rename_all)
    }

    fn try_get_transparent(&self) -> Result<Option<()>> {
        let mut iter = self
            .iter()
            .filter(|&d| matches!(d, &DisplayType::Transparent))
            .map(|_| ());
        let transparent = iter.next();
        if iter.next().is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Only one transparent attribute is allowed",
            ));
        }
        Ok(transparent)
    }

    fn try_get_skip(&self) -> Result<Option<SkipType>> {
        let mut iter = self.iter().filter_map(|d| SkipType::from_display_type(d));
        let skip = iter.next();
        if iter.next().is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Only one skip attribute is allowed",
            ));
        }
        Ok(skip)
    }
}

#[derive(Debug, Clone)]
struct DisplayAttrs {
    transparent: bool,
    skip: Option<SkipType>,
    rename: Option<RenameType>,
    rename_all: Option<RenameAllType>,
}

fn spanned_tokens(s: &syn::LitStr) -> parse::Result<TokenStream2> {
    let stream = syn::parse_str(&s.value())?;
    Ok(respan(stream, s.span()))
}

fn respan(stream: TokenStream2, span: Span) -> TokenStream2 {
    stream
        .into_iter()
        .map(|token| respan_token(token, span))
        .collect()
}

fn respan_token(mut token: TokenTree, span: Span) -> TokenTree {
    if let TokenTree::Group(g) = &mut token {
        *g = Group::new(g.delimiter(), respan(g.stream(), span));
    }
    token.set_span(span);
    token
}

fn rename_named_field(name: String, rename_type: RenameType) -> String {
    match rename_type {
        RenameType::Str(s) => s,
        RenameType::Pascal => name.to_pascal_case(),
        RenameType::Snake => name.to_snake_case(),
        RenameType::Kebab => name.to_kebab_case(),
        RenameType::Camel => name.to_lower_camel_case(),
    }
}

fn parse_attributes(attrs: &[syn::Attribute]) -> Result<DisplayAttrs> {
    let attrs = attrs
        .iter()
        .filter_map(|attr| {
            let meta = match attr.parse_meta() {
                Ok(meta) => meta,
                Err(e) => return Some(Err(e)),
            };
            Ok(if let syn::Meta::List(list) = meta {
                if list.path.is_ident("tree_display") {
                    Some(list.nested.into_iter().filter_map(|n| Ok(match n {
                        syn::NestedMeta::Meta(syn::Meta::Path(path)) => {
                            if path.is_ident("transparent") {
                                Some(DisplayType::Transparent)
                            } else if path.is_ident("skip") {
                                Some(DisplayType::Skip)
                            } else if path.is_ident("skip_if") {
                                return Some(Err(syn::Error::new(Span::call_site(), "rename requires a string literal as an argument, that contains a function")));
                            } else if path.is_ident("skip_if_false") {
                                Some(DisplayType::SkipIfFalse)
                            } else if path.is_ident("skip_if_true") {
                                Some(DisplayType::SkipIfTrue)
                            } else if path.is_ident("skip_if_none") {
                                Some(DisplayType::SkipIfNone)
                            } else if path.is_ident("skip_if_empty") {
                                Some(DisplayType::SkipIfEmpty)
                            } else if path.is_ident("rename") {
                                return Some(Err(syn::Error::new(Span::call_site(), "rename requires a string literal as an argument")));
                            } else if path.is_ident("rename_pascal") {
                                Some(DisplayType::RenamePascal)
                            } else if path.is_ident("rename_snake") {
                                Some(DisplayType::RenameSnake)
                            } else if path.is_ident("rename_kebab") {
                                Some(DisplayType::RenameKebab)
                            } else if path.is_ident("rename_camel") {
                                Some(DisplayType::RenameCamel)
                            } else if path.is_ident("rename_all_pascal") {
                                Some(DisplayType::RenameAllPascal)
                            } else if path.is_ident("rename_all_snake") {
                                Some(DisplayType::RenameAllSnake)
                            } else if path.is_ident("rename_all_kebab") {
                                Some(DisplayType::RenameAllKebab)
                            } else if path.is_ident("rename_all_camel") {
                                Some(DisplayType::RenameAllCamel)
                            } else {
                                None
                            }
                        }
                        syn::NestedMeta::Meta(syn::Meta::NameValue(name)) => {
                            if let syn::Lit::Str(lit_str) = &name.lit {
                                if name.path.is_ident("rename") {
                                    Some(DisplayType::Rename(lit_str.value()))
                                } else if name.path.is_ident("skip_if") {
                                    let tokens = spanned_tokens(lit_str).expect("Failed to parse tokens for skip_if");
                                    let parsed = syn::parse2(tokens).expect("Failed to parse skip_if");
                                    Some(DisplayType::SkipIf(parsed))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }).transpose()).collect::<Result<Vec<_>>>())
                } else {
                    None
                }
            } else {
                None
            }).transpose()
        })
        .flatten()
        .collect::<Result<Vec<_>>>()?;
    let attrs = attrs.into_iter().flatten().collect::<Vec<_>>();
    println!("{:#?}", attrs);
    Ok(DisplayAttrs {
        transparent: attrs.try_get_transparent()?.is_some(),
        skip: attrs.try_get_skip()?,
        rename: attrs.try_get_rename()?,
        rename_all: attrs.try_get_rename_all()?,
    })
}

fn gen_named_fields(fields: FieldsNamed) -> Result<TokenStream2> {
    let field_render_tuple_code = fields.named.iter().map(|_| {
        quote! {
            true,
        }
    });
    let fields_code = fields.named.iter().enumerate().map(move |(i, field)| {
        // TODO: expect to error
        let attrs = parse_attributes(&field.attrs).expect("Unable to parse attributes");
        if attrs.rename_all.is_some() {
            return Err(syn::Error::new(Span::call_site(), "rename_all can only be used on a struct"));
        }
        let field_name = field.ident.as_ref().ok_or(syn::Error::new(Span::call_site(), "Fields must have a name"))?;
        let name_span = field_name.span();
        let field_name_string = attrs.rename.map(|rename_type| rename_named_field(field_name.to_string(), rename_type)).unwrap_or(field_name.to_string());
        eprintln!("{:?}", field_name_string);
        let field_name_stringified = LitStr::new(&field_name_string, name_span);
        let to_render_index = syn::Index::from(i);
        let skip_code = if let Some(skip) = attrs.skip {
            let condition = if let SkipType::If(skip_if) = skip {
                quote! {
                    if #skip_if (self.#field_name) {
                        to_render. #to_render_index = false;
                    } else {
                        last_field = #i;
                    }
                }
            } else if let SkipType::IfFalse = skip {
                quote! {
                    !(self.#field_name)
                }
             } else if let SkipType::IfTrue = skip {
                quote! {
                    (self.#field_name)
                }
            } else if let SkipType::IfNone = skip {
                quote! {
                    (self.#field_name).is_none()
                }
            } else if let SkipType::IfEmpty = skip {
                quote! {
                    (self.#field_name).is_empty()
                }
            } else if let SkipType::Always = skip {
                quote! {
                    true
                }
            } else {
                unreachable!()
            };
            quote! {
                if #condition {
                    to_render. #to_render_index = false;
                } else {
                    last_field = #i;
                }
            }
        } else {
            quote! {
                last_field = #i;
            }
        };
        let render_code = quote! {
            let mut indent_modified = indent.to_string();
            if to_render. #to_render_index {
                eprintln!("{}", #field_name_stringified);
                if last_field > #i {
                    indent_modified.push_str("|  ");
                    write!(f, "{}├──{}", indent, #field_name_stringified)?;
                } else {
                    indent_modified.push_str("   ");
                    write!(f, "{}└──{}", indent, #field_name_stringified)?;
                }
                if show_types {
                    self.#field_name.type_name_fmt(f)?;
                }
                writeln!(f)?;

                tree_display::TreeDisplay::tree_fmt(&self.#field_name, f, &indent_modified, show_types, dense)?;
            }
        };
        Ok((skip_code, render_code))
    }).collect::<Result<Vec<_>>>()?;
    let (skipping, rendering): (Vec<_>, Vec<_>) = fields_code.into_iter().unzip();
    Ok(quote! {
        let mut last_field: usize = 0;
        let mut to_render = ( #(#field_render_tuple_code)* );
        #(#skipping)*
        #(#rendering)*
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

                let named_fields_code = gen_named_fields(fields)?;

                quote! {
                    impl tree_display::TreeDisplay for #name {
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, indent: &str, show_types: bool, dense: bool) -> ::std::fmt::Result {
                            if !dense {
                                writeln!(f, "{}|", indent)?;
                            }
                            #named_fields_code
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
                let named_fields_code = gen_named_fields(fields)?;
                quote! {
                    impl #generics tree_display::TreeDisplay for #name #generics #where_clause {
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, indent: &str, show_types: bool, dense: bool) -> ::std::fmt::Result {
                            if !dense {
                                writeln!(f, "{}|", indent)?;
                            }
                            #named_fields_code
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
