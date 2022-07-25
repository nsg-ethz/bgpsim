use yew::prelude::*;

#[derive(Properties, PartialEq, Eq)]
pub struct DividerProps {
    pub text: Option<String>,
}

#[function_component(Divider)]
pub fn divider(props: &DividerProps) -> Html {
    if let Some(text) = props.text.as_ref() {
        html! {
            <div class="w-full flex py-3 items-center">
                <div class="flex-grow border-t border-gray-300"></div>
                <span class="flex-shrink mx-4 text-gray-300">{text}</span>
                <div class="flex-grow border-t border-gray-300"></div>
            </div>
        }
    } else {
        html! {
            <div class="w-full flex py-3 items-center">
                <div class="flex-grow border-t border-gray-300"></div>
            </div>
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct DividerButtonProps {
    pub on_click: Callback<MouseEvent>,
    pub children: Children,
    pub hidden: Option<bool>,
}

#[function_component(DividerButton)]
pub fn divider_button(props: &DividerButtonProps) -> Html {
    let line_class = if props.hidden.unwrap_or(false) {
        "flex-grow"
    } else {
        "flex-grow border-t border-gray-300"
    };
    html! {
        <div class="w-full flex py-3 items-center">
            <div class={line_class}></div>
            <button class="rounded-full bg-white drop-shadow-md hover:drop-shadow-lg p-2" onclick={props.on_click.clone()}>
                { for props.children.iter() }
            </button>
            <div class={line_class}></div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ExpandableDividerProps {
    pub text: Option<String>,
    pub children: Children,
}

#[function_component(ExpandableDivider)]
pub fn expandable_divider(props: &ExpandableDividerProps) -> Html {
    let shown = use_state(|| false);
    let onclick = {
        let shown = shown.clone();
        Callback::from(move |_| shown.set(!*shown))
    };

    let text = props.text.clone().unwrap_or_default();
    let text = if *shown {
        html! { <span class="inline-flex items-center">{ text } <yew_lucide::ChevronUp class="w-4 h-4" /> </span> }
    } else {
        html! { <span class="inline-flex items-center">{ text } <yew_lucide::ChevronDown class="w-4 h-4" /> </span> }
    };

    html! {
        <div class="w-full space-y-2">
            <div class="w-full flex py-3 items-center">
                <div class="flex-grow border-t border-gray-300"></div>
                <button class="flex-shrink mx-4 text-gray-300 hover:text-gray-700 transition duration-150 ease-in-out" {onclick}>{text}</button>
                <div class="flex-grow border-t border-gray-300"></div>
            </div>
        {
            if *shown {
                html! {{ for props.children.iter() }}
            } else { html!{} }
        }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ExpandableSectionProps {
    pub text: String,
    pub children: Children,
}

#[function_component(ExpandableSection)]
pub fn expandable_section(props: &ExpandableSectionProps) -> Html {
    let shown = use_state(|| false);
    let onclick = {
        let shown = shown.clone();
        Callback::from(move |_| shown.set(!*shown))
    };

    let icon = if *shown {
        html! { <yew_lucide::ChevronUp class="w-4 h-4" /> }
    } else {
        html! { <yew_lucide::ChevronDown class="w-4 h-4" /> }
    };
    html! {
        <div class="w-full space-y-2">
            <button class="w-full inline-flex items-center text-gray-300 hover:text-gray-700 transition transition-150 ease-in-out" {onclick}>
                {icon}
                <span class="flex-shrink mx-2">{&props.text}</span>
            </button>
            {
                if *shown {
                    html! {{ for props.children.iter() }}
                } else { html!{} }
            }
        </div>
    }
}
