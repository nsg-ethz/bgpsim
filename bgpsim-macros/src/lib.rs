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

#![doc(html_logo_url = "https://bgpsim.github.io/dark_only.svg")]

use proc_macro::TokenStream;

mod formatter;
mod ip;
mod net;
use ip::PrefixInput;
use net::Net;
use syn::parse_macro_input;

/// Create a `Network` using a domain specific language. This proc-macro will check at compile time
/// that all invariants are satisfied.
///
/// # Syntax
/// The content can contain the following parts:
///
/// - `links`: An enumeration of links in the network. Each link is written as `SRC -> DST: WEIGHT`,
///   where both `SRC` and `DST` are identifiers of a node, and `WEIGHT` is a number defining the
///   weight. By default, this notation will automatically create the link, and set the link weight
///   in both directions. However, you can also set the link weight in the opposite direction by
///   writing `DST -> SRC: WEIGHT`.
///
/// - `sessions`: An enumeration of all BGP sessions in the network. Each session is written as `SRC
///   -> DST[: TYPE]`, where both `SRC` and `DST` are identifiers of a node. The `TYPE` is optiona,
///   and can be omitted. If the type is omitted, then it will be a `BgpSessionType::IBgpPeer` for
///   internal sessions, and `BgpSessionType::EBgp` for external sessions. The `TYPE` can be one of
///   the following identifiers:
///
///   - `ebgp`, which maps to `BgpSessionType::EBgp`,
///   - `peer`, which maps to `BgpSessionType::IBgpPeer`,
///   - `client`, which maps to `BgpSessionType::IBgpClient`.
///
///   This macro will **automatically add links between nodes for external sessions** if they are
///   not already defined in `links`.
///
/// - `routes`: An enumeration of all BGP announcements from external routers. Each announcement is
///   written as `SRC -> PREFIX as {path: P, [med: M], [communities: C]}`. The symbols mean the
///   following:
///   - `SRC` is the external router that announces the prefix.
///   - `PREFIX` is the prefix that should be announced. The prefix can either be a number, a string
///     containing an IP prefix (see [`prefix!`]), or an identifier of a local variable that was
///     already defined earlier.
///   - `P` is the AS path and is required. It can be either a single number (which will be turned
///     into a path of length 1), an array of numbers representing the path, or any other arbitrary
///     expression that evaluates to `impl Iterator<Item = I> where I: Into<AsId>`.
///   - `M` is the MED value but is optional. If omitted, then the MED value will not be set in the
///     announcement. `M` must be either a number, or an expression that evaluates to `Option<u32>`.
///   - `C` is the set of communities present in the route, and is optional. Similar to `P`, it can
///     also either take a single number, an array of numbers, or any other arbitrary expression
///     that evaluates to `impl Iterator<Item = I> where I: Into<u32>`.
///
/// - `Prefix`: The type of the prefix. Choose either `SinglePrefix`, `SimplePrefix`, or
///   `Ipv4Prefix` here (optional).
///
/// - `Queue`: The type of the queue (optional).
///
/// - `queue`: The expression to create the empty queue. If no queue is provided, then the expanded
///   macro will use `Default::default()`.
///
/// - `return`: A nested tuple of identifiers that referr to previously defined nodes.
///
/// # Defining external routers
/// Every node identifier can also be written like a macro invocation by appending a `!(AS_ID)`,
/// where `AS_ID` is a literal number. In that case, this node will be trned into an external router
/// that uses the given AS number. You only need to annotate an external router once!
///
/// # Example
/// ```rust
/// use bgpsim::prelude::*;
///
/// let (net, ((b0, b1), (e0, e1))) = net! {
///     Prefix = Ipv4Prefix;
///     links = {
///         b0 -> r0: 1;
///         r0 -> r1: 1;
///         r1 -> b1: 1;
///     };
///     sessions = {
///         b1 -> e1!(1);
///         b0 -> e0!(2);
///         r0 -> r1: peer;
///         r0 -> b0: client;
///         r1 -> b1: client;
///     };
///     routes = {
///         e0 -> "10.0.0.0/8" as {path: [1, 3, 4], med: 100, community: 20};
///         e1 -> "10.0.0.0/8" as {path: [2, 4]};
///     };
///     return ((b0, b1), (e0, e1))
/// };
/// ```
///
/// This example will be expanded into the following code. This code was cleaned-up, so the
/// different parts can be seen better.
///
/// ```rust
/// use bgpsim::prelude::*;
/// // these imports are added for compactness
/// use ipnet::Ipv4Net;
/// use std::net::Ipv4Addr;
///
/// let (_net, ((b0, b1), (e0, e1))) = {
///     let mut _net: Network<Ipv4Prefix, _> = Network::new(BasicEventQueue::default());
///     let b0 = _net.add_router("b0");
///     let b1 = _net.add_router("b1");
///     let r0 = _net.add_router("r0");
///     let r1 = _net.add_router("r1");
///     let e0 = _net.add_external_router("e0", 2u32);
///     let e1 = _net.add_external_router("e1", 1u32);
///
///     _net.add_link(b0, r0);
///     _net.add_link(r1, b1);
///     _net.add_link(r0, r1);
///     _net.add_link(b1, e1);
///     _net.add_link(b0, e0);
///
///     _net.set_link_weight(b0, r0, 1f64).unwrap();
///     _net.set_link_weight(r0, b0, 1f64).unwrap();
///     _net.set_link_weight(r1, b1, 1f64).unwrap();
///     _net.set_link_weight(b1, r1, 1f64).unwrap();
///     _net.set_link_weight(r0, r1, 1f64).unwrap();
///     _net.set_link_weight(r1, r0, 1f64).unwrap();
///
///     _net.set_bgp_session(b0, e0, Some(BgpSessionType::EBgp)).unwrap();
///     _net.set_bgp_session(r1, b1, Some(BgpSessionType::IBgpClient)).unwrap();
///     _net.set_bgp_session(r0, r1, Some(BgpSessionType::IBgpPeer)).unwrap();
///     _net.set_bgp_session(b1, e1, Some(BgpSessionType::EBgp)).unwrap();
///     _net.set_bgp_session(r0, b0, Some(BgpSessionType::IBgpClient)).unwrap();
///
///     _net.advertise_external_route(
///             e0,
///             Ipv4Net::new(Ipv4Addr::new(10, 0, 0, 0),8).unwrap(),
///             [1, 3, 4],
///             Some(100),
///             [20],
///         ).unwrap();
///     _net.advertise_external_route(
///             e1,
///             Ipv4Net::new(Ipv4Addr::new(10, 0, 0, 0),8).unwrap(),
///             [2, 4],
///             None,
///             [],
///         ).unwrap();
///     (_net, ((b0, b1), (e0, e1)))
/// };
/// ```
///
/// ## Order or assigned Router-IDs
///
/// The router-IDs are assigned in order of their first occurrence. The first named router will be
/// assigned id 0, the second 1, and so on. The first occurrence must not necessarily be in the
/// `routers` block, but it also includes the mentioning of a router in a link or BGP session. Here
/// is an example:
///
/// ```rust
/// use bgpsim::prelude::*;
///
/// let (net, ((b0, b1), (r0, r1), (e0, e1))) = net! {
///     Prefix = Ipv4Prefix;
///     links = {
///         b0 -> r0: 1;
///         r0 -> r1: 1;
///         r1 -> b1: 1;
///     };
///     sessions = {
///         b1 -> e1!(1);
///         b0 -> e0!(2);
///         r0 -> r1: peer;
///         r0 -> b0: client;
///         r1 -> b1: client;
///     };
///     routes = {
///         e0 -> "10.0.0.0/8" as {path: [1, 3, 4], med: 100, community: 20};
///         e1 -> "10.0.0.0/8" as {path: [2, 4]};
///     };
///     return ((b0, b1), (r0, r1), (e0, e1))
/// };
///
/// assert_eq!(b0.index(), 0);
/// assert_eq!(r0.index(), 1);
/// assert_eq!(r1.index(), 2);
/// assert_eq!(b1.index(), 3);
/// assert_eq!(e1.index(), 4);
/// assert_eq!(e0.index(), 5);
/// ```
#[proc_macro]
pub fn net(input: TokenStream) -> TokenStream {
    // 1. Use syn to parse the input tokens into a syntax tree.
    // 2. Use quote to generate new tokens based on what we parsed.
    // 3. Return the generated tokens.
    parse_macro_input!(input as Net).quote()
}

/// Create a `Prefix` from an [`ipnet::Ipv4Net`] string. If you provide an `as`, you can
/// specify to which type the resulting `Ipv4Net` will be casted. If you omit the type parameter
/// after `as`, then the macro will simply invoke `.into()` on the generated `IpvtNet`.
///
/// ```
/// # use bgpsim_macros::*;
/// # use ipnet::Ipv4Net as P;
/// // `p` will be an `Ipv4Net`
/// let p = prefix!("192.168.0.0/24");
///
/// // `p` will have type `P`, but `P` must implement `From<Ipv4Net>`.
/// let p = prefix!("192.168.0.0/24" as P);
/// let p: P = prefix!("192.168.0.0/24" as);
/// ```
#[proc_macro]
pub fn prefix(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as PrefixInput).quote()
}

/// Automatically implement the NetworkFormatter for the given type. The strings are generated
/// similar to the derived `std::fmt::Debug` implementation.
///
/// You can control the way in which individual fields are formatted. To do so, you can use the
/// `#[formatter(...)]` attribute. You can use the following values:
///
/// - `skip` will skip that field entirely.
/// - `fmt = ...` controls which function to use for the (single-line) formatting. You have the
///   following options:
///   - `path::to::fn`: A path to a function that takes a reference to the value and to the network
///     the same function signature as `NetworkFormatter::fmt`. If you pick a
///     custom function without specifying a `multiline` attribute, then the same function will be
///     used when formatting the field for multiple lines.
///   - `"fmt"`: The default (single-line) formatter (used by default). See `NetworkFormatter::fmt`.
///   - `"fmt_set`: Format any iterable as a set. See `NetworkFormatterSequence::fmt_set`.
///   - `"fmt_list`: Format any iterable as a list. See `NetworkFormatterSequence::fmt_list`.
///   - `"fmt_path`: Format any iterable as a path, in the form of `a -> b -> c`. See
///     `NetworkFormatterSequence::fmt_path`.
///   - `"fmt_map`: Format the content as a mapping. See `NetworkFormatterMap::fmt_map`.
///   - `"fmt_map`: Format the content as a mapping. See `NetworkFormatterMap::fmt_map`.
///   - `"fmt_path_options`: Format a nested iterator as a path option set, in the form of `a -> b |
///     a -> b -> c`. See `NetworkFormatterNestedSequence::fmt_path_options`.
///   - `"fmt_path_set`: Format a nested iterator as a path option set, in the form of `{a -> b,
///     a -> b -> c}`. See `NetworkFormatterNestedSequence::fmt_path_set`.
///   - `"fmt_ext`: Format any iterable using the extension formatter, see
///     `NetworkFormatterExt::fmt_ext`.
/// - `multiline = ...` controls which function to use for the multiline formatting. By default, it
///   will pick the multi-line variant of the `fmt` option (for instance, setting `fmt = "fmt_set"`
///   will automatically configure `multiline = "fmt_set_multiline"`). In addition to those, you
///   have the following options:
///   - `path::to::fn`: A path to a function that takes a reference to the value, to the network,
///     and an usize counting the current indentation level. It must have the same function
///     signature as `NetworkFormatter::fmt_multiline_indent`.
///   - `"fmt_multiline"`: The default multi-line formatter (used by default). See
///     `NetworkFormatter::fmt_multiline_indent`.
///   - `"fmt_set_multiline`: Format any iterable as a set. See
///     `NetworkFormatterSequence::fmt_set_multiline`.
///   - `"fmt_list_multiline`: Format any iterable as a list. See
///     `NetworkFormatterSequence::fmt_list_multiline`.
///   - `"fmt_map_multiline`: Format the content as a mapping. See
///     `NetworkFormatterMap::fmt_map_multiline`.
///   - `"fmt_path_multiline`: Format the content as a set of paths. See
///     `NetworkFormatterNestedSequence::fmt_path_multiline`.
///
/// ```
/// use bgpsim::prelude::*;
/// # use std::collections::HashSet;
///
/// #[derive(NetworkFormatter)]
/// struct Foo {
///     /// Will be printed regularly
///     counter: usize
///     /// Will be hidden
///     #[formatter(skip)]
///     internal_counter: usize
///     /// This will print a path instead of a list
///     #[formatter(fmt = "fmt_path")]
///     path: Vec<RouterId>,
///     /// Do not print this field with multiple lines
///     #[formatter(multiline = "fmt")]
///     visited: HashSet<RouterId>,
/// }
/// ```
#[proc_macro_derive(NetworkFormatter, attributes(formatter))]
pub fn network_formatter_derive(input: TokenStream) -> TokenStream {
    formatter::derive(input)
}
