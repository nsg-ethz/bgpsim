// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022-2023 Tibor Schneider <sctibor@ethz.ch>
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

use bgpsim::prelude::InteractiveNetwork;
use gloo_events::EventListener;
use gloo_utils::{document, window};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::Net,
    point::Point,
    state::{Layer, Selected, State},
};

const STEPS: &[TourStep] = &[
    TourStep::Text(
        &[
            #[cfg(feature = "atomic_bgp")]
            "Welcome to the online simulator for Chameleon.",
            #[cfg(not(feature = "atomic_bgp"))]
            "Welcome to BgpSim, the online simulator for BGP networks.",
            "In a few steps, this tutorial will show you how to use this simulator."
        ],
        &[]
    ),
    TourStep::Element {
        element_id: "layer-selection",
        alternative: None,
        actions: &[
            #[cfg(feature = "atomic_bgp")]
            Action::ChooseLayer(Layer::RouteProp),
            #[cfg(not(feature = "atomic_bgp"))]
            Action::ChooseLayer(Layer::FwState),
        ],
        paragraphs: &["The simulator offers visualization layers for many different aspects of the network. You can select which aspect should be visualized using this button. The main section below can visualize the forwarding state, the routing state (how routes are propagated), the IGP configuration, or the BGP configuration."],
        align: Align::Bottom,
    },
    TourStep::Element {
        element_id: "prefix-selection",
        alternative: None,
        actions: &[Action::CreateFirstRouter, Action::SelectFirstRouter],
        paragraphs: &["Some layers only visualize the state for a given prefix. This selection allows you to change that prefix!"],
        align: Align::Bottom,
    },
    TourStep::Element {
        element_id: "add-new-router",
        alternative: Some(&["The simulator distinguishes between internal routers and external networks (routers). External networks only advertise BGP routes, while internal routers run BGP and OSPF."]),
        actions: &[],
        paragraphs: &[
            "The simulator distinguishes between internal routers and external networks (routers). External networks only advertise BGP routes, while internal routers run BGP and OSPF.",
            "You can add internal routers or external networks using this button."
        ],
        align: Align::Bottom,
    },
    TourStep::Element {
        element_id: "selected-router",
        alternative: None,
        actions: &[],
        paragraphs: &["You can rearrange the network by dragging nodes arround. By right-clicking on a node, you can create a new link or establish a new BGP session."],
        align: Align::Bottom,
    },
    TourStep::Element {
        element_id: "sidebar",
        alternative: None,
        actions: &[],
        paragraphs: &[
            "After selecting a router, the sidebar shows all configuration options for that router. In this window, you can modify the OSPF and BGP configuration.",
            #[cfg(feature = "atomic_bgp")]
            "To make the scenarios work properly, please do not change these config options, but you can inspect them.",
        ],
        align: Align::Left,
    },
    TourStep::Element {
        element_id: "queue-controls",
        alternative: None,
        actions: &[Action::ShowQueue],
        paragraphs: &[
            "The BGP simulator is based on an event queue. The simulator is in manual simulation mode, meaning BGP message are not automatically processed.",
            "The button in the middle will execute the next euqueued event.",
            "The left button will execute all events until either all messages are handled, or any forwarding policy is violated.",
            "Finally, the right button shows displays the queue in the sidebar, where you can arbitrarily reorder messages (as long as the message ordering of a single session is not violated)."
        ],
        align: Align::BottomLeft,
    },
    #[cfg(feature = "atomic_bgp")]
    TourStep::Element {
        element_id: "migration-button",
        alternative: None,
        actions: &[Action::ShowMigration],
        paragraphs: &[
            "The current scenario comes with a predefined migration plan. By clicking this button, the complete migration plan shows on the right.",
        ],
        align: Align::BottomLeft,
    },
    #[cfg(feature = "atomic_bgp")]
    TourStep::Element {
        element_id: "sidebar",
        alternative: None,
        actions: &[],
        paragraphs: &[
            "The migration is separated into three pahses: the setup, the main update phase, and the cleanup. Each phase is further divided into multiple steps.",
            "Chameleon requires each step to be completed before moving to the next one. Within each step, different commands can be executed in any order, as long as their precondition is satisfied.",
            "The sidebar will show all individual commands, including their pre- and postconditions. By clicking on any command (that has its preconditions satisfied), the modifications are performed.",
            "Note, that the network does by default not automatically process BGP messages. You will need to process some messages of the queue in order to complete the migration.",
        ],
        align: Align::Left,
    },
    #[cfg(feature = "atomic_bgp")]
    TourStep::Element {
        element_id: "specification-button",
        alternative: None,
        actions: &[Action::ShowSpecification],
        paragraphs: &[
            "The simulator comes with a built-in verifier. At every step, the simulator will check all forwarding properties, and notify you as soon as any property is violated.",
            "By clicking on this button, the sidebar on the right will show all properties (and which of them are violated)."
        ],
        align: Align::BottomLeft,
    },
    TourStep::Text(
        &[
            "You now understand the basics of the simulator.",
            #[cfg(feature = "atomic_bgp")]
            "Please consider reading the paper for information on how the migration plan is generated, and which guarantees it can provide.",
        ],
        &[Action::SelectNothing]
    ),
];

const HIGHLIGHT_PADDING: f64 = 20.0;
const BOX_PADDING: f64 = 40.0;
const BOX_WIDTH: f64 = 400.0;
const BOX_HEIGHT: f64 = 200.0;

#[function_component]
pub fn Tour() -> Html {
    let (net, _) = use_store::<Net>();

    let tour_complete = use_selector(|state: &State| state.is_tour_complete());

    let step = use_state_eq(|| 0);
    // create a trigger on resize, that will simply re-compute the component.
    let trigger = use_force_update();
    let _onresize = use_state(|| {
        EventListener::new(window().as_ref(), "resize", move |_| trigger.force_update())
    });

    if *tour_complete {
        step.set(0);
        return html!();
    }

    // check if we exceeded all steps
    if *step >= STEPS.len() {
        Dispatch::<State>::new().reduce_mut(|s| s.set_tour_complete());
        return html!();
    }

    // get the screen dimension
    let width = f64::try_from(window().inner_width().unwrap()).unwrap();
    let height = f64::try_from(window().inner_height().unwrap()).unwrap();

    let first = *step == 0;
    let last = *step + 1 == STEPS.len();

    let current_step = &STEPS[*step];

    let (highlight, popup_pos, paragraphs) = match current_step {
        TourStep::Text(paragraphs, actions) => {
            for action in actions.iter() {
                action.apply(&net);
            }
            (
                html! {},
                format!(
                    "left: {}px; top: {}px;",
                    (width - BOX_WIDTH) * 0.5,
                    (height - BOX_HEIGHT) * 0.5
                ),
                paragraphs,
            )
        }
        TourStep::Element {
            element_id,
            alternative,
            actions,
            align,
            paragraphs,
        } => {
            // perform the actions
            for action in actions.iter() {
                action.apply(&net);
            }

            // then, get the element by ID. If it doesn't exist, then we simply skip that step.
            if let Some(elem) = document().get_element_by_id(element_id) {
                let rect = elem.get_bounding_client_rect();
                let highlight_pos = format!(
                    "width: {}px; height: {}px; top: {}px; left: {}px;",
                    rect.width() + 2.0 * HIGHLIGHT_PADDING,
                    rect.height() + 2.0 * HIGHLIGHT_PADDING,
                    rect.y() - HIGHLIGHT_PADDING,
                    rect.x() - HIGHLIGHT_PADDING
                );
                let highlight = html! { <div class="absolute rounded-xl blur-md bg-white" style={highlight_pos}></div> };

                let popup_pos: String = match align {
                    Align::Top => format!(
                        "left: {}px; bottom: {}px;",
                        rect.x(),
                        height - rect.y() + BOX_PADDING
                    ),
                    Align::Left => format!(
                        "right: {}px; top: {}px;",
                        width - rect.x() + BOX_PADDING,
                        rect.y()
                    ),
                    Align::Bottom => format!(
                        "left: {}px; top: {}px;",
                        rect.x(),
                        rect.y() + rect.height() + BOX_PADDING
                    ),
                    Align::BottomLeft => format!(
                        "left: {}px; top: {}px;",
                        rect.x() + rect.width() - BOX_WIDTH,
                        rect.y() + rect.height() + BOX_PADDING
                    ),
                    Align::Right => format!(
                        "left: {}px; top: {}px;",
                        rect.x() + rect.width() + BOX_PADDING,
                        rect.y()
                    ),
                };

                (highlight, popup_pos, paragraphs)
            } else if let Some(alternative) = alternative {
                (
                    html! {},
                    format!(
                        "left: {}px; top: {}px;",
                        (width - BOX_WIDTH) * 0.5,
                        (height - BOX_HEIGHT) * 0.5
                    ),
                    alternative,
                )
            } else {
                step.set(*step + 1);
                return html! {};
            }
        }
    };

    let popup_box_style = format!("{popup_pos} width: {BOX_WIDTH}px; min-height:{BOX_HEIGHT}px;");
    let content: Html = paragraphs
        .iter()
        .map(|s| html! {<p class="mb-3">{s}</p>})
        .collect();

    let step_c = step.clone();
    let skip_tour = Callback::from(move |_| step_c.set(STEPS.len()));
    let step_c = step.clone();
    let next_step = Callback::from(move |_| step_c.set(*step_c + 1));
    let step_c = step.clone();
    let prev_step = Callback::from(move |_| step_c.set(*step_c - 1));

    html! {
        <>
            <div class="absolute z-30 h-screen w-screen mix-blend-multiply overflow-hidden" style="background-color: #666666;">
                { highlight }
            </div>
            <div class="absolute z-30 rounded-md shadow-md bg-base-1 p-6 text-main flex flex-col gap-8" style={popup_box_style}>
                <div class="flex-1">
                    { content }
                </div>
                <div class="flex flex-row">
                    if first {
                        <button class="" onclick={skip_tour}>{"Skip Tour"}</button>
                    } else {
                        <button class="" onclick={prev_step}>{"Back"}</button>
                    }
                    <div class="flex-1"></div>
                    <button class="rounded-md py-2 px-4 shadow-md border border-base-4 bg-base-2" onclick={next_step}>{if last {"Start"} else {"Next"}}</button>
                </div>
            </div>
        </>
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TourStep {
    Text(&'static [&'static str], &'static [Action]),
    Element {
        element_id: &'static str,
        alternative: Option<&'static [&'static str]>,
        actions: &'static [Action],
        paragraphs: &'static [&'static str],
        align: Align,
    },
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum Align {
    Top,
    Left,
    Bottom,
    BottomLeft,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
enum Action {
    ChooseLayer(Layer),
    CreateFirstRouter,
    SelectFirstRouter,
    ShowQueue,
    #[cfg(feature = "atomic_bgp")]
    ShowMigration,
    #[cfg(feature = "atomic_bgp")]
    ShowSpecification,
    SelectNothing,
}

impl Action {
    pub fn apply(&self, net: impl AsRef<Net>) {
        let net = net.as_ref();
        match self {
            Action::ChooseLayer(l) => {
                Dispatch::<State>::new().reduce_mut(|state| state.set_layer(l.clone()));
            }
            Action::CreateFirstRouter => {
                if net.net().get_routers().len() == 0 {
                    Dispatch::<Net>::new().reduce_mut(|n| {
                        let id = n.net_mut().add_router("Zürich");
                        n.pos_mut().insert(id, Point::new(0.5, 0.5));
                    });
                }
            }
            Action::SelectFirstRouter => {
                let first_router = net.net().get_routers()[0];
                Dispatch::<State>::new().reduce_mut(move |state| {
                    state.set_selected(Selected::Router(first_router, false))
                });
            }
            Action::ShowQueue => {
                if !net.net().auto_simulation_enabled() {
                    Dispatch::<State>::new()
                        .reduce_mut(|state| state.set_selected(Selected::Queue));
                }
            }
            #[cfg(feature = "atomic_bgp")]
            Action::ShowMigration => {
                if net.migration().iter().map(|x| x.len()).sum::<usize>() > 0 {
                    Dispatch::<State>::new()
                        .reduce_mut(|state| state.set_selected(Selected::Migration));
                }
            }
            #[cfg(feature = "atomic_bgp")]
            Action::ShowSpecification => {
                if !net.spec().is_empty() {
                    Dispatch::<State>::new()
                        .reduce_mut(|state| state.set_selected(Selected::Verifier));
                }
            }
            Action::SelectNothing => {
                Dispatch::<State>::new().reduce_mut(|state| state.set_selected(Selected::None));
            }
        }
    }
}
