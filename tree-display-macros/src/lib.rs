extern crate proc_macro;
use ::proc_macro::TokenStream;
use ::proc_macro2::{Group, Span, TokenStream as TokenStream2, TokenTree};
use ::quote::quote;
use ::syn::{Result, *};
use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};

#[proc_macro_derive(TreeDisplay, attributes(tree_display))]
pub fn rule_system_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as _);

    TokenStream::from(match impl_my_trait(ast) {
        Ok(it) => it,
        Err(err) => err.to_compile_error(),
    })
}

// TODO: remove fields/variants with skip on code generation level
// TODO: Add ctx to pass parameters down
// TODO: change dense to sparcity with Option<usize> for how many "gaps"
// TODO: consider moving more logic downward for attributes that require field info to be implemented more easily
// TODO: trim indent when making newline

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum DisplayType {
    Flatten,
    Transparent,
    Untagged,
    Tag(String),
    Content(String),
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

    fn from_all_display_type(display_type: &DisplayType) -> Option<RenameType> {
        match display_type {
            DisplayType::RenameAllPascal => Some(RenameType::Pascal),
            DisplayType::RenameAllSnake => Some(RenameType::Snake),
            DisplayType::RenameAllKebab => Some(RenameType::Kebab),
            DisplayType::RenameAllCamel => Some(RenameType::Camel),
            _ => None,
        }
    }
}

trait VecExt {
    fn try_get_rename(&self) -> Result<Option<RenameType>>;
    fn try_get_rename_all(&self) -> Result<Option<RenameType>>;
    fn try_get_flatten(&self) -> Result<Option<()>>;
    fn try_get_tag(&self) -> Result<Option<TagType>>;
    fn try_get_transparent(&self) -> Result<Option<()>>;
    fn try_get_skip(&self) -> Result<Option<SkipType>>;
}

impl VecExt for Vec<DisplayType> {
    fn try_get_rename(&self) -> Result<Option<RenameType>> {
        let mut iter = self.iter().filter_map(RenameType::from_display_type);
        let rename = iter.next();
        if iter.next().is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Only one rename attribute is allowed",
            ));
        }
        Ok(rename)
    }

    fn try_get_rename_all(&self) -> Result<Option<RenameType>> {
        let mut iter = self.iter().filter_map(RenameType::from_all_display_type);
        let rename_all = iter.next();
        if iter.next().is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Only one rename_all attribute is allowed",
            ));
        }
        Ok(rename_all)
    }

    fn try_get_flatten(&self) -> Result<Option<()>> {
        let mut iter = self
            .iter()
            .filter(|&d| matches!(d, &DisplayType::Flatten))
            .map(|_| ());
        let flatten = iter.next();
        if iter.next().is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Only one flatten attribute is allowed",
            ));
        }
        Ok(flatten)
    }

    fn try_get_skip(&self) -> Result<Option<SkipType>> {
        let mut iter = self.iter().filter_map(SkipType::from_display_type);
        let skip = iter.next();
        if iter.next().is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Only one skip attribute is allowed",
            ));
        }
        Ok(skip)
    }

    fn try_get_tag(&self) -> Result<Option<TagType>> {
        let mut tag_iter = self.iter().filter_map(|d| match d {
            DisplayType::Tag(s) => Some(s.clone()),
            _ => None,
        });
        let tag = tag_iter.next();
        if tag_iter.next().is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Only one tag attribute is allowed",
            ));
        }
        let mut content_iter = self.iter().filter_map(|d| match d {
            DisplayType::Content(s) => Some(s.clone()),
            _ => None,
        });
        let content = content_iter.next();
        if content_iter.next().is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Only one content attribute is allowed",
            ));
        }
        let mut untagged_iter = self.iter().filter_map(|d| match d {
            DisplayType::Untagged => Some(()),
            _ => None,
        });
        let untagged = untagged_iter.next().is_some();
        if untagged_iter.next().is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Only one untagged attribute is allowed",
            ));
        }

        if untagged && (tag.is_some() || content.is_some()) {
            Err(syn::Error::new(
                Span::call_site(),
                "Cannot use both untagged and tag or content",
            ))
        } else if content.is_some() && tag.is_none() {
            Err(syn::Error::new(
                Span::call_site(),
                "Cannot use content without tag",
            ))
        } else if let (Some(tag), false) = (tag, untagged) {
            Ok(Some(TagType::Tagged { tag, content }))
        } else if untagged {
            Ok(Some(TagType::Untagged))
        } else {
            Ok(None)
        }
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

fn rename_named_field(name: String, rename_type: &RenameType) -> String {
    match rename_type {
        RenameType::Str(s) => s.clone(),
        RenameType::Pascal => name.to_pascal_case(),
        RenameType::Snake => name.to_snake_case(),
        RenameType::Kebab => name.to_kebab_case(),
        RenameType::Camel => name.to_lower_camel_case(),
    }
}

// TODO: Error for wrong attributes
fn parse_attributes(attrs: &[syn::Attribute]) -> Result<Vec<DisplayType>> {
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
                            if path.is_ident("flatten") {
                                Some(DisplayType::Flatten)
                            } else if path.is_ident("transparent") {
                                Some(DisplayType::Transparent)
                            } else if path.is_ident("tag") {
                                return Some(Err(syn::Error::new(Span::call_site(), "tag requires a string literal as an argument")));
                            } else if path.is_ident("content") {
                                return Some(Err(syn::Error::new(Span::call_site(), "content requires a string literal as an argument")));
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
                                } else if name.path.is_ident("tag") {
                                    Some(DisplayType::Tag(lit_str.value()))
                                } else if name.path.is_ident("content") {
                                    Some(DisplayType::Content(lit_str.value()))
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
    Ok(attrs.into_iter().flatten().collect::<Vec<_>>())
}

#[derive(Debug, Clone)]
struct FieldAttributes {
    flatten: bool,
    skip: Option<SkipType>,
    rename: Option<RenameType>,
}

#[derive(Debug, Clone)]
enum TagType {
    Tagged {
        tag: String,
        content: Option<String>,
    },
    Untagged,
}

#[derive(Debug, Clone)]
struct ContainerAttributes {
    transparent: bool,
    tag: Option<TagType>,
    rename_all: Option<RenameType>,
}

#[derive(Debug, Clone)]
struct VariantAttributes {
    skip: Option<SkipType>,
    rename: Option<RenameType>,
    rename_all: Option<RenameType>,
}

fn parse_field_attributes(attrs: &[syn::Attribute]) -> Result<FieldAttributes> {
    let parsed_attrs = parse_attributes(attrs)?;

    if parsed_attrs.try_get_rename_all()?.is_some() {
        return Err(syn::Error::new(
            Span::call_site(),
            "rename_all is not supported for fields",
        ));
    }

    if parsed_attrs.try_get_transparent()?.is_some() {
        return Err(syn::Error::new(
            Span::call_site(),
            "transparent is not supported for fields",
        ));
    }

    Ok(FieldAttributes {
        flatten: parsed_attrs.try_get_flatten()?.is_some(),
        skip: parsed_attrs.try_get_skip()?,
        rename: parsed_attrs.try_get_rename()?,
    })
}

fn parse_container_attributes(attrs: &[syn::Attribute]) -> Result<ContainerAttributes> {
    let parsed_attrs = parse_attributes(attrs)?;

    if parsed_attrs.try_get_flatten()?.is_some() {
        return Err(syn::Error::new(
            Span::call_site(),
            "flatten is not supported for containers",
        ));
    }

    Ok(ContainerAttributes {
        transparent: parsed_attrs.try_get_transparent()?.is_some(),
        tag: parsed_attrs.try_get_tag()?,
        rename_all: parsed_attrs.try_get_rename_all()?,
    })
}

fn parse_variant_attributes(attrs: &[syn::Attribute]) -> Result<VariantAttributes> {
    let parsed_attrs = parse_attributes(attrs)?;

    if parsed_attrs.try_get_flatten()?.is_some() {
        return Err(syn::Error::new(
            Span::call_site(),
            "flatten is not supported for variants",
        ));
    }

    if parsed_attrs.try_get_transparent()?.is_some() {
        return Err(syn::Error::new(
            Span::call_site(),
            "transparent is not supported for variants",
        ));
    }

    if parsed_attrs.try_get_tag()?.is_some() {
        return Err(syn::Error::new(
            Span::call_site(),
            "tag is not supported for variants",
        ));
    }

    Ok(VariantAttributes {
        skip: parsed_attrs.try_get_skip()?,
        rename: parsed_attrs.try_get_rename()?,
        rename_all: parsed_attrs.try_get_rename_all()?,
    })
}

// TODO: fn parse_container_attributes(attrs: &[syn::Attribute]) -> Result<DisplayAttrs>
// TODO: fn parse_field_attributes(attrs: &[syn::Attribute]) -> Result<DisplayAttrs>
// TODO: fn parse_variant_attributes(attrs: &[syn::Attribute]) -> Result<DisplayAttrs>
// TODO: specific type for each?

fn gen_named_fields(fields: FieldsNamed, rename_all: Option<RenameType>) -> Result<TokenStream2> {
    let field_render_tuple_code = fields.named.iter().map(|_| {
        quote! {
            true,
        }
    });

    let mut fields_with_attrs = fields
        .named
        .iter()
        .map(|field| parse_field_attributes(&field.attrs).map(|attrs| (field, attrs)))
        .collect::<Result<Vec<_>>>()?;

    fields_with_attrs.retain(|(_, attrs)| !matches!(attrs.skip, Some(SkipType::Always)));

    let fields_code = fields_with_attrs.into_iter().enumerate().map(move |(i, (field, attrs))| {
        let field_name = field.ident.as_ref().ok_or_else(|| syn::Error::new(Span::call_site(), "Fields must have a name"))?;
        let name_span = field_name.span();
        let field_name_string = attrs.rename.as_ref().map_or(rename_all.as_ref(),  Some).map(|rename_type| rename_named_field(field_name.to_string(), rename_type)).unwrap_or_else(|| field_name.to_string());
        let field_name_stringified = LitStr::new(&field_name_string, name_span);
        let to_render_index = syn::Index::from(i);
        let skip_code = if let Some(skip) = attrs.skip {
            let condition = if let SkipType::If(skip_if) = skip {
                quote! {
                    if #skip_if (#field_name) {
                        to_render. #to_render_index = false;
                    } else {
                        last_field = #i;
                    }
                }
            } else if let SkipType::IfFalse = skip {
                quote! {
                    !(#field_name)
                }
             } else if let SkipType::IfTrue = skip {
                quote! {
                    (#field_name)
                }
            } else if let SkipType::IfNone = skip {
                quote! {
                    (#field_name).is_none()
                }
            } else if let SkipType::IfEmpty = skip {
                quote! {
                    (#field_name).is_empty()
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
        // TODO: do flatten at compile time, not at runtime
        let flatten = attrs.flatten;
        let render_code = quote! {
            let mut indent_modified = ctx.indent.to_string();
            if to_render. #to_render_index {
                if (! #flatten && last_field > #i) || tctx.is_flattened_and_last == Some(false) {
                    indent_modified.push_str("|  ");
                    write!(f, "{}├──{}", ctx.indent, #field_name_stringified)?;
                } else if ! #flatten {
                    indent_modified.push_str("   ");
                    write!(f, "{}└──{}", ctx.indent, #field_name_stringified)?;
                }
                if ! #flatten && ctx.show_types {
                    #field_name.type_name_fmt(f)?;
                }
                if ! #flatten {
                    writeln!(f)?;
                }

                let mut tctx = tree_display::TransientContext { ..Default::default() };
                if #flatten {
                    tctx.is_flattened_and_last = Some(last_field <= #i);
                }
                tree_display::TreeDisplay::tree_fmt(&#field_name, f, tree_display::Context { indent: &indent_modified, ..ctx }, tctx)?;
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
                    let mut indent_modified = ctx.indent.to_string();
                    indent_modified.push_str("|  ");
                    write!(f, "{}├──{}", ctx.indent, #field_accessor)?;
                    if ctx.show_types {
                        self.#field_accessor.type_name_fmt(f)?;
                    }
                    writeln!(f)?;
                    tree_display::TreeDisplay::tree_fmt(&self.#field_accessor, f, tree_display::Context { indent: &indent_modified, ..ctx }, Default::default())?;
                }
            } else {
                quote! {
                    let mut indent_modified = ctx.indent.to_string();
                    indent_modified.push_str("   ");
                    write!(f, "{}└──{}", ctx.indent, #field_accessor)?;
                    if ctx.show_types {
                        self.#field_accessor.type_name_fmt(f)?;
                    }
                    writeln!(f)?;
                    tree_display::TreeDisplay::tree_fmt(&self.#field_accessor, f, tree_display::Context { indent: &indent_modified, ..ctx }, Default::default())?;
                }
            }
        })
}

fn impl_my_trait(ast: DeriveInput) -> Result<TokenStream2> {
    Ok({
        let name = ast.ident;
        let where_clause = ast.generics.where_clause.clone();
        let generics = ast.generics;
        let attrs = parse_container_attributes(&ast.attrs)?;

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
                        writeln!(f, "{}└──{}", ctx.indent, #variant_name_stringified)?;
                        if let Some(sparcity) = ctx.sparcity {
                            (0..sparcity.get()).try_for_each(|_| {
                                writeln!(f, "{}   |", ctx.indent)
                            })?;
                        }
                        let mut indent_modified = ctx.indent.to_string();
                        indent_modified.push_str("   ");
                    };

                    match v.fields {
                        Fields::Named(fields) => {
                            let span = name.span();
                            let name_stringified = LitStr::new(&name.to_string(), span);
                            let mut i = 0;
                            let fields_with_names = fields
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
                                #(#fields_with_names ,)*
                            };

                            let named_fields_code = gen_named_fields(fields, attrs.rename_all.clone())?;
                            Ok(quote! {
                                #name::#variant_name { #destructure_code } => {
                                    #variant_name_code
                                    let mut indent = ctx.indent.to_string();
                                    indent.push_str("   ");
                                    let ctx = tree_display::Context {
                                        indent: &indent,
                                        ..ctx
                                    };
                                    #named_fields_code
                                }
                            })
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
                                    #ident.tree_fmt(f, tree_display::Context { indent: &indent_modified, ..ctx }, Default::default())?;
                                }
                            });

                            Ok(quote! {
                                #name::#variant_name(#destructure_code) => {
                                    #variant_name_code
                                    #(#fields_fmt)*
                                }
                            })
                        }
                        Fields::Unit => Ok(quote! {
                                #name::#variant_name => {
                                    writeln!(f, "{}└──{}", ctx.indent, #variant_name_stringified)?;
                                    if let Some(sparcity) = ctx.sparcity {
                                        (0..sparcity.get()).try_for_each(|_| {
                                            writeln!(f, "{}", ctx.indent)
                                        })?;
                                    }
                                    let mut indent_modified = ctx.indent.to_string();
                            }
                        }),
                    }
                }).collect::<Result<Vec<_>>>()?;

                quote! {
                    impl tree_display::TreeDisplay for #name {
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, ctx: tree_display::Context, tctx: tree_display::TransientContext) -> ::std::fmt::Result {
                            if let Some(sparcity) = ctx.sparcity {
                                (0..sparcity.get()).try_for_each(|_| {
                                    writeln!(f, "{}|", ctx.indent)
                                })?;
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

                let fields_iter = fields.named.iter().map(|f| f.ident.clone());

                let field_code = quote! {
                    let Self { #(#fields_iter ,)* } = self;
                };

                let named_fields_code = gen_named_fields(fields, attrs.rename_all)?;

                quote! {
                    impl tree_display::TreeDisplay for #name {
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, ctx: tree_display::Context, tctx: tree_display::TransientContext) -> ::std::fmt::Result {
                            #field_code

                            if let Some(sparcity) = ctx.sparcity {
                                (0..sparcity.get()).try_for_each(|_| {
                                    writeln!(f, "{}|", ctx.indent)
                                })?;
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

                let fields_iter = fields.named.iter().map(|f| f.ident.clone());

                let field_code = quote! {
                    let Self { #(#fields_iter ,)* } = self;
                };

                let named_fields_code = gen_named_fields(fields, attrs.rename_all)?;
                quote! {
                    impl #generics tree_display::TreeDisplay for #name #generics #where_clause {
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, ctx: tree_display::Context, tctx: tree_display::TransientContext) -> ::std::fmt::Result {
                            #field_code

                            if let Some(sparcity) = ctx.sparcity {
                                (0..sparcity.get()).try_for_each(|_| {
                                    writeln!(f, "{}|", ctx.indent)
                                })?;
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
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, ctx: tree_display::Context, tctx: tree_display::TransientContext) -> ::std::fmt::Result {
                            if let Some(sparcity) = ctx.sparcity {
                                (0..sparcity.get()).try_for_each(|_| {
                                    writeln!(f, "{}|", ctx.indent)
                                })?;
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
                        fn tree_fmt(&self, f: &mut ::std::fmt::Formatter<'_>, ctx: tree_display::Context, tctx: tree_display::TransientContext) -> ::std::fmt::Result {
                            if let Some(sparcity) = ctx.sparcity {
                                (0..sparcity.get()).try_for_each(|_| {
                                    writeln!(f, "{}", ctx.indent)
                                })?;
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
