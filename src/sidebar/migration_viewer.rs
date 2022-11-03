use netsim::prelude::NetworkFormatter;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{net::Net, sidebar::Divider};

#[function_component(MigrationViewer)]
pub fn migration_viewer() -> Html {
    let (net, _) = use_store::<Net>();

    if net.migration().is_empty() {
        html! {
            <div class="h-full w-full flex flex-col justify-center items-center">
                <p class="text-gray-300 italic"> { "Migration is empty!" } </p>
            </div>
        }
    } else {
        let content = (0..net.migration().len())
            .map(|idx| html!( <> <ModifierViewer {idx} /> <Divider /> </> ))
            .collect::<Html>();

        html! {
            <div class="w-full space-y-2 mt-2">
                <Divider text={"Specification".to_string()}/>
                { content }
            </div>
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct ModifierViewerProps {
    pub idx: usize,
}

#[function_component(ModifierViewer)]
pub fn modifier_viewer(props: &ModifierViewerProps) -> Html {
    let (net, _) = use_store::<Net>();
    let idx = props.idx;

    let p = net.migration().get(idx).cloned();

    if let Some(modifier) = p {
        let text = modifier.fmt(&net.net());
        html! {
            <p class="w-full flex m-4">
                { text }
            </p>
        }
    } else {
        html! {}
    }
}
