// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2023 Tibor Schneider <sctibor@ethz.ch>
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
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    LitStr, Result, Token, Type,
};

#[derive(Clone, Debug)]
pub(crate) struct PrefixInputNoCast {
    pub byte0: u8,
    pub byte1: u8,
    pub byte2: u8,
    pub byte3: u8,
    pub prefix_len: u8,
    pub span: Span,
}

impl Eq for PrefixInputNoCast {}

impl std::hash::Hash for PrefixInputNoCast {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.byte0.hash(state);
        self.byte1.hash(state);
        self.byte2.hash(state);
        self.byte3.hash(state);
        self.prefix_len.hash(state);
    }
}

impl PartialEq for PrefixInputNoCast {
    fn eq(&self, other: &Self) -> bool {
        self.byte0 == other.byte0
            && self.byte1 == other.byte1
            && self.byte2 == other.byte2
            && self.byte3 == other.byte3
            && self.prefix_len == other.prefix_len
    }
}

impl Parse for PrefixInputNoCast {
    fn parse(input: ParseStream) -> Result<Self> {
        let str: LitStr = input.parse()?;
        let span = str.span();
        let s = str.value();
        let (ip, mask) = s.trim().split_once('/').ok_or_else(|| {
            input.error(
                "Expected a `/` in the string literal of the IP address containing the mask.",
            )
        })?;
        let prefix_len = mask
            .parse::<u8>()
            .map_err(|_| input.error("Cannot parse the netmask as a number!"))?;
        let ip = ip
            .split('.')
            .enumerate()
            .map(|(i, x)| {
                x.parse::<u8>().map_err(|_| {
                    input.error(format!(
                        "Cannot parse the {} byte of the IP address as an u8: {}",
                        match i {
                            0 => "first".to_string(),
                            1 => "second".to_string(),
                            2 => "third".to_string(),
                            3 => "fourth".to_string(),
                            i => format!("{}th", i + 1),
                        },
                        x
                    ))
                })
            })
            .collect::<Result<Vec<u8>>>()?;
        if ip.len() != 4 {
            return Err(input.error("The IP address must contain 4 parts."));
        }

        if prefix_len > 32 {
            return Err(input.error("The prefix length must be between 0 and 32!"));
        }

        Ok(Self {
            byte0: ip[0],
            byte1: ip[1],
            byte2: ip[2],
            byte3: ip[3],
            prefix_len,
            span,
        })
    }
}

pub(crate) struct PrefixInput {
    pub prefix: PrefixInputNoCast,
    pub cast: bool,
    pub target_type: Option<Type>,
}

impl Parse for PrefixInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let prefix = input.parse()?;
        let mut cast = false;
        let mut target_type = None;
        if input.parse::<Token![as]>().is_ok() {
            cast = true;
            target_type = input.parse::<Type>().ok();
        }

        Ok(Self {
            prefix,
            cast,
            target_type,
        })
    }
}

impl PrefixInputNoCast {
    pub fn quote(&self) -> proc_macro2::TokenStream {
        let Self {
            byte0,
            byte1,
            byte2,
            byte3,
            prefix_len,
            ..
        } = self;
        quote! {
            ::ipnet::Ipv4Net::new(::std::net::Ipv4Addr::new(#byte0, #byte1, #byte2, #byte3), #prefix_len).unwrap()
        }
    }
}

impl PrefixInput {
    pub fn quote(&self) -> TokenStream {
        let Self {
            prefix,
            cast,
            target_type,
        } = self;
        let prefix = prefix.quote();
        match (cast, target_type) {
            (false, _) => prefix,
            (true, None) => quote! { #prefix.into() },
            (true, Some(ty)) => quote! { #ty::from(#prefix) },
        }
        .into()
    }
}
