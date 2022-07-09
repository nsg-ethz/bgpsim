use yew::prelude::*;

pub struct Element {}

pub enum Msg {}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub text: String,
    pub children: Children,
    pub class: Option<Classes>,
    pub small: Option<bool>,
}

impl Component for Element {
    type Message = Msg;
    type Properties = Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        Element {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let class = classes! { "text-gray-700", "text-right", "pr-4", &ctx.props().class };
        let (d1class, d2class) = if ctx.props().small.unwrap_or(false) {
            ("basis-1/4 flex-none", "basis-3/4 flex-none")
        } else {
            ("basis-2/5 flex-none", "basis-3/5 flex-none")
        };
        html! {
            <div class="w-full flex">
                <div class={d1class}>
                    <p {class}>{ctx.props().text.as_str()}</p>
                </div>
                <div class={d2class}>
                    { for ctx.props().children.iter() }
                </div>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }
}
