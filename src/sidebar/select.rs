use std::marker::PhantomData;

use gloo_utils::window;
use yew::prelude::*;

pub struct Select<T> {
    menu_shown: bool,
    pop_above: bool,
    phantom: PhantomData<T>,
}

pub enum Msg<T> {
    ToggleMenu(MouseEvent),
    HideMenu,
    OnSelect(T),
}

#[derive(Properties, PartialEq)]
pub struct Properties<T: Clone + PartialEq> {
    pub text: String,
    pub options: Vec<(T, String)>,
    pub on_select: Callback<T>,
    pub button_class: Option<Classes>,
}

impl<T: Clone + PartialEq + 'static> Component for Select<T> {
    type Message = Msg<T>;
    type Properties = Properties<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Select {
            menu_shown: false,
            pop_above: false,
            phantom: PhantomData,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|e| Msg::ToggleMenu(e));
        let onclick_close = ctx.link().callback(|_| Msg::HideMenu);
        let disabled = ctx.props().options.len() <= 1;

        let base_class = "w-full py-0.5 px-2 flex items-center border border-gray-300 text-gray-700 bg-white rounded";
        let mut button_class = if let Some(c) = ctx.props().button_class.clone() {
            classes!(base_class, c)
        } else {
            Classes::from(base_class)
        };
        if !disabled {
            button_class = classes! {button_class, "hover:text-black", "hover:shadow", "transition", "duration-150", "ease-in-out"};
        }
        let style = if self.pop_above {
            format!(
                "top: -{}rem",
                (ctx.props().options.len() as f64 * 2.0).min(11.0) + 2.5
            )
        } else {
            String::new()
        };
        html! {
            <>
                if self.menu_shown {
                    <button
                     class="absolute left-0 -top-[0rem] insert-0 h-screen w-screen cursor-default focus:outline-none"
                     onclick={onclick_close} />
                }
                <button class={button_class} {onclick} {disabled}>
                    <div class="flex-1"> <p> {&ctx.props().text} </p> </div>
                    {
                        if disabled {
                            html!{}
                        } else {
                            html!{ <yew_lucide::ChevronDown class="w-4 h-4" /> }
                        }
                    }
                </button>
                <div class="relative">
                    if self.menu_shown {
                        <div class={classes!("absolute", "w-full", "shadow-lg", "border", "rounded", "py-1", "bg-white", "right-0", "max-h-48", "overflow-auto")} {style}>
                        {
                            ctx.props().options.iter().map(|(val, text)| {
                                let v = val.clone();
                                let onclick = ctx.link().callback(move |_| Msg::OnSelect(v.clone()));
                                html! {
                                    <button class="flex w-full justify-between items-center px-4 py-1 hover:bg-gray-100" {onclick}>{ text }</button>
                                }
                            }).collect::<Html>()
                        }
                        </div>
                    }
                </div>
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleMenu(e) => {
                self.menu_shown = !self.menu_shown;
                let cur_y = e.screen_y();
                let max_y = window().inner_height().unwrap().as_f64().unwrap() as i32;
                if max_y - cur_y < 96 {
                    self.pop_above = true;
                } else {
                    self.pop_above = false;
                }
                true
            }
            Msg::HideMenu => {
                if self.menu_shown {
                    self.menu_shown = false;
                    true
                } else {
                    false
                }
            }
            Msg::OnSelect(val) => {
                self.menu_shown = false;
                ctx.props().on_select.emit(val);
                true
            }
        }
    }
}
