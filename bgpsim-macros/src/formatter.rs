// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2025 Tibor Schneider <sctibor@ethz.ch>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TS2;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DataStruct, DeriveInput, Fields};

pub(crate) fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        mut generics,
        data,
        ..
    } = parse_macro_input!(input as DeriveInput);
    generics.make_where_clause();

    let x = generics.params.iter();
    let impl_generics = quote! {
        <'__n, __P: ::bgpsim::types::Prefix, __Q, __OSPF: ::bgpsim::ospf::OspfImpl, __R, #(#x),*>
    };

    let type_params = generics
        .const_params()
        .map(|x| x.ident.clone())
        .map(|x| quote!(#x))
        .chain(
            generics
                .lifetimes()
                .map(|x| x.lifetime.clone())
                .map(|x| quote!(#x)),
        )
        .chain(
            generics
                .type_params()
                .map(|x| x.ident.clone())
                .map(|x| quote!(#x)),
        );
    let given_bounds = generics
        .where_clause
        .as_ref()
        .unwrap()
        .predicates
        .iter()
        .collect::<Vec<_>>();
    let type_bounds = generics
        .type_params()
        .map(|x| x.ident.clone())
        .map(|x| quote!(#x: ::bgpsim::formatter::NetworkFormatter<'__n, __P, __Q, __OSPF, __R>))
        .collect::<Vec<_>>();

    let fmt_impl = match generate_fmt_impl(&ident, &data) {
        Ok(ts) => ts,
        Err(e) => return e.to_compile_error().into(),
    };
    let fmt_multiline_impl = match generate_fmt_multiline_impl(&ident, &data) {
        Ok(ts) => ts,
        Err(e) => return e.to_compile_error().into(),
    };

    quote!{
        #[automatically_derived]
        impl #impl_generics ::bgpsim::formatter::NetworkFormatter<'__n, __P, __Q, __OSPF, __R> for #ident<#(#type_params),*>
        where
            #(#given_bounds),*
            #(#type_bounds),*
        {
            fn fmt(&self, __net: &'__n ::bgpsim::network::Network<__P, __Q, __OSPF, __R>) -> String {
                #fmt_impl
            }

            fn fmt_multiline_indent(&self, __net: &'__n ::bgpsim::network::Network<__P, __Q, __OSPF, __R>, __indent: usize) -> String {
                #fmt_multiline_impl
            }
        }
    }.into()
}

fn generate_fmt_impl(name: &syn::Ident, data: &syn::Data) -> syn::Result<TS2> {
    match data {
        syn::Data::Struct(DataStruct { fields, .. }) => fmt_fields_impl(name, fields, true),
        syn::Data::Enum(data_enum) => {
            let mut cases = Vec::new();

            for variant in data_enum.variants.iter() {
                let ident = variant.ident.clone();
                let code = fmt_fields_impl(&ident, &variant.fields, false)?;
                let case = match &variant.fields {
                    Fields::Named(fields_named) => {
                        let fields = fields_named.named.iter().filter_map(|x| x.ident.as_ref());
                        quote!(Self::#ident{#(#fields),*} => { #code })
                    }
                    Fields::Unnamed(fields_unnamed) => {
                        let fields = fields_unnamed
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| quote::format_ident!("__f{}", i));
                        quote!(Self::#ident(#(#fields),*) => { #code })
                    }
                    Fields::Unit => quote!(Self::#ident => { #code }),
                };
                cases.push(case);
            }

            Ok(quote!(match self {
                #(#cases),*
            }))
        }
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            name,
            "Cannot automatically derive NetworkFormatter for an untagged union",
        )),
    }
}

fn fmt_fields_impl(name: &syn::Ident, fields: &syn::Fields, in_struct: bool) -> syn::Result<TS2> {
    match fields {
        Fields::Unit => {
            let content = name.to_string();
            Ok(quote!(::std::string::String::from(#content)))
        }
        Fields::Named(fields) => {
            let mut header = format!("{name} {{");
            let empty = fields
                .named
                .iter()
                .filter_map(|x| Options::from_attributes(&x.attrs).ok())
                .filter(|x| !x.skip)
                .count()
                == 0;
            if empty {
                header.push('}');
                return Ok(quote!(::std::string::String::from(#header)));
            }
            header.push(' ');
            let mut prog = quote!(
                let mut s = String::from(#header);
            );
            let mut first = true;
            for field in &fields.named {
                let options = Options::from_attributes(&field.attrs)?;
                if options.skip {
                    continue;
                }
                let ident = field.ident.as_ref().unwrap();
                let field = if in_struct {
                    quote!(&self.#ident)
                } else {
                    quote!(#ident)
                };
                let call = options.fmt.call(field, false);
                let ident_header = if first {
                    format!("{ident}: ")
                } else {
                    format!(", {ident}: ")
                };
                first = false;
                prog.extend([quote!(
                    s.push_str(#ident_header);
                    s.push_str(&#call);
                )]);
            }
            prog.extend([quote!(s.push_str(" }");)]);
            prog.extend([quote!(s)]);
            Ok(prog)
        }
        Fields::Unnamed(fields) => {
            let header = format!("{name}(");
            let mut prog = quote!(
                let mut s = String::from(#header);
            );
            let mut first = true;
            for (idx, field) in fields.unnamed.iter().enumerate() {
                let options = Options::from_attributes(&field.attrs)?;
                if options.skip {
                    continue;
                }
                if !first {
                    prog.extend([quote!(s.push_str(", ");)]);
                }
                first = false;
                let field = if in_struct {
                    let idx = syn::Index::from(idx);
                    quote!(&self.#idx)
                } else {
                    let ident = quote::format_ident!("__f{}", idx);
                    quote!(#ident)
                };
                let call = options.fmt.call(field, false);
                prog.extend([quote!(s.push_str(&#call);)]);
            }
            prog.extend([quote!(s.push(')');)]);
            prog.extend([quote!(s)]);
            Ok(prog)
        }
    }
}

fn generate_fmt_multiline_impl(name: &syn::Ident, data: &syn::Data) -> syn::Result<TS2> {
    match data {
        syn::Data::Struct(DataStruct { fields, .. }) => {
            fmt_multiline_fields_impl(name, fields, true)
        }
        syn::Data::Enum(data_enum) => {
            let mut cases = Vec::new();

            for variant in data_enum.variants.iter() {
                let ident = variant.ident.clone();
                let code = fmt_multiline_fields_impl(&ident, &variant.fields, false)?;
                let case = match &variant.fields {
                    Fields::Named(fields_named) => {
                        let fields = fields_named.named.iter().filter_map(|x| x.ident.as_ref());
                        quote!(Self::#ident{#(#fields),*} => { #code })
                    }
                    Fields::Unnamed(fields_unnamed) => {
                        let fields = fields_unnamed
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| quote::format_ident!("__f{}", i));
                        quote!(Self::#ident(#(#fields),*) => { #code })
                    }
                    Fields::Unit => quote!(Self::#ident => { #code }),
                };
                cases.push(case);
            }

            Ok(quote!(match self {
                #(#cases),*
            }))
        }
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            name,
            "Cannot automatically derive NetworkFormatter for an untagged union",
        )),
    }
}

fn fmt_multiline_fields_impl(
    name: &syn::Ident,
    fields: &syn::Fields,
    in_struct: bool,
) -> syn::Result<TS2> {
    match fields {
        Fields::Unit => {
            let content = name.to_string();
            Ok(quote!(::std::string::String::from(#content)))
        }
        Fields::Named(fields) => {
            let mut header = format!("{name} {{");
            let empty = fields
                .named
                .iter()
                .filter_map(|x| Options::from_attributes(&x.attrs).ok())
                .filter(|x| !x.skip)
                .count()
                == 0;
            if empty {
                header.push('}');
                return Ok(quote!(::std::string::String::from(#header)));
            }
            let mut prog = quote!(
                let spc = " ".repeat(__indent);
                let mut s = String::from(#header);
            );
            let mut first = true;
            for field in &fields.named {
                let options = Options::from_attributes(&field.attrs)?;
                if options.skip {
                    continue;
                }
                let ident = field.ident.as_ref().unwrap();
                let field = if in_struct {
                    quote!(&self.#ident)
                } else {
                    quote!(#ident)
                };
                let call = options.fmt_multiline.call(field, true);
                let header = format!("{ident}: ");
                let new_line = if first { "\n  " } else { ",\n  " };
                first = false;
                prog.extend([quote!(
                    s.push_str(#new_line);
                    s.push_str(&spc);
                    s.push_str(#header);
                    s.push_str(&#call);
                )]);
            }
            prog.extend([quote!(s.push_str("\n}");)]);
            prog.extend([quote!(s)]);
            Ok(prog)
        }
        Fields::Unnamed(fields) => {
            let header = format!("{name}(");
            let mut prog = quote!(
                let mut s = String::from(#header);
            );
            let single_line = fields
                .unnamed
                .iter()
                .filter_map(|x| Options::from_attributes(&x.attrs).ok())
                .filter(|x| !x.skip)
                .count()
                <= 1;
            let mut first = true;
            for (idx, field) in fields.unnamed.iter().enumerate() {
                let options = Options::from_attributes(&field.attrs)?;
                if options.skip {
                    continue;
                }
                let field = if in_struct {
                    let idx = syn::Index::from(idx);
                    quote!(&self.#idx)
                } else {
                    let ident = quote::format_ident!("__f{}", idx);
                    quote!(#ident)
                };
                let call = options.fmt_multiline.call(field, !single_line);
                let new_line = if single_line {
                    ""
                } else if first {
                    "\n  "
                } else {
                    ",\n  "
                };
                first = false;

                prog.extend([quote!(
                    s.push_str(#new_line);
                    s.push_str(&#call);
                )]);
            }
            if single_line {
                prog.extend([quote!(s.push_str(")");)]);
            } else {
                prog.extend([quote!(s.push_str("\n)");)]);
            }
            prog.extend([quote!(s)]);
            Ok(prog)
        }
    }
}

struct Options {
    skip: bool,
    fmt: Formatter,
    fmt_multiline: Formatter,
}

impl Options {
    fn from_attributes(attrs: &[syn::Attribute]) -> syn::Result<Options> {
        let mut skip = false;
        let mut fmt = Formatter::Fmt;
        let mut fmt_multiline = None;

        for attr in attrs {
            let syn::Meta::List(meta) = &attr.meta else {
                continue;
            };
            let Some(key) = meta.path.get_ident() else {
                continue;
            };
            if *key != "formatter" {
                continue;
            };

            let attributes: Args = syn::parse(meta.tokens.clone().into())?;
            for meta in attributes.0 {
                let (path, value) = match meta.clone() {
                    syn::Meta::Path(path) => (path, None),
                    syn::Meta::List(meta_list) => {
                        return Err(syn::Error::new_spanned(meta_list, "Invalid attribute"))
                    }
                    syn::Meta::NameValue(m) => (m.path, Some(m.value)),
                };
                let Some(key) = path.get_ident() else {
                    return Err(syn::Error::new_spanned(
                        path.clone(),
                        "The `key` in the attribute must be an identifier",
                    ));
                };
                match key.to_string().as_str() {
                    "skip" => {
                        if let Some(v) = value {
                            return Err(syn::Error::new_spanned(
                                v,
                                "Expecting `skip` without a value",
                            ));
                        }
                        skip = true;
                    }
                    "fmt" => {
                        let Some(value) = value else {
                            return Err(syn::Error::new_spanned(
                                path,
                                "Expecting a value for the `fmt` attribute",
                            ));
                        };
                        fmt = Formatter::from_expr(value, false)?;
                        // assert that that fmt can be used in singleline
                        if !fmt.singleline() {
                            return Err(syn::Error::new_spanned(
                                meta.clone(),
                                format!("The formatter `{fmt}` can only be used in `multiline`."),
                            ));
                        }
                    }
                    "multiline" => {
                        let Some(value) = value else {
                            return Err(syn::Error::new_spanned(
                                path,
                                "Expecting a value for the `multiline` attribute",
                            ));
                        };
                        fmt_multiline = Some(Formatter::from_expr(value, true)?);
                    }
                    x => {
                        return Err(syn::Error::new_spanned(
                            path.clone(),
                            format!("Unknown attribute key: {x}"),
                        ))
                    }
                }
            }
        }

        let fmt_multiline = fmt_multiline.unwrap_or_else(|| fmt.multiline_variant());
        Ok(Options {
            skip,
            fmt,
            fmt_multiline,
        })
    }
}

struct Args(Vec<syn::Meta>);
impl syn::parse::Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let list =
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated(input)?;
        Ok(Self(Vec::from_iter(list)))
    }
}

#[derive(Clone, Debug)]
enum Formatter {
    Fmt,
    FmtMultiline,
    Set,
    SetMultiline,
    Map,
    MapMultiline,
    List,
    ListMultiline,
    Path,
    PathOptions,
    PathSet,
    PathMultiline,
    Ext,
    Custom(syn::Path),
    CustomMultiline(syn::Path),
}

impl Formatter {
    fn from_expr(expr: syn::Expr, in_multiline: bool) -> syn::Result<Self> {
        match expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(name),
                ..
            }) => Self::from_name(name),
            syn::Expr::Path(expr_path) if in_multiline => Ok(Self::CustomMultiline(expr_path.path)),
            syn::Expr::Path(expr_path) => Ok(Self::Custom(expr_path.path)),
            _ => Err(syn::Error::new_spanned(expr, "Unsupported expression.")),
        }
    }

    fn from_name(name: syn::LitStr) -> syn::Result<Self> {
        match name.value().as_ref() {
            "fmt" => Ok(Self::Fmt),
            "fmt_multiline" => Ok(Self::FmtMultiline),
            "fmt_set" => Ok(Self::Set),
            "fmt_set_multiline" => Ok(Self::SetMultiline),
            "fmt_list" => Ok(Self::List),
            "fmt_list_multiline" => Ok(Self::ListMultiline),
            "fmt_map" => Ok(Self::Map),
            "fmt_map_multiline" => Ok(Self::MapMultiline),
            "fmt_path" => Ok(Self::Path),
            "fmt_path_options" => Ok(Self::PathOptions),
            "fmt_path_set" => Ok(Self::PathSet),
            "fmt_path_multiline" => Ok(Self::PathMultiline),
            "fmt_ext" => Ok(Self::Ext),
            x => Err(syn::Error::new_spanned(
                name,
                format!("Unknown build-in network formatter function: `{x}`"),
            )),
        }
    }
}

impl Formatter {
    fn singleline(&self) -> bool {
        match self {
            Formatter::Fmt
            | Formatter::Set
            | Formatter::Map
            | Formatter::List
            | Formatter::Path
            | Formatter::PathOptions
            | Formatter::PathSet
            | Formatter::Ext
            | Formatter::Custom(_) => true,
            Formatter::FmtMultiline
            | Formatter::SetMultiline
            | Formatter::MapMultiline
            | Formatter::ListMultiline
            | Formatter::PathMultiline
            | Formatter::CustomMultiline(_) => false,
        }
    }

    fn multiline(&self) -> bool {
        !self.singleline()
    }

    fn multiline_variant(&self) -> Self {
        match self {
            Formatter::Fmt => Formatter::FmtMultiline,
            Formatter::Set => Formatter::SetMultiline,
            Formatter::Map => Formatter::MapMultiline,
            Formatter::List => Formatter::ListMultiline,
            Formatter::PathSet => Formatter::PathMultiline,
            Formatter::FmtMultiline
            | Formatter::SetMultiline
            | Formatter::MapMultiline
            | Formatter::ListMultiline
            | Formatter::PathMultiline
            | Formatter::PathOptions
            | Formatter::Ext
            | Formatter::Path
            | Formatter::Custom(_)
            | Formatter::CustomMultiline(_) => self.clone(),
        }
    }

    fn path(&self) -> syn::Path {
        match self {
            Formatter::Fmt => {
                parse_quote!(::bgpsim::formatter::NetworkFormatter::fmt)
            }
            Formatter::FmtMultiline => {
                parse_quote!(::bgpsim::formatter::NetworkFormatter::fmt_multiline_indent)
            }
            Formatter::Set => {
                parse_quote!(::bgpsim::formatter::NetworkFormatterSequence::fmt_set)
            }
            Formatter::SetMultiline => {
                parse_quote!(::bgpsim::formatter::NetworkFormatterSequence::fmt_set_multiline)
            }
            Formatter::List => {
                parse_quote!(::bgpsim::formatter::NetworkFormatterSequence::fmt_list)
            }
            Formatter::ListMultiline => {
                parse_quote!(::bgpsim::formatter::NetworkFormatterSequence::fmt_list_multiline)
            }
            Formatter::Map => {
                parse_quote!(::bgpsim::formatter::NetworkFormatterMap::fmt_map)
            }
            Formatter::MapMultiline => {
                parse_quote!(::bgpsim::formatter::NetworkFormatterMap::fmt_map_multiline)
            }
            Formatter::Path => {
                parse_quote!(::bgpsim::formatter::NetworkFormatterSequence::fmt_path)
            }
            Formatter::PathOptions => {
                parse_quote!(::bgpsim::formatter::NetworkFormatterNestedSequence::fmt_path_options)
            }
            Formatter::PathSet => {
                parse_quote!(::bgpsim::formatter::NetworkFormatterNestedSequence::fmt_path_set)
            }
            Formatter::PathMultiline => {
                parse_quote!(
                    ::bgpsim::formatter::NetworkFormatterNestedSequence::fmt_path_multiline
                )
            }
            Formatter::Ext => {
                parse_quote!(::bgpsim::formatter::NetworkFormatterExt::fmt_ext)
            }
            Formatter::Custom(path) | Formatter::CustomMultiline(path) => path.clone(),
        }
    }

    fn call(&self, field: impl quote::ToTokens, increment_indent: bool) -> TS2 {
        let indent = if increment_indent {
            quote!(__indent + 2)
        } else {
            quote!(__indent)
        };
        let path = self.path();
        if self.multiline() {
            quote!(#path(#field, __net, #indent))
        } else {
            quote!(#path(#field, __net))
        }
    }
}

impl std::fmt::Display for Formatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Formatter::Fmt => f.write_str("fmt"),
            Formatter::FmtMultiline => f.write_str("fmt_multiline"),
            Formatter::Set => f.write_str("fmt_set"),
            Formatter::SetMultiline => f.write_str("fmt_set_multiline"),
            Formatter::Map => f.write_str("fmt_map"),
            Formatter::MapMultiline => f.write_str("fmt_map_multiline"),
            Formatter::List => f.write_str("fmt_list"),
            Formatter::ListMultiline => f.write_str("fmt_list_multiline"),
            Formatter::Path => f.write_str("fmt_path"),
            Formatter::PathOptions => f.write_str("fmt_path_options"),
            Formatter::PathSet => f.write_str("fmt_path_set"),
            Formatter::PathMultiline => f.write_str("fmt_path_multiline"),
            Formatter::Ext => f.write_str("fmt_ext"),
            Formatter::Custom(_) => f.write_str("CUSTOM"),
            Formatter::CustomMultiline(_) => f.write_str("CUSTOM"),
        }
    }
}
