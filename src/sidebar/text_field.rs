use web_sys::HtmlInputElement;
use yew::prelude::*;

pub struct TextField {
    current_text: String,
    original_text: String,
    node_ref: NodeRef,
}

pub enum Msg {
    Keypress(KeyboardEvent),
    Change,
    Set,
}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub text: String,
    pub button_text: Option<String>,
    pub correct: bool,
    pub placeholder: Option<String>,
    pub on_change: Callback<String>,
    pub on_set: Callback<String>,
}

impl Component for TextField {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        TextField {
            current_text: ctx.props().text.clone(),
            original_text: ctx.props().text.clone(),
            node_ref: Default::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let changed = self.current_text != ctx.props().text;
        let colors = match (changed, ctx.props().correct) {
            (true, true) => {
                classes! {"text-gray-700", "border-blue-300", "focus:border-blue-600", "focus:text-gray-700"}
            }
            (true, false) => {
                classes! {"text-gray-700", "border-red-300", "focus:border-red-600", "focus:text-gray-700"}
            }
            (false, _) => {
                classes! {"text-gray-500", "border-gray-300", "focus:border-blue-600", "focus:text-gray-700"}
            }
        };
        let class = classes! {"flex-1", "w-16", "px-3", "text-base", "font-normal", "bg-white", "bg-clip-padding", "border", "border-solid", "rounded", "transition", "ease-in-out", "m-0", "focus:outline-none", colors};

        let node_ref = self.node_ref.clone();

        let onchange = ctx.link().callback(|_| Msg::Change);
        let onkeypress = ctx.link().callback(Msg::Keypress);
        let onpaste = ctx.link().callback(|_| Msg::Change);
        let oninput = ctx.link().callback(|_| Msg::Change);
        let onclick = ctx.link().callback(|_| Msg::Set);
        let enabled = changed && ctx.props().correct;
        let button_class = if enabled {
            classes! {"ml-2", "px-2", "flex-none", "text-gray-700", "rounded", "shadow-md", "hover:shadow-lg", "transition", "ease-in-out", "border", "border-gray-300", "focus:border-blue-600", "focus:outline-none"}
        } else {
            classes! {"ml-2", "px-2", "flex-none", "rounded", "bg-gray-50", "transition", "ease-in-out", "border", "border-gray-200", "focus:outline-none", "text-gray-200"}
        };

        let button_text = ctx
            .props()
            .button_text
            .clone()
            .unwrap_or_else(|| "Set".to_string());

        let placeholder = ctx.props().placeholder.clone().unwrap_or_default();

        html! {
            <div class="w-full flex">
                <input type="text" {class} value={self.current_text.clone()} {placeholder} {onchange} {onkeypress} {onpaste} {oninput} ref={node_ref}/>
                {
                    if enabled {
                        html!{<button class={button_class} {onclick}> {button_text} </button>}
                    } else {
                        html!{<button class={button_class} disabled=true> {button_text} </button>}
                    }
                }
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Change => {
                let elem = self.node_ref.cast::<HtmlInputElement>().unwrap();
                let val = elem.value();
                let updated = val != self.current_text;
                self.current_text = val;
                // call the callback
                ctx.props().on_change.emit(self.current_text.clone());
                updated
            }
            Msg::Set => {
                ctx.props().on_set.emit(self.current_text.clone());
                false
            }
            Msg::Keypress(e) => self.update(
                ctx,
                if e.code() == "Enter" {
                    Msg::Set
                } else {
                    Msg::Change
                },
            ),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        if self.original_text != ctx.props().text {
            self.current_text = ctx.props().text.clone();
            self.original_text = ctx.props().text.clone();
        }
        true
    }
}
