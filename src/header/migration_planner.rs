use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::Net,
    state::{Selected, State},
};

pub struct MigrationButton {
    net: Rc<Net>,
    state_dispatch: Dispatch<State>,
}

pub enum Msg {
    State(Rc<State>),
    StateNet(Rc<Net>),
    Show,
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {}

impl Component for MigrationButton {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        let state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        MigrationButton {
            net: Default::default(),
            state_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let open_planner = ctx.link().callback(|_| Msg::Show);

        let migration = self.net.migration();
        let step = self.net.migration_step;

        if migration.len() == 0 {
            return html!();
        }

        let class = "rounded-full z-10 p-2 px-4 drop-shadow hover:drop-shadow-lg bg-white text-gray-700 hover:text-black pointer-events-auto";
        let badge_class = "absolute inline-block top-2 right-2 bottom-auto left-auto translate-x-2/4 -translate-y-1/2 scale-x-100 scale-y-100 py-1 px-2.5 text-xs leading-none text-center whitespace-nowrap align-baseline font-bold bg-blue-700 text-white rounded-full z-10";

        html! {
            <button {class} onclick={open_planner}>
                { "Migration" }
                <div class={badge_class}>{migration.len()} {"/"} {step}</div>
            </button>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(_) => false,
            Msg::StateNet(n) => {
                self.net = n;
                true
            }
            Msg::Show => {
                self.state_dispatch
                    .reduce_mut(|s| s.set_selected(Selected::Migration));
                false
            }
        }
    }
}
