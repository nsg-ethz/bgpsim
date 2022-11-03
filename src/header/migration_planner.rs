use std::{ops::DerefMut, rc::Rc};

use netsim::config::NetworkConfig;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::Net,
    state::{Selected, State},
};

pub struct MigrationButton {
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
    state_dispatch: Dispatch<State>,
}

pub enum Msg {
    State(Rc<State>),
    StateNet(Rc<Net>),
    Show,
    Step,
    StepAll,
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {}

impl Component for MigrationButton {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        let state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        MigrationButton {
            net: Default::default(),
            net_dispatch,
            state_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let open_planner = ctx.link().callback(|_| Msg::Show);
        let step = ctx.link().callback(|_| Msg::Step);
        let step_all = ctx.link().callback(|_| Msg::StepAll);

        let migration = self.net.migration();

        if migration.len() == 0 {
            return html!();
        }

        let class = "space-x-4 rounded-full z-10 p-2 px-4 drop-shadow bg-white text-gray-700 flex justify-between items-center pointer-events-auto";
        let badge_class = "absolute inline-block top-2 right-2 bottom-auto left-auto translate-x-2/4 -translate-y-1/2 scale-x-100 scale-y-100 py-1 px-2.5 text-xs leading-none text-center whitespace-nowrap align-baseline font-bold bg-blue-700 text-white rounded-full z-10";

        let play_class = if migration.is_empty() {
            "text-gray-400 cursor-default pointer-events-none"
        } else {
            "text-gray-700 hover:text-green-700 pointer-events-auto"
        };

        let step_class = if migration.is_empty() {
            "text-gray-400 cursor-default pointer-events-none"
        } else {
            "text-gray-700 hover:text-blue-700 pointer-events-auto"
        };

        html! {
            <div {class}>
                <p class="mr-4">{ "Migration:" } </p>
                <button class={play_class} onclick={step_all}> <yew_lucide::ListVideo class="w-6 h-6"/> </button>
                <button class={step_class} onclick={step}> <yew_lucide::Forward class="w-6 h-6"/> </button>
                <div class={badge_class}>{migration.len()}</div>
                <button class="text-gray-700 hover:text-black" onclick={open_planner}> <yew_lucide::ListOrdered class="w-6 h-6"/> </button>
            </div>
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
            Msg::StepAll => {
                self.net_dispatch.reduce_mut(|n| {
                    let migration = std::mem::take(n.migration_mut().deref_mut());
                    for expr in migration {
                        n.net_mut().apply_modifier_unchecked(&expr).unwrap();
                    }
                });
                false
            }
            Msg::Step => {
                self.net_dispatch.reduce_mut(|n| {
                    if n.migration_mut().is_empty() {
                        // do nothing
                    } else {
                        let expr = n.migration_mut().remove(0);
                        n.net_mut().apply_modifier_unchecked(&expr).unwrap();
                    }
                });
                false
            }
        }
    }
}
