use std::rc::Rc;

use netsim::{formatter::NetworkFormatter, route_map::RouteMapSet, types::RouterId};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::net::Net;

use super::Select;

pub struct RouteMapSetCfg {
    v_r: NodeRef,
    v_s: String,
    v_v: SetValue,
    v_c: bool,
    net: Rc<Net>,
    _net_dispatch: Dispatch<BasicStore<Net>>,
}

pub enum Msg {
    StateNet(Rc<Net>),
    KindUpdate(RouteMapSet),
    InputUpdate,
    ValueUpdate,
    ValueUpdateRouter(RouterId),
    Delete,
}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub router: RouterId,
    pub index: usize,
    pub set: RouteMapSet,
    pub on_update: Callback<(usize, Option<RouteMapSet>)>,
}

impl Component for RouteMapSetCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        let mut s = RouteMapSetCfg {
            v_r: NodeRef::default(),
            v_s: String::new(),
            v_v: SetValue::None,
            v_c: true,
            net: Default::default(),
            _net_dispatch,
        };
        s.update_from_props(&ctx.props().set);
        s
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // first, get the network store.
        let kind_text = set_kind_text(&ctx.props().set);

        let is_nh = matches!(ctx.props().set, RouteMapSet::NextHop(_));

        let value_html = if is_nh {
            let options = self
                .net
                .net
                .get_topology()
                .node_indices()
                .map(|n| (n, n.fmt(&self.net.net).to_string()))
                .collect::<Vec<_>>();
            let current_text = self.v_v.fmt(&self.net);
            let on_select = ctx.link().callback(Msg::ValueUpdateRouter);
            html! {<div class="flex-1 ml-2"><Select<RouterId> text={current_text} {options} {on_select} button_class={Classes::from("text-sm")} /></div>}
        } else if matches!(self.v_v, SetValue::None) {
            html! {<div class="flex-1 ml-2"></div>}
        } else {
            let input_class = "flex-1 w-8 px-3 text-base font-normal bg-white bg-clip-padding border border-solid rounded transition ease-in-out m-0 focus:outline-none";
            let tt_class =
                "text-gray-700 border-blue-300 focus:border-blue-600 focus:text-gray-700";
            let tf_class = "text-gray-700 border-red-300 focus:border-red-600 focus:text-gray-700";
            let f_class = "text-gray-500 border-gray-300 focus:border-blue-600 focus:text-gray-700";
            let v_class = match (
                set_value(&ctx.props().set).fmt(&self.net) != self.v_s,
                self.v_c,
            ) {
                (true, true) => tt_class,
                (true, false) => tf_class,
                (false, _) => f_class,
            };
            let v_class = classes!(input_class, v_class, "ml-2");
            let v_u = ctx.link().callback(|_| Msg::InputUpdate);
            html! {
                <input type="text" class={v_class} value={self.v_s.clone()} onchange={v_u.reform(|_| ())} onkeypress={v_u.reform(|_| ())} onpaste={v_u.reform(|_| ())} oninput={v_u.reform(|_| ())} ref={self.v_r.clone()}/>
            }
        };

        let on_select = ctx.link().callback(Msg::KindUpdate);
        let on_delete = ctx.link().callback(|_| Msg::Delete);

        html! {
            <div class="w-full flex">
                <div class="w-36"><Select<RouteMapSet> text={kind_text} options={set_kind_options(ctx.props().router)} {on_select} button_class={Classes::from("text-sm")} /></div>
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
                self.v_v = SetValue::Router(r);
                self.update(ctx, Msg::ValueUpdate);
            }
            Msg::ValueUpdate => {
                if let Some(set) = set_update(&ctx.props().set, self.v_v) {
                    ctx.props().on_update.emit((ctx.props().index, Some(set)))
                }
            }
            Msg::Delete => ctx.props().on_update.emit((ctx.props().index, None)),
            Msg::InputUpdate => {
                self.v_s = self.v_r.cast::<HtmlInputElement>().unwrap().value();
                if let Some(val) = self.v_v.update(&self.v_s) {
                    self.v_v = val;
                    self.v_c = true;
                    self.update(ctx, Msg::ValueUpdate);
                } else {
                    self.v_c = false;
                }
            }
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.update_from_props(&ctx.props().set);
        true
    }
}

impl RouteMapSetCfg {
    fn update_from_props(&mut self, set: &RouteMapSet) {
        self.v_v = set_value(set);
        self.v_s = self.v_v.fmt(self.net.as_ref());
        self.v_c = true;
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum SetValue {
    None,
    Integer(u32),
    Float(f32),
    Router(RouterId),
}

impl SetValue {
    fn update(self, s: &str) -> Option<Self> {
        match self {
            SetValue::None => Some(SetValue::None),
            SetValue::Integer(_) => s.parse().ok().map(SetValue::Integer),
            SetValue::Float(_) => s.parse().ok().map(SetValue::Float),
            SetValue::Router(_) => s.parse().ok().map(|i: u32| SetValue::Router(i.into())),
        }
    }

    fn fmt(&self, net: &Net) -> String {
        match self {
            SetValue::None => String::new(),
            SetValue::Integer(x) => x.to_string(),
            SetValue::Float(x) => x.to_string(),
            SetValue::Router(r) => r.fmt(&net.net).to_string(),
        }
    }
}

fn set_kind_text(set: &RouteMapSet) -> &'static str {
    match set {
        RouteMapSet::NextHop(_) => "set Next Hop",
        RouteMapSet::LocalPref(Some(_)) => "set Local Pref",
        RouteMapSet::LocalPref(None) => "clear Local Pref",
        RouteMapSet::Med(Some(_)) => "set MED",
        RouteMapSet::Med(None) => "clear MED",
        RouteMapSet::IgpCost(_) => "IGP weight",
        RouteMapSet::SetCommunity(_) => "Set community",
        RouteMapSet::DelCommunity(_) => "Del community",
    }
}

fn set_kind_options(router: RouterId) -> Vec<(RouteMapSet, String)> {
    [
        RouteMapSet::NextHop(router),
        RouteMapSet::LocalPref(Some(100)),
        RouteMapSet::LocalPref(None),
        RouteMapSet::Med(Some(100)),
        RouteMapSet::Med(None),
        RouteMapSet::IgpCost(1.0),
        RouteMapSet::SetCommunity(0),
        RouteMapSet::DelCommunity(0),
    ]
    .map(|kind| {
        let text = set_kind_text(&kind).to_string();
        (kind, text)
    })
    .into_iter()
    .collect()
}

fn set_value(set: &RouteMapSet) -> SetValue {
    match set {
        RouteMapSet::NextHop(x) => SetValue::Router(*x),
        RouteMapSet::LocalPref(Some(x)) => SetValue::Integer(*x),
        RouteMapSet::LocalPref(None) => SetValue::None,
        RouteMapSet::Med(Some(x)) => SetValue::Integer(*x),
        RouteMapSet::Med(None) => SetValue::None,
        RouteMapSet::IgpCost(x) => SetValue::Float(*x),
        RouteMapSet::SetCommunity(x) => SetValue::Integer(*x),
        RouteMapSet::DelCommunity(x) => SetValue::Integer(*x),
    }
}

fn set_update(set: &RouteMapSet, val: SetValue) -> Option<RouteMapSet> {
    Some(match (set, val) {
        (RouteMapSet::NextHop(_), SetValue::Router(x)) => RouteMapSet::NextHop(x),
        (RouteMapSet::LocalPref(Some(_)), SetValue::Integer(x)) => RouteMapSet::LocalPref(Some(x)),
        (RouteMapSet::LocalPref(None), SetValue::None) => RouteMapSet::LocalPref(None),
        (RouteMapSet::Med(Some(_)), SetValue::Integer(x)) => RouteMapSet::Med(Some(x)),
        (RouteMapSet::Med(None), SetValue::None) => RouteMapSet::Med(None),
        (RouteMapSet::IgpCost(_), SetValue::Float(x)) => RouteMapSet::IgpCost(x),
        (RouteMapSet::SetCommunity(_), SetValue::Integer(x)) => RouteMapSet::SetCommunity(x),
        (RouteMapSet::DelCommunity(_), SetValue::Integer(x)) => RouteMapSet::DelCommunity(x),
        _ => return None,
    })
}
