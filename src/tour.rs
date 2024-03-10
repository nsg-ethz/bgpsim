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
use web_sys::Element;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    callback, clone,
    net::Net,
    state::{Layer, Selected, State},
};

const STEPS: &[TourStep] = &[
    TourStep::Text {
        paragraphs: &[
            "Welcome to BgpSim, the online simulator for BGP networks.",
            "In a few steps, this tutorial will show you how to use this simulator."
        ],
        actions: &[],
    },
    TourStep::Element {
        element_id: "layer-selection",
        alternative: None,
        actions: &[
            Action::ChooseLayer(Layer::FwState),
        ],
        paragraphs: &[
            "The simulator offers visualization layers for many different aspects of the network. You can select which aspect should be visualized using this button. The visualization layers include:" ,
            "- the forwarding state (how packets are forwarded),",
            "- the routing state (how routes are propagated),",
            "- the IGP configuration (link weights), or",
            "- the BGP configuration (BGP sessions and route-maps)."],
        align: Align::Bottom,
    },
    TourStep::Element {
        element_id: "prefix-selection",
        alternative: None,
        actions: &[Action::CreateFirstRouter, Action::SelectFirstRouter],
        paragraphs: &["Some layers only visualize the state for a given prefix. This input field allows you to change that prefix!"],
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
        align: Align::Fit,
    },
    TourStep::Element {
        element_id: "sidebar",
        alternative: None,
        actions: &[],
        paragraphs: &[
            "After selecting a router, the sidebar shows all configuration options for that router. Here, you can modify the OSPF and BGP configuration.",
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
        align: Align::Bottom,
    },
    TourStep::Element {
        element_id: "specification-button",
        alternative: None,
        actions: &[Action::ShowSpecification],
        paragraphs: &[
            "The simulator comes with a built-in verifier. At every step, the simulator will check all forwarding properties, and notify you as soon as any property is violated.",
            "By clicking on this button, the sidebar on the right will show all properties (and which of them are violated)."
        ],
        align: Align::Bottom,
    },
    TourStep::Text {
        paragraphs: &[
            "You now understand the basics of the simulator.",
        ],
        actions: &[Action::SelectNothing]
    },
];

const HIGHLIGHT_PADDING: f64 = 20.0;
const BOX_WIDTH: f64 = 400.0;
const BOX_HEIGHT: f64 = 200.0;
const PADDING: f64 = 10.0;

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

    // ensure that the box size is updated
    let help_win_ref = use_node_ref();
    let box_size = use_reducer_eq(clone!(help_win_ref -> move || BoxSize::new(help_win_ref)));
    use_effect(clone!(box_size -> move || box_size.dispatch(())));

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

    let (box_width, box_height) = (box_size.width, box_size.height);

    let first = *step == 0;
    let last = *step + 1 == STEPS.len();

    let current_step = &STEPS[*step];

    let (highlight, popup_pos, paragraphs) = match current_step {
        TourStep::Text {
            paragraphs,
            actions,
            ..
        } => {
            for action in actions.iter() {
                action.apply(&net);
            }
            (
                html! {},
                format!(
                    "left: {}px; top: {}px;",
                    (width - box_width) * 0.5,
                    (height - box_height) * 0.5
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
            ..
        } => {
            // perform the actions
            for action in actions.iter() {
                action.apply(&net);
            }

            // then, get the element by ID. If it doesn't exist, then we simply skip that step.
            if let Some(elem) = document().get_element_by_id(element_id) {
                let rect = elem.get_bounding_client_rect();
                let mid_x = rect.x() + (rect.width() * 0.5) - (box_width * 0.5);
                let mid_y = rect.y() + (rect.height() * 0.5) - (box_height * 0.5);
                let highlight_pos = format!(
                    "width: {}px; height: {}px; top: {}px; left: {}px;",
                    rect.width() + 2.0 * HIGHLIGHT_PADDING,
                    rect.height() + 2.0 * HIGHLIGHT_PADDING,
                    rect.y() - HIGHLIGHT_PADDING,
                    rect.x() - HIGHLIGHT_PADDING
                );
                let highlight = html! { <div class="absolute rounded-xl blur-md bg-white" style={highlight_pos}></div> };

                // change the alignment if it is Fit
                let align = if *align == Align::Fit {
                    // check if we have enough space at the bottom
                    if rect.y() + box_height + 2.0 * PADDING < height {
                        Align::Bottom
                    } else if rect.y() - box_height - 2.0 * PADDING > 0.0 {
                        Align::Top
                    } else if rect.x() + box_width + 2.0 * PADDING < width {
                        Align::Right
                    } else if rect.x() - box_width - 2.0 * PADDING > 0.0 {
                        Align::Left
                    } else {
                        log::warn!("Cannot fit the help box on the screen!");
                        Align::Bottom
                    }
                } else {
                    *align
                };

                let popup_pos: String = match align {
                    Align::Top => format!(
                        "left: {}px; bottom: {}px;",
                        mid_x.min(width - box_width - PADDING).max(PADDING),
                        height - rect.y() + PADDING
                    ),
                    Align::Left => format!(
                        "right: {}px; top: {}px;",
                        width - rect.x() + PADDING,
                        mid_y.min(height - box_height - PADDING).max(PADDING),
                    ),
                    Align::Bottom => format!(
                        "left: {}px; top: {}px;",
                        mid_x.min(width - box_width - PADDING).max(PADDING),
                        rect.y() + rect.height() + PADDING
                    ),
                    Align::Right => format!(
                        "left: {}px; top: {}px;",
                        rect.x() + rect.width() + PADDING,
                        mid_y.min(height - box_height - PADDING).max(PADDING),
                    ),
                    Align::Fit => unreachable!("Case handled before!"),
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

    let skip_tour = callback!(step -> move |_| step.set(STEPS.len()));
    let next_step = callback!(step -> move |_| step.set(*step + 1));
    let prev_step = callback!(step -> move |_| step.set(*step - 1));

    html! {
        <>
            <div class="absolute z-30 h-screen w-screen mix-blend-multiply overflow-hidden" style="background-color: #666666;">
                { highlight }
            </div>
            <div ref={help_win_ref} class="absolute z-30 rounded-md shadow-md bg-base-1 p-6 text-main flex flex-col gap-8" style={popup_box_style}>
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

#[derive(Debug, Clone)]
struct BoxSize {
    width: f64,
    height: f64,
    node_ref: NodeRef,
}

impl PartialEq for BoxSize {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && self.height == other.height
    }
}

impl BoxSize {
    pub fn new(node_ref: NodeRef) -> Self {
        Self {
            width: BOX_WIDTH,
            height: BOX_HEIGHT,
            node_ref,
        }
    }
}

impl Reducible for BoxSize {
    type Action = ();

    fn reduce(self: std::rc::Rc<Self>, _action: Self::Action) -> std::rc::Rc<Self> {
        let (width, height) = self
            .node_ref
            .cast::<Element>()
            .map(|e| (e.client_width() as f64, e.client_height() as f64))
            .unwrap_or((BOX_WIDTH, BOX_HEIGHT));
        Self {
            width,
            height,
            node_ref: self.node_ref.clone(),
        }
        .into()
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TourStep {
    Text {
        actions: &'static [Action],
        paragraphs: &'static [&'static str],
    },
    Element {
        element_id: &'static str,
        alternative: Option<&'static [&'static str]>,
        actions: &'static [Action],
        paragraphs: &'static [&'static str],
        align: Align,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum Align {
    Top,
    Left,
    Bottom,
    Right,
    Fit,
}

#[derive(Debug, Clone, PartialEq)]
enum Action {
    ChooseLayer(Layer),
    CreateFirstRouter,
    SelectFirstRouter,
    ShowQueue,
    ShowSpecification,
    SelectNothing,
}

impl Action {
    pub fn apply(&self, net: impl AsRef<Net>) {
        let net = net.as_ref();
        match self {
            Action::ChooseLayer(l) => {
                Dispatch::<State>::new().reduce_mut(|state| state.set_layer(*l));
            }
            Action::CreateFirstRouter => {
                let center = net.dim.center_point();
                if net.net().internal_indices().next().is_none() {
                    Dispatch::<Net>::new().reduce_mut(|n| {
                        let id = n.net_mut().add_router("Zürich");
                        n.pos_mut().insert(id, center);
                    });
                }
            }
            Action::SelectFirstRouter => {
                let first_router = net.net().internal_indices().next().unwrap();
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
