// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022 Tibor Schneider
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

use gloo_events::EventListener;
use gloo_utils::{document, window};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::state::{Selected, State};

const STEPS: &[TourStep] = &[
    TourStep::Text(&[
        #[cfg(feature = "atomic_bgp")]
        "Welcome to the online simulator for Chameleon.",
        #[cfg(not(feature = "atomic_bgp"))]
        "Welcome to BgpSim, the online simulator for BGP networks.",
        "In a few steps, this tutorial will show you how to use this simulator."
    ]),
    TourStep::Element {
        element_id: "layer-selection",
        selected: None,
        paragraphs: &["The simulator offers visualization of many different aspects of the network. You can select which aspect should be visualized using this button. The main section below can visualize the forwarding state, the routing state (how routes are propagated), the IGP configuration, or the BGP configuration."],
        align: Align::Bottom,
    },
];

const HIGHLIGHT_PADDING: f64 = 20.0;
const BOX_PADDING: f64 = 40.0;
const BOX_WIDTH: f64 = 400.0;
const BOX_HEIGHT: f64 = 200.0;

#[function_component]
pub fn Tour() -> Html {
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

    let (highlight, popup_pos) = match current_step {
        TourStep::Text(_) => (
            html! {},
            format!(
                "left: {}px; top: {}px;",
                (width - BOX_WIDTH) * 0.5,
                (height - BOX_HEIGHT) * 0.5
            ),
        ),
        TourStep::Element {
            element_id,
            selected,
            align,
            ..
        } => {
            // first, if selected is Some, go to that state
            if let Some(s) = selected {
                Dispatch::<State>::new().reduce_mut(|state| state.set_selected(s.clone()));
            }

            // then, get the element by ID. If it doesn't exist, then we simply skip that step.
            let Some(elem) = document().get_element_by_id(element_id) else {
                step.set(*step + 1);
                return html!{}
            };

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
                Align::Right => format!(
                    "left: {}px; top: {}px;",
                    rect.x() + rect.width() + BOX_PADDING,
                    rect.y()
                ),
            };

            (highlight, popup_pos)
        }
    };

    let popup_box_style = format!("{popup_pos} width: {BOX_WIDTH}px; height:{BOX_HEIGHT}px;");
    let content: Html = current_step
        .paragraphs()
        .iter()
        .map(|s| html! {<p class="mb-3">{s}</p>})
        .collect();

    let progress = format!("{} / {}", (*step + 1), STEPS.len());

    let step_c = step.clone();
    let skip_tour = Callback::from(move |_| step_c.set(STEPS.len()));
    let step_c = step.clone();
    let next_step = Callback::from(move |_| step_c.set(*step_c + 1));
    let step_c = step.clone();
    let prev_step = Callback::from(move |_| step_c.set(*step_c - 1));

    html! {
        <>
            <div class="absolute z-30 h-screen w-screen mix-blend-multiply bg-neutral-400">
                { highlight }
            </div>
            <div class="absolute z-30 h-48 w-96 rounded-md shadow-md bg-base-1 p-3 text-main flex flex-col" style={popup_box_style}>
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
                    <div class="text-base-4 py-2">{progress}</div>
                    <div class="flex-1"></div>
                    <button class="rounded-md py-2 px-4 shadow-md border border-base-4 bg-base-2" onclick={next_step}>{if last {"Start"} else {"Next"}}</button>
                </div>
            </div>
        </>
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TourStep {
    Text(&'static [&'static str]),
    Element {
        element_id: &'static str,
        selected: Option<Selected>,
        paragraphs: &'static [&'static str],
        align: Align,
    },
}

impl TourStep {
    pub fn paragraphs(&self) -> &'static [&'static str] {
        match self {
            TourStep::Text(x) => x,
            TourStep::Element { paragraphs, .. } => paragraphs,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Align {
    Top,
    Left,
    Bottom,
    Right,
}
