# yew-lucide

[![crates.io version](https://img.shields.io/crates/v/yew-lucide.svg?style=flat-square)](https://crates.io/crates/yew-lucide)
[![crates.io downloads](https://img.shields.io/crates/d/yew-lucide.svg?style=flat-square)](https://crates.io/crates/yew-lucide)

Forked from [Yew Lucide](https://gitlab.com/john_t/yew-lucide)

## What is yew-lucide?

yew-lucide is a collection of simply beautiful open source icons for Yew. Each icon is designed on a 24x24 grid with an emphasis on simplicity, consistency and readability.

## Based on Lucide Icons

This will be updated to keep in track with the latest lucide.

https://lucide.dev/

## Usage

```rust
use yew::{function_component, html};
use yew_lucide::Camera;

#[function_component(App)]
fn app() -> Html {
    html! { <Camera /> }
}

fn main() {
    yew::start_app::<App>();
}
```

Icons can be configured with inline props:

```rust
<Camera color="red" size=48 />
```
