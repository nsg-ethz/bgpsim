use yew::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum Icon {
    ChevronDown,
    Menu,
    X,
}

impl Icon {
    pub fn html(&self) -> Html {
        match self {
            Self::ChevronDown => html! { <yew_lucide::ChevronDown />},
            Self::Menu => html! { <yew_lucide::Menu />},
            Self::X => html! { <yew_lucide::X />},
        }
    }
}
