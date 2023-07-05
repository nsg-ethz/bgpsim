// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022-2023 Tibor Schneider <sctibor@ethz.ch>
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

use std::collections::{hash_map::Entry, HashMap, HashSet};

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    braced,
    ext::IdentExt,
    parenthesized,
    parse::{discouraged::Speculative, Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Error, Expr, ExprLit, FieldValue, Ident, Lit, LitFloat, LitInt, LitStr, Result, Token, Type,
};

use crate::ip::PrefixInputNoCast;

pub(crate) struct Net {
    ty: Option<Type>,
    prefix_ty: Option<Type>,
    queue_ty: Option<Type>,
    queue: Option<Expr>,
    nodes: HashMap<Ident, Option<(u32, Span)>>,
    links: HashMap<(Ident, Ident), (f64, Span)>,
    sessions: HashMap<(Ident, Ident), Option<Ident>>,
    routes: Vec<Route<Ident>>,
    returns: Option<Returns>,
}

impl Parse for Net {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut net = Net {
            ty: Default::default(),
            prefix_ty: Default::default(),
            queue_ty: Default::default(),
            queue: Default::default(),
            nodes: Default::default(),
            links: Default::default(),
            sessions: Default::default(),
            routes: Default::default(),
            returns: Default::default(),
        };

        while !input.is_empty() {
            let block: Ident = input.call(Ident::parse_any)?;
            match block.to_string().as_str() {
                "links" => {
                    let _: Token![=] = input.parse()?;
                    net.parse_links(input)?;
                }
                "sessions" => {
                    let _: Token![=] = input.parse()?;
                    net.parse_sessions(input)?;
                }
                "routes" => {
                    let _: Token![=] = input.parse()?;
                    net.parse_routes(input)?;
                }
                "queue" => {
                    let _: Token![=] = input.parse()?;
                    if net.queue.is_some() {
                        return Err(Error::new(block.span(), "You cannot define the `queue` more than once!"));
                    }
                    net.queue = Some(input.parse()?);
                }
                "Queue" => {
                    let _: Token![=] = input.parse()?;
                    if net.queue_ty.is_some() {
                        return Err(Error::new(block.span(), "You cannot define the `Queue` type more than once!"));
                    }
                    if net.ty.is_some() {
                        return Err(Error::new(block.span(), "You cannot define both the `Queue` type and the `Network` type."));
                    }
                    net.queue_ty = Some(input.parse()?);
                }
                "Prefix" => {
                    let _: Token![=] = input.parse()?;
                    if net.prefix_ty.is_some() {
                        return Err(Error::new(block.span(), "You cannot define the `Prefix` type more than once!"));
                    }
                    if net.ty.is_some() {
                        return Err(Error::new(block.span(), "You cannot define both the `Prefix` type and the `Network` type."));
                    }
                    net.prefix_ty = Some(input.parse()?);
                }
                "Type" => {
                    let _: Token![=] = input.parse()?;
                    if net.ty.is_some() {
                        return Err(Error::new(block.span(), "You cannot define the `type` more than once!"));
                    }
                    if net.queue_ty.is_some() {
                        return Err(Error::new(block.span(), "You cannot define both the `Queue` type and the `Network` type."));
                    }
                    if net.prefix_ty.is_some() {
                        return Err(Error::new(block.span(), "You cannot define both the `Prefix` type and the `Network` type."));
                    }
                    net.ty = Some(input.parse()?);
                }
                "return" => {
                    if net.returns.is_some() {
                        return Err(Error::new(block.span(), "You cannot define the `return` more than once!"));
                    }
                    net.returns = Some(input.parse()?);
                }
                _ => return Err(Error::new(
                    block.span(),
                    format!(
                        "Unexpected identifier! Expected `links`, `sessions`, `routes`, `queue`, `Queue`, `Prefix`, `Type`, or `return`, but found `{block}`"
                    )
                )),
            }
            if !input.is_empty() {
                let _: Token![;] = input.parse()?;
            }
        }

        net.check_input()?;
        net.add_external_links();

        Ok(net)
    }
}

impl Net {
    pub fn quote(self) -> TokenStream {
        let nodes = self
            .nodes
            .iter()
            .map(|(ident, ext)| {
                let router_name = ident.to_string();
                if let Some((as_id, _)) = ext.as_ref() {
                    quote! { let #ident = _net.add_external_router(#router_name, #as_id); }
                } else {
                    quote! { let #ident = _net.add_router(#router_name); }
                }
            })
            .collect::<Vec<_>>();

        let links = self
            .links
            .iter()
            .map(|((src, dst), (weight, _))| {
                if self.links.contains_key(&(dst.clone(), src.clone())) {
                    if src > dst {
                        quote! {
                            _net.set_link_weight(#src, #dst, #weight).unwrap();
                        }
                    } else {
                        quote! {
                            _net.add_link(#src, #dst);
                            _net.set_link_weight(#src, #dst, #weight).unwrap();
                        }
                    }
                } else {
                    quote! {
                        _net.add_link(#src, #dst);
                        _net.set_link_weight(#src, #dst, #weight).unwrap();
                        _net.set_link_weight(#dst, #src, #weight).unwrap();
                    }
                }
            })
            .collect::<Vec<_>>();

        let sessions = self
            .sessions
            .iter()
            .map(|((src, dst), ty)| {
                let is_external = self.external_session(src, dst);
                let ty = match (
                    SessionType::try_from(ty).expect("Already checked!"),
                    is_external,
                ) {
                    (_, true) => quote! {::bgpsim::prelude::BgpSessionType::EBgp},
                    (SessionType::Empty, false) | (SessionType::IBgpPeer, false) => {
                        quote! {::bgpsim::prelude::BgpSessionType::IBgpPeer}
                    }
                    (SessionType::IBgpClient, false) => {
                        quote! {::bgpsim::prelude::BgpSessionType::IBgpClient}
                    }
                    _ => unreachable!(),
                };
                quote! {
                    _net.set_bgp_session(#src, #dst, Some(#ty)).unwrap();
                }
            })
            .collect::<Vec<_>>();

        let routes = self
            .routes
            .iter()
            .map(|r| {
                let source = r.src.clone();
                let prefix = r.prefix.quote();
                let as_path = r.as_path.quote(|path| quote!([#(#path),*]));
                let med = r.med.quote(|med| if let Some(med) = med {
                    quote!(Some(#med))
                } else {
                    quote!(None)
                });
                let communities = r.communities.quote(|c| quote!([#(#c),*]));
                quote! {
                    _net.advertise_external_route(#source, #prefix, #as_path, #med, #communities).unwrap();
                }
            })
            .collect::<Vec<_>>();

        let queue = if let Some(q) = self.queue {
            quote!(#q)
        } else {
            quote!(::bgpsim::prelude::BasicEventQueue::default())
        };

        let ty = if let Some(ty) = self.ty {
            quote!(#ty)
        } else {
            let queue_ty = self
                .queue_ty
                .map(|ty| quote!(#ty))
                .unwrap_or_else(|| quote!(_));
            let prefix_ty = self
                .prefix_ty
                .map(|ty| quote!(#ty))
                .unwrap_or_else(|| quote!(_));
            quote!(::bgpsim::prelude::Network<#prefix_ty, #queue_ty>)
        };

        let returns = if let Some(returns) = self.returns {
            let returns = returns.quote();
            quote!((_net, #returns))
        } else {
            quote!(_net)
        };

        quote! {
            {
                let mut _net: #ty = ::bgpsim::prelude::Network::new(#queue);
                #(#nodes)*
                #(#links)*
                #(#sessions)*
                #(#routes)*
                #returns
            }
        }
        .into()
    }

    fn parse_links(&mut self, input: ParseStream) -> Result<()> {
        // must start with a paren.
        let links;
        braced!(links in input);
        let links: Punctuated<Link, Token![;]> = links.parse_terminated(Link::parse)?;

        for Link {
            src,
            dst,
            weight,
            weight_span,
        } in links.into_iter()
        {
            let src = self.register_node(src)?;
            let dst = self.register_node(dst)?;
            match self.links.entry((src, dst)) {
                Entry::Occupied(e) if e.get().0 == weight => {}
                Entry::Occupied(e) => {
                    let mut err = Error::new(
                        weight_span,
                        "The same link was declared earlier with a different weight!",
                    );
                    err.combine(Error::new(e.get().1, "Link weas declared here"));
                    return Err(err);
                }
                Entry::Vacant(e) => {
                    e.insert((weight, weight_span));
                }
            }
        }

        Ok(())
    }

    fn parse_sessions(&mut self, input: ParseStream) -> Result<()> {
        // must start with a paren.
        let sessions;
        braced!(sessions in input);
        let sessions: Punctuated<_, Token![;]> = sessions.parse_terminated(BgpSession::parse)?;

        for BgpSession { src, dst, ty } in sessions.into_iter() {
            let src = self.register_node(src)?;
            let dst = self.register_node(dst)?;
            if let Some(((a, b), c)) = self
                .sessions
                .get_key_value(&(src.clone(), dst.clone()))
                .or_else(|| self.sessions.get_key_value(&(dst.clone(), src.clone())))
            {
                let this_span = src
                    .span()
                    .join(ty.map(|x| x.span()).unwrap_or_else(|| dst.span()));
                let mut err = Error::new(
                    this_span.expect("Cannot get the span 1"),
                    "Declared a BGP session between two nodes twice!",
                );
                let last_span = a
                    .span()
                    .join(c.as_ref().map(|x| x.span()).unwrap_or_else(|| b.span()));
                err.combine(Error::new(
                    last_span.expect("Cannot get the span 2"),
                    "The BGP session is already declared here!",
                ));
                return Err(err);
            }
            self.sessions.insert((src, dst), ty);
        }

        Ok(())
    }

    fn parse_routes(&mut self, input: ParseStream) -> Result<()> {
        // must start with a paren.
        let routes;
        braced!(routes in input);
        let routes: Punctuated<_, Token![;]> = routes.parse_terminated(Route::parse)?;

        for Route {
            src,
            prefix,
            as_path,
            med,
            communities,
        } in routes
        {
            let src = self.register_node(src)?;
            self.routes.push(Route {
                src,
                prefix,
                as_path,
                med,
                communities,
            });
        }

        Ok(())
    }

    fn register_node(&mut self, node: Node) -> Result<Ident> {
        let entry = self.nodes.entry(node.ident.clone());
        if let Some((as_id, span)) = node.ext {
            match entry {
                Entry::Occupied(mut e) => match e.get() {
                    None => {
                        e.insert(Some((as_id, span)));
                    }
                    Some((x, _)) if *x == as_id => {}
                    Some((_, before)) => {
                        let mut err = Error::new(
                            span,
                            "Declared an AS Number for an external router twice with a different number!"
                        );
                        err.combine(Error::new(
                            *before,
                            "The AS Number was originally defined here.",
                        ));
                        return Err(err);
                    }
                },
                Entry::Vacant(e) => {
                    e.insert(Some((as_id, span)));
                }
            }
        } else {
            entry.or_insert(None);
        }
        Ok(node.ident)
    }

    fn check_input(&self) -> Result<()> {
        if let Some((src, dst)) = self.sessions.keys().find(|(src, dst)| {
            self.nodes.get(src).unwrap().is_some() && self.nodes.get(dst).unwrap().is_some()
        }) {
            return Err(Error::new(
                src.span().join(dst.span()).expect("Cannot get the span 3"),
                "BGP Sessions between two external routers are not allowed!",
            ));
        }

        self.sessions.iter().try_for_each(|((src, dst), ident)| {
            SessionType::check(ident, self.external_session(src, dst))
        })?;

        if let Some(src) = self
            .routes
            .iter()
            .map(|r| &r.src)
            .find(|src| !self.is_external(src))
        {
            return Err(Error::new(
                src.span(),
                format!(
                    "Only external routers are allowed to advertise a route! Maybe use `{src}!(1)`?"
                ),
            ));
        }

        let mut announcements: HashSet<(&Ident, &Prefix)> = Default::default();
        for route in self.routes.iter() {
            if let Some((a, b)) = announcements.get(&(&route.src, &route.prefix)) {
                let mut err = Error::new(
                    route
                        .src
                        .span()
                        .join(route.prefix.span())
                        .expect("Cannot get the span 4"),
                    "A router cannot advertise the same prefix twice!",
                );
                err.combine(Error::new(
                    a.span().join(b.span()).expect("Cannot get the span 5"),
                    "The same route was defined here.",
                ));
                return Err(err);
            }
            announcements.insert((&route.src, &route.prefix));
        }

        Ok(())
    }

    fn add_external_links(&mut self) {
        let external_sessions = self
            .sessions
            .keys()
            .filter(|(src, dst)| self.external_session(src, dst))
            .map(|(src, dst)| (src.clone(), dst.clone()))
            .collect::<Vec<_>>();
        for (src, dst) in external_sessions {
            if !(self.links.contains_key(&(src.clone(), dst.clone()))
                || self.links.contains_key(&(dst.clone(), src.clone())))
            {
                let span = Span::call_site();
                self.links.insert((src, dst), (1.0, span));
            }
        }
    }

    fn is_external(&self, node: &Ident) -> bool {
        self.nodes.get(node).unwrap().is_some()
    }

    fn external_session(&self, src: &Ident, dst: &Ident) -> bool {
        self.is_external(src) || self.is_external(dst)
    }
}

struct BgpSession {
    src: Node,
    dst: Node,
    ty: Option<Ident>,
}

impl Parse for BgpSession {
    fn parse(input: ParseStream) -> Result<Self> {
        let src: Node = input.parse()?;
        let _: Token![->] = input.parse()?;
        let dst: Node = input.parse()?;
        let ty = if input.peek(Token![:]) {
            let _: Token![:] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(BgpSession { src, dst, ty })
    }
}

struct Link {
    src: Node,
    dst: Node,
    weight: f64,
    weight_span: Span,
}

impl Parse for Link {
    fn parse(input: ParseStream) -> Result<Self> {
        let src: Node = input.parse()?;
        let _: Token![->] = input.parse()?;
        let dst: Node = input.parse()?;
        let _: Token![:] = input.parse()?;
        let (weight, weight_span): (f64, Span) = if input.peek(LitInt) {
            let elem: LitInt = input.parse()?;
            let num: u128 = elem.base10_parse()?;
            (num as f64, elem.span())
        } else if input.peek(LitFloat) {
            let elem: LitFloat = input.parse()?;
            (elem.base10_parse()?, elem.span())
        } else {
            return Err(input.error("Expected a number (either a decimal number or a float)!"));
        };
        Ok(Link {
            src,
            dst,
            weight,
            weight_span,
        })
    }
}

enum MaybeExpr<T> {
    Expr(Expr),
    Other(T),
}

impl<T> MaybeExpr<T> {
    fn quote<'a, F>(&'a self, f: F) -> proc_macro2::TokenStream
    where
        F: FnOnce(&'a T) -> proc_macro2::TokenStream,
    {
        match self {
            MaybeExpr::Expr(e) => quote!(#e),
            MaybeExpr::Other(t) => f(t),
        }
    }
}

struct Route<N> {
    src: N,
    prefix: Prefix,
    as_path: MaybeExpr<Vec<LitInt>>,
    med: MaybeExpr<Option<LitInt>>,
    communities: MaybeExpr<Vec<LitInt>>,
}

impl Parse for Route<Node> {
    fn parse(input: ParseStream) -> Result<Self> {
        let src: Node = input.parse()?;
        let _: Token![->] = input.parse()?;
        let prefix: Prefix = input.parse()?;
        let _: Token![as] = input.parse()?;
        let content;
        braced!(content in input);
        let route_span = content.span();
        let route: Punctuated<_, Token![,]> = content.parse_terminated(FieldValue::parse)?;

        let missing_as_path = Error::new(route_span, "Missing an AS path!");

        let mut as_path = None;
        let mut med = MaybeExpr::Other(None);
        let mut communities = MaybeExpr::Other(Vec::new());

        fn parse_list_ints(expr: Expr) -> Result<MaybeExpr<Vec<LitInt>>> {
            match expr {
                Expr::Array(a) => a
                    .elems
                    .into_iter()
                    .map(|e| {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Int(num), ..
                        }) = e
                        {
                            Ok(num)
                        } else {
                            Err(Error::new(e.span(), "Expected a number!"))
                        }
                    })
                    .collect::<Result<Vec<LitInt>>>()
                    .map(MaybeExpr::Other),
                Expr::Lit(ExprLit {
                    lit: Lit::Int(num), ..
                }) => Ok(MaybeExpr::Other(vec![num])),
                expr => Ok(MaybeExpr::Expr(expr)),
            }
        }

        fn parse_med(expr: Expr) -> MaybeExpr<Option<LitInt>> {
            match expr {
                Expr::Lit(ExprLit {
                    lit: Lit::Int(num), ..
                }) => MaybeExpr::Other(Some(num)),
                expr => MaybeExpr::Expr(expr),
            }
        }

        for field in route {
            match field.member {
                syn::Member::Named(n) => match n.to_string().as_str() {
                    "as_path" | "path" => {
                        as_path = Some(parse_list_ints(field.expr)?);
                    }
                    "med" => {
                        med = parse_med(field.expr);
                    }
                    "community" | "communities" => {
                        communities = parse_list_ints(field.expr)?;
                    }
                    _ => {
                        return Err(Error::new(
                            n.span(),
                            "Unknown field! Expected either `path`, `med`, or `communities`",
                        ))
                    }
                },
                syn::Member::Unnamed(i) => {
                    return Err(Error::new(i.span(), "Only named attributes are allowed!"));
                }
            }
        }

        Ok(Self {
            src,
            prefix,
            as_path: as_path.ok_or(missing_as_path)?,
            med,
            communities,
        })
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum Prefix {
    Num(LitInt),
    Ip(PrefixInputNoCast),
    Ident(Ident),
}
impl Prefix {
    fn quote(&self) -> proc_macro2::TokenStream {
        match self {
            Prefix::Num(p) => quote!(#p),
            Prefix::Ip(p) => p.quote(),
            Prefix::Ident(p) => quote!(#p),
        }
    }

    fn span(&self) -> Span {
        match self {
            Prefix::Num(x) => x.span(),
            Prefix::Ip(x) => x.span,
            Prefix::Ident(x) => x.span(),
        }
    }
}

impl Parse for Prefix {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork_1 = input.fork();
        let fork_2 = input.fork();
        let span = input.span();
        if let Ok(num) = fork_1.parse() {
            input.advance_to(&fork_1);
            Ok(Prefix::Num(num))
        } else if fork_2.parse::<LitStr>().is_ok() {
            input.parse().map(Prefix::Ip)
        } else if let Ok(ident) = input.parse() {
            Ok(Prefix::Ident(ident))
        } else {
            Err(Error::new(
                span,
                "Expected either a number, a string literal containing an IP prefix, or an ident!",
            ))
        }
    }
}

struct Node {
    ident: Ident,
    ext: Option<(u32, Span)>,
}

impl Parse for Node {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        if input.peek(Token![!]) {
            let _: Token![!] = input.parse()?;
            let paren_parser;
            parenthesized!(paren_parser in input);
            let as_id: LitInt = paren_parser.parse()?;
            Ok(Self {
                ident,
                ext: Some((as_id.base10_parse()?, as_id.span())),
            })
        } else {
            Ok(Self { ident, ext: None })
        }
    }
}

enum Returns {
    Node(Ident),
    Tuple(Vec<Returns>),
}

impl Parse for Returns {
    fn parse(input: ParseStream) -> Result<Self> {
        Self::parse_expr(input.parse()?)
    }
}

impl Returns {
    fn parse_expr(expr: Expr) -> Result<Self> {
        match expr {
            Expr::Path(p) => {
                let p = p.path;
                if p.segments.len() != 1 || p.leading_colon.is_some() {
                    return Err(Error::new(p.span(), "Expected an ident, but found a path"));
                }
                Ok(Returns::Node(p.segments.into_iter().next().unwrap().ident))
            }
            Expr::Tuple(t) => t
                .elems
                .into_iter()
                .map(Returns::parse_expr)
                .collect::<Result<Vec<Returns>>>()
                .map(Self::Tuple),
            _ => Err(Error::new(
                expr.span(),
                "Expected either a literal or a tuple!",
            )),
        }
    }

    fn quote(self) -> proc_macro2::TokenStream {
        match self {
            Returns::Node(r) => quote!(#r),
            Returns::Tuple(t) => {
                let elements = t.into_iter().map(Returns::quote);
                quote!((#(#elements),*))
            }
        }
    }
}

#[derive(Clone, Copy)]
enum SessionType {
    Empty,
    EBgp,
    IBgpPeer,
    IBgpClient,
}

impl SessionType {
    fn try_from(value: &Option<Ident>) -> Result<Self> {
        if let Some(value) = value.as_ref() {
            match value.to_string().to_lowercase().as_ref() {
                "ebgp" | "external" => Ok(Self::EBgp),
                "peer" | "ibgppeer" => Ok(Self::IBgpPeer),
                "client" | "ibgpclient" => Ok(Self::IBgpClient),
                _ => Err(Error::new(
                    value.span(),
                    format!(
                        "Unknown BGP session type! Expected either `ebgp`, `peer`, or `client`, but got `{value}`!"
                    )
                ))
            }
        } else {
            Ok(Self::Empty)
        }
    }

    fn check(value: &Option<Ident>, external: bool) -> Result<()> {
        let s = Self::try_from(value)?;
        match (s, external) {
            (SessionType::Empty, _)
            | (SessionType::EBgp, true)
            | (SessionType::IBgpPeer, false)
            | (SessionType::IBgpClient, false) => Ok(()),
            (SessionType::EBgp, false) => Err(Error::new(
                value.as_ref().unwrap().span(),
                "Cannot establish an eBGP session between two internal routers!",
            )),
            (SessionType::IBgpPeer, true) |
            (SessionType::IBgpClient, true) => Err(Error::new(
                value.as_ref().unwrap().span(),
                "A BGP session type between an internal and external router must be either empty or `ebgp`!",
            )),
        }
    }
}
