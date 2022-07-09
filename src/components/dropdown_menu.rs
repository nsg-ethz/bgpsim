use std::marker::PhantomData;

use yew::prelude::*;

use crate::icons::Icon;

pub struct DropdownMenu<T> {
    phantom: PhantomData<T>,
    menu_shown: bool,
}

pub enum Msg<T: Copy> {
    ToggleMenu,
    HideMenu,
    Select(T),
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ButtonStyle {
    Button(Color),
    Underline(Color),
    UnderlineHover(Color),
    Flat,
}

impl ButtonStyle {
    pub fn classes(&self) -> Classes {
        let default_c = classes! {
            "flex", "leading_normal", "focus:outline-none", "focus:ring-0",
            "transition", "duration-150", "ease-in-out"
        };
        let buttonless_c = classes! {
            "px-2", "text-gray-600", "hover:text-black"
        };
        match self {
            ButtonStyle::Button(c) => {
                classes! {
                    "px-4", "py-2", "rounded", "shadow-md", "hover:shadow-lg",
                    default_c, c.button_classes()
                }
            }
            ButtonStyle::Underline(c) => {
                classes! {
                    "border-b", "border-gray-300", "hover:border-b-2",
                    default_c, buttonless_c, c.underline_hover_classes()
                }
            }
            ButtonStyle::UnderlineHover(c) => {
                classes! {
                    "flex", "border-b-2", "border-transparent", "hover:border-b-2", "hover:border-blue-600",
                    default_c, buttonless_c, c.underline_hover_classes()
                }
            }
            ButtonStyle::Flat => {
                classes! { default_c, buttonless_c }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Color {
    Blue,
    Purple,
    Green,
    Red,
    Yellow,
    Light,
    Dark,
}

impl Color {
    pub fn button_classes(&self) -> Classes {
        match self {
            Color::Blue => {
                classes! {"bg-blue-600", "hover:bg-blue-700", "active:bg-blue-700", "text-white"}
            }
            Color::Purple => {
                classes! {"bg-purple-600", "hover:bg-purple-700", "active:bg-purple-700", "text-white"}
            }
            Color::Green => {
                classes! {"bg-green-500", "hover:bg-green-600", "active:bg-green-600", "text-white"}
            }
            Color::Red => {
                classes! {"bg-red-600", "hover:bg-red-700", "active:bg-red-700", "text-white"}
            }
            Color::Yellow => {
                classes! {"bg-yellow-500", "hover:bg-yellow-600", "active:bg-yellow-600", "text-white"}
            }
            Color::Light => {
                classes! {"bg-gray-200", "hover:bg-gray-300", "active:bg-gray-300", "text-gray-700"}
            }
            Color::Dark => {
                classes! {"bg-gray-800", "hover:bg-gray-900", "active:bg-gray-900", "text-white"}
            }
        }
    }
    pub fn underline_hover_classes(&self) -> Classes {
        match self {
            Color::Blue => {
                classes! {"hover:border-blue-600"}
            }
            Color::Purple => {
                classes! {"hover:border-purple-600"}
            }
            Color::Green => {
                classes! {"hover:border-green-500"}
            }
            Color::Red => {
                classes! {"hover:border-red-600"}
            }
            Color::Yellow => {
                classes! {"hover:border-yellow-600"}
            }
            Color::Light => {
                classes! {"hover:border-gray-200"}
            }
            Color::Dark => {
                classes! {"hover:border-gray-800"}
            }
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct Properties<T: PartialEq> {
    /// Color for the button
    pub button_style: ButtonStyle,
    /// Class for the button
    pub button_class: Option<Classes>,
    /// Content of the button
    pub button_icon: Option<Icon>,
    /// Content of the button
    pub button_text: String,
    /// Left (true) or right (false) expansion of the dropdown
    pub expand_left: bool,
    /// Options to choose from, with the value that is passed to the callback on a selection.
    pub options: Vec<(String, T)>,
    /// Callback executed once anything was selected.
    pub on_select: Callback<T>,
}

impl<T: Copy + PartialEq + 'static> Component for DropdownMenu<T> {
    type Message = Msg<T>;
    type Properties = Properties<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        DropdownMenu {
            phantom: PhantomData,
            menu_shown: false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_| Msg::ToggleMenu);
        let onclick_close = ctx.link().callback(|_| Msg::HideMenu);
        let button_text = ctx.props().button_text.clone();
        let mut button_class = ctx.props().button_class.clone().unwrap_or_default();
        let icon = ctx
            .props()
            .button_icon
            .map(|i| i.html())
            .unwrap_or_default();
        button_class.extend(ctx.props().button_style.classes());
        let orientation = if ctx.props().expand_left {
            "left-0"
        } else {
            "right-0"
        };
        html! {
            <>
                if self.menu_shown {
                    <button
                     class="absolute left-0 top-0 insert-0 h-screen w-screen cursor-default focus:outline-none"
                     onclick={onclick_close} />
                }
                <div class="relative">
                    <button class={button_class} {onclick}> <div class="mr-1">{icon}</div> {button_text} </button>
                    if self.menu_shown {
                        <div class={classes!("absolute", "shadow-lg", "border", "rounded", "py-1", "bg-white", orientation)}>
                        {
                            ctx.props().options.iter().map(|(text, val)| {
                                let v = *val;
                                let onclick = ctx.link().callback(move |_| Msg::Select(v));
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
            Msg::ToggleMenu => {
                self.menu_shown = !self.menu_shown;
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
            Msg::Select(val) => {
                self.menu_shown = false;
                ctx.props().on_select.emit(val);
                true
            }
        }
    }
}
