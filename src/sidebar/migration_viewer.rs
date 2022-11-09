use netsim::prelude::NetworkFormatter;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::Net,
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
        let step = net.migration_step;
        let content = (0..net.migration().len())
            .map(|major| html!( <AtomicCommandGroupViewer {major} active={major == step} />))
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
pub struct AtomicCommandGroupProps {
    pub major: usize,
    pub active: bool,
}

#[function_component(AtomicCommandGroupViewer)]
pub fn atomic_command_group_viewer(props: &AtomicCommandGroupProps) -> Html {
    let (net, _) = use_store::<Net>();
    let major = props.major;

    let num_cmds = net
        .migration()
        .get(major)
        .map(|x| x.len())
        .unwrap_or_default();

    let active = props.active;

    let content: Html = (0..num_cmds)
        .map(|minor| html! {<AtomicCommandViewer {major} {minor} {active} />})
        .collect();

    let text = if active {
        format!("Step {} (current)", major)
    } else {
        format!("Step {}", major)
    };

    html! {
        <ExpandableSection {text}>
            { content }
        </ExpandableSection>
    }
}

#[derive(Properties, PartialEq)]
pub struct AtomicCommandProps {
    pub major: usize,
    pub minor: usize,
    pub active: bool,
}

#[function_component(AtomicCommandViewer)]
pub fn atomic_command_viewer(props: &AtomicCommandProps) -> Html {
    let (net, _) = use_store::<Net>();
    let major = props.major;
    let minor = props.minor;

    let cmd = net
        .migration()
        .get(major)
        .and_then(|x| x.get(minor))
        .cloned();

    if let Some(cmd) = cmd {
        let pre = cmd.precondition.fmt(&net.net());
        let text = cmd.command.fmt(&net.net());
        let post = cmd.postcondition.fmt(&net.net());
        html! {
            <>
                <p class="w-full flex m-4"> { pre } </p><br />
                <p class="w-full flex m-4"> { text } </p><br />
                <p class="w-full flex m-4"> { post } </p><br />
            </>
        }
    } else {
        html! {}
    }
}
