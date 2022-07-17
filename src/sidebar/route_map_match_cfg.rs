use std::rc::Rc;

use netsim::{
    formatter::NetworkFormatter,
    route_map::{RouteMapMatch, RouteMapMatchAsPath, RouteMapMatchClause},
    types::RouterId,
};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::net::Net;

use super::Select;

pub struct RouteMapMatchCfg {
    v1r: NodeRef,
    v2r: NodeRef,
    v1s: String,
    v2s: String,
    v1v: MatchValue,
    v2v: MatchValue,
    v1c: bool,
    v2c: bool,
    net: Rc<Net>,
    _net_dispatch: Dispatch<Net>,
}

pub enum Msg {
    StateNet(Rc<Net>),
    KindUpdate(RouteMapMatch),
    Input1Update,
    Input2Update,
    ValueUpdate,
    ValueUpdateRouter(RouterId),
    Delete,
}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub router: RouterId,
    pub index: usize,
    pub m: RouteMapMatch,
    pub on_update: Callback<(usize, Option<RouteMapMatch>)>,
}

impl Component for RouteMapMatchCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        let mut s = RouteMapMatchCfg {
            v1r: NodeRef::default(),
            v2r: NodeRef::default(),
            v1s: String::new(),
            v2s: String::new(),
            v1v: MatchValue::None,
            v2v: MatchValue::None,
            v1c: true,
            v2c: true,
            net: Default::default(),
            _net_dispatch,
        };
        s.update_from_props(&ctx.props().m);
        s
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // first, get the network store.
        let peers: Vec<RouterId> = self
            .net
            .net()
            .get_device(ctx.props().router)
            .internal()
            .map(|r| r.get_bgp_sessions().map(|(r, _)| *r).collect())
            .unwrap_or_default();

        let kind_text = match_kind_text(&ctx.props().m);

        let is_peer = matches!(ctx.props().m, RouteMapMatch::Neighbor(_));
        let is_nh = matches!(ctx.props().m, RouteMapMatch::NextHop(_));

        let value_html = if is_peer || is_nh {
            let options: Vec<(RouterId, String)> = if is_peer {
                peers
                    .iter()
                    .map(|n| (*n, n.fmt(&self.net.net()).to_string()))
                    .collect()
            } else {
                self.net
                    .net()
                    .get_topology()
                    .node_indices()
                    .map(|n| (n, n.fmt(&self.net.net()).to_string()))
                    .collect::<Vec<_>>()
            };
            let current_text = self.v1v.fmt(&self.net);
            let on_select = ctx.link().callback(Msg::ValueUpdateRouter);
            html! {<div class="flex-1 ml-2"><Select<RouterId> text={current_text} {options} {on_select} button_class={Classes::from("text-sm")} /></div>}
        } else {
            let input_class = "flex-1 w-8 px-3 text-base font-normal bg-white bg-clip-padding border border-solid rounded transition ease-in-out m-0 focus:outline-none";
            let tt_class =
                "text-gray-700 border-blue-300 focus:border-blue-600 focus:text-gray-700";
            let tf_class = "text-gray-700 border-red-300 focus:border-red-600 focus:text-gray-700";
            let f_class = "text-gray-500 border-gray-300 focus:border-blue-600 focus:text-gray-700";
            let (original_v1, original_v2) = match_values(&ctx.props().m);
            let v1class = match (original_v1.fmt(&self.net) != self.v1s, self.v1c) {
                (true, true) => tt_class,
                (true, false) => tf_class,
                (false, _) => f_class,
            };
            let v1class = classes!(input_class, v1class, "ml-2");
            let v1u = ctx.link().callback(|_| Msg::Input1Update);
            let v1_html = html! {
                <input type="text" class={v1class} value={self.v1s.clone()} onchange={v1u.reform(|_| ())} onkeypress={v1u.reform(|_| ())} onpaste={v1u.reform(|_| ())} oninput={v1u.reform(|_| ())} ref={self.v1r.clone()}/>
            };
            if !matches!(self.v2v, MatchValue::None) {
                let v2class = match (original_v2.fmt(&self.net) != self.v2s, self.v2c) {
                    (true, true) => tt_class,
                    (true, false) => tf_class,
                    (false, _) => f_class,
                };
                let v2class = classes!(input_class, v2class, "ml-0");
                let v2u = ctx.link().callback(|_| Msg::Input2Update);
                html! {
                    <>
                        {v1_html} {"-"}
                        <input type="text" class={v2class} value={self.v2s.clone()} onchange={v2u.reform(|_| ())} onkeypress={v2u.reform(|_| ())} onpaste={v2u.reform(|_| ())} oninput={v2u.reform(|_| ())} ref={self.v2r.clone()}/>
                    </>
                }
            } else {
                v1_html
            }
        };

        let on_select = ctx.link().callback(Msg::KindUpdate);
        let on_delete = ctx.link().callback(|_| Msg::Delete);

        html! {
            <div class="w-full flex">
                <div class="w-40"><Select<RouteMapMatch> text={kind_text} options={match_kind_options(&peers)} {on_select} button_class={Classes::from("text-sm")} /></div>
                { value_html }
                <button class="ml-2 hover hover:text-red-700 focus:outline-none transition duration-150 ease-in-out" onclick={on_delete}> <yew_lucide::X class="w-3 h-3 text-center" /> </button>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateNet(n) => {
                self.net = n;
            }
            Msg::KindUpdate(k) => ctx.props().on_update.emit((ctx.props().index, Some(k))),
            Msg::ValueUpdateRouter(r) => {
                self.v1v = MatchValue::Router(r);
                Component::update(self, ctx, Msg::ValueUpdate);
            }
            Msg::ValueUpdate => {
                if let Some(m) = match_update(&ctx.props().m, self.v1v, self.v2v) {
                    ctx.props().on_update.emit((ctx.props().index, Some(m)))
                }
            }
            Msg::Delete => ctx.props().on_update.emit((ctx.props().index, None)),
            Msg::Input1Update => {
                self.v1s = self
                    .v1r
                    .cast::<HtmlInputElement>()
                    .map(|e| e.value())
                    .unwrap_or_default();
                if let Some(val) = self.v1v.update(&self.v1s) {
                    self.v1v = val;
                    self.v1c = true;
                    Component::update(self, ctx, Msg::ValueUpdate);
                } else {
                    self.v1c = false;
                }
            }
            Msg::Input2Update => {
                self.v2s = self
                    .v2r
                    .cast::<HtmlInputElement>()
                    .map(|e| e.value())
                    .unwrap_or_default();
                if let Some(val) = self.v2v.update(&self.v2s) {
                    self.v2v = val;
                    self.v2c = true;
                    Component::update(self, ctx, Msg::ValueUpdate);
                } else {
                    self.v2c = false;
                }
            }
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.update_from_props(&ctx.props().m);
        true
    }
}

impl RouteMapMatchCfg {
    fn update_from_props(&mut self, m: &RouteMapMatch) {
        (self.v1v, self.v2v) = match_values(m);
        (self.v1s, self.v2s) = (
            self.v1v.fmt(self.net.as_ref()),
            self.v2v.fmt(self.net.as_ref()),
        );
        (self.v1c, self.v2c) = (true, true)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum MatchValue {
    None,
    Integer(u32),
    Router(RouterId),
}

impl MatchValue {
    fn update(self, s: &str) -> Option<Self> {
        match self {
            MatchValue::None => Some(MatchValue::None),
            MatchValue::Integer(_) => s.parse().ok().map(MatchValue::Integer),
            MatchValue::Router(_) => s.parse().ok().map(|i: u32| MatchValue::Router(i.into())),
        }
    }

    fn fmt(&self, net: &Net) -> String {
        match self {
            MatchValue::None => String::new(),
            MatchValue::Integer(x) => x.to_string(),
            MatchValue::Router(r) => r.fmt(&net.net()).to_string(),
        }
    }
}

fn match_kind_text(m: &RouteMapMatch) -> &'static str {
    match m {
        RouteMapMatch::Neighbor(_) => "Peer is",
        RouteMapMatch::Prefix(RouteMapMatchClause::Equal(_)) => "Prefix is",
        RouteMapMatch::Prefix(_) => "Prefix in",
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Contains(_)) => "Path has",
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(RouteMapMatchClause::Equal(_))) => {
            "Path len is"
        }
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(_)) => "Path len in",
        RouteMapMatch::NextHop(_) => "Next-Hop is",
        RouteMapMatch::Community(_) => "Community has",
    }
}

fn match_kind_options(peers: &[RouterId]) -> Vec<(RouteMapMatch, String)> {
    [
        RouteMapMatch::Neighbor(peers.get(0).copied().unwrap_or_else(|| 0.into())),
        RouteMapMatch::Prefix(RouteMapMatchClause::Equal(0.into())),
        RouteMapMatch::Prefix(RouteMapMatchClause::Range(0.into(), 0.into())),
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Contains(0.into())),
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(RouteMapMatchClause::Equal(0))),
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(RouteMapMatchClause::Range(
            0, 0,
        ))),
        RouteMapMatch::NextHop(0.into()),
        RouteMapMatch::Community(0),
    ]
    .map(|kind| {
        let text = match_kind_text(&kind).to_string();
        (kind, text)
    })
    .into_iter()
    .collect()
}

fn match_values(m: &RouteMapMatch) -> (MatchValue, MatchValue) {
    match m {
        RouteMapMatch::Neighbor(v) => (MatchValue::Router(*v), MatchValue::None),
        RouteMapMatch::Prefix(RouteMapMatchClause::Equal(v)) => {
            (MatchValue::Integer(v.0), MatchValue::None)
        }
        RouteMapMatch::Prefix(RouteMapMatchClause::Range(v1, v2)) => {
            (MatchValue::Integer(v1.0), MatchValue::Integer(v2.0))
        }
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Contains(v)) => {
            (MatchValue::Integer(v.0), MatchValue::None)
        }
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(RouteMapMatchClause::Equal(v))) => {
            (MatchValue::Integer(*v as u32), MatchValue::None)
        }
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(RouteMapMatchClause::Range(v1, v2))) => (
            MatchValue::Integer(*v1 as u32),
            MatchValue::Integer(*v2 as u32),
        ),
        RouteMapMatch::NextHop(v) => (MatchValue::Router(*v), MatchValue::None),
        RouteMapMatch::Community(v) => (MatchValue::Integer(*v), MatchValue::None),
        _ => (MatchValue::None, MatchValue::None),
    }
}

fn match_update(m: &RouteMapMatch, val1: MatchValue, val2: MatchValue) -> Option<RouteMapMatch> {
    Some(match (m, val1, val2) {
        (RouteMapMatch::Neighbor(_), MatchValue::Router(r), MatchValue::None) => {
            RouteMapMatch::Neighbor(r)
        }
        (
            RouteMapMatch::Prefix(RouteMapMatchClause::Equal(_)),
            MatchValue::Integer(x),
            MatchValue::None,
        ) => RouteMapMatch::Prefix(RouteMapMatchClause::Equal(x.into())),
        (
            RouteMapMatch::Prefix(RouteMapMatchClause::Range(_, _)),
            MatchValue::Integer(x),
            MatchValue::Integer(y),
        ) => RouteMapMatch::Prefix(RouteMapMatchClause::Range(x.into(), y.into())),
        (
            RouteMapMatch::AsPath(RouteMapMatchAsPath::Contains(_)),
            MatchValue::Integer(x),
            MatchValue::None,
        ) => RouteMapMatch::AsPath(RouteMapMatchAsPath::Contains(x.into())),
        (
            RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(RouteMapMatchClause::Equal(_))),
            MatchValue::Integer(x),
            MatchValue::None,
        ) => RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(RouteMapMatchClause::Equal(
            x as usize,
        ))),
        (
            RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(RouteMapMatchClause::Range(_, _))),
            MatchValue::Integer(x),
            MatchValue::Integer(y),
        ) => RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(RouteMapMatchClause::Range(
            x as usize, y as usize,
        ))),
        (RouteMapMatch::NextHop(_), MatchValue::Router(r), MatchValue::None) => {
            RouteMapMatch::NextHop(r)
        }
        (RouteMapMatch::Community(_), MatchValue::Integer(x), MatchValue::None) => {
            RouteMapMatch::Community(x)
        }
        _ => return None,
    })
}
