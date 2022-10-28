use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

use crate::state::State;

pub struct MigrationCfg {
    state: Rc<State>,
    state_dispatch: Dispatch<State>,
}

pub enum Msg {
    State(Rc<State>),
}

impl Component for MigrationCfg {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        MigrationCfg {
            state: Default::default(),
            state_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let migration = self.state.get_migratoin();

        if migration.is_empty() {
            return html! {
                <div class="h-full w-full flex flex-col justify-center items-center">
                    <p class="text-gray-300 italic"> { "Migration is empty!" } </p>
                </div>
            };
        }

        html! {
            <div class="w-full space-y-2">
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(s) => {
                self.state = s;
                true
            }
        }
    }
}
