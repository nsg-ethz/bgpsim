use netsim::{config::NetworkConfig, prelude::NetworkFormatter};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{MigrationState, Net},
    sidebar::{Divider, ExpandableSection},
};

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
            .map(|major| html!( <AtomicCommandGroupViewer {major} />))
            .collect::<Html>();

        html! {
            <div class="w-full space-y-2 mt-2">
                <Divider text={"Migration".to_string()}/>
                { content }
            </div>
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct AtomicCommandGroupProps {
    pub major: usize,
}

#[function_component(AtomicCommandGroupViewer)]
pub fn atomic_command_group_viewer(props: &AtomicCommandGroupProps) -> Html {
    let (net, _) = use_store::<Net>();
    let major = props.major;
    let active = major == net.migration_step();

    let num_cmds = net
        .migration()
        .get(major)
        .map(|x| x.len())
        .unwrap_or_default();

    let content: Html = (0..num_cmds)
        .map(|minor| html! {<AtomicCommandViewer {major} {minor} />})
        .collect();

    let text = if active {
        format!("Step {} (current)", major + 1)
    } else {
        format!("Step {}", major + 1)
    };

    html! {
        <ExpandableSection {text}>
            <div class="flex flex-col space-y-4 pb-4">
                { content }
            </div>
        </ExpandableSection>
    }
}

#[derive(Properties, PartialEq)]
pub struct AtomicCommandProps {
    pub major: usize,
    pub minor: usize,
}

#[function_component(AtomicCommandViewer)]
pub fn atomic_command_viewer(props: &AtomicCommandProps) -> Html {
    let (net, net_dispatch) = use_store::<Net>();

    let major = props.major;
    let minor = props.minor;

    let cmd = net
        .migration()
        .get(major)
        .and_then(|x| x.get(minor))
        .cloned();

    let entry_class = "flex space-x-4 px-4 py-2";
    let box_class =
        "flex flex-col rounded-md my-2 py-2 bg-gray-50 shadow-md border-gray-200 border divide-y space-y text-sm";

    if let Some(cmd) = cmd {
        let pre = cmd.precondition.fmt(&net.net());
        let text = cmd.command.fmt(&net.net());
        let post = cmd.postcondition.fmt(&net.net());
        let (class, sym1, sym2, sym3, onclick) = match net
            .migration_state()
            .get(major)
            .and_then(|x| x.get(minor))
        {
            Some(MigrationState::WaitPre) => (
                "text-gray-700",
                html!(<yew_lucide::Clock class="text-red-600 w-4 h-4 self-center"/>),
                html!(<div class="w-4 h-4 self-center"></div>),
                html!(<div class="w-4 h-4 self-center"></div>),
                Callback::default(),
            ),
            Some(MigrationState::Ready) => {
                let cmd = cmd.command;
                (
                        "hover:shadow-lg hover:text-black hover:bg-gray-100 transition ease-in-out duration-150 cursor-pointer",
                        html!(<yew_lucide::Check class="text-green-600 w-4 h-4 self-center"/>),
                        html!(<yew_lucide::ArrowRight class="w-4 h-4 self-center" />),
                        html!(<div class="w-4 h-4 self-center"></div>),
                        net_dispatch.reduce_mut_callback(move |n| {
                            n.migration_state_mut()[major][minor] = MigrationState::WaitPost;
                            n.net_mut().apply_modifier_unchecked(&cmd).unwrap();
                        }),
                    )
            }
            Some(MigrationState::WaitPost) => (
                "text-gray-700",
                html!(<yew_lucide::Check class="text-green-600 w-4 h-4 self-center" />),
                html!(<yew_lucide::Check class="text-green-600 w-4 h-4 self-center" />),
                html!(<yew_lucide::Clock class="text-red-600 w-4 h-4 self-center" />),
                Callback::default(),
            ),
            _ => (
                "text-gray-400",
                html!(<yew_lucide::Check class="text-green-600 w-4 h-4 self-center" />),
                html!(<yew_lucide::Check class="text-green-600 w-4 h-4 self-center" />),
                html!(<yew_lucide::Check class="text-green-600 w-4 h-4 self-center" />),
                Callback::default(),
            ),
        };
        let class = classes!(box_class, class);
        html! {
            <div {class} {onclick}>
                <div class={entry_class}> {sym1} <p class="flex-1"> { pre } </p></div>
                <div class={entry_class}> {sym2} <p class="flex-1"> { text } </p></div>
                <div class={entry_class}> {sym3} <p class="flex-1"> { post } </p></div>
            </div>
        }
    } else {
        html! {}
    }
}
