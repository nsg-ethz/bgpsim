mod components;
mod dim;
mod draw;
mod header;
mod icons;
mod net;
pub mod point;
mod sidebar;
pub mod state;
use draw::canvas::Canvas;
use header::Header;
use sidebar::Sidebar;

use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <div class="flex flex-col w-screen h-screen">
            <div class="flex">
                <Header />
            </div>
            <div class="flex h-full w-full bg-gray-100">
                <Canvas />
                <Sidebar />
            </div>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
