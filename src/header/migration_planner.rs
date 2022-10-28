use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

use crate::state::{Selected, State};

pub struct MigrationButton {
    state: Rc<State>,
    state_dispatch: Dispatch<State>,
}

pub enum Msg {
    State(Rc<State>),
    Show,
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {}

impl Component for MigrationButton {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        MigrationButton {
            state: Default::default(),
            state_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_| Msg::Show);

        let count = self.state.get_migratoin().len();
        if count == 0 {
            return html!();
        }

        let badge_class = "absolute inline-block top-2 right-2 bottom-auto left-auto translate-x-2/4 -translate-y-1/2 scale-x-100 scale-y-100 py-1 px-2.5 text-xs leading-none text-center whitespace-nowrap align-baseline font-bold bg-blue-700 text-white rounded-full z-10";
        let class = "space-x-4 rounded-full z-10 p-2 px-4 drop-shadow bg-white text-gray-700 pointer-events-auto";

        html! {
            <div class="relative">
                <button {class} {onclick}>{ "Migration" }</button>
                <div class={badge_class}>{count}</div>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(s) => {
                self.state = s;
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
