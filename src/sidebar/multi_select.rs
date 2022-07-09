use std::marker::PhantomData;

use yew::prelude::*;

pub struct MultiSelect<T> {
    phantom: PhantomData<T>,
    menu_shown: bool,
}

pub enum Msg<T> {
    ToggleMenu,
    HideMenu,
    ToggleElement(T),
    RemoveElement(T),
}

#[derive(Properties, PartialEq)]
pub struct Properties<T: Clone + PartialEq> {
    pub options: Vec<(T, String, bool)>,
    pub on_add: Callback<T>,
    pub on_remove: Callback<T>,
}

impl<T: Clone + PartialEq + 'static> Component for MultiSelect<T> {
    type Message = Msg<T>;
    type Properties = Properties<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        MultiSelect {
            phantom: PhantomData,
            menu_shown: false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_| Msg::ToggleMenu);
        let onclick_close = ctx.link().callback(|_| Msg::HideMenu);
        let disabled = ctx.props().options.iter().filter(|(_, _, b)| !*b).count() == 0;
        let mut button_class = classes! {"w-full", "p-0.5", "flex", "border", "border-gray-300", "bg-white", "rounded"};
        if !disabled {
            button_class = classes! {button_class, "hover:shadow", "rounded", "transition", "duration-150", "ease-in-out"};
        }
        html! {
            <>
                if self.menu_shown {
                    <button
                     class="absolute left-0 top-0 insert-0 h-screen w-screen cursor-default focus:outline-none"
                     onclick={onclick_close} />
                }
                <div class={button_class}>
                    <div class="flex-auto flex flex-wrap">
                    {
                        ctx.props().options.iter().filter(|(_, _, b)| *b).cloned().map(|(entry, text, _)| {
                            html!{ <MultiSelectItem<T> entry={entry.clone()} {text} on_remove={ctx.link().callback(move |_| Msg::RemoveElement(entry.clone()))} /> }
                        }).collect::<Html>()
                    }
                    </div>
                    <div class="text-gray-500 w-8 ml-0.5 py-1 pl-2 pr-1 border-l flex items-center border-gray-300">
                    {
                        if !disabled {
                            html!{<button class="" {onclick} {disabled}> <yew_lucide::ChevronDown class="w-4 h-4" /> </button>}
                        } else { html! {} }
                    }
                    </div>
                </div>
                if self.menu_shown {
                    <div class="relative">
                        <div class={classes!("absolute", "w-full", "shadow-lg", "border", "rounded", "py-1", "bg-white", "right-0", "max-h-48", "overflow-auto")}>
                        {
                            ctx.props().options.iter().filter(|(_, _, b)| !*b).map(|(val, text, _)| {
                                let v = val.clone();
                                let onclick = ctx.link().callback(move |_| Msg::ToggleElement(v.clone()));
                                html! {
                                    <button class="flex w-full justify-between items-center px-4 py-1 hover:bg-gray-100" {onclick}>{ text }</button>
                                }
                            }).collect::<Html>()
                        }
                        </div>
                    </div>
                }
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
            Msg::ToggleElement(e) => {
                if ctx
                    .props()
                    .options
                    .iter()
                    .filter(|(t, _, _)| t == &e)
                    .map(|(_, _, b)| !*b)
                    .next()
                    .unwrap()
                {
                    ctx.props().on_add.emit(e);
                } else {
                    ctx.props().on_remove.emit(e);
                }
                if self.menu_shown {
                    self.menu_shown = false;
                    true
                } else {
                    false
                }
            }
            Msg::RemoveElement(e) => {
                ctx.props().on_remove.emit(e);
                false
            }
        }
    }
}
#[derive(Properties, PartialEq)]
pub struct ItemProperties<T: Clone + PartialEq> {
    pub text: String,
    pub entry: T,
    pub on_remove: Callback<T>,
}

#[function_component(MultiSelectItem)]
fn multi_select_item<T: Clone + PartialEq + 'static>(props: &ItemProperties<T>) -> Html {
    let onclick = {
        let entry = props.entry.clone();
        props.on_remove.reform(move |_| entry.clone())
    };
    html! {
        <div class="px-3 py-0 m-0.5 rounded text-gray-700 bg-gray-200 text-sm flex flex-row items-center">
            { props.text.as_str() }
            <button class="pl-2 hover hover:text-red-700 focus:outline-none transition duration-150 ease-in-out" {onclick}>
                <yew_lucide::X class="w-3 h-3 text-center" />
            </button>
        </div>
    }
}
