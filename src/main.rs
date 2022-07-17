mod dim;
mod draw;
mod header;
mod net;
mod point;
mod sidebar;
mod state;
mod tooltip;
use draw::canvas::Canvas;
use header::Header;
use sidebar::Sidebar;
use tooltip::Tooltip;

use yew::{prelude::*, Renderer};

#[function_component(App)]
fn app() -> Html {
    let header_ref = use_node_ref();
    html! {
        <div class="flex w-screen h-screen max-h-screen max-w-screen bg-gray-50 overflow-scroll">
            <Tooltip />
            <Header node_ref={header_ref.clone()} />
            <Canvas header_ref={header_ref.clone()} />
            <Sidebar />
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    Renderer::<App>::new().render();
}
