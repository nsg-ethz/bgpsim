#![allow(clippy::let_unit_value)]

mod dim;
mod draw;
mod header;
mod latex_export;
mod net;
mod point;
mod sidebar;
mod state;
mod tooltip;
use draw::canvas::Canvas;
use header::Header;
use sidebar::Sidebar;
use tooltip::Tooltip;

use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::net::Net;

#[function_component(App)]
fn app() -> Html {
    let header_ref = use_node_ref();
    html! {
        <div class="flex w-screen h-screen max-h-screen max-w-screen bg-gray-50 overflow-scroll">
            <Tooltip />
            <div class="relative flex-1 h-full p-0 bg-gray-50">
              <Header node_ref={header_ref.clone()} />
              <Canvas header_ref={header_ref.clone()} />
            </div>
            <Sidebar />
        </div>
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Routable)]
enum Route {
    #[not_found]
    #[at("/")]
    Home,
    #[at("/i/:d")]
    ImportNet { d: String },
}

fn switch(route: &Route) -> Html {
    match route {
        Route::Home => html! {<App />},
        Route::ImportNet { d } => {
            let net_dispatch = Dispatch::<Net>::new();
            net_dispatch.reduce_mut(|n| n.import_url(d));
            html! { <Redirect<Route> to={Route::Home} /> }
        }
    }
}

#[function_component(Entry)]
fn entry() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Switch::render(switch)} />
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Entry>();
}
