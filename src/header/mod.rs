use std::rc::Rc;

use netsim::{formatter::NetworkFormatter, types::Prefix};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    components::{dropdown_menu::ButtonStyle, DropdownMenu},
    icons::Icon,
    net::Net,
    state::{Hover, Layer, State},
};

pub enum Msg {
    State(Rc<State>),
    StateNet(Rc<Net>),
    MenuAction(MenuAction),
    ChangeLayer(Layer),
    ChangePrefix(Prefix),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Import,
    Export,
    Settings,
    Help,
}

pub struct Header {
    header_text: String,
    from_hover: bool,
    layer_text: String,
    state: Rc<State>,
    net: Rc<Net>,
    prefixes: Vec<Prefix>,
    state_dispatch: Dispatch<BasicStore<State>>,
    _net_dispatch: Dispatch<BasicStore<Net>>,
}

impl Component for Header {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let state_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
        let _net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        Self {
            header_text: String::new(),
            from_hover: false,
            layer_text: Layer::default().to_string(),
            prefixes: Default::default(),
            state: Default::default(),
            net: Default::default(),
            state_dispatch,
            _net_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let text_class = if self.from_hover {
            "ml-2 flex-1 font-bold italic"
        } else {
            "ml-2 flex-1 font-bold"
        };

        let menu_button_text = "Menu".to_string();
        let on_menu_select = ctx.link().callback(Msg::MenuAction);

        let layer_button_text = format!("Layer: {}", self.layer_text);
        let on_layer_select = ctx.link().callback(Msg::ChangeLayer);

        let prefix_button_text = self
            .state
            .prefix()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "No Prefix".to_string());
        let on_prefix_select = ctx.link().callback(Msg::ChangePrefix);
        html! {
            <div class="w-full flex-1 p-4 bg-white drop-shadow-lg flex">
                <DropdownMenu<MenuAction>
                    button_style={ButtonStyle::Flat}
                    button_icon={Icon::Menu}
                    button_text={menu_button_text}
                    expand_left=true
                    options={vec![
                        ("Import".to_string(), MenuAction::Import),
                        ("Export".to_string(), MenuAction::Export),
                        ("Settings".to_string(), MenuAction::Settings),
                        ("Help".to_string(), MenuAction::Help)]}
                    on_select={on_menu_select} />
                <p class={text_class}>{ self.header_text.as_str() }</p>
                if self.state.layer() == Layer::FwState {
                    <DropdownMenu<Prefix>
                        button_style={ButtonStyle::Flat}
                        button_icon={Icon::ChevronDown}
                        button_text={prefix_button_text}
                        expand_left=false
                        options={self.prefixes.clone().into_iter().map(|p| (p.to_string(), p)).collect::<Vec<(String, Prefix)>>()}
                        on_select={on_prefix_select} />
                }
                <DropdownMenu<Layer>
                    button_style={ButtonStyle::Flat}
                    button_icon={Icon::ChevronDown}
                    button_text={layer_button_text}
                    expand_left=false
                    options={vec![
                        (Layer::FwState.to_string(), Layer::FwState),
                        (Layer::Igp.to_string(), Layer::Igp),
                        (Layer::Bgp.to_string(), Layer::Bgp)]}
                    on_select={on_layer_select} />
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(s) => {
                self.state = s;
                (self.header_text, self.from_hover) = header_msg(&self.state, &self.net);
                self.layer_text = self.state.layer().to_string();
                true
            }
            Msg::StateNet(n) => {
                self.net = n;
                let (new_msg, from_hover) = header_msg(&self.state, &self.net);
                let new_prefixes =
                    Vec::from_iter(self.net.net.get_known_prefixes().iter().cloned());
                if (&new_msg, from_hover, &new_prefixes)
                    != (&self.header_text, self.from_hover, &self.prefixes)
                {
                    self.header_text = new_msg;
                    self.from_hover = from_hover;
                    self.prefixes = new_prefixes;
                    if self.state.prefix().is_none() && !self.prefixes.is_empty() {
                        let p = *self.prefixes.first().unwrap();
                        self.state_dispatch.reduce(move |s| s.set_prefix(Some(p)));
                    } else if self.state.prefix().is_some() && self.prefixes.is_empty() {
                        self.state_dispatch.reduce(move |s| s.set_prefix(None));
                    }
                    true
                } else {
                    false
                }
            }
            Msg::MenuAction(a) => {
                match a {
                    MenuAction::Import => log::info!("Import"),
                    MenuAction::Export => log::info!("Export"),
                    MenuAction::Settings => log::info!("Settings"),
                    MenuAction::Help => log::info!("Help"),
                }
                true
            }
            Msg::ChangeLayer(l) => {
                self.state_dispatch.reduce(move |s| s.set_layer(l));
                false
            }
            Msg::ChangePrefix(p) => {
                self.state_dispatch.reduce(move |s| s.set_prefix(Some(p)));
                false
            }
        }
    }
}

fn header_msg(state: &State, net: &Net) -> (String, bool) {
    match state.hover() {
        Hover::None => (
            match state.selected() {
                crate::state::Selected::None => String::new(),
                crate::state::Selected::Router(r) => format!("Router: {}", r.fmt(&net.net)),
                crate::state::Selected::BgpSession(src, dst) => format!(
                    "Bgp Session between {} and {}",
                    src.fmt(&net.net),
                    dst.fmt(&net.net)
                ),
            },
            false,
        ),
        Hover::Router(r) => (format!("Router: {}", r.fmt(&net.net)), true),
        Hover::BgpSession(src, dst) => (
            format!(
                "Bgp Session between {} and {}",
                src.fmt(&net.net),
                dst.fmt(&net.net)
            ),
            true,
        ),
    }
}
