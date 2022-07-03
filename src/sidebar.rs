use yew::prelude::*;

#[function_component(Sidebar)]
pub fn sidebar() -> Html {
    html! {
        <div class="w-96 h-full p-4 align-middle">
            <div class="w-full h-full p-4 bg-white shadow-lg flex rounded-lg">
                <p>{ "Sidebar" }</p>
            </div>
        </div>
    }
}
