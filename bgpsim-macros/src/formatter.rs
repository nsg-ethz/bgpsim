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
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

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
        <'__n, __P: ::bgpsim::types::Prefix, __Q, __OSPF: ::bgpsim::ospf::OspfImpl, #(#x),*>
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
        .map(|x| quote!(#x: ::bgpsim::formatter::NetworkFormatter<'__n, __P, __Q, __OSPF>))
        .collect::<Vec<_>>();

    let fmt_impl = generate_fmt_impl(&ident, &data);
    let fmt_multiline_impl = generate_fmt_multiline_impl(&ident, &data);

    quote!{
        impl #impl_generics ::bgpsim::formatter::NetworkFormatter<'__n, __P, __Q, __OSPF> for #ident<#(#type_params),*>
        where
            #(#given_bounds),*
            #(#type_bounds),*
        {
            fn fmt(&self, __net: &'__n ::bgpsim::network::Network<__P, __Q, __OSPF>) -> String {
                #fmt_impl
            }

            fn fmt_multiline_indent(&self, __net: &'__n ::bgpsim::network::Network<__P, __Q, __OSPF>, __indent: usize) -> String {
                #fmt_multiline_impl
            }
        }
    }.into()
}

fn generate_fmt_impl(name: &syn::Ident, data: &Data) -> TS2 {
    match data {
        Data::Struct(DataStruct { fields, .. }) => fmt_fields_impl(name, fields, true),
        Data::Enum(data_enum) => {
            let mut cases = Vec::new();

            for variant in data_enum.variants.iter() {
                let ident = variant.ident.clone();
                let code = fmt_fields_impl(&ident, &variant.fields, false);
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

            quote!(match self {
                #(#cases),*
            })
        }
        Data::Union(_) => {
            quote!(compile_error!(
                "Cannot automatically derive NetworkFormatter for an untagged union"
            ))
        }
    }
}

fn fmt_fields_impl(name: &syn::Ident, fields: &syn::Fields, in_struct: bool) -> TS2 {
    match fields {
        Fields::Unit => {
            let content = name.to_string();
            quote!(::std::string::String::from(#content))
        }
        Fields::Named(fields) => {
            let header = format!("{name} {{ ");
            let mut prog = quote!(
                let mut s = String::from(#header);
            );
            for (i, field) in fields.named.iter().enumerate() {
                let ident = field.ident.as_ref().unwrap();
                let ident_header = if i > 0 {
                    format!(", {ident}: ")
                } else {
                    format!("{ident}: ")
                };
                let field = if in_struct {
                    quote!(&self.#ident)
                } else {
                    quote!(#ident)
                };
                prog.extend([quote!(
                    s.push_str(#ident_header);
                    s.push_str(&::bgpsim::formatter::NetworkFormatter::fmt(#field, __net));
                )])
            }
            prog.extend([quote!(s.push_str(" }");)]);
            prog.extend([quote!(s)]);
            prog
        }
        Fields::Unnamed(fields) => {
            let header = format!("{name}(");
            let mut prog = quote!(
                let mut s = String::from(#header);
            );
            for (i, _) in fields.unnamed.iter().enumerate() {
                if i > 0 {
                    prog.extend([quote!(s.push_str(", ");)]);
                }
                let field = if in_struct {
                    let idx = syn::Index::from(i);
                    quote!(&self.#idx)
                } else {
                    let ident = quote::format_ident!("__f{}", i);
                    quote!(#ident)
                };
                prog.extend([
                    quote!(s.push_str(&::bgpsim::formatter::NetworkFormatter::fmt(#field, __net));),
                ])
            }
            prog.extend([quote!(s.push(')');)]);
            prog.extend([quote!(s)]);
            prog
        }
    }
}

fn generate_fmt_multiline_impl(name: &syn::Ident, data: &Data) -> TS2 {
    match data {
        Data::Struct(DataStruct { fields, .. }) => fmt_multiline_fields_impl(name, fields, true),
        Data::Enum(data_enum) => {
            let mut cases = Vec::new();

            for variant in data_enum.variants.iter() {
                let ident = variant.ident.clone();
                let code = fmt_multiline_fields_impl(&ident, &variant.fields, false);
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

            quote!(match self {
                #(#cases),*
            })
        }
        Data::Union(_) => {
            quote!(compile_error!(
                "Cannot automatically derive NetworkFormatter for an untagged union"
            ))
        }
    }
}

fn fmt_multiline_fields_impl(name: &syn::Ident, fields: &syn::Fields, in_struct: bool) -> TS2 {
    match fields {
        Fields::Unit => {
            let content = name.to_string();
            quote!(::std::string::String::from(#content))
        }
        Fields::Named(fields) => {
            let header = format!("{name} {{");
            let mut prog = quote!(
                let spc = " ".repeat(__indent);
                let mut s = String::from(#header);
            );
            for (i, field) in fields.named.iter().enumerate() {
                let ident = field.ident.as_ref().unwrap();
                let field = if in_struct {
                    quote!(&self.#ident)
                } else {
                    quote!(#ident)
                };
                let header = format!("{ident}: ");
                let new_line = if i == 0 { "\n  " } else { ",\n  " };

                prog.extend([quote!(
                    s.push_str(#new_line);
                    s.push_str(&spc);
                    s.push_str(#header);
                    s.push_str(&::bgpsim::formatter::NetworkFormatter::fmt_multiline_indent(#field, __net, __indent + 2));
                )]);
            }
            prog.extend([quote!(s.push_str("\n}");)]);
            prog.extend([quote!(s)]);
            prog
        }
        Fields::Unnamed(fields) => {
            let header = format!("{name}(");
            let mut prog = quote!(
                let mut s = String::from(#header);
            );
            for (i, _) in fields.unnamed.iter().enumerate() {
                let field = if in_struct {
                    let idx = syn::Index::from(i);
                    quote!(&self.#idx)
                } else {
                    let ident = quote::format_ident!("__f{}", i);
                    quote!(#ident)
                };
                let new_line = if i == 0 { "\n  " } else { ",\n  " };

                prog.extend([quote!(
                    s.push_str(#new_line);
                    s.push_str(&::bgpsim::formatter::NetworkFormatter::fmt_multiline_indent(#field, __net, __indent + 2));
                )])
            }
            prog.extend([quote!(s.push_str("\n)");)]);
            prog.extend([quote!(s)]);
            prog
        }
    }
}
