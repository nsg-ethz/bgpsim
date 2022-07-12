pub mod button;
pub mod divider;
pub mod element;
pub mod external_router_cfg;
pub mod multi_select;
mod queue_cfg;
mod route_map_cfg;
mod route_map_match_cfg;
mod route_map_set_cfg;
pub mod router_cfg;
pub mod select;
pub mod text_field;
pub mod toggle;
pub mod topology_cfg;

pub use button::Button;
pub use divider::{Divider, ExpandableDivider, ExpandableSection};
pub use element::Element;
pub use multi_select::MultiSelect;
pub use select::Select;
pub use text_field::TextField;
pub use toggle::Toggle;

use external_router_cfg::ExternalRouterCfg;
use queue_cfg::QueueCfg;
use router_cfg::RouterCfg;

use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::Net,
    state::{Selected, State},
};

pub struct Sidebar {
    state: Rc<State>,
    net: Rc<Net>,
    _state_dispatch: Dispatch<BasicStore<State>>,
    _net_dispatch: Dispatch<BasicStore<Net>>,
}

pub enum Msg {
    State(Rc<State>),
    StateNet(Rc<Net>),
}

impl Component for Sidebar {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let _state_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
        let _net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        Sidebar {
            state: Default::default(),
            net: Default::default(),
            _state_dispatch,
            _net_dispatch,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let content = match self.state.selected() {
            Selected::None => html! {
                <div class="h-full w-full flex flex-col justify-center items-center">
                    <p class="text-gray-300 italic"> { "nothing selected!" } </p>
                </div>
            },
            Selected::Router(r) if self.net.net.get_device(r).is_internal() => {
                html! { <RouterCfg router={r} /> }
            }
            Selected::Router(r) => html! { <ExternalRouterCfg router={r} /> },
            Selected::Queue => html! { <QueueCfg /> },
        };

        html! {
            <div class="w-96 h-full max-h-full pr-4 py-4 align-middle">
                <div class="w-full h-full max-h-full px-4 bg-white shadow-lg flex flex-col rounded-lg overflow-scroll">
                    { content }
                </div>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(s) => {
                self.state = s;
                true
            }
            Msg::StateNet(n) => {
                self.net = n;
                true
            }
        }
    }
}
