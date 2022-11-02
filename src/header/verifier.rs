use std::rc::Rc;

use netsim::policies::Policy;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::Net,
    state::{Selected, State},
};

pub struct Verifier {
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
    state_dispatch: Dispatch<State>,
    skip_update: bool,
}

pub enum Msg {
    State(Rc<State>),
    StateNet(Rc<Net>),
    Show,
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {}

impl Component for Verifier {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        Verifier {
            net: Default::default(),
            net_dispatch,
            state_dispatch,
            skip_update: false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let num_violations = self
            .net
            .spec()
            .values()
            .flatten()
            .filter(|(_, r)| r.is_err())
            .count();

        if self.net.spec().is_empty() {
            html!()
        } else if num_violations == 0 {
            let class = "space-x-4 rounded-full z-10 p-2 px-4 drop-shadow bg-white text-green-700 pointer-events-auto";
            let onclick = ctx.link().callback(|_| Msg::Show);
            html! {
                <button {class} {onclick}><yew_lucide::Check class="w-6 h-6"/></button>
            }
        } else {
            let badge_class = "absolute inline-block top-2 right-2 bottom-auto left-auto translate-x-2/4 -translate-y-1/2 scale-x-100 scale-y-100 py-1 px-2.5 text-xs leading-none text-center whitespace-nowrap align-baseline font-bold bg-red-700 text-white rounded-full z-10";
            let class = "space-x-4 rounded-full z-10 p-2 px-4 drop-shadow bg-white text-red-700 pointer-events-auto";
            let onclick = ctx.link().callback(|_| Msg::Show);
            html! {
                <div class="relative">
                    <button {class} {onclick}><yew_lucide::X class="w-6 h-6"/></button>
                    <div class={badge_class}>{num_violations}</div>
                </div>
            }
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(_) => false,
            Msg::StateNet(n) => {
                if self.skip_update {
                    self.skip_update = false;
                    false
                } else {
                    self.net = n;
                    self.net_dispatch.reduce_mut(verify);
                    self.skip_update = true;
                    true
                }
            }
            Msg::Show => {
                self.state_dispatch
                    .reduce_mut(|s| s.set_selected(Selected::Verifier));
                false
            }
        }
    }
}

fn verify(net: &mut Net) {
    let mut fw_state = net.net().get_forwarding_state();
    net.spec_mut()
        .values_mut()
        .flat_map(|x| x.iter_mut())
        .for_each(|(policy, val)| *val = policy.check(&mut fw_state));
}
