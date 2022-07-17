use yew::Properties;

#[derive(Properties, PartialEq)]
pub struct IconProps {
    #[prop_or_default]
    pub class: &'static str,
    #[prop_or("24")]
    pub size: &'static str,
    #[prop_or("none")]
    pub fill: &'static str,
    #[prop_or("currentColor")]
    pub color: &'static str,
    #[prop_or("2")]
    pub stroke_width: &'static str,
    #[prop_or("round")]
    pub stroke_linecap: &'static str,
    #[prop_or("round")]
    pub stroke_linejoin: &'static str,
}

mod r#clock_7 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clock7)]
pub fn r#clock_7(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" cx="12" r="10"  /><polyline points="12 6 12 12 9.5 16"  />
        </svg>
    }
}

}
pub use r#clock_7::Clock7;
mod r#file_down {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileDown)]
pub fn r#file_down(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><path d="M12 18v-6"  /><path d="m9 15 3 3 3-3"  />
        </svg>
    }
}

}
pub use r#file_down::FileDown;
mod r#arrow_up_right {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowUpRight)]
pub fn r#arrow_up_right(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y1="17" x1="7" x2="17" y2="7"  /><polyline points="7 7 17 7 17 17"  />
        </svg>
    }
}

}
pub use r#arrow_up_right::ArrowUpRight;
mod r#youtube {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Youtube)]
pub fn r#youtube(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 19c-2.3 0-6.4-.2-8.1-.6-.7-.2-1.2-.7-1.4-1.4-.3-1.1-.5-3.4-.5-5s.2-3.9.5-5c.2-.7.7-1.2 1.4-1.4C5.6 5.2 9.7 5 12 5s6.4.2 8.1.6c.7.2 1.2.7 1.4 1.4.3 1.1.5 3.4.5 5s-.2 3.9-.5 5c-.2.7-.7 1.2-1.4 1.4-1.7.4-5.8.6-8.1.6 0 0 0 0 0 0z"  /><polygon points="10 15 15 12 10 9"  />
        </svg>
    }
}

}
pub use r#youtube::Youtube;
mod r#zoom_out {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ZoomOut)]
pub fn r#zoom_out(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="11" r="8" cx="11"  /><line x1="21" y2="16.65" y1="21" x2="16.65"  /><line y2="11" y1="11" x2="14" x1="8"  />
        </svg>
    }
}

}
pub use r#zoom_out::ZoomOut;
mod r#accessibility {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Accessibility)]
pub fn r#accessibility(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="1" cx="16" cy="4"  /><path d="m18 19 1-7-5.87.94"  /><path d="m5 8 3-3 5.5 3-2.21 3.1"  /><path d="M4.24 14.48c-.19.58-.27 1.2-.23 1.84a5 5 0 0 0 5.31 4.67c.65-.04 1.25-.2 1.8-.46"  /><path d="M13.76 17.52c.19-.58.27-1.2.23-1.84a5 5 0 0 0-5.31-4.67c-.65.04-1.25.2-1.8.46"  />
        </svg>
    }
}

}
pub use r#accessibility::Accessibility;
mod r#phone {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Phone)]
pub fn r#phone(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"  />
        </svg>
    }
}

}
pub use r#phone::Phone;
mod r#file_output {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileOutput)]
pub fn r#file_output(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 22h14a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v4"  /><polyline points="14 2 14 8 20 8"  /><path d="M2 15h10"  /><path d="m5 12-3 3 3 3"  />
        </svg>
    }
}

}
pub use r#file_output::FileOutput;
mod r#plus_square {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PlusSquare)]
pub fn r#plus_square(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect height="18" rx="2" x="3" width="18" ry="2" y="3"  /><line x1="12" y1="8" y2="16" x2="12"  /><line y1="12" x1="8" x2="16" y2="12"  />
        </svg>
    }
}

}
pub use r#plus_square::PlusSquare;
mod r#zoom_in {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ZoomIn)]
pub fn r#zoom_in(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="11" cy="11" r="8"  /><line x1="21" x2="16.65" y1="21" y2="16.65"  /><line y2="14" y1="8" x2="11" x1="11"  /><line x1="8" y2="11" x2="14" y1="11"  />
        </svg>
    }
}

}
pub use r#zoom_in::ZoomIn;
mod r#align_right {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignRight)]
pub fn r#align_right(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x1="21" x2="3" y1="6" y2="6"  /><line x2="9" y1="12" x1="21" y2="12"  /><line y2="18" x1="21" y1="18" x2="7"  />
        </svg>
    }
}

}
pub use r#align_right::AlignRight;
mod r#list {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(List)]
pub fn r#list(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x2="21" x1="8" y1="6" y2="6"  /><line x1="8" y1="12" y2="12" x2="21"  /><line x2="21" y1="18" y2="18" x1="8"  /><line x1="3" y1="6" y2="6" x2="3.01"  /><line x1="3" y1="12" x2="3.01" y2="12"  /><line x2="3.01" y2="18" x1="3" y1="18"  />
        </svg>
    }
}

}
pub use r#list::List;
mod r#tablet {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Tablet)]
pub fn r#tablet(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect width="16" height="20" y="2" x="4" ry="2" rx="2"  /><line x1="12" x2="12.01" y1="18" y2="18"  />
        </svg>
    }
}

}
pub use r#tablet::Tablet;
mod r#tornado {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Tornado)]
pub fn r#tornado(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 4H3"  /><path d="M18 8H6"  /><path d="M19 12H9"  /><path d="M16 16h-6"  /><path d="M11 20H9"  />
        </svg>
    }
}

}
pub use r#tornado::Tornado;
mod r#alarm_clock_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlarmClockOff)]
pub fn r#alarm_clock_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M6.87 6.87a8 8 0 1 0 11.26 11.26"  /><path d="M19.9 14.25A7.44 7.44 0 0 0 20 13a8 8 0 0 0-8-8 7.44 7.44 0 0 0-1.25.1"  /><path d="m22 6-3-3"  /><path d="m6 19-2 2"  /><path d="m2 2 20 20"  /><path d="M4 4 2 6"  />
        </svg>
    }
}

}
pub use r#alarm_clock_off::AlarmClockOff;
mod r#check {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Check)]
pub fn r#check(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="20 6 9 17 4 12"  />
        </svg>
    }
}

}
pub use r#check::Check;
mod r#hammer {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Hammer)]
pub fn r#hammer(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m15 12-8.5 8.5c-.83.83-2.17.83-3 0 0 0 0 0 0 0a2.12 2.12 0 0 1 0-3L12 9"  /><path d="M17.64 15 22 10.64"  /><path d="m20.91 11.7-1.25-1.25c-.6-.6-.93-1.4-.93-2.25v-.86L16.01 4.6a5.56 5.56 0 0 0-3.94-1.64H9l.92.82A6.18 6.18 0 0 1 12 8.4v1.56l2 2h2.47l2.26 1.91"  />
        </svg>
    }
}

}
pub use r#hammer::Hammer;
mod r#folder_search_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderSearch2)]
pub fn r#folder_search_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"  /><circle cx="11.5" cy="12.5" r="2.5"  /><path d="M13.27 14.27 15 16"  />
        </svg>
    }
}

}
pub use r#folder_search_2::FolderSearch2;
mod r#clock {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clock)]
pub fn r#clock(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="12" r="10"  /><polyline points="12 6 12 12 16 14"  />
        </svg>
    }
}

}
pub use r#clock::Clock;
mod r#share_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Share2)]
pub fn r#share_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="5" r="3" cx="18"  /><circle cy="12" cx="6" r="3"  /><circle cx="18" cy="19" r="3"  /><line x2="15.42" x1="8.59" y1="13.51" y2="17.49"  /><line x2="8.59" x1="15.41" y1="6.51" y2="10.49"  />
        </svg>
    }
}

}
pub use r#share_2::Share2;
mod r#loader_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Loader2)]
pub fn r#loader_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 12a9 9 0 1 1-6.219-8.56"  />
        </svg>
    }
}

}
pub use r#loader_2::Loader2;
mod r#contact {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Contact)]
pub fn r#contact(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M17 18a2 2 0 0 0-2-2H9a2 2 0 0 0-2 2"  /><rect rx="2" x="3" width="18" height="18" y="4"  /><circle cy="10" cx="12" r="2"  /><line y2="4" x2="8" y1="2" x1="8"  /><line y2="4" x1="16" y1="2" x2="16"  />
        </svg>
    }
}

}
pub use r#contact::Contact;
mod r#lasso {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Lasso)]
pub fn r#lasso(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M7 22a5 5 0 0 1-2-4"  /><path d="M3.3 14A6.8 6.8 0 0 1 2 10c0-4.4 4.5-8 10-8s10 3.6 10 8-4.5 8-10 8a12 12 0 0 1-5-1"  /><path d="M5 18a2 2 0 1 0 0-4 2 2 0 0 0 0 4z"  />
        </svg>
    }
}

}
pub use r#lasso::Lasso;
mod r#align_start_vertical {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignStartVertical)]
pub fn r#align_start_vertical(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="6" y="14" width="9" height="6" rx="2"  /><rect rx="2" x="6" width="16" height="6" y="4"  /><path d="M2 2v20"  />
        </svg>
    }
}

}
pub use r#align_start_vertical::AlignStartVertical;
mod r#cloud_rain {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CloudRain)]
pub fn r#cloud_rain(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242"  /><path d="M16 14v6"  /><path d="M8 14v6"  /><path d="M12 16v6"  />
        </svg>
    }
}

}
pub use r#cloud_rain::CloudRain;
mod r#image {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Image)]
pub fn r#image(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="3" rx="2" ry="2" width="18" y="3" height="18"  /><circle r="2" cy="9" cx="9"  /><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"  />
        </svg>
    }
}

}
pub use r#image::Image;
mod r#beer {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Beer)]
pub fn r#beer(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M17 11h1a3 3 0 0 1 0 6h-1"  /><path d="M9 12v6"  /><path d="M13 12v6"  /><path d="M14 7.5c-1 0-1.44.5-3 .5s-2-.5-3-.5-1.72.5-2.5.5a2.5 2.5 0 0 1 0-5c.78 0 1.57.5 2.5.5S9.44 2 11 2s2 1.5 3 1.5 1.72-.5 2.5-.5a2.5 2.5 0 0 1 0 5c-.78 0-1.5-.5-2.5-.5Z"  /><path d="M5 8v12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V8"  />
        </svg>
    }
}

}
pub use r#beer::Beer;
mod r#database {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Database)]
pub fn r#database(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <ellipse ry="3" rx="9" cx="12" cy="5"  /><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"  /><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"  />
        </svg>
    }
}

}
pub use r#database::Database;
mod r#banana {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Banana)]
pub fn r#banana(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 13c3.5-2 8-2 10 2a5.5 5.5 0 0 1 8 5"  /><path d="M5.15 17.89c5.52-1.52 8.65-6.89 7-12C11.55 4 11.5 2 13 2c3.22 0 5 5.5 5 8 0 6.5-4.2 12-10.49 12C5.11 22 2 22 2 20c0-1.5 1.14-1.55 3.15-2.11Z"  />
        </svg>
    }
}

}
pub use r#banana::Banana;
mod r#clover {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clover)]
pub fn r#clover(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16.2 3.8a2.7 2.7 0 0 0-3.81 0l-.4.38-.4-.4a2.7 2.7 0 0 0-3.82 0C6.73 4.85 6.67 6.64 8 8l4 4 4-4c1.33-1.36 1.27-3.15.2-4.2z"  /><path d="M8 8c-1.36-1.33-3.15-1.27-4.2-.2a2.7 2.7 0 0 0 0 3.81l.38.4-.4.4a2.7 2.7 0 0 0 0 3.82C4.85 17.27 6.64 17.33 8 16"  /><path d="M16 16c1.36 1.33 3.15 1.27 4.2.2a2.7 2.7 0 0 0 0-3.81l-.38-.4.4-.4a2.7 2.7 0 0 0 0-3.82C19.15 6.73 17.36 6.67 16 8"  /><path d="M7.8 20.2a2.7 2.7 0 0 0 3.81 0l.4-.38.4.4a2.7 2.7 0 0 0 3.82 0c1.06-1.06 1.12-2.85-.21-4.21l-4-4-4 4c-1.33 1.36-1.27 3.15-.2 4.2z"  /><path d="m7 17-5 5"  />
        </svg>
    }
}

}
pub use r#clover::Clover;
mod r#corner_right_down {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CornerRightDown)]
pub fn r#corner_right_down(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="10 15 15 20 20 15"  /><path d="M4 4h7a4 4 0 0 1 4 4v12"  />
        </svg>
    }
}

}
pub use r#corner_right_down::CornerRightDown;
mod r#file_x {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileX)]
pub fn r#file_x(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><line x2="14.5" y2="17.5" x1="9.5" y1="12.5"  /><line y1="12.5" y2="17.5" x2="9.5" x1="14.5"  />
        </svg>
    }
}

}
pub use r#file_x::FileX;
mod r#folder_symlink {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderSymlink)]
pub fn r#folder_symlink(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 9V5c0-1.1.9-2 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H20a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H2"  /><path d="m8 16 3-3-3-3"  /><path d="M2 16v-1a2 2 0 0 1 2-2h6"  />
        </svg>
    }
}

}
pub use r#folder_symlink::FolderSymlink;
mod r#git_fork {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(GitFork)]
pub fn r#git_fork(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="18" r="3"  /><circle cx="6" cy="6" r="3"  /><circle r="3" cy="6" cx="18"  /><path d="M18 9v1a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V9"  /><path d="M12 12v3"  />
        </svg>
    }
}

}
pub use r#git_fork::GitFork;
mod r#scan_line {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ScanLine)]
pub fn r#scan_line(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 7V5a2 2 0 0 1 2-2h2"  /><path d="M17 3h2a2 2 0 0 1 2 2v2"  /><path d="M21 17v2a2 2 0 0 1-2 2h-2"  /><path d="M7 21H5a2 2 0 0 1-2-2v-2"  /><line x2="17" x1="7" y1="12" y2="12"  />
        </svg>
    }
}

}
pub use r#scan_line::ScanLine;
mod r#shirt {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Shirt)]
pub fn r#shirt(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20.38 3.46 16 2a4 4 0 0 1-8 0L3.62 3.46a2 2 0 0 0-1.34 2.23l.58 3.47a1 1 0 0 0 .99.84H6v10c0 1.1.9 2 2 2h8a2 2 0 0 0 2-2V10h2.15a1 1 0 0 0 .99-.84l.58-3.47a2 2 0 0 0-1.34-2.23z"  />
        </svg>
    }
}

}
pub use r#shirt::Shirt;
mod r#airplay {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Airplay)]
pub fn r#airplay(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M5 17H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2h-1"  /><polygon points="12 15 17 21 7 21 12 15"  />
        </svg>
    }
}

}
pub use r#airplay::Airplay;
mod r#alarm_plus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlarmPlus)]
pub fn r#alarm_plus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 21a8 8 0 1 0 0-16 8 8 0 0 0 0 16z"  /><path d="M5 3 2 6"  /><path d="m22 6-3-3"  /><path d="m6 19-2 2"  /><path d="m18 19 2 2"  /><path d="M12 10v6"  /><path d="M9 13h6"  />
        </svg>
    }
}

}
pub use r#alarm_plus::AlarmPlus;
mod r#signal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Signal)]
pub fn r#signal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 20h.01"  /><path d="M7 20v-4"  /><path d="M12 20v-8"  /><path d="M17 20V8"  /><path d="M22 4v16"  />
        </svg>
    }
}

}
pub use r#signal::Signal;
mod r#aperture {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Aperture)]
pub fn r#aperture(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" cx="12" r="10"  /><line x1="14.31" x2="20.05" y2="17.94" y1="8"  /><line x1="9.69" y2="8" y1="8" x2="21.17"  /><line y1="12" x2="13.12" x1="7.38" y2="2.06"  /><line y2="6.06" y1="16" x2="3.95" x1="9.69"  /><line y1="16" y2="16" x1="14.31" x2="2.83"  /><line x2="10.88" y2="21.94" y1="12" x1="16.62"  />
        </svg>
    }
}

}
pub use r#aperture::Aperture;
mod r#chevrons_up_down {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ChevronsUpDown)]
pub fn r#chevrons_up_down(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m7 15 5 5 5-5"  /><path d="m7 9 5-5 5 5"  />
        </svg>
    }
}

}
pub use r#chevrons_up_down::ChevronsUpDown;
mod r#codepen {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Codepen)]
pub fn r#codepen(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="12 2 22 8.5 22 15.5 12 22 2 15.5 2 8.5 12 2"  /><line y2="15.5" x1="12" y1="22" x2="12"  /><polyline points="22 8.5 12 15.5 2 8.5"  /><polyline points="2 15.5 12 8.5 22 15.5"  /><line y1="2" x2="12" x1="12" y2="8.5"  />
        </svg>
    }
}

}
pub use r#codepen::Codepen;
mod r#file_spreadsheet {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileSpreadsheet)]
pub fn r#file_spreadsheet(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><path d="M8 13h2"  /><path d="M8 17h2"  /><path d="M14 13h2"  /><path d="M14 17h2"  />
        </svg>
    }
}

}
pub use r#file_spreadsheet::FileSpreadsheet;
mod r#hash {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Hash)]
pub fn r#hash(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x2="20" y1="9" y2="9" x1="4"  /><line x1="4" y2="15" y1="15" x2="20"  /><line x1="10" y1="3" x2="8" y2="21"  /><line x1="16" y1="3" x2="14" y2="21"  />
        </svg>
    }
}

}
pub use r#hash::Hash;
mod r#podcast {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Podcast)]
pub fn r#podcast(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="11" cx="12" r="1"  /><path d="M11 17a1 1 0 0 1 2 0c0 .5-.34 3-.5 4.5a.5.5 0 0 1-1 0c-.16-1.5-.5-4-.5-4.5Z"  /><path d="M8 14a5 5 0 1 1 8 0"  /><path d="M17 18.5a9 9 0 1 0-10 0"  />
        </svg>
    }
}

}
pub use r#podcast::Podcast;
mod r#slash {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Slash)]
pub fn r#slash(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" r="10" cx="12"  /><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"  />
        </svg>
    }
}

}
pub use r#slash::Slash;
mod r#flashlight_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FlashlightOff)]
pub fn r#flashlight_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16 16v4a2 2 0 0 1-2 2h-4a2 2 0 0 1-2-2V10c0-2-2-2-2-4"  /><path d="M7 2h11v4c0 2-2 2-2 4v1"  /><line y1="6" y2="6" x1="11" x2="18"  /><line y1="2" x1="2" x2="22" y2="22"  />
        </svg>
    }
}

}
pub use r#flashlight_off::FlashlightOff;
mod r#rocket {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Rocket)]
pub fn r#rocket(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"  /><path d="m12 15-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"  /><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"  /><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"  />
        </svg>
    }
}

}
pub use r#rocket::Rocket;
mod r#crosshair {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Crosshair)]
pub fn r#crosshair(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cy="12" cx="12"  /><line x1="22" y2="12" y1="12" x2="18"  /><line x1="6" y2="12" y1="12" x2="2"  /><line x1="12" x2="12" y1="6" y2="2"  /><line x2="12" x1="12" y1="22" y2="18"  />
        </svg>
    }
}

}
pub use r#crosshair::Crosshair;
mod r#screen_share {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ScreenShare)]
pub fn r#screen_share(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M13 3H4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-3"  /><path d="M8 21h8"  /><path d="M12 17v4"  /><path d="m17 8 5-5"  /><path d="M17 3h5v5"  />
        </svg>
    }
}

}
pub use r#screen_share::ScreenShare;
mod r#folder_input {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderInput)]
pub fn r#folder_input(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 9V5c0-1.1.9-2 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H20a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-1"  /><path d="M2 13h10"  /><path d="m9 16 3-3-3-3"  />
        </svg>
    }
}

}
pub use r#folder_input::FolderInput;
mod r#headphones {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Headphones)]
pub fn r#headphones(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 18v-6a9 9 0 0 1 18 0v6"  /><path d="M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3zM3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3z"  />
        </svg>
    }
}

}
pub use r#headphones::Headphones;
mod r#speaker {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Speaker)]
pub fn r#speaker(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect width="16" x="4" ry="2" y="2" height="20" rx="2"  /><circle cx="12" r="4" cy="14"  /><line x1="12" y2="6" x2="12.01" y1="6"  />
        </svg>
    }
}

}
pub use r#speaker::Speaker;
mod r#banknote {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Banknote)]
pub fn r#banknote(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect height="12" y="6" rx="2" x="2" width="20"  /><circle cy="12" cx="12" r="2"  /><path d="M6 12h.01M18 12h.01"  />
        </svg>
    }
}

}
pub use r#banknote::Banknote;
mod r#bed_double {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BedDouble)]
pub fn r#bed_double(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 20v-8a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v8"  /><path d="M4 10V6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v4"  /><path d="M12 4v6"  /><path d="M2 18h20"  />
        </svg>
    }
}

}
pub use r#bed_double::BedDouble;
mod r#files {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Files)]
pub fn r#files(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M15.5 2H8.6c-.4 0-.8.2-1.1.5-.3.3-.5.7-.5 1.1v12.8c0 .4.2.8.5 1.1.3.3.7.5 1.1.5h9.8c.4 0 .8-.2 1.1-.5.3-.3.5-.7.5-1.1V6.5L15.5 2z"  /><path d="M3 7.6v12.8c0 .4.2.8.5 1.1.3.3.7.5 1.1.5h9.8"  /><path d="M15 2v5h5"  />
        </svg>
    }
}

}
pub use r#files::Files;
mod r#lock {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Lock)]
pub fn r#lock(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="3" rx="2" ry="2" y="11" width="18" height="11"  /><path d="M7 11V7a5 5 0 0 1 10 0v4"  />
        </svg>
    }
}

}
pub use r#lock::Lock;
mod r#mic_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MicOff)]
pub fn r#mic_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x1="2" y1="2" x2="22" y2="22"  /><path d="M18.89 13.23A7.12 7.12 0 0 0 19 12v-2"  /><path d="M5 10v2a7 7 0 0 0 12 5"  /><path d="M15 9.34V5a3 3 0 0 0-5.68-1.33"  /><path d="M9 9v3a3 3 0 0 0 5.12 2.12"  /><line y1="19" x2="12" x1="12" y2="22"  />
        </svg>
    }
}

}
pub use r#mic_off::MicOff;
mod r#chevron_left {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ChevronLeft)]
pub fn r#chevron_left(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="15 18 9 12 15 6"  />
        </svg>
    }
}

}
pub use r#chevron_left::ChevronLeft;
mod r#user_cog {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(UserCog)]
pub fn r#user_cog(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"  /><circle cy="7" cx="9" r="4"  /><circle cy="11" r="2" cx="19"  /><path d="M19 8v1"  /><path d="M19 13v1"  /><path d="m21.6 9.5-.87.5"  /><path d="m17.27 12-.87.5"  /><path d="m21.6 12.5-.87-.5"  /><path d="m17.27 10-.87-.5"  />
        </svg>
    }
}

}
pub use r#user_cog::UserCog;
mod r#volume_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Volume2)]
pub fn r#volume_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"  /><path d="M15.54 8.46a5 5 0 0 1 0 7.07"  /><path d="M19.07 4.93a10 10 0 0 1 0 14.14"  />
        </svg>
    }
}

}
pub use r#volume_2::Volume2;
mod r#file_input {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileInput)]
pub fn r#file_input(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 22h14a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v4"  /><polyline points="14 2 14 8 20 8"  /><path d="M2 15h10"  /><path d="m9 18 3-3-3-3"  />
        </svg>
    }
}

}
pub use r#file_input::FileInput;
mod r#wrench {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Wrench)]
pub fn r#wrench(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"  />
        </svg>
    }
}

}
pub use r#wrench::Wrench;
mod r#clock_8 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clock8)]
pub fn r#clock_8(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cx="12" cy="12"  /><polyline points="12 6 12 12 8 14"  />
        </svg>
    }
}

}
pub use r#clock_8::Clock8;
mod r#pencil {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Pencil)]
pub fn r#pencil(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y1="2" x2="22" x1="18" y2="6"  /><path d="M7.5 20.5 19 9l-4-4L3.5 16.5 2 22z"  />
        </svg>
    }
}

}
pub use r#pencil::Pencil;
mod r#currency {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Currency)]
pub fn r#currency(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" r="8" cx="12"  /><line x1="3" x2="6" y1="3" y2="6"  /><line x1="21" y1="3" x2="18" y2="6"  /><line x1="3" y1="21" y2="18" x2="6"  /><line x1="21" x2="18" y2="18" y1="21"  />
        </svg>
    }
}

}
pub use r#currency::Currency;
mod r#file_type {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileType)]
pub fn r#file_type(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><path d="M9 13v-1h6v1"  /><path d="M11 18h2"  /><path d="M12 12v6"  />
        </svg>
    }
}

}
pub use r#file_type::FileType;
mod r#baggage_claim {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BaggageClaim)]
pub fn r#baggage_claim(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M22 18H6a2 2 0 0 1-2-2V7a2 2 0 0 0-2-2"  /><path d="M17 14V4a2 2 0 0 0-2-2h-1a2 2 0 0 0-2 2v10"  /><rect y="6" rx="1" width="13" x="8" height="8"  /><circle cx="18" cy="20" r="2"  /><circle cy="20" r="2" cx="9"  />
        </svg>
    }
}

}
pub use r#baggage_claim::BaggageClaim;
mod r#maximize_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Maximize2)]
pub fn r#maximize_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="15 3 21 3 21 9"  /><polyline points="9 21 3 21 3 15"  /><line x1="21" y1="3" y2="10" x2="14"  /><line x2="10" y2="14" y1="21" x1="3"  />
        </svg>
    }
}

}
pub use r#maximize_2::Maximize2;
mod r#vibrate {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Vibrate)]
pub fn r#vibrate(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m2 8 2 2-2 2 2 2-2 2"  /><path d="m22 8-2 2 2 2-2 2 2 2"  /><rect y="5" rx="1" height="14" x="8" width="8"  />
        </svg>
    }
}

}
pub use r#vibrate::Vibrate;
mod r#bed_single {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BedSingle)]
pub fn r#bed_single(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 20v-8a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v8"  /><path d="M5 10V6a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v4"  /><path d="M3 18h18"  />
        </svg>
    }
}

}
pub use r#bed_single::BedSingle;
mod r#calendar_heart {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CalendarHeart)]
pub fn r#calendar_heart(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 10V6a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v14c0 1.1.9 2 2 2h7"  /><path d="M16 2v4"  /><path d="M8 2v4"  /><path d="M3 10h18"  /><path d="M21.29 14.7a2.43 2.43 0 0 0-2.65-.52c-.3.12-.57.3-.8.53l-.34.34-.35-.34a2.43 2.43 0 0 0-2.65-.53c-.3.12-.56.3-.79.53-.95.94-1 2.53.2 3.74L17.5 22l3.6-3.55c1.2-1.21 1.14-2.8.19-3.74Z"  />
        </svg>
    }
}

}
pub use r#calendar_heart::CalendarHeart;
mod r#divide {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Divide)]
pub fn r#divide(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="6" r="1" cx="12"  /><line x2="19" y1="12" y2="12" x1="5"  /><circle r="1" cy="18" cx="12"  />
        </svg>
    }
}

}
pub use r#divide::Divide;
mod r#sticky_note {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(StickyNote)]
pub fn r#sticky_note(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M15.5 3H5a2 2 0 0 0-2 2v14c0 1.1.9 2 2 2h14a2 2 0 0 0 2-2V8.5L15.5 3Z"  /><path d="M15 3v6h6"  />
        </svg>
    }
}

}
pub use r#sticky_note::StickyNote;
mod r#sort_desc {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SortDesc)]
pub fn r#sort_desc(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M11 5h10"  /><path d="M11 9h7"  /><path d="M11 13h4"  /><path d="m3 17 3 3 3-3"  /><path d="M6 18V4"  />
        </svg>
    }
}

}
pub use r#sort_desc::SortDesc;
mod r#form_input {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FormInput)]
pub fn r#form_input(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="6" height="12" x="2" width="20" rx="2"  /><path d="M12 12h.01"  /><path d="M17 12h.01"  /><path d="M7 12h.01"  />
        </svg>
    }
}

}
pub use r#form_input::FormInput;
mod r#arrow_left {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowLeft)]
pub fn r#arrow_left(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x1="19" y1="12" x2="5" y2="12"  /><polyline points="12 19 5 12 12 5"  />
        </svg>
    }
}

}
pub use r#arrow_left::ArrowLeft;
mod r#user_x {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(UserX)]
pub fn r#user_x(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"  /><circle r="4" cx="9" cy="7"  /><line y1="8" x2="22" x1="17" y2="13"  /><line y1="8" x2="17" x1="22" y2="13"  />
        </svg>
    }
}

}
pub use r#user_x::UserX;
mod r#lamp_wall_up {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LampWallUp)]
pub fn r#lamp_wall_up(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M11 4h6l3 7H8l3-7Z"  /><path d="M14 11v5a2 2 0 0 1-2 2H8"  /><path d="M4 15h2a2 2 0 0 1 2 2v2a2 2 0 0 1-2 2H4v-6Z"  />
        </svg>
    }
}

}
pub use r#lamp_wall_up::LampWallUp;
mod r#axis_3_d {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Axis3D)]
pub fn r#axis_3_d(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 4v16h16"  /><path d="m4 20 7-7"  />
        </svg>
    }
}

}
pub use r#axis_3_d::Axis3D;
mod r#japanese_yen {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(JapaneseYen)]
pub fn r#japanese_yen(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 9.5V21m0-11.5L6 3m6 6.5L18 3"  /><path d="M6 15h12"  /><path d="M6 11h12"  />
        </svg>
    }
}

}
pub use r#japanese_yen::JapaneseYen;
mod r#tree_pine {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(TreePine)]
pub fn r#tree_pine(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m17 14 3 3.3a1 1 0 0 1-.7 1.7H4.7a1 1 0 0 1-.7-1.7L7 14h-.3a1 1 0 0 1-.7-1.7L9 9h-.2A1 1 0 0 1 8 7.3L12 3l4 4.3a1 1 0 0 1-.8 1.7H15l3 3.3a1 1 0 0 1-.7 1.7H17Z"  /><path d="M12 22v-3"  />
        </svg>
    }
}

}
pub use r#tree_pine::TreePine;
mod r#bell {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Bell)]
pub fn r#bell(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"  /><path d="M13.73 21a2 2 0 0 1-3.46 0"  />
        </svg>
    }
}

}
pub use r#bell::Bell;
mod r#align_vertical_justify_end {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignVerticalJustifyEnd)]
pub fn r#align_vertical_justify_end(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect height="6" y="12" width="14" rx="2" x="5"  /><rect y="2" width="10" height="6" rx="2" x="7"  /><path d="M2 22h20"  />
        </svg>
    }
}

}
pub use r#align_vertical_justify_end::AlignVerticalJustifyEnd;
mod r#divide_circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(DivideCircle)]
pub fn r#divide_circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="12" x2="16" x1="8" y1="12"  /><line x2="12" y1="16" x1="12" y2="16"  /><line x1="12" x2="12" y2="8" y1="8"  /><circle r="10" cy="12" cx="12"  />
        </svg>
    }
}

}
pub use r#divide_circle::DivideCircle;
mod r#file_lock_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileLock2)]
pub fn r#file_lock_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 5V4a2 2 0 0 1 2-2h8.5L20 7.5V20a2 2 0 0 1-2 2H4"  /><polyline points="14 2 14 8 20 8"  /><rect height="5" rx="1" width="8" y="13" x="2"  /><path d="M8 13v-2a2 2 0 1 0-4 0v2"  />
        </svg>
    }
}

}
pub use r#file_lock_2::FileLock2;
mod r#align_left {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignLeft)]
pub fn r#align_left(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="6" x1="21" y1="6" x2="3"  /><line x2="3" y2="12" x1="15" y1="12"  /><line x1="17" x2="3" y1="18" y2="18"  />
        </svg>
    }
}

}
pub use r#align_left::AlignLeft;
mod r#reply_all {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ReplyAll)]
pub fn r#reply_all(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="7 17 2 12 7 7"  /><polyline points="12 17 7 12 12 7"  /><path d="M22 18v-2a4 4 0 0 0-4-4H7"  />
        </svg>
    }
}

}
pub use r#reply_all::ReplyAll;
mod r#view {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(View)]
pub fn r#view(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M5 12s2.545-5 7-5c4.454 0 7 5 7 5s-2.546 5-7 5c-4.455 0-7-5-7-5z"  /><path d="M12 13a1 1 0 1 0 0-2 1 1 0 0 0 0 2z"  /><path d="M21 17v2a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-2"  /><path d="M21 7V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v2"  />
        </svg>
    }
}

}
pub use r#view::View;
mod r#align_horizontal_distribute_start {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignHorizontalDistributeStart)]
pub fn r#align_horizontal_distribute_start(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect height="14" rx="2" x="4" y="5" width="6"  /><rect y="7" x="14" width="6" height="10" rx="2"  /><path d="M4 2v20"  /><path d="M14 2v20"  />
        </svg>
    }
}

}
pub use r#align_horizontal_distribute_start::AlignHorizontalDistributeStart;
mod r#gem {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Gem)]
pub fn r#gem(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="6 3 18 3 22 9 12 22 2 9"  /><path d="m12 22 4-13-3-6"  /><path d="M12 22 8 9l3-6"  /><path d="M2 9h20"  />
        </svg>
    }
}

}
pub use r#gem::Gem;
mod r#align_start_horizontal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignStartHorizontal)]
pub fn r#align_start_horizontal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect width="6" rx="2" x="4" y="6" height="16"  /><rect rx="2" y="6" width="6" height="9" x="14"  /><path d="M22 2H2"  />
        </svg>
    }
}

}
pub use r#align_start_horizontal::AlignStartHorizontal;
mod r#plus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Plus)]
pub fn r#plus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x2="12" x1="12" y1="5" y2="19"  /><line x2="19" x1="5" y2="12" y1="12"  />
        </svg>
    }
}

}
pub use r#plus::Plus;
mod r#signal_high {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SignalHigh)]
pub fn r#signal_high(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 20h.01"  /><path d="M7 20v-4"  /><path d="M12 20v-8"  /><path d="M17 20V8"  />
        </svg>
    }
}

}
pub use r#signal_high::SignalHigh;
mod r#corner_up_right {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CornerUpRight)]
pub fn r#corner_up_right(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="15 14 20 9 15 4"  /><path d="M4 20v-7a4 4 0 0 1 4-4h12"  />
        </svg>
    }
}

}
pub use r#corner_up_right::CornerUpRight;
mod r#wrap_text {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(WrapText)]
pub fn r#wrap_text(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x2="21" x1="3" y1="6" y2="6"  /><path d="M3 12h15a3 3 0 1 1 0 6h-4"  /><polyline points="16 16 14 18 16 20"  /><line x1="3" y1="18" y2="18" x2="10"  />
        </svg>
    }
}

}
pub use r#wrap_text::WrapText;
mod r#zap_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ZapOff)]
pub fn r#zap_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="12.41 6.75 13 2 10.57 4.92"  /><polyline points="18.57 12.91 21 10 15.66 10"  /><polyline points="8 8 3 14 12 14 11 22 16 16"  /><line x1="2" y1="2" x2="22" y2="22"  />
        </svg>
    }
}

}
pub use r#zap_off::ZapOff;
mod r#person_standing {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PersonStanding)]
pub fn r#person_standing(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="5" r="1"  /><path d="m9 20 3-6 3 6"  /><path d="m6 8 6 2 6-2"  /><path d="M12 10v4"  />
        </svg>
    }
}

}
pub use r#person_standing::PersonStanding;
mod r#circle_slashed {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CircleSlashed)]
pub fn r#circle_slashed(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cx="12" cy="12"  /><path d="M22 2 2 22"  />
        </svg>
    }
}

}
pub use r#circle_slashed::CircleSlashed;
mod r#twitter {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Twitter)]
pub fn r#twitter(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M22 4s-.7 2.1-2 3.4c1.6 10-9.4 17.3-18 11.6 2.2.1 4.4-.6 6-2C3 15.5.5 9.6 3 5c2.2 2.6 5.6 4.1 9 4-.9-4.2 4-6.6 7-3.8 1.1 0 3-1.2 3-1.2z"  />
        </svg>
    }
}

}
pub use r#twitter::Twitter;
mod r#baseline {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Baseline)]
pub fn r#baseline(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 20h16"  /><path d="m6 16 6-12 6 12"  /><path d="M8 12h8"  />
        </svg>
    }
}

}
pub use r#baseline::Baseline;
mod r#key {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Key)]
pub fn r#key(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m21 2-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0 3 3L22 7l-3-3m-3.5 3.5L19 4"  />
        </svg>
    }
}

}
pub use r#key::Key;
mod r#battery_full {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BatteryFull)]
pub fn r#battery_full(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="2" rx="2" y="7" width="16" height="10" ry="2"  /><line y2="13" x1="22" x2="22" y1="11"  /><line x2="6" y2="13" y1="11" x1="6"  /><line x1="10" y1="11" x2="10" y2="13"  /><line y1="11" x2="14" y2="13" x1="14"  />
        </svg>
    }
}

}
pub use r#battery_full::BatteryFull;
mod r#battery_low {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BatteryLow)]
pub fn r#battery_low(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="7" x="2" height="10" rx="2" ry="2" width="16"  /><line y2="13" x1="22" y1="11" x2="22"  /><line x1="6" x2="6" y1="11" y2="13"  />
        </svg>
    }
}

}
pub use r#battery_low::BatteryLow;
mod r#file_search_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileSearch2)]
pub fn r#file_search_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><circle r="2.5" cx="11.5" cy="14.5"  /><path d="M13.25 16.25 15 18"  />
        </svg>
    }
}

}
pub use r#file_search_2::FileSearch2;
mod r#angry {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Angry)]
pub fn r#angry(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="12" r="10"  /><path d="M16 16s-1.5-2-4-2-4 2-4 2"  /><path d="M7.5 8 10 9"  /><path d="m14 9 2.5-1"  /><path d="M9 10h0"  /><path d="M15 10h0"  />
        </svg>
    }
}

}
pub use r#angry::Angry;
mod r#grip_vertical {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(GripVertical)]
pub fn r#grip_vertical(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="1" cy="12" cx="9"  /><circle cx="9" r="1" cy="5"  /><circle cx="9" r="1" cy="19"  /><circle cy="12" cx="15" r="1"  /><circle cy="5" r="1" cx="15"  /><circle cx="15" r="1" cy="19"  />
        </svg>
    }
}

}
pub use r#grip_vertical::GripVertical;
mod r#life_buoy {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LifeBuoy)]
pub fn r#life_buoy(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cx="12" cy="12"  /><circle cy="12" r="4" cx="12"  /><line y2="9.17" x2="9.17" y1="4.93" x1="4.93"  /><line y1="14.83" x2="19.07" y2="19.07" x1="14.83"  /><line x2="19.07" x1="14.83" y1="9.17" y2="4.93"  /><line x1="14.83" y2="5.64" y1="9.17" x2="18.36"  /><line y2="14.83" x1="4.93" x2="9.17" y1="19.07"  />
        </svg>
    }
}

}
pub use r#life_buoy::LifeBuoy;
mod r#scroll {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Scroll)]
pub fn r#scroll(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10 17v2a2 2 0 0 1-2 2v0a2 2 0 0 1-2-2V5a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v3h3"  /><path d="M22 17v2a2 2 0 0 1-2 2H8"  /><path d="M19 17V5a2 2 0 0 0-2-2H4"  /><path d="M22 17H10"  />
        </svg>
    }
}

}
pub use r#scroll::Scroll;
mod r#share {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Share)]
pub fn r#share(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"  /><polyline points="16 6 12 2 8 6"  /><line y1="2" x1="12" x2="12" y2="15"  />
        </svg>
    }
}

}
pub use r#share::Share;
mod r#stop_circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(StopCircle)]
pub fn r#stop_circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" cx="12" r="10"  /><rect y="9" height="6" width="6" x="9"  />
        </svg>
    }
}

}
pub use r#stop_circle::StopCircle;
mod r#diff {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Diff)]
pub fn r#diff(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 3v14"  /><path d="M5 10h14"  /><path d="M5 21h14"  />
        </svg>
    }
}

}
pub use r#diff::Diff;
mod r#arrow_right_circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowRightCircle)]
pub fn r#arrow_right_circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="12" r="10"  /><polyline points="12 16 16 12 12 8"  /><line x2="16" y2="12" x1="8" y1="12"  />
        </svg>
    }
}

}
pub use r#arrow_right_circle::ArrowRightCircle;
mod r#clock_3 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clock3)]
pub fn r#clock_3(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cy="12" cx="12"  /><polyline points="12 6 12 12 16.5 12"  />
        </svg>
    }
}

}
pub use r#clock_3::Clock3;
mod r#log_in {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LogIn)]
pub fn r#log_in(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"  /><polyline points="10 17 15 12 10 7"  /><line y2="12" y1="12" x2="3" x1="15"  />
        </svg>
    }
}

}
pub use r#log_in::LogIn;
mod r#bluetooth_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BluetoothOff)]
pub fn r#bluetooth_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m17 17-5 5V12l-5 5"  /><path d="m2 2 20 20"  /><path d="M14.5 9.5 17 7l-5-5v4.5"  />
        </svg>
    }
}

}
pub use r#bluetooth_off::BluetoothOff;
mod r#highlighter {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Highlighter)]
pub fn r#highlighter(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m9 11-6 6v3h9l3-3"  /><path d="m22 12-4.6 4.6a2 2 0 0 1-2.8 0l-5.2-5.2a2 2 0 0 1 0-2.8L14 4"  />
        </svg>
    }
}

}
pub use r#highlighter::Highlighter;
mod r#ice_cream {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(IceCream)]
pub fn r#ice_cream(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m7 11 4.08 10.35a1 1 0 0 0 1.84 0L17 11"  /><path d="M17 7A5 5 0 0 0 7 7"  /><path d="M17 7a2 2 0 0 1 0 4H7a2 2 0 0 1 0-4"  />
        </svg>
    }
}

}
pub use r#ice_cream::IceCream;
mod r#list_end {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ListEnd)]
pub fn r#list_end(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16 12H3"  /><path d="M16 6H3"  /><path d="M10 18H3"  /><path d="M21 6v10a2 2 0 0 1-2 2h-4"  /><path d="m16 16-2 2 2 2"  />
        </svg>
    }
}

}
pub use r#list_end::ListEnd;
mod r#list_minus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ListMinus)]
pub fn r#list_minus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M11 12H3"  /><path d="M16 6H3"  /><path d="M16 18H3"  /><path d="M21 12h-6"  />
        </svg>
    }
}

}
pub use r#list_minus::ListMinus;
mod r#minus_square {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MinusSquare)]
pub fn r#minus_square(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect rx="2" ry="2" width="18" height="18" x="3" y="3"  /><line x2="16" x1="8" y1="12" y2="12"  />
        </svg>
    }
}

}
pub use r#minus_square::MinusSquare;
mod r#grid {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Grid)]
pub fn r#grid(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect rx="2" y="3" ry="2" x="3" height="18" width="18"  /><line x1="3" x2="21" y2="9" y1="9"  /><line y1="15" x2="21" x1="3" y2="15"  /><line y1="3" y2="21" x2="9" x1="9"  /><line x1="15" y1="3" y2="21" x2="15"  />
        </svg>
    }
}

}
pub use r#grid::Grid;
mod r#codesandbox {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Codesandbox)]
pub fn r#codesandbox(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"  /><polyline points="7.5 4.21 12 6.81 16.5 4.21"  /><polyline points="7.5 19.79 7.5 14.6 3 12"  /><polyline points="21 12 16.5 14.6 16.5 19.79"  /><polyline points="3.27 6.96 12 12.01 20.73 6.96"  /><line y1="22.08" x1="12" x2="12" y2="12"  />
        </svg>
    }
}

}
pub use r#codesandbox::Codesandbox;
mod r#user_minus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(UserMinus)]
pub fn r#user_minus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"  /><circle cx="9" cy="7" r="4"  /><line y1="11" x1="22" x2="16" y2="11"  />
        </svg>
    }
}

}
pub use r#user_minus::UserMinus;
mod r#bar_chart_3 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BarChart3)]
pub fn r#bar_chart_3(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 3v18h18"  /><path d="M18 17V9"  /><path d="M13 17V5"  /><path d="M8 17v-3"  />
        </svg>
    }
}

}
pub use r#bar_chart_3::BarChart3;
mod r#move_diagonal_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MoveDiagonal2)]
pub fn r#move_diagonal_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="5 11 5 5 11 5"  /><polyline points="19 13 19 19 13 19"  /><line x1="5" y2="19" x2="19" y1="5"  />
        </svg>
    }
}

}
pub use r#move_diagonal_2::MoveDiagonal2;
mod r#rotate_cw {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(RotateCw)]
pub fn r#rotate_cw(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 2v6h-6"  /><path d="M21 13a9 9 0 1 1-3-7.7L21 8"  />
        </svg>
    }
}

}
pub use r#rotate_cw::RotateCw;
mod r#swords {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Swords)]
pub fn r#swords(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="14.5 17.5 3 6 3 3 6 3 17.5 14.5"  /><line y1="19" y2="13" x1="13" x2="19"  /><line x2="20" y1="16" y2="20" x1="16"  /><line y2="19" x2="21" y1="21" x1="19"  /><polyline points="14.5 6.5 18 3 21 3 21 6 17.5 9.5"  /><line y1="14" x2="9" y2="18" x1="5"  /><line x1="7" y1="17" x2="4" y2="20"  /><line x1="3" y2="21" x2="5" y1="19"  />
        </svg>
    }
}

}
pub use r#swords::Swords;
mod r#ticket {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Ticket)]
pub fn r#ticket(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 7v2a3 3 0 1 1 0 6v2c0 1.1.9 2 2 2h14a2 2 0 0 0 2-2v-2a3 3 0 1 1 0-6V7a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2Z"  /><path d="M13 5v2"  /><path d="M13 17v2"  /><path d="M13 11v2"  />
        </svg>
    }
}

}
pub use r#ticket::Ticket;
mod r#cast {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Cast)]
pub fn r#cast(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 8V6a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-6"  /><path d="M2 12a9 9 0 0 1 8 8"  /><path d="M2 16a5 5 0 0 1 4 4"  /><line x1="2" x2="2.01" y1="20" y2="20"  />
        </svg>
    }
}

}
pub use r#cast::Cast;
mod r#sticker {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Sticker)]
pub fn r#sticker(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M15.5 3H5a2 2 0 0 0-2 2v14c0 1.1.9 2 2 2h14a2 2 0 0 0 2-2V8.5L15.5 3Z"  /><path d="M15 3v6h6"  /><path d="M10 16s.8 1 2 1c1.3 0 2-1 2-1"  /><path d="M8 13h0"  /><path d="M16 13h0"  />
        </svg>
    }
}

}
pub use r#sticker::Sticker;
mod r#car {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Car)]
pub fn r#car(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14 16H9m10 0h3v-3.15a1 1 0 0 0-.84-.99L16 11l-2.7-3.6a1 1 0 0 0-.8-.4H5.24a2 2 0 0 0-1.8 1.1l-.8 1.63A6 6 0 0 0 2 12.42V16h2"  /><circle cx="6.5" cy="16.5" r="2.5"  /><circle cy="16.5" cx="16.5" r="2.5"  />
        </svg>
    }
}

}
pub use r#car::Car;
mod r#equal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Equal)]
pub fn r#equal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="9" x2="19" x1="5" y1="9"  /><line y2="15" y1="15" x1="5" x2="19"  />
        </svg>
    }
}

}
pub use r#equal::Equal;
mod r#gauge {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Gauge)]
pub fn r#gauge(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m12 15 3.5-3.5"  /><path d="M20.3 18c.4-1 .7-2.2.7-3.4C21 9.8 17 6 12 6s-9 3.8-9 8.6c0 1.2.3 2.4.7 3.4"  />
        </svg>
    }
}

}
pub use r#gauge::Gauge;
mod r#laptop_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Laptop2)]
pub fn r#laptop_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="3" height="12" ry="2" rx="2" y="4" width="18"  /><line y1="20" x2="22" x1="2" y2="20"  />
        </svg>
    }
}

}
pub use r#laptop_2::Laptop2;
mod r#move_3_d {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Move3D)]
pub fn r#move_3_d(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M5 3v16h16"  /><path d="m5 19 6-6"  /><path d="m2 6 3-3 3 3"  /><path d="m18 16 3 3-3 3"  />
        </svg>
    }
}

}
pub use r#move_3_d::Move3D;
mod r#file_key {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileKey)]
pub fn r#file_key(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><circle cx="10" cy="16" r="2"  /><path d="m16 10-4.5 4.5"  /><path d="m15 11 1 1"  />
        </svg>
    }
}

}
pub use r#file_key::FileKey;
mod r#navigation_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Navigation2)]
pub fn r#navigation_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="12 2 19 21 12 17 5 21 12 2"  />
        </svg>
    }
}

}
pub use r#navigation_2::Navigation2;
mod r#refresh_ccw {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(RefreshCcw)]
pub fn r#refresh_ccw(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 2v6h6"  /><path d="M21 12A9 9 0 0 0 6 5.3L3 8"  /><path d="M21 22v-6h-6"  /><path d="M3 12a9 9 0 0 0 15 6.7l3-2.7"  />
        </svg>
    }
}

}
pub use r#refresh_ccw::RefreshCcw;
mod r#snowflake {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Snowflake)]
pub fn r#snowflake(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y1="12" x2="22" x1="2" y2="12"  /><line x1="12" x2="12" y2="22" y1="2"  /><path d="m20 16-4-4 4-4"  /><path d="m4 8 4 4-4 4"  /><path d="m16 4-4 4-4-4"  /><path d="m8 20 4-4 4 4"  />
        </svg>
    }
}

}
pub use r#snowflake::Snowflake;
mod r#sunrise {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Sunrise)]
pub fn r#sunrise(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 2v8"  /><path d="m4.93 10.93 1.41 1.41"  /><path d="M2 18h2"  /><path d="M20 18h2"  /><path d="m19.07 10.93-1.41 1.41"  /><path d="M22 22H2"  /><path d="m8 6 4-4 4 4"  /><path d="M16 18a4 4 0 0 0-8 0"  />
        </svg>
    }
}

}
pub use r#sunrise::Sunrise;
mod r#clipboard_x {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ClipboardX)]
pub fn r#clipboard_x(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect rx="1" width="8" y="2" x="8" height="4" ry="1"  /><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"  /><path d="m15 11-6 6"  /><path d="m9 11 6 6"  />
        </svg>
    }
}

}
pub use r#clipboard_x::ClipboardX;
mod r#skip_forward {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SkipForward)]
pub fn r#skip_forward(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="5 4 15 12 5 20 5 4"  /><line x1="19" x2="19" y1="5" y2="19"  />
        </svg>
    }
}

}
pub use r#skip_forward::SkipForward;
mod r#download {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Download)]
pub fn r#download(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"  /><polyline points="7 10 12 15 17 10"  /><line y2="3" y1="15" x1="12" x2="12"  />
        </svg>
    }
}

}
pub use r#download::Download;
mod r#align_center_vertical {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignCenterVertical)]
pub fn r#align_center_vertical(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 2v20"  /><path d="M8 10H4a2 2 0 0 1-2-2V6c0-1.1.9-2 2-2h4"  /><path d="M16 10h4a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2h-4"  /><path d="M8 20H7a2 2 0 0 1-2-2v-2c0-1.1.9-2 2-2h1"  /><path d="M16 14h1a2 2 0 0 1 2 2v2a2 2 0 0 1-2 2h-1"  />
        </svg>
    }
}

}
pub use r#align_center_vertical::AlignCenterVertical;
mod r#wine {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Wine)]
pub fn r#wine(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M8 22h8"  /><path d="M7 10h10"  /><path d="M12 15v7"  /><path d="M12 15a5 5 0 0 0 5-5c0-2-.5-4-2-8H9c-1.5 4-2 6-2 8a5 5 0 0 0 5 5Z"  />
        </svg>
    }
}

}
pub use r#wine::Wine;
mod r#menu {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Menu)]
pub fn r#menu(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="12" x2="20" x1="4" y1="12"  /><line y1="6" x2="20" x1="4" y2="6"  /><line x1="4" x2="20" y1="18" y2="18"  />
        </svg>
    }
}

}
pub use r#menu::Menu;
mod r#move_horizontal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MoveHorizontal)]
pub fn r#move_horizontal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="18 8 22 12 18 16"  /><polyline points="6 8 2 12 6 16"  /><line x1="2" x2="22" y2="12" y1="12"  />
        </svg>
    }
}

}
pub use r#move_horizontal::MoveHorizontal;
mod r#signal_low {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SignalLow)]
pub fn r#signal_low(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 20h.01"  /><path d="M7 20v-4"  />
        </svg>
    }
}

}
pub use r#signal_low::SignalLow;
mod r#align_justify {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignJustify)]
pub fn r#align_justify(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x2="21" y2="6" x1="3" y1="6"  /><line x2="21" y1="12" y2="12" x1="3"  /><line y2="18" x2="21" y1="18" x1="3"  />
        </svg>
    }
}

}
pub use r#align_justify::AlignJustify;
mod r#cigarette {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Cigarette)]
pub fn r#cigarette(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18 12H2v4h16"  /><path d="M22 12v4"  /><path d="M7 12v4"  /><path d="M18 8c0-2.5-2-2.5-2-5"  /><path d="M22 8c0-2.5-2-2.5-2-5"  />
        </svg>
    }
}

}
pub use r#cigarette::Cigarette;
mod r#cloud_lightning {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CloudLightning)]
pub fn r#cloud_lightning(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M6 16.326A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 .5 8.973"  /><path d="m13 12-3 5h4l-3 5"  />
        </svg>
    }
}

}
pub use r#cloud_lightning::CloudLightning;
mod r#mountain {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Mountain)]
pub fn r#mountain(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m8 3 4 8 5-5 5 15H2L8 3z"  />
        </svg>
    }
}

}
pub use r#mountain::Mountain;
mod r#clipboard_edit {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ClipboardEdit)]
pub fn r#clipboard_edit(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect ry="1" width="8" rx="1" height="4" x="8" y="2"  /><path d="M10.42 12.61a2.1 2.1 0 1 1 2.97 2.97L7.95 21 4 22l.99-3.95 5.43-5.44Z"  /><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-5.5"  /><path d="M4 13.5V6a2 2 0 0 1 2-2h2"  />
        </svg>
    }
}

}
pub use r#clipboard_edit::ClipboardEdit;
mod r#mouse_pointer_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MousePointer2)]
pub fn r#mouse_pointer_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m4 4 7.07 17 2.51-7.39L21 11.07z"  />
        </svg>
    }
}

}
pub use r#mouse_pointer_2::MousePointer2;
mod r#chevron_first {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ChevronFirst)]
pub fn r#chevron_first(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="17 18 11 12 17 6"  /><path d="M7 6v12"  />
        </svg>
    }
}

}
pub use r#chevron_first::ChevronFirst;
mod r#copyright {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Copyright)]
pub fn r#copyright(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" r="10" cy="12"  /><path d="M15 9.354a4 4 0 1 0 0 5.292"  />
        </svg>
    }
}

}
pub use r#copyright::Copyright;
mod r#laugh {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Laugh)]
pub fn r#laugh(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" cx="12" r="10"  /><path d="M18 13a6 6 0 0 1-6 5 6 6 0 0 1-6-5h12Z"  /><line y1="9" x2="9.01" y2="9" x1="9"  /><line y1="9" x1="15" y2="9" x2="15.01"  />
        </svg>
    }
}

}
pub use r#laugh::Laugh;
mod r#bot {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Bot)]
pub fn r#bot(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect rx="2" y="11" height="10" width="18" x="3"  /><circle cx="12" cy="5" r="2"  /><path d="M12 7v4"  /><line x2="8" y2="16" y1="16" x1="8"  /><line y1="16" x2="16" y2="16" x1="16"  />
        </svg>
    }
}

}
pub use r#bot::Bot;
mod r#music_4 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Music4)]
pub fn r#music_4(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9 18V5l12-2v13"  /><path d="m9 9 12-2"  /><circle cy="18" cx="6" r="3"  /><circle cy="16" cx="18" r="3"  />
        </svg>
    }
}

}
pub use r#music_4::Music4;
mod r#venetian_mask {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(VenetianMask)]
pub fn r#venetian_mask(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 12a5 5 0 0 0 5 5 8 8 0 0 1 5 2 8 8 0 0 1 5-2 5 5 0 0 0 5-5V7h-5a8 8 0 0 0-5 2 8 8 0 0 0-5-2H2Z"  /><path d="M6 11c1.5 0 3 .5 3 2-2 0-3 0-3-2Z"  /><path d="M18 11c-1.5 0-3 .5-3 2 2 0 3 0 3-2Z"  />
        </svg>
    }
}

}
pub use r#venetian_mask::VenetianMask;
mod r#package_x {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PackageX)]
pub fn r#package_x(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 10V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l2-1.14"  /><path d="M16.5 9.4 7.55 4.24"  /><polyline points="3.29 7 12 12 20.71 7"  /><line y1="22" x2="12" y2="12" x1="12"  /><path d="m17 13 5 5m-5 0 5-5"  />
        </svg>
    }
}

}
pub use r#package_x::PackageX;
mod r#annoyed {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Annoyed)]
pub fn r#annoyed(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" r="10" cy="12"  /><path d="M8 15h8"  /><path d="M8 9h2"  /><path d="M14 9h2"  />
        </svg>
    }
}

}
pub use r#annoyed::Annoyed;
mod r#file_plus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FilePlus)]
pub fn r#file_plus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><line x1="12" y2="12" x2="12" y1="18"  /><line x2="15" y1="15" y2="15" x1="9"  />
        </svg>
    }
}

}
pub use r#file_plus::FilePlus;
mod r#file_volume {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileVolume)]
pub fn r#file_volume(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 22h14a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v3"  /><polyline points="14 2 14 8 20 8"  /><path d="m7 10-3 2H2v4h2l3 2v-8Z"  /><path d="M11 11c.64.8 1 1.87 1 3s-.36 2.2-1 3"  />
        </svg>
    }
}

}
pub use r#file_volume::FileVolume;
mod r#axe {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Axe)]
pub fn r#axe(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m14 12-8.501 8.501a2.12 2.12 0 0 1-2.998 0h-.002a2.12 2.12 0 0 1 0-2.998L11 9.002"  /><path d="m9 7 4-4 6 6h3l-.13.648a7.648 7.648 0 0 1-5.081 5.756L15 16v-3z"  />
        </svg>
    }
}

}
pub use r#axe::Axe;
mod r#clock_5 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clock5)]
pub fn r#clock_5(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" r="10" cx="12"  /><polyline points="12 6 12 12 14.5 16"  />
        </svg>
    }
}

}
pub use r#clock_5::Clock5;
mod r#phone_missed {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PhoneMissed)]
pub fn r#phone_missed(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y1="2" y2="8" x2="16" x1="22"  /><line x2="22" x1="16" y2="8" y1="2"  /><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"  />
        </svg>
    }
}

}
pub use r#phone_missed::PhoneMissed;
mod r#sword {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Sword)]
pub fn r#sword(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="14.5 17.5 3 6 3 3 6 3 17.5 14.5"  /><line y1="19" x2="19" y2="13" x1="13"  /><line x2="20" y2="20" x1="16" y1="16"  /><line x1="19" x2="21" y1="21" y2="19"  />
        </svg>
    }
}

}
pub use r#sword::Sword;
mod r#upload_cloud {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(UploadCloud)]
pub fn r#upload_cloud(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242"  /><path d="M12 12v9"  /><path d="m16 16-4-4-4 4"  />
        </svg>
    }
}

}
pub use r#upload_cloud::UploadCloud;
mod r#pin_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PinOff)]
pub fn r#pin_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x1="2" y1="2" y2="22" x2="22"  /><line y2="22" x1="12" y1="17" x2="12"  /><path d="M9 9v1.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V17h12"  /><path d="M15 9.34V6h1a2 2 0 0 0 0-4H7.89"  />
        </svg>
    }
}

}
pub use r#pin_off::PinOff;
mod r#bell_ring {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BellRing)]
pub fn r#bell_ring(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"  /><path d="M13.73 21a2 2 0 0 1-3.46 0"  /><path d="M2 8c0-2.2.7-4.3 2-6"  /><path d="M22 8a10 10 0 0 0-2-6"  />
        </svg>
    }
}

}
pub use r#bell_ring::BellRing;
mod r#coins {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Coins)]
pub fn r#coins(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="8" r="6" cx="8"  /><path d="M18.09 10.37A6 6 0 1 1 10.34 18"  /><path d="M7 6h1v4"  /><path d="m16.71 13.88.7.71-2.82 2.82"  />
        </svg>
    }
}

}
pub use r#coins::Coins;
mod r#corner_left_up {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CornerLeftUp)]
pub fn r#corner_left_up(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="14 9 9 4 4 9"  /><path d="M20 20h-7a4 4 0 0 1-4-4V4"  />
        </svg>
    }
}

}
pub use r#corner_left_up::CornerLeftUp;
mod r#calendar_x_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CalendarX2)]
pub fn r#calendar_x_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 13V6a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h8"  /><line y1="2" x2="16" y2="6" x1="16"  /><line x2="8" x1="8" y1="2" y2="6"  /><line y2="10" x1="3" x2="21" y1="10"  /><line x2="22" y1="17" y2="22" x1="17"  /><line x2="22" y2="17" x1="17" y1="22"  />
        </svg>
    }
}

}
pub use r#calendar_x_2::CalendarX2;
mod r#folder_key {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderKey)]
pub fn r#folder_key(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10 20H4a2 2 0 0 1-2-2V5c0-1.1.9-2 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H20a2 2 0 0 1 2 2v2"  /><circle cx="16" cy="20" r="2"  /><path d="m22 14-4.5 4.5"  /><path d="m21 15 1 1"  />
        </svg>
    }
}

}
pub use r#folder_key::FolderKey;
mod r#mouse_pointer_click {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MousePointerClick)]
pub fn r#mouse_pointer_click(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m9 9 5 12 1.774-5.226L21 14 9 9z"  /><path d="m16.071 16.071 4.243 4.243"  /><path d="m7.188 2.239.777 2.897M5.136 7.965l-2.898-.777M13.95 4.05l-2.122 2.122m-5.657 5.656-2.12 2.122"  />
        </svg>
    }
}

}
pub use r#mouse_pointer_click::MousePointerClick;
mod r#bus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Bus)]
pub fn r#bus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M19 17h2l.64-2.54c.24-.959.24-1.962 0-2.92l-1.07-4.27A3 3 0 0 0 17.66 5H4a2 2 0 0 0-2 2v10h2"  /><path d="M14 17H9"  /><circle r="2.5" cx="6.5" cy="17.5"  /><circle cx="16.5" r="2.5" cy="17.5"  />
        </svg>
    }
}

}
pub use r#bus::Bus;
mod r#corner_right_up {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CornerRightUp)]
pub fn r#corner_right_up(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="10 9 15 4 20 9"  /><path d="M4 20h7a4 4 0 0 0 4-4V4"  />
        </svg>
    }
}

}
pub use r#corner_right_up::CornerRightUp;
mod r#map {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Map)]
pub fn r#map(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="3 6 9 3 15 6 21 3 21 18 15 21 9 18 3 21"  /><line x2="9" x1="9" y1="3" y2="18"  /><line y1="6" y2="21" x1="15" x2="15"  />
        </svg>
    }
}

}
pub use r#map::Map;
mod r#outdent {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Outdent)]
pub fn r#outdent(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="7 8 3 12 7 16"  /><line y1="12" x2="11" y2="12" x1="21"  /><line x2="11" x1="21" y2="6" y1="6"  /><line x2="11" y2="18" y1="18" x1="21"  />
        </svg>
    }
}

}
pub use r#outdent::Outdent;
mod r#scale_3_d {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Scale3D)]
pub fn r#scale_3_d(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M5 7v12h12"  /><path d="m5 19 6-6"  /><rect rx="1" y="3" width="4" x="3" height="4"  /><rect x="17" rx="1" y="17" height="4" width="4"  />
        </svg>
    }
}

}
pub use r#scale_3_d::Scale3D;
mod r#skull {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Skull)]
pub fn r#skull(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="1" cx="9" cy="12"  /><circle r="1" cy="12" cx="15"  /><path d="M8 20v2h8v-2"  /><path d="m12.5 17-.5-1-.5 1h1z"  /><path d="M16 20a2 2 0 0 0 1.56-3.25 8 8 0 1 0-11.12 0A2 2 0 0 0 8 20"  />
        </svg>
    }
}

}
pub use r#skull::Skull;
mod r#server {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Server)]
pub fn r#server(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect rx="2" ry="2" x="2" y="2" width="20" height="8"  /><rect x="2" y="14" rx="2" ry="2" width="20" height="8"  /><line y1="6" x2="6.01" x1="6" y2="6"  /><line x1="6" y1="18" y2="18" x2="6.01"  />
        </svg>
    }
}

}
pub use r#server::Server;
mod r#library {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Library)]
pub fn r#library(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m16 6 4 14"  /><path d="M12 6v14"  /><path d="M8 8v12"  /><path d="M4 4v16"  />
        </svg>
    }
}

}
pub use r#library::Library;
mod r#shrink {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Shrink)]
pub fn r#shrink(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m15 15 6 6m-6-6v4.8m0-4.8h4.8"  /><path d="M9 19.8V15m0 0H4.2M9 15l-6 6"  /><path d="M15 4.2V9m0 0h4.8M15 9l6-6"  /><path d="M9 4.2V9m0 0H4.2M9 9 3 3"  />
        </svg>
    }
}

}
pub use r#shrink::Shrink;
mod r#sunset {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Sunset)]
pub fn r#sunset(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 10V2"  /><path d="m4.93 10.93 1.41 1.41"  /><path d="M2 18h2"  /><path d="M20 18h2"  /><path d="m19.07 10.93-1.41 1.41"  /><path d="M22 22H2"  /><path d="m16 6-4 4-4-4"  /><path d="M16 18a4 4 0 0 0-8 0"  />
        </svg>
    }
}

}
pub use r#sunset::Sunset;
mod r#verified {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Verified)]
pub fn r#verified(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 3c-1.2 0-2.4.6-3 1.7A3.6 3.6 0 0 0 4.6 9c-1 .6-1.7 1.8-1.7 3s.7 2.4 1.7 3c-.3 1.2 0 2.5 1 3.4.8.8 2.1 1.2 3.3 1 .6 1 1.8 1.6 3 1.6s2.4-.6 3-1.7c1.2.3 2.5 0 3.4-1 .8-.8 1.2-2 1-3.3 1-.6 1.6-1.8 1.6-3s-.6-2.4-1.7-3c.3-1.2 0-2.5-1-3.4a3.7 3.7 0 0 0-3.3-1c-.6-1-1.8-1.6-3-1.6Z"  /><path d="m9 12 2 2 4-4"  />
        </svg>
    }
}

}
pub use r#verified::Verified;
mod r#clipboard_check {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ClipboardCheck)]
pub fn r#clipboard_check(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect rx="1" x="8" y="2" width="8" height="4" ry="1"  /><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"  /><path d="m9 14 2 2 4-4"  />
        </svg>
    }
}

}
pub use r#clipboard_check::ClipboardCheck;
mod r#git_commit {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(GitCommit)]
pub fn r#git_commit(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" r="3" cy="12"  /><line x2="9" y1="12" y2="12" x1="3"  /><line y1="12" x1="15" x2="21" y2="12"  />
        </svg>
    }
}

}
pub use r#git_commit::GitCommit;
mod r#quote {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Quote)]
pub fn r#quote(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 21c3 0 7-1 7-8V5c0-1.25-.756-2.017-2-2H4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z"  /><path d="M15 21c3 0 7-1 7-8V5c0-1.25-.757-2.017-2-2h-4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2h.75c0 2.25.25 4-2.75 4v3c0 1 0 1 1 1z"  />
        </svg>
    }
}

}
pub use r#quote::Quote;
mod r#bluetooth_connected {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BluetoothConnected)]
pub fn r#bluetooth_connected(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m7 7 10 10-5 5V2l5 5L7 17"  /><line x1="18" y1="12" y2="12" x2="21"  /><line y1="12" x1="3" y2="12" x2="6"  />
        </svg>
    }
}

}
pub use r#bluetooth_connected::BluetoothConnected;
mod r#music_3 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Music3)]
pub fn r#music_3(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="18" r="4"  /><path d="M16 18V2"  />
        </svg>
    }
}

}
pub use r#music_3::Music3;
mod r#utensils {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Utensils)]
pub fn r#utensils(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 2v7c0 1.1.9 2 2 2h4a2 2 0 0 0 2-2V2"  /><path d="M7 2v20"  /><path d="M21 15V2v0a5 5 0 0 0-5 5v6c0 1.1.9 2 2 2h3Zm0 0v7"  />
        </svg>
    }
}

}
pub use r#utensils::Utensils;
mod r#pause_octagon {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PauseOctagon)]
pub fn r#pause_octagon(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10 15V9"  /><path d="M14 15V9"  /><path d="M7.714 2h8.572L22 7.714v8.572L16.286 22H7.714L2 16.286V7.714L7.714 2z"  />
        </svg>
    }
}

}
pub use r#pause_octagon::PauseOctagon;
mod r#volume_1 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Volume1)]
pub fn r#volume_1(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"  /><path d="M15.54 8.46a5 5 0 0 1 0 7.07"  />
        </svg>
    }
}

}
pub use r#volume_1::Volume1;
mod r#option {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Option)]
pub fn r#option(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 3h6l6 18h6"  /><path d="M14 3h7"  />
        </svg>
    }
}

}
pub use r#option::Option;
mod r#expand {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Expand)]
pub fn r#expand(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m21 21-6-6m6 6v-4.8m0 4.8h-4.8"  /><path d="M3 16.2V21m0 0h4.8M3 21l6-6"  /><path d="M21 7.8V3m0 0h-4.8M21 3l-6 6"  /><path d="M3 7.8V3m0 0h4.8M3 3l6 6"  />
        </svg>
    }
}

}
pub use r#expand::Expand;
mod r#send {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Send)]
pub fn r#send(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="13" x1="22" y1="2" x2="11"  /><polygon points="22 2 15 22 11 13 2 9 22 2"  />
        </svg>
    }
}

}
pub use r#send::Send;
mod r#diamond {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Diamond)]
pub fn r#diamond(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect height="15.56" rx="2.41" transform="rotate(45 12 1)" x="12" width="15.56" y="1"  />
        </svg>
    }
}

}
pub use r#diamond::Diamond;
mod r#folder_x {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderX)]
pub fn r#folder_x(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"  /><path d="m9.5 10.5 5 5"  /><path d="m14.5 10.5-5 5"  />
        </svg>
    }
}

}
pub use r#folder_x::FolderX;
mod r#bed {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Bed)]
pub fn r#bed(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 4v16"  /><path d="M2 8h18a2 2 0 0 1 2 2v10"  /><path d="M2 17h20"  /><path d="M6 8v9"  />
        </svg>
    }
}

}
pub use r#bed::Bed;
mod r#italic {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Italic)]
pub fn r#italic(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x1="19" x2="10" y2="4" y1="4"  /><line x2="5" y1="20" y2="20" x1="14"  /><line y1="4" x2="9" x1="15" y2="20"  />
        </svg>
    }
}

}
pub use r#italic::Italic;
mod r#file_text {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileText)]
pub fn r#file_text(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><line x2="8" y1="13" x1="16" y2="13"  /><line x1="16" x2="8" y1="17" y2="17"  /><line x1="10" x2="8" y1="9" y2="9"  />
        </svg>
    }
}

}
pub use r#file_text::FileText;
mod r#arrow_right {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowRight)]
pub fn r#arrow_right(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x1="5" y1="12" y2="12" x2="19"  /><polyline points="12 5 19 12 12 19"  />
        </svg>
    }
}

}
pub use r#arrow_right::ArrowRight;
mod r#gift {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Gift)]
pub fn r#gift(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="20 12 20 22 4 22 4 12"  /><rect x="2" width="20" height="5" y="7"  /><line x1="12" y2="7" y1="22" x2="12"  /><path d="M12 7H7.5a2.5 2.5 0 0 1 0-5C11 2 12 7 12 7z"  /><path d="M12 7h4.5a2.5 2.5 0 0 0 0-5C13 2 12 7 12 7z"  />
        </svg>
    }
}

}
pub use r#gift::Gift;
mod r#hard_hat {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(HardHat)]
pub fn r#hard_hat(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 18a1 1 0 0 0 1 1h18a1 1 0 0 0 1-1v-2a1 1 0 0 0-1-1H3a1 1 0 0 0-1 1v2z"  /><path d="M10 10V5a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v5"  /><path d="M4 15v-3a6 6 0 0 1 6-6h0"  /><path d="M14 6h0a6 6 0 0 1 6 6v3"  />
        </svg>
    }
}

}
pub use r#hard_hat::HardHat;
mod r#align_vertical_distribute_center {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignVerticalDistributeCenter)]
pub fn r#align_vertical_distribute_center(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect rx="2" width="14" x="5" y="14" height="6"  /><rect x="7" y="4" width="10" rx="2" height="6"  /><path d="M22 7h-5"  /><path d="M7 7H1"  /><path d="M22 17h-3"  /><path d="M5 17H2"  />
        </svg>
    }
}

}
pub use r#align_vertical_distribute_center::AlignVerticalDistributeCenter;
mod r#arrow_big_left {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowBigLeft)]
pub fn r#arrow_big_left(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m3 12 7-7v4h11v6H10v4z"  />
        </svg>
    }
}

}
pub use r#arrow_big_left::ArrowBigLeft;
mod r#align_vertical_justify_center {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignVerticalJustifyCenter)]
pub fn r#align_vertical_justify_center(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect height="6" rx="2" y="16" width="14" x="5"  /><rect x="7" width="10" height="6" y="2" rx="2"  /><path d="M2 12h20"  />
        </svg>
    }
}

}
pub use r#align_vertical_justify_center::AlignVerticalJustifyCenter;
mod r#party_popper {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PartyPopper)]
pub fn r#party_popper(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M5.8 11.3 2 22l10.7-3.79"  /><path d="M4 3h.01"  /><path d="M22 8h.01"  /><path d="M15 2h.01"  /><path d="M22 20h.01"  /><path d="m22 2-2.24.75a2.9 2.9 0 0 0-1.96 3.12v0c.1.86-.57 1.63-1.45 1.63h-.38c-.86 0-1.6.6-1.76 1.44L14 10"  /><path d="m22 13-.82-.33c-.86-.34-1.82.2-1.98 1.11v0c-.11.7-.72 1.22-1.43 1.22H17"  /><path d="m11 2 .33.82c.34.86-.2 1.82-1.11 1.98v0C9.52 4.9 9 5.52 9 6.23V7"  /><path d="M11 13c1.93 1.93 2.83 4.17 2 5-.83.83-3.07-.07-5-2-1.93-1.93-2.83-4.17-2-5 .83-.83 3.07.07 5 2Z"  />
        </svg>
    }
}

}
pub use r#party_popper::PartyPopper;
mod r#align_horizontal_justify_start {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignHorizontalJustifyStart)]
pub fn r#align_horizontal_justify_start(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="6" y="5" rx="2" height="14" width="6"  /><rect width="6" x="16" y="7" rx="2" height="10"  /><path d="M2 2v20"  />
        </svg>
    }
}

}
pub use r#align_horizontal_justify_start::AlignHorizontalJustifyStart;
mod r#redo_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Redo2)]
pub fn r#redo_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m15 14 5-5-5-5"  /><path d="M20 9H9.5A5.5 5.5 0 0 0 4 14.5v0A5.5 5.5 0 0 0 9.5 20H13"  />
        </svg>
    }
}

}
pub use r#redo_2::Redo2;
mod r#fingerprint {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Fingerprint)]
pub fn r#fingerprint(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 12C2 6.5 6.5 2 12 2a10 10 0 0 1 8 4"  /><path d="M5 19.5C5.5 18 6 15 6 12c0-.7.12-1.37.34-2"  /><path d="M17.29 21.02c.12-.6.43-2.3.5-3.02"  /><path d="M12 10a2 2 0 0 0-2 2c0 1.02-.1 2.51-.26 4"  /><path d="M8.65 22c.21-.66.45-1.32.57-2"  /><path d="M14 13.12c0 2.38 0 6.38-1 8.88"  /><path d="M2 16h.01"  /><path d="M21.8 16c.2-2 .131-5.354 0-6"  /><path d="M9 6.8a6 6 0 0 1 9 5.2c0 .47 0 1.17-.02 2"  />
        </svg>
    }
}

}
pub use r#fingerprint::Fingerprint;
mod r#file_edit {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileEdit)]
pub fn r#file_edit(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 13.5V4a2 2 0 0 1 2-2h8.5L20 7.5V20a2 2 0 0 1-2 2h-5.5"  /><polyline points="14 2 14 8 20 8"  /><path d="M10.42 12.61a2.1 2.1 0 1 1 2.97 2.97L7.95 21 4 22l.99-3.95 5.43-5.44Z"  />
        </svg>
    }
}

}
pub use r#file_edit::FileEdit;
mod r#target {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Target)]
pub fn r#target(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" r="10" cx="12"  /><circle r="6" cy="12" cx="12"  /><circle cx="12" cy="12" r="2"  />
        </svg>
    }
}

}
pub use r#target::Target;
mod r#rocking_chair {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(RockingChair)]
pub fn r#rocking_chair(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="3.5 2 6.5 12.5 18 12.5"  /><line x1="9.5" x2="5.5" y1="12.5" y2="20"  /><line y2="20" x2="18.5" x1="15" y1="12.5"  /><path d="M2.75 18a13 13 0 0 0 18.5 0"  />
        </svg>
    }
}

}
pub use r#rocking_chair::RockingChair;
mod r#star {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Star)]
pub fn r#star(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"  />
        </svg>
    }
}

}
pub use r#star::Star;
mod r#corner_down_left {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CornerDownLeft)]
pub fn r#corner_down_left(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="9 10 4 15 9 20"  /><path d="M20 4v7a4 4 0 0 1-4 4H4"  />
        </svg>
    }
}

}
pub use r#corner_down_left::CornerDownLeft;
mod r#edit_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Edit2)]
pub fn r#edit_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"  />
        </svg>
    }
}

}
pub use r#edit_2::Edit2;
mod r#file_signature {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileSignature)]
pub fn r#file_signature(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20 19.5v.5a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8.5L18 5.5"  /><path d="M8 18h1"  /><path d="M18.42 9.61a2.1 2.1 0 1 1 2.97 2.97L16.95 17 13 18l.99-3.95 4.43-4.44Z"  />
        </svg>
    }
}

}
pub use r#file_signature::FileSignature;
mod r#alarm_clock {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlarmClock)]
pub fn r#alarm_clock(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="13" r="8"  /><path d="M12 9v4l2 2"  /><path d="M5 3 2 6"  /><path d="m22 6-3-3"  /><path d="m6 19-2 2"  /><path d="m18 19 2 2"  />
        </svg>
    }
}

}
pub use r#alarm_clock::AlarmClock;
mod r#file_check_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileCheck2)]
pub fn r#file_check_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 22h14a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v4"  /><polyline points="14 2 14 8 20 8"  /><path d="m3 15 2 2 4-4"  />
        </svg>
    }
}

}
pub use r#file_check_2::FileCheck2;
mod r#backpack {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Backpack)]
pub fn r#backpack(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 20V10a4 4 0 0 1 4-4h8a4 4 0 0 1 4 4v10a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2Z"  /><path d="M9 6V4a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v2"  /><path d="M8 21v-5a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v5"  /><path d="M8 10h8"  /><path d="M8 18h8"  />
        </svg>
    }
}

}
pub use r#backpack::Backpack;
mod r#locate_fixed {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LocateFixed)]
pub fn r#locate_fixed(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y1="12" x2="5" y2="12" x1="2"  /><line y1="12" x2="22" x1="19" y2="12"  /><line y1="2" y2="5" x1="12" x2="12"  /><line x1="12" y1="19" y2="22" x2="12"  /><circle cx="12" cy="12" r="7"  /><circle r="3" cx="12" cy="12"  />
        </svg>
    }
}

}
pub use r#locate_fixed::LocateFixed;
mod r#arrow_big_down {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowBigDown)]
pub fn r#arrow_big_down(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9 3h6v11h4l-7 7-7-7h4z"  />
        </svg>
    }
}

}
pub use r#arrow_big_down::ArrowBigDown;
mod r#chevron_up {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ChevronUp)]
pub fn r#chevron_up(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="18 15 12 9 6 15"  />
        </svg>
    }
}

}
pub use r#chevron_up::ChevronUp;
mod r#flame {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Flame)]
pub fn r#flame(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z"  />
        </svg>
    }
}

}
pub use r#flame::Flame;
mod r#file_up {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileUp)]
pub fn r#file_up(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><path d="M12 12v6"  /><path d="m15 15-3-3-3 3"  />
        </svg>
    }
}

}
pub use r#file_up::FileUp;
mod r#arrow_up_circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowUpCircle)]
pub fn r#arrow_up_circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="12" r="10"  /><polyline points="16 12 12 8 8 12"  /><line x1="12" x2="12" y1="16" y2="8"  />
        </svg>
    }
}

}
pub use r#arrow_up_circle::ArrowUpCircle;
mod r#cloud_cog {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CloudCog)]
pub fn r#cloud_cog(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20 16.2A4.5 4.5 0 0 0 17.5 8h-1.8A7 7 0 1 0 4 14.9"  /><circle r="3" cx="12" cy="17"  /><path d="M12 13v1"  /><path d="M12 20v1"  /><path d="M16 17h-1"  /><path d="M9 17H8"  /><path d="m15 14-.88.88"  /><path d="M9.88 19.12 9 20"  /><path d="m15 20-.88-.88"  /><path d="M9.88 14.88 9 14"  />
        </svg>
    }
}

}
pub use r#cloud_cog::CloudCog;
mod r#filter {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Filter)]
pub fn r#filter(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"  />
        </svg>
    }
}

}
pub use r#filter::Filter;
mod r#glasses {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Glasses)]
pub fn r#glasses(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="4" cy="15" cx="6"  /><circle r="4" cy="15" cx="18"  /><path d="M14 15a2 2 0 0 0-2-2 2 2 0 0 0-2 2"  /><path d="M2.5 13 5 7c.7-1.3 1.4-2 3-2"  /><path d="M21.5 13 19 7c-.7-1.3-1.5-2-3-2"  />
        </svg>
    }
}

}
pub use r#glasses::Glasses;
mod r#inbox {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Inbox)]
pub fn r#inbox(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="22 12 16 12 14 15 10 15 8 12 2 12"  /><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"  />
        </svg>
    }
}

}
pub use r#inbox::Inbox;
mod r#paint_bucket {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PaintBucket)]
pub fn r#paint_bucket(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m19 11-8-8-8.6 8.6a2 2 0 0 0 0 2.8l5.2 5.2c.8.8 2 .8 2.8 0L19 11Z"  /><path d="m5 2 5 5"  /><path d="M2 13h15"  /><path d="M22 20a2 2 0 1 1-4 0c0-1.6 1.7-2.4 2-4 .3 1.6 2 2.4 2 4Z"  />
        </svg>
    }
}

}
pub use r#paint_bucket::PaintBucket;
mod r#mouse_pointer {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MousePointer)]
pub fn r#mouse_pointer(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m3 3 7.07 16.97 2.51-7.39 7.39-2.51L3 3z"  /><path d="m13 13 6 6"  />
        </svg>
    }
}

}
pub use r#mouse_pointer::MousePointer;
mod r#stretch_vertical {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(StretchVertical)]
pub fn r#stretch_vertical(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="4" y="2" rx="2" width="6" height="20"  /><rect y="2" width="6" height="20" rx="2" x="14"  />
        </svg>
    }
}

}
pub use r#stretch_vertical::StretchVertical;
mod r#air_vent {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AirVent)]
pub fn r#air_vent(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M6 12H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"  /><path d="M6 8h12"  /><path d="M18.3 17.7a2.5 2.5 0 0 1-3.16 3.83 2.53 2.53 0 0 1-1.14-2V12"  /><path d="M6.6 15.6A2 2 0 1 0 10 17v-5"  />
        </svg>
    }
}

}
pub use r#air_vent::AirVent;
mod r#euro {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Euro)]
pub fn r#euro(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 10h12"  /><path d="M4 14h9"  /><path d="M19 6a7.7 7.7 0 0 0-5.2-2A7.9 7.9 0 0 0 6 12c0 4.4 3.5 8 7.8 8 2 0 3.8-.8 5.2-2"  />
        </svg>
    }
}

}
pub use r#euro::Euro;
mod r#delete {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Delete)]
pub fn r#delete(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20 5H9l-7 7 7 7h11a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2Z"  /><line x1="18" y1="9" x2="12" y2="15"  /><line y1="9" x2="18" y2="15" x1="12"  />
        </svg>
    }
}

}
pub use r#delete::Delete;
mod r#clock_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clock2)]
pub fn r#clock_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cx="12" cy="12"  /><polyline points="12 6 12 12 16 10"  />
        </svg>
    }
}

}
pub use r#clock_2::Clock2;
mod r#droplet {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Droplet)]
pub fn r#droplet(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 22a7 7 0 0 0 7-7c0-2-1-3.9-3-5.5s-3.5-4-4-6.5c-.5 2.5-2 4.9-4 6.5C6 11.1 5 13 5 15a7 7 0 0 0 7 7z"  />
        </svg>
    }
}

}
pub use r#droplet::Droplet;
mod r#map_pin {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MapPin)]
pub fn r#map_pin(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z"  /><circle cy="10" cx="12" r="3"  />
        </svg>
    }
}

}
pub use r#map_pin::MapPin;
mod r#milestone {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Milestone)]
pub fn r#milestone(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18 6H5a2 2 0 0 0-2 2v3a2 2 0 0 0 2 2h13l4-3.5L18 6Z"  /><path d="M12 13v9"  /><path d="M12 2v4"  />
        </svg>
    }
}

}
pub use r#milestone::Milestone;
mod r#palette {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Palette)]
pub fn r#palette(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="6.5" cx="13.5" r=".5"  /><circle r=".5" cx="17.5" cy="10.5"  /><circle cy="7.5" r=".5" cx="8.5"  /><circle cy="12.5" r=".5" cx="6.5"  /><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"  />
        </svg>
    }
}

}
pub use r#palette::Palette;
mod r#building_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Building2)]
pub fn r#building_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M6 22V4c0-.27 0-.55.07-.82a1.477 1.477 0 0 1 1.1-1.11C7.46 2 8.73 2 9 2h7c.27 0 .55 0 .82.07a1.477 1.477 0 0 1 1.11 1.1c.07.28.07.56.07.83v18H6Z"  /><path d="M2 14v6c0 1.1.9 2 2 2h2V12H4c-.27 0-.55 0-.82.07-.27.07-.52.2-.72.4-.19.19-.32.44-.39.71A3.4 3.4 0 0 0 2 14Z"  /><path d="M20.82 9.07A3.4 3.4 0 0 0 20 9h-2v13h2a2 2 0 0 0 2-2v-9c0-.28 0-.55-.07-.82-.07-.27-.2-.52-.4-.72-.19-.19-.44-.32-.71-.39Z"  /><path d="M10 6h4"  /><path d="M10 10h4"  /><path d="M10 14h4"  /><path d="M10 18h4"  />
        </svg>
    }
}

}
pub use r#building_2::Building2;
mod r#repeat {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Repeat)]
pub fn r#repeat(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m17 2 4 4-4 4"  /><path d="M3 11v-1a4 4 0 0 1 4-4h14"  /><path d="m7 22-4-4 4-4"  /><path d="M21 13v1a4 4 0 0 1-4 4H3"  />
        </svg>
    }
}

}
pub use r#repeat::Repeat;
mod r#crop {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Crop)]
pub fn r#crop(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M6 2v14a2 2 0 0 0 2 2h14"  /><path d="M18 22V8a2 2 0 0 0-2-2H2"  />
        </svg>
    }
}

}
pub use r#crop::Crop;
mod r#equal_not {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(EqualNot)]
pub fn r#equal_not(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x1="5" x2="19" y2="9" y1="9"  /><line x1="5" x2="19" y2="15" y1="15"  /><line x1="19" y1="5" y2="19" x2="5"  />
        </svg>
    }
}

}
pub use r#equal_not::EqualNot;
mod r#shield_check {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ShieldCheck)]
pub fn r#shield_check(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"  /><path d="m9 12 2 2 4-4"  />
        </svg>
    }
}

}
pub use r#shield_check::ShieldCheck;
mod r#clipboard_signature {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ClipboardSignature)]
pub fn r#clipboard_signature(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="2" height="4" x="8" rx="1" width="8" ry="1"  /><path d="M8 4H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-.5"  /><path d="M16 4h2a2 2 0 0 1 1.73 1"  /><path d="M18.42 9.61a2.1 2.1 0 1 1 2.97 2.97L16.95 17 13 18l.99-3.95 4.43-4.44Z"  /><path d="M8 18h1"  />
        </svg>
    }
}

}
pub use r#clipboard_signature::ClipboardSignature;
mod r#paperclip {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Paperclip)]
pub fn r#paperclip(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m21.44 11.05-9.19 9.19a6 6 0 0 1-8.49-8.49l8.57-8.57A4 4 0 1 1 18 8.84l-8.59 8.57a2 2 0 0 1-2.83-2.83l8.49-8.48"  />
        </svg>
    }
}

}
pub use r#paperclip::Paperclip;
mod r#folder_clock {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderClock)]
pub fn r#folder_clock(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M7 20H4a2 2 0 0 1-2-2V5c0-1.1.9-2 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H20a2 2 0 0 1 2 2"  /><circle r="6" cx="16" cy="16"  /><path d="M16 14v2l1 1"  />
        </svg>
    }
}

}
pub use r#folder_clock::FolderClock;
mod r#clock_4 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clock4)]
pub fn r#clock_4(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cy="12" cx="12"  /><polyline points="12 6 12 12 16 14"  />
        </svg>
    }
}

}
pub use r#clock_4::Clock4;
mod r#message_square {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MessageSquare)]
pub fn r#message_square(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"  />
        </svg>
    }
}

}
pub use r#message_square::MessageSquare;
mod r#clock_6 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clock6)]
pub fn r#clock_6(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cy="12" cx="12"  /><polyline points="12 6 12 12 12 16.5"  />
        </svg>
    }
}

}
pub use r#clock_6::Clock6;
mod r#croissant {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Croissant)]
pub fn r#croissant(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m4.6 13.11 5.79-3.21c1.89-1.05 4.79 1.78 3.71 3.71l-3.22 5.81C8.8 23.16.79 15.23 4.6 13.11Z"  /><path d="m10.5 9.5-1-2.29C9.2 6.48 8.8 6 8 6H4.5C2.79 6 2 6.5 2 8.5a7.71 7.71 0 0 0 2 4.83"  /><path d="M8 6c0-1.55.24-4-2-4-2 0-2.5 2.17-2.5 4"  /><path d="m14.5 13.5 2.29 1c.73.3 1.21.7 1.21 1.5v3.5c0 1.71-.5 2.5-2.5 2.5a7.71 7.71 0 0 1-4.83-2"  /><path d="M18 16c1.55 0 4-.24 4 2 0 2-2.17 2.5-4 2.5"  />
        </svg>
    }
}

}
pub use r#croissant::Croissant;
mod r#flag_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FlagOff)]
pub fn r#flag_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M8 2c3 0 5 2 8 2s4-1 4-1v11"  /><path d="M4 22V4"  /><path d="M4 15s1-1 4-1 5 2 8 2"  /><line y1="2" y2="22" x1="2" x2="22"  />
        </svg>
    }
}

}
pub use r#flag_off::FlagOff;
mod r#copy {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Copy)]
pub fn r#copy(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="9" y="9" rx="2" ry="2" width="13" height="13"  /><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"  />
        </svg>
    }
}

}
pub use r#copy::Copy;
mod r#pause_circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PauseCircle)]
pub fn r#pause_circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cy="12" cx="12"  /><line y1="15" y2="9" x2="10" x1="10"  /><line x2="14" x1="14" y1="15" y2="9"  />
        </svg>
    }
}

}
pub use r#pause_circle::PauseCircle;
mod r#slice {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Slice)]
pub fn r#slice(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m8 14-6 6h9v-3"  /><path d="M18.37 3.63 8 14l3 3L21.37 6.63a2.12 2.12 0 1 0-3-3Z"  />
        </svg>
    }
}

}
pub use r#slice::Slice;
mod r#toggle_left {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ToggleLeft)]
pub fn r#toggle_left(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="1" width="22" rx="7" ry="7" y="5" height="14"  /><circle cy="12" cx="8" r="3"  />
        </svg>
    }
}

}
pub use r#toggle_left::ToggleLeft;
mod r#radio {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Radio)]
pub fn r#radio(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="2" cy="12" cx="12"  /><path d="M4.93 19.07a10 10 0 0 1 0-14.14"  /><path d="M7.76 16.24a6 6 0 0 1-1.3-1.95 6 6 0 0 1 0-4.59 6 6 0 0 1 1.3-1.95"  /><path d="M16.24 7.76a6 6 0 0 1 1.3 2 6 6 0 0 1 0 4.59 6 6 0 0 1-1.3 1.95"  /><path d="M19.07 4.93a10 10 0 0 1 0 14.14"  />
        </svg>
    }
}

}
pub use r#radio::Radio;
mod r#plug_zap {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PlugZap)]
pub fn r#plug_zap(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m13 2-2 2.5h3L12 7"  /><path d="M12 22v-3"  /><path d="M10 13v-2.5"  /><path d="M10 12.5v-2"  /><path d="M14 12.5v-2"  /><path d="M16 15a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v2a2 2 0 0 0 2 2h4a2 2 0 0 0 2-2v-2z"  />
        </svg>
    }
}

}
pub use r#plug_zap::PlugZap;
mod r#indian_rupee {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(IndianRupee)]
pub fn r#indian_rupee(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M6 3h12"  /><path d="M6 8h12"  /><path d="m6 13 8.5 8"  /><path d="M6 13h3"  /><path d="M9 13c6.667 0 6.667-10 0-10"  />
        </svg>
    }
}

}
pub use r#indian_rupee::IndianRupee;
mod r#calendar_check_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CalendarCheck2)]
pub fn r#calendar_check_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 14V6a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h8"  /><line y2="6" x1="16" y1="2" x2="16"  /><line y1="2" x1="8" x2="8" y2="6"  /><line x1="3" y2="10" y1="10" x2="21"  /><path d="m16 20 2 2 4-4"  />
        </svg>
    }
}

}
pub use r#calendar_check_2::CalendarCheck2;
mod r#gamepad {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Gamepad)]
pub fn r#gamepad(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x1="6" y2="12" x2="10" y1="12"  /><line x2="8" y2="14" x1="8" y1="10"  /><line y1="13" x2="15.01" y2="13" x1="15"  /><line y2="11" x1="18" x2="18.01" y1="11"  /><rect x="2" y="6" width="20" height="12" rx="2"  />
        </svg>
    }
}

}
pub use r#gamepad::Gamepad;
mod r#apple {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Apple)]
pub fn r#apple(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 20.94c1.5 0 2.75 1.06 4 1.06 3 0 6-8 6-12.22A4.91 4.91 0 0 0 17 5c-2.22 0-4 1.44-5 2-1-.56-2.78-2-5-2a4.9 4.9 0 0 0-5 4.78C2 14 5 22 8 22c1.25 0 2.5-1.06 4-1.06Z"  /><path d="M10 2c1 .5 2 2 2 5"  />
        </svg>
    }
}

}
pub use r#apple::Apple;
mod r#cpu {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Cpu)]
pub fn r#cpu(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect ry="2" height="16" x="4" width="16" y="4" rx="2"  /><rect y="9" width="6" x="9" height="6"  /><line y2="4" y1="2" x2="9" x1="9"  /><line x2="15" y2="4" y1="2" x1="15"  /><line x1="9" y1="21" y2="22" x2="9"  /><line x2="15" x1="15" y1="20" y2="22"  /><line y1="9" x2="22" y2="9" x1="20"  /><line x2="22" x1="20" y1="14" y2="14"  /><line y2="9" x2="4" x1="2" y1="9"  /><line x1="2" y1="14" x2="4" y2="14"  />
        </svg>
    }
}

}
pub use r#cpu::Cpu;
mod r#dices {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Dices)]
pub fn r#dices(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect width="12" y="10" ry="2" rx="2" x="2" height="12"  /><path d="m17.92 14 3.5-3.5a2.24 2.24 0 0 0 0-3l-5-4.92a2.24 2.24 0 0 0-3 0L10 6"  /><path d="M6 18h.01"  /><path d="M10 14h.01"  /><path d="M15 6h.01"  /><path d="M18 9h.01"  />
        </svg>
    }
}

}
pub use r#dices::Dices;
mod r#align_vertical_space_around {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignVerticalSpaceAround)]
pub fn r#align_vertical_space_around(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="7" width="10" y="9" height="6" rx="2"  /><path d="M22 20H2"  /><path d="M22 4H2"  />
        </svg>
    }
}

}
pub use r#align_vertical_space_around::AlignVerticalSpaceAround;
mod r#crown {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Crown)]
pub fn r#crown(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m2 4 3 12h14l3-12-6 7-4-7-4 7-6-7zm3 16h14"  />
        </svg>
    }
}

}
pub use r#crown::Crown;
mod r#git_branch_plus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(GitBranchPlus)]
pub fn r#git_branch_plus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M6 3v12"  /><path d="M18 9a3 3 0 1 0 0-6 3 3 0 0 0 0 6z"  /><path d="M6 21a3 3 0 1 0 0-6 3 3 0 0 0 0 6z"  /><path d="M15 6a9 9 0 0 0-9 9"  /><path d="M18 15v6"  /><path d="M21 18h-6"  />
        </svg>
    }
}

}
pub use r#git_branch_plus::GitBranchPlus;
mod r#align_horizontal_distribute_end {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignHorizontalDistributeEnd)]
pub fn r#align_horizontal_distribute_end(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="5" rx="2" x="4" width="6" height="14"  /><rect x="14" y="7" rx="2" height="10" width="6"  /><path d="M10 2v20"  /><path d="M20 2v20"  />
        </svg>
    }
}

}
pub use r#align_horizontal_distribute_end::AlignHorizontalDistributeEnd;
mod r#list_x {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ListX)]
pub fn r#list_x(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M11 12H3"  /><path d="M16 6H3"  /><path d="M16 18H3"  /><path d="m19 10-4 4"  /><path d="m15 10 4 4"  />
        </svg>
    }
}

}
pub use r#list_x::ListX;
mod r#luggage {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Luggage)]
pub fn r#luggage(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M6 20h0a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2h0"  /><path d="M8 18V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v14"  /><path d="M10 20h4"  /><circle cy="20" cx="16" r="2"  /><circle cx="8" cy="20" r="2"  />
        </svg>
    }
}

}
pub use r#luggage::Luggage;
mod r#sigma {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Sigma)]
pub fn r#sigma(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18 7V4H6l6 8-6 8h12v-3"  />
        </svg>
    }
}

}
pub use r#sigma::Sigma;
mod r#redo {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Redo)]
pub fn r#redo(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 7v6h-6"  /><path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3l3 2.7"  />
        </svg>
    }
}

}
pub use r#redo::Redo;
mod r#skip_back {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SkipBack)]
pub fn r#skip_back(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="19 20 9 12 19 4 19 20"  /><line x2="5" y1="19" x1="5" y2="5"  />
        </svg>
    }
}

}
pub use r#skip_back::SkipBack;
mod r#moon {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Moon)]
pub fn r#moon(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 3a6.364 6.364 0 0 0 9 9 9 9 0 1 1-9-9Z"  />
        </svg>
    }
}

}
pub use r#moon::Moon;
mod r#anchor {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Anchor)]
pub fn r#anchor(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="3" cx="12" cy="5"  /><line y1="22" y2="8" x1="12" x2="12"  /><path d="M5 12H2a10 10 0 0 0 20 0h-3"  />
        </svg>
    }
}

}
pub use r#anchor::Anchor;
mod r#circle_ellipsis {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CircleEllipsis)]
pub fn r#circle_ellipsis(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cx="12" cy="12"  /><path d="M17 12h.01"  /><path d="M12 12h.01"  /><path d="M7 12h.01"  />
        </svg>
    }
}

}
pub use r#circle_ellipsis::CircleEllipsis;
mod r#file_heart {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileHeart)]
pub fn r#file_heart(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 6V4a2 2 0 0 1 2-2h8.5L20 7.5V20a2 2 0 0 1-2 2H4"  /><polyline points="14 2 14 8 20 8"  /><path d="M10.29 10.7a2.43 2.43 0 0 0-2.66-.52c-.29.12-.56.3-.78.53l-.35.34-.35-.34a2.43 2.43 0 0 0-2.65-.53c-.3.12-.56.3-.79.53-.95.94-1 2.53.2 3.74L6.5 18l3.6-3.55c1.2-1.21 1.14-2.8.19-3.74Z"  />
        </svg>
    }
}

}
pub use r#file_heart::FileHeart;
mod r#dice_1 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Dice1)]
pub fn r#dice_1(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect rx="2" width="18" height="18" ry="2" x="3" y="3"  /><path d="M12 12h.01"  />
        </svg>
    }
}

}
pub use r#dice_1::Dice1;
mod r#file_minus_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileMinus2)]
pub fn r#file_minus_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 22h14a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v4"  /><polyline points="14 2 14 8 20 8"  /><path d="M3 15h6"  />
        </svg>
    }
}

}
pub use r#file_minus_2::FileMinus2;
mod r#flask_round {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FlaskRound)]
pub fn r#flask_round(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10 2v7.31"  /><path d="M14 9.3V1.99"  /><path d="M8.5 2h7"  /><path d="M14 9.3a6.5 6.5 0 1 1-4 0"  /><path d="M5.58 16.5h12.85"  />
        </svg>
    }
}

}
pub use r#flask_round::FlaskRound;
mod r#forward {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Forward)]
pub fn r#forward(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="15 17 20 12 15 7"  /><path d="M4 18v-2a4 4 0 0 1 4-4h12"  />
        </svg>
    }
}

}
pub use r#forward::Forward;
mod r#glass_water {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(GlassWater)]
pub fn r#glass_water(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M15.2 22H8.8a2 2 0 0 1-2-1.79L5 3h14l-1.81 17.21A2 2 0 0 1 15.2 22Z"  /><path d="M6 12a5 5 0 0 1 6 0 5 5 0 0 0 6 0"  />
        </svg>
    }
}

}
pub use r#glass_water::GlassWater;
mod r#phone_incoming {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PhoneIncoming)]
pub fn r#phone_incoming(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="16 2 16 8 22 8"  /><line y1="2" y2="8" x2="16" x1="22"  /><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"  />
        </svg>
    }
}

}
pub use r#phone_incoming::PhoneIncoming;
mod r#calendar_days {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CalendarDays)]
pub fn r#calendar_days(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect rx="2" ry="2" x="3" y="4" width="18" height="18"  /><line x1="16" y1="2" x2="16" y2="6"  /><line x1="8" y1="2" x2="8" y2="6"  /><line x1="3" x2="21" y2="10" y1="10"  /><path d="M8 14h.01"  /><path d="M12 14h.01"  /><path d="M16 14h.01"  /><path d="M8 18h.01"  /><path d="M12 18h.01"  /><path d="M16 18h.01"  />
        </svg>
    }
}

}
pub use r#calendar_days::CalendarDays;
mod r#save {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Save)]
pub fn r#save(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"  /><polyline points="17 21 17 13 7 13 7 21"  /><polyline points="7 3 7 8 15 8"  />
        </svg>
    }
}

}
pub use r#save::Save;
mod r#smile {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Smile)]
pub fn r#smile(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="12" r="10"  /><path d="M8 14s1.5 2 4 2 4-2 4-2"  /><line y2="9" y1="9" x2="9.01" x1="9"  /><line y2="9" y1="9" x1="15" x2="15.01"  />
        </svg>
    }
}

}
pub use r#smile::Smile;
mod r#star_half {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(StarHalf)]
pub fn r#star_half(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 17.8 5.8 21 7 14.1 2 9.3l7-1L12 2"  />
        </svg>
    }
}

}
pub use r#star_half::StarHalf;
mod r#volume {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Volume)]
pub fn r#volume(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"  />
        </svg>
    }
}

}
pub use r#volume::Volume;
mod r#clipboard {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clipboard)]
pub fn r#clipboard(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="2" rx="1" width="8" height="4" x="8" ry="1"  /><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"  />
        </svg>
    }
}

}
pub use r#clipboard::Clipboard;
mod r#x_circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(XCircle)]
pub fn r#x_circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="12" r="10"  /><line x1="15" y1="9" y2="15" x2="9"  /><line y2="15" x1="9" x2="15" y1="9"  />
        </svg>
    }
}

}
pub use r#x_circle::XCircle;
mod r#alert_triangle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlertTriangle)]
pub fn r#alert_triangle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"  /><line x1="12" x2="12" y2="13" y1="9"  /><line x1="12" y1="17" x2="12.01" y2="17"  />
        </svg>
    }
}

}
pub use r#alert_triangle::AlertTriangle;
mod r#bath {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Bath)]
pub fn r#bath(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9 6 6.5 3.5a1.5 1.5 0 0 0-1-.5C4.683 3 4 3.683 4 4.5V17a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-5"  /><line y2="7" x1="10" x2="8" y1="5"  /><line x1="2" x2="22" y2="12" y1="12"  /><line y2="21" x1="7" x2="7" y1="19"  /><line x2="17" x1="17" y1="19" y2="21"  />
        </svg>
    }
}

}
pub use r#bath::Bath;
mod r#move_vertical {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MoveVertical)]
pub fn r#move_vertical(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="8 18 12 22 16 18"  /><polyline points="8 6 12 2 16 6"  /><line x2="12" y2="22" x1="12" y1="2"  />
        </svg>
    }
}

}
pub use r#move_vertical::MoveVertical;
mod r#screen_share_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ScreenShareOff)]
pub fn r#screen_share_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M13 3H4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-3"  /><path d="M8 21h8"  /><path d="M12 17v4"  /><path d="m22 3-5 5"  /><path d="m17 3 5 5"  />
        </svg>
    }
}

}
pub use r#screen_share_off::ScreenShareOff;
mod r#terminal_square {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(TerminalSquare)]
pub fn r#terminal_square(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m7 11 2-2-2-2"  /><path d="M11 13h4"  /><rect ry="2" x="3" width="18" y="3" height="18" rx="2"  />
        </svg>
    }
}

}
pub use r#terminal_square::TerminalSquare;
mod r#pin {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Pin)]
pub fn r#pin(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y1="17" x1="12" y2="22" x2="12"  /><path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"  />
        </svg>
    }
}

}
pub use r#pin::Pin;
mod r#square {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Square)]
pub fn r#square(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="3" y="3" rx="2" height="18" ry="2" width="18"  />
        </svg>
    }
}

}
pub use r#square::Square;
mod r#flower {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Flower)]
pub fn r#flower(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 7.5a4.5 4.5 0 1 1 4.5 4.5M12 7.5A4.5 4.5 0 1 0 7.5 12M12 7.5V9m-4.5 3a4.5 4.5 0 1 0 4.5 4.5M7.5 12H9m7.5 0a4.5 4.5 0 1 1-4.5 4.5m4.5-4.5H15m-3 4.5V15"  /><circle cy="12" cx="12" r="3"  /><path d="m8 16 1.5-1.5"  /><path d="M14.5 9.5 16 8"  /><path d="m8 8 1.5 1.5"  /><path d="M14.5 14.5 16 16"  />
        </svg>
    }
}

}
pub use r#flower::Flower;
mod r#percent {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Percent)]
pub fn r#percent(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="19" y1="5" x1="19" x2="5"  /><circle cy="6.5" cx="6.5" r="2.5"  /><circle cx="17.5" r="2.5" cy="17.5"  />
        </svg>
    }
}

}
pub use r#percent::Percent;
mod r#wifi {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Wifi)]
pub fn r#wifi(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M5 13a10 10 0 0 1 14 0"  /><path d="M8.5 16.5a5 5 0 0 1 7 0"  /><path d="M2 8.82a15 15 0 0 1 20 0"  /><line y1="20" x1="12" x2="12.01" y2="20"  />
        </svg>
    }
}

}
pub use r#wifi::Wifi;
mod r#hard_drive {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(HardDrive)]
pub fn r#hard_drive(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="12" x1="22" y1="12" x2="2"  /><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"  /><line x2="6.01" x1="6" y1="16" y2="16"  /><line x1="10" x2="10.01" y1="16" y2="16"  />
        </svg>
    }
}

}
pub use r#hard_drive::HardDrive;
mod r#boxes {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Boxes)]
pub fn r#boxes(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2.97 12.92A2 2 0 0 0 2 14.63v3.24a2 2 0 0 0 .97 1.71l3 1.8a2 2 0 0 0 2.06 0L12 19v-5.5l-5-3-4.03 2.42Z"  /><path d="m7 16.5-4.74-2.85"  /><path d="m7 16.5 5-3"  /><path d="M7 16.5v5.17"  /><path d="M12 13.5V19l3.97 2.38a2 2 0 0 0 2.06 0l3-1.8a2 2 0 0 0 .97-1.71v-3.24a2 2 0 0 0-.97-1.71L17 10.5l-5 3Z"  /><path d="m17 16.5-5-3"  /><path d="m17 16.5 4.74-2.85"  /><path d="M17 16.5v5.17"  /><path d="M7.97 4.42A2 2 0 0 0 7 6.13v4.37l5 3 5-3V6.13a2 2 0 0 0-.97-1.71l-3-1.8a2 2 0 0 0-2.06 0l-3 1.8Z"  /><path d="M12 8 7.26 5.15"  /><path d="m12 8 4.74-2.85"  /><path d="M12 13.5V8"  />
        </svg>
    }
}

}
pub use r#boxes::Boxes;
mod r#component {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Component)]
pub fn r#component(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M5.5 8.5 9 12l-3.5 3.5L2 12l3.5-3.5Z"  /><path d="m12 2 3.5 3.5L12 9 8.5 5.5 12 2Z"  /><path d="M18.5 8.5 22 12l-3.5 3.5L15 12l3.5-3.5Z"  /><path d="m12 15 3.5 3.5L12 22l-3.5-3.5L12 15Z"  />
        </svg>
    }
}

}
pub use r#component::Component;
mod r#sidebar_close {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SidebarClose)]
pub fn r#sidebar_close(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="3" rx="2" y="3" ry="2" width="18" height="18"  /><path d="M9 3v18"  /><path d="m16 15-3-3 3-3"  />
        </svg>
    }
}

}
pub use r#sidebar_close::SidebarClose;
mod r#power_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PowerOff)]
pub fn r#power_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18.36 6.64A9 9 0 0 1 20.77 15"  /><path d="M6.16 6.16a9 9 0 1 0 12.68 12.68"  /><path d="M12 2v4"  /><path d="m2 2 20 20"  />
        </svg>
    }
}

}
pub use r#power_off::PowerOff;
mod r#folder_plus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderPlus)]
pub fn r#folder_plus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"  /><line x2="12" x1="12" y1="10" y2="16"  /><line x1="9" y1="13" y2="13" x2="15"  />
        </svg>
    }
}

}
pub use r#folder_plus::FolderPlus;
mod r#bar_chart_4 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BarChart4)]
pub fn r#bar_chart_4(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 3v18h18"  /><path d="M13 17V9"  /><path d="M18 17V5"  /><path d="M8 17v-3"  />
        </svg>
    }
}

}
pub use r#bar_chart_4::BarChart4;
mod r#battery_charging {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BatteryCharging)]
pub fn r#battery_charging(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M15 7h1a2 2 0 0 1 2 2v6a2 2 0 0 1-2 2h-2"  /><path d="M6 7H4a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2h1"  /><path d="m11 7-3 5h4l-3 5"  /><line x2="22" x1="22" y1="11" y2="13"  />
        </svg>
    }
}

}
pub use r#battery_charging::BatteryCharging;
mod r#calendar {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Calendar)]
pub fn r#calendar(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect rx="2" width="18" y="4" x="3" height="18" ry="2"  /><line y2="6" x2="16" x1="16" y1="2"  /><line x1="8" y2="6" y1="2" x2="8"  /><line x1="3" x2="21" y2="10" y1="10"  />
        </svg>
    }
}

}
pub use r#calendar::Calendar;
mod r#leaf {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Leaf)]
pub fn r#leaf(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10Z"  /><path d="M2 21c0-3 1.85-5.36 5.08-6C9.5 14.52 12 13 13 12"  />
        </svg>
    }
}

}
pub use r#leaf::Leaf;
mod r#folder_down {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderDown)]
pub fn r#folder_down(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"  /><path d="M12 10v6"  /><path d="m15 13-3 3-3-3"  />
        </svg>
    }
}

}
pub use r#folder_down::FolderDown;
mod r#phone_forwarded {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PhoneForwarded)]
pub fn r#phone_forwarded(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="18 2 22 6 18 10"  /><line x1="14" x2="22" y2="6" y1="6"  /><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"  />
        </svg>
    }
}

}
pub use r#phone_forwarded::PhoneForwarded;
mod r#server_crash {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ServerCrash)]
pub fn r#server_crash(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M6 10H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2h-2"  /><path d="M6 14H4a2 2 0 0 0-2 2v4a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-4a2 2 0 0 0-2-2h-2"  /><path d="M6 6h.01"  /><path d="M6 18h.01"  /><path d="m13 6-4 6h6l-4 6"  />
        </svg>
    }
}

}
pub use r#server_crash::ServerCrash;
mod r#droplets {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Droplets)]
pub fn r#droplets(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M7 16.3c2.2 0 4-1.83 4-4.05 0-1.16-.57-2.26-1.71-3.19S7.29 6.75 7 5.3c-.29 1.45-1.14 2.84-2.29 3.76S3 11.1 3 12.25c0 2.22 1.8 4.05 4 4.05z"  /><path d="M12.56 6.6A10.97 10.97 0 0 0 14 3.02c.5 2.5 2 4.9 4 6.5s3 3.5 3 5.5a6.98 6.98 0 0 1-11.91 4.97"  />
        </svg>
    }
}

}
pub use r#droplets::Droplets;
mod r#file_archive {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileArchive)]
pub fn r#file_archive(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 22V4c0-.5.2-1 .6-1.4C5 2.2 5.5 2 6 2h8.5L20 7.5V20c0 .5-.2 1-.6 1.4-.4.4-.9.6-1.4.6h-2"  /><polyline points="14 2 14 8 20 8"  /><circle cy="20" cx="10" r="2"  /><path d="M10 7V6"  /><path d="M10 12v-1"  /><path d="M10 18v-2"  />
        </svg>
    }
}

}
pub use r#file_archive::FileArchive;
mod r#file_axis_3_d {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileAxis3D)]
pub fn r#file_axis_3_d(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><path d="M8 10v8h8"  /><path d="m8 18 4-4"  />
        </svg>
    }
}

}
pub use r#file_axis_3_d::FileAxis3D;
mod r#folder_heart {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderHeart)]
pub fn r#folder_heart(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M11 20H4a2 2 0 0 1-2-2V5c0-1.1.9-2 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H20a2 2 0 0 1 2 2v1.5"  /><path d="M21.29 13.7a2.43 2.43 0 0 0-2.65-.52c-.3.12-.57.3-.8.53l-.34.34-.35-.34a2.43 2.43 0 0 0-2.65-.53c-.3.12-.56.3-.79.53-.95.94-1 2.53.2 3.74L17.5 21l3.6-3.55c1.2-1.21 1.14-2.8.19-3.74Z"  />
        </svg>
    }
}

}
pub use r#folder_heart::FolderHeart;
mod r#linkedin {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Linkedin)]
pub fn r#linkedin(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16 8a6 6 0 0 1 6 6v7h-4v-7a2 2 0 0 0-2-2 2 2 0 0 0-2 2v7h-4v-7a6 6 0 0 1 6-6z"  /><rect height="12" y="9" x="2" width="4"  /><circle cy="4" cx="4" r="2"  />
        </svg>
    }
}

}
pub use r#linkedin::Linkedin;
mod r#reply {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Reply)]
pub fn r#reply(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="9 17 4 12 9 7"  /><path d="M20 18v-2a4 4 0 0 0-4-4H4"  />
        </svg>
    }
}

}
pub use r#reply::Reply;
mod r#table {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Table)]
pub fn r#table(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect width="18" y="3" x="3" height="18" rx="2" ry="2"  /><line y1="9" x2="21" y2="9" x1="3"  /><line y1="15" y2="15" x2="21" x1="3"  /><line x2="12" x1="12" y1="3" y2="21"  />
        </svg>
    }
}

}
pub use r#table::Table;
mod r#chevrons_up {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ChevronsUp)]
pub fn r#chevrons_up(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="17 11 12 6 7 11"  /><polyline points="17 18 12 13 7 18"  />
        </svg>
    }
}

}
pub use r#chevrons_up::ChevronsUp;
mod r#sun_medium {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SunMedium)]
pub fn r#sun_medium(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8z"  /><path d="M12 3v1"  /><path d="M12 20v1"  /><path d="M3 12h1"  /><path d="M20 12h1"  /><path d="m18.364 5.636-.707.707"  /><path d="m6.343 17.657-.707.707"  /><path d="m5.636 5.636.707.707"  /><path d="m17.657 17.657.707.707"  />
        </svg>
    }
}

}
pub use r#sun_medium::SunMedium;
mod r#shield_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ShieldOff)]
pub fn r#shield_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M19.69 14a6.9 6.9 0 0 0 .31-2V5l-8-3-3.16 1.18"  /><path d="M4.73 4.73 4 5v7c0 6 8 10 8 10a20.29 20.29 0 0 0 5.62-4.38"  /><line y2="22" y1="2" x2="22" x1="2"  />
        </svg>
    }
}

}
pub use r#shield_off::ShieldOff;
mod r#x {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(X)]
pub fn r#x(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x2="6" y2="18" y1="6" x1="18"  /><line x1="6" y1="6" y2="18" x2="18"  />
        </svg>
    }
}

}
pub use r#x::X;
mod r#camera_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CameraOff)]
pub fn r#camera_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x1="2" y1="2" y2="22" x2="22"  /><path d="M7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16"  /><path d="M9.5 4h5L17 7h3a2 2 0 0 1 2 2v7.5"  /><path d="M14.121 15.121A3 3 0 1 1 9.88 10.88"  />
        </svg>
    }
}

}
pub use r#camera_off::CameraOff;
mod r#focus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Focus)]
pub fn r#focus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="3" cx="12" cy="12"  /><path d="M3 7V5a2 2 0 0 1 2-2h2"  /><path d="M17 3h2a2 2 0 0 1 2 2v2"  /><path d="M21 17v2a2 2 0 0 1-2 2h-2"  /><path d="M7 21H5a2 2 0 0 1-2-2v-2"  />
        </svg>
    }
}

}
pub use r#focus::Focus;
mod r#box_select {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BoxSelect)]
pub fn r#box_select(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 2a2 2 0 0 0-2 2"  /><line x2="10" y1="2" y2="2" x1="8"  /><line y2="2" x1="14" y1="2" x2="16"  /><path d="M4 22a2 2 0 0 1-2-2"  /><line y1="8" x2="22" x1="22" y2="10"  /><line y1="14" x2="22" x1="22" y2="16"  /><path d="M22 20a2 2 0 0 1-2 2"  /><line x1="14" x2="16" y2="22" y1="22"  /><line x2="10" y2="22" x1="8" y1="22"  /><path d="M20 2a2 2 0 0 1 2 2"  /><line x1="2" y1="14" x2="2" y2="16"  /><line x2="2" x1="2" y1="8" y2="10"  />
        </svg>
    }
}

}
pub use r#box_select::BoxSelect;
mod r#music {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Music)]
pub fn r#music(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9 18V5l12-2v13"  /><circle r="3" cx="6" cy="18"  /><circle r="3" cx="18" cy="16"  />
        </svg>
    }
}

}
pub use r#music::Music;
mod r#magnet {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Magnet)]
pub fn r#magnet(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m6 15-4-4 6.75-6.77a7.79 7.79 0 0 1 11 11L13 22l-4-4 6.39-6.36a2.14 2.14 0 0 0-3-3L6 15"  /><path d="m5 8 4 4"  /><path d="m12 15 4 4"  />
        </svg>
    }
}

}
pub use r#magnet::Magnet;
mod r#search {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Search)]
pub fn r#search(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="11" r="8" cx="11"  /><line x1="21" x2="16.65" y1="21" y2="16.65"  />
        </svg>
    }
}

}
pub use r#search::Search;
mod r#switch_camera {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SwitchCamera)]
pub fn r#switch_camera(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M11 19H4a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h5"  /><path d="M13 5h7a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2h-5"  /><circle cy="12" cx="12" r="3"  /><path d="m18 22-3-3 3-3"  /><path d="m6 2 3 3-3 3"  />
        </svg>
    }
}

}
pub use r#switch_camera::SwitchCamera;
mod r#pie_chart {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PieChart)]
pub fn r#pie_chart(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21.21 15.89A10 10 0 1 1 8 2.83"  /><path d="M22 12A10 10 0 0 0 12 2v10z"  />
        </svg>
    }
}

}
pub use r#pie_chart::PieChart;
mod r#align_horizontal_justify_end {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignHorizontalJustifyEnd)]
pub fn r#align_horizontal_justify_end(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect rx="2" height="14" y="5" width="6" x="2"  /><rect rx="2" y="7" x="12" height="10" width="6"  /><path d="M22 2v20"  />
        </svg>
    }
}

}
pub use r#align_horizontal_justify_end::AlignHorizontalJustifyEnd;
mod r#power {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Power)]
pub fn r#power(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18.36 6.64a9 9 0 1 1-12.73 0"  /><line x2="12" y2="12" y1="2" x1="12"  />
        </svg>
    }
}

}
pub use r#power::Power;
mod r#qr_code {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(QrCode)]
pub fn r#qr_code(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="2" x="2" width="8" height="8"  /><path d="M6 6h.01"  /><rect height="8" y="2" width="8" x="14"  /><path d="M18 6h.01"  /><rect width="8" y="14" x="2" height="8"  /><path d="M6 18h.01"  /><path d="M14 14h.01"  /><path d="M18 18h.01"  /><path d="M18 22h4v-4"  /><path d="M14 18v4"  /><path d="M22 14h-4"  />
        </svg>
    }
}

}
pub use r#qr_code::QrCode;
mod r#russian_ruble {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(RussianRuble)]
pub fn r#russian_ruble(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14 11c5.333 0 5.333-8 0-8"  /><path d="M6 11h8"  /><path d="M6 15h8"  /><path d="M9 21V3"  /><path d="M9 3h5"  />
        </svg>
    }
}

}
pub use r#russian_ruble::RussianRuble;
mod r#sort_asc {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SortAsc)]
pub fn r#sort_asc(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M11 5h4"  /><path d="M11 9h7"  /><path d="M11 13h10"  /><path d="m3 17 3 3 3-3"  /><path d="M6 18V4"  />
        </svg>
    }
}

}
pub use r#sort_asc::SortAsc;
mod r#bluetooth_searching {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BluetoothSearching)]
pub fn r#bluetooth_searching(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m7 7 10 10-5 5V2l5 5L7 17"  /><path d="M20.83 14.83a4 4 0 0 0 0-5.66"  /><path d="M18 12h.01"  />
        </svg>
    }
}

}
pub use r#bluetooth_searching::BluetoothSearching;
mod r#shrub {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Shrub)]
pub fn r#shrub(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 22v-7l-2-2"  /><path d="M17 8v.8A6 6 0 0 1 13.8 20v0H10v0A6.5 6.5 0 0 1 7 8h0a5 5 0 0 1 10 0Z"  /><path d="m14 14-2 2"  />
        </svg>
    }
}

}
pub use r#shrub::Shrub;
mod r#chevron_down {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ChevronDown)]
pub fn r#chevron_down(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="6 9 12 15 18 9"  />
        </svg>
    }
}

}
pub use r#chevron_down::ChevronDown;
mod r#sliders {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Sliders)]
pub fn r#sliders(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="14" x1="4" y1="21" x2="4"  /><line y2="3" x1="4" y1="10" x2="4"  /><line x1="12" y1="21" x2="12" y2="12"  /><line y1="8" y2="3" x1="12" x2="12"  /><line x1="20" y1="21" y2="16" x2="20"  /><line x1="20" y1="12" y2="3" x2="20"  /><line y2="14" x2="6" y1="14" x1="2"  /><line x2="14" y1="8" x1="10" y2="8"  /><line x1="18" y2="16" x2="22" y1="16"  />
        </svg>
    }
}

}
pub use r#sliders::Sliders;
mod r#plane {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Plane)]
pub fn r#plane(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M17.8 19.2 16 11l3.5-3.5C21 6 21.5 4 21 3c-1-.5-3 0-4.5 1.5L13 8 4.8 6.2c-.5-.1-.9.1-1.1.5l-.3.5c-.2.5-.1 1 .3 1.3L9 12l-2 3H4l-1 1 3 2 2 3 1-1v-3l3-2 3.5 5.3c.3.4.8.5 1.3.3l.5-.2c.4-.3.6-.7.5-1.2z"  />
        </svg>
    }
}

}
pub use r#plane::Plane;
mod r#check_circle_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CheckCircle2)]
pub fn r#check_circle_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z"  /><path d="m9 12 2 2 4-4"  />
        </svg>
    }
}

}
pub use r#check_circle_2::CheckCircle2;
mod r#arrow_big_right {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowBigRight)]
pub fn r#arrow_big_right(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m21 12-7-7v4H3v6h11v4z"  />
        </svg>
    }
}

}
pub use r#arrow_big_right::ArrowBigRight;
mod r#haze {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Haze)]
pub fn r#haze(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m5.2 6.2 1.4 1.4"  /><path d="M2 13h2"  /><path d="M20 13h2"  /><path d="m17.4 7.6 1.4-1.4"  /><path d="M22 17H2"  /><path d="M22 21H2"  /><path d="M16 13a4 4 0 0 0-8 0"  /><path d="M12 5V2.5"  />
        </svg>
    }
}

}
pub use r#haze::Haze;
mod r#calendar_range {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CalendarRange)]
pub fn r#calendar_range(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="4" height="18" x="3" ry="2" width="18" rx="2"  /><line x1="16" x2="16" y1="2" y2="6"  /><line y1="2" x2="8" x1="8" y2="6"  /><line x1="3" x2="21" y1="10" y2="10"  /><path d="M17 14h-6"  /><path d="M13 18H7"  /><path d="M7 14h.01"  /><path d="M17 18h.01"  />
        </svg>
    }
}

}
pub use r#calendar_range::CalendarRange;
mod r#lamp_desk {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LampDesk)]
pub fn r#lamp_desk(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m14 5-3 3 2 7 8-8-7-2Z"  /><path d="m14 5-3 3-3-3 3-3 3 3Z"  /><path d="M9.5 6.5 4 12l3 6"  /><path d="M3 22v-2c0-1.1.9-2 2-2h4a2 2 0 0 1 2 2v2H3Z"  />
        </svg>
    }
}

}
pub use r#lamp_desk::LampDesk;
mod r#align_horizontal_distribute_center {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignHorizontalDistributeCenter)]
pub fn r#align_horizontal_distribute_center(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="5" rx="2" x="4" width="6" height="14"  /><rect x="14" rx="2" y="7" height="10" width="6"  /><path d="M17 22v-5"  /><path d="M17 7V2"  /><path d="M7 22v-3"  /><path d="M7 5V2"  />
        </svg>
    }
}

}
pub use r#align_horizontal_distribute_center::AlignHorizontalDistributeCenter;
mod r#book_open {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BookOpen)]
pub fn r#book_open(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"  /><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"  />
        </svg>
    }
}

}
pub use r#book_open::BookOpen;
mod r#navigation {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Navigation)]
pub fn r#navigation(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="3 11 22 2 13 21 11 13 3 11"  />
        </svg>
    }
}

}
pub use r#navigation::Navigation;
mod r#mountain_snow {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MountainSnow)]
pub fn r#mountain_snow(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m8 3 4 8 5-5 5 15H2L8 3z"  /><path d="M4.14 15.08c2.62-1.57 5.24-1.43 7.86.42 2.74 1.94 5.49 2 8.23.19"  />
        </svg>
    }
}

}
pub use r#mountain_snow::MountainSnow;
mod r#cloud_sun_rain {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CloudSunRain)]
pub fn r#cloud_sun_rain(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 2v2"  /><path d="m4.93 4.93 1.41 1.41"  /><path d="M20 12h2"  /><path d="m19.07 4.93-1.41 1.41"  /><path d="M15.947 12.65a4 4 0 0 0-5.925-4.128"  /><path d="M3 20a5 5 0 1 1 8.9-4H13a3 3 0 0 1 2 5.24"  /><path d="M11 20v2"  /><path d="M7 19v2"  />
        </svg>
    }
}

}
pub use r#cloud_sun_rain::CloudSunRain;
mod r#image_minus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ImageMinus)]
pub fn r#image_minus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 9v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h7"  /><line x2="22" x1="16" y2="5" y1="5"  /><circle r="2" cx="9" cy="9"  /><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"  />
        </svg>
    }
}

}
pub use r#image_minus::ImageMinus;
mod r#file_line_chart {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileLineChart)]
pub fn r#file_line_chart(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><path d="m16 13-3.5 3.5-2-2L8 17"  />
        </svg>
    }
}

}
pub use r#file_line_chart::FileLineChart;
mod r#cloud_hail {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CloudHail)]
pub fn r#cloud_hail(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242"  /><path d="M16 14v2"  /><path d="M8 14v2"  /><path d="M16 20h.01"  /><path d="M8 20h.01"  /><path d="M12 16v2"  /><path d="M12 22h.01"  />
        </svg>
    }
}

}
pub use r#cloud_hail::CloudHail;
mod r#mic {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Mic)]
pub fn r#mic(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"  /><path d="M19 10v2a7 7 0 0 1-14 0v-2"  /><line y1="19" x2="12" y2="22" x1="12"  />
        </svg>
    }
}

}
pub use r#mic::Mic;
mod r#microscope {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Microscope)]
pub fn r#microscope(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M6 18h8"  /><path d="M3 22h18"  /><path d="M14 22a7 7 0 1 0 0-14h-1"  /><path d="M9 14h2"  /><path d="M8 6h4"  /><path d="M13 10V6.5a.5.5 0 0 0-.5-.5.5.5 0 0 1-.5-.5V3a1 1 0 0 0-1-1H9a1 1 0 0 0-1 1v2.5a.5.5 0 0 1-.5.5.5.5 0 0 0-.5.5V10c0 1.1.9 2 2 2h2a2 2 0 0 0 2-2Z"  />
        </svg>
    }
}

}
pub use r#microscope::Microscope;
mod r#ghost {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Ghost)]
pub fn r#ghost(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9 10h.01"  /><path d="M15 10h.01"  /><path d="M12 2a8 8 0 0 0-8 8v12l3-3 2.5 2.5L12 19l2.5 2.5L17 19l3 3V10a8 8 0 0 0-8-8z"  />
        </svg>
    }
}

}
pub use r#ghost::Ghost;
mod r#monitor_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MonitorOff)]
pub fn r#monitor_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M17 17H4a2 2 0 0 1-2-2V5c0-1.5 1-2 1-2"  /><path d="M22 15V5a2 2 0 0 0-2-2H9"  /><path d="M8 21h8"  /><path d="M12 17v4"  /><path d="m2 2 20 20"  />
        </svg>
    }
}

}
pub use r#monitor_off::MonitorOff;
mod r#plus_circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PlusCircle)]
pub fn r#plus_circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" r="10" cx="12"  /><line x1="12" x2="12" y1="8" y2="16"  /><line y1="12" x2="16" x1="8" y2="12"  />
        </svg>
    }
}

}
pub use r#plus_circle::PlusCircle;
mod r#book {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Book)]
pub fn r#book(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"  /><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"  />
        </svg>
    }
}

}
pub use r#book::Book;
mod r#package_search {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PackageSearch)]
pub fn r#package_search(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 10V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l2-1.14"  /><path d="M16.5 9.4 7.55 4.24"  /><polyline points="3.29 7 12 12 20.71 7"  /><line y1="22" x2="12" y2="12" x1="12"  /><circle r="2.5" cx="18.5" cy="15.5"  /><path d="M20.27 17.27 22 19"  />
        </svg>
    }
}

}
pub use r#package_search::PackageSearch;
mod r#camera {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Camera)]
pub fn r#camera(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3l-2.5-3z"  /><circle cx="12" r="3" cy="13"  />
        </svg>
    }
}

}
pub use r#camera::Camera;
mod r#rss {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Rss)]
pub fn r#rss(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 11a9 9 0 0 1 9 9"  /><path d="M4 4a16 16 0 0 1 16 16"  /><circle cx="5" cy="19" r="1"  />
        </svg>
    }
}

}
pub use r#rss::Rss;
mod r#bar_chart {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BarChart)]
pub fn r#bar_chart(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x1="12" y1="20" x2="12" y2="10"  /><line x1="18" y2="4" y1="20" x2="18"  /><line x1="6" y1="20" x2="6" y2="16"  />
        </svg>
    }
}

}
pub use r#bar_chart::BarChart;
mod r#lamp_ceiling {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LampCeiling)]
pub fn r#lamp_ceiling(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 2v5"  /><path d="M6 7h12l4 9H2l4-9Z"  /><path d="M9.17 16a3 3 0 1 0 5.66 0"  />
        </svg>
    }
}

}
pub use r#lamp_ceiling::LampCeiling;
mod r#align_vertical_distribute_start {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignVerticalDistributeStart)]
pub fn r#align_vertical_distribute_start(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="14" width="14" rx="2" x="5" height="6"  /><rect width="10" rx="2" x="7" height="6" y="4"  /><path d="M2 14h20"  /><path d="M2 4h20"  />
        </svg>
    }
}

}
pub use r#align_vertical_distribute_start::AlignVerticalDistributeStart;
mod r#building {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Building)]
pub fn r#building(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect ry="2" width="16" x="4" height="20" rx="2" y="2"  /><path d="M9 22v-4h6v4"  /><path d="M8 6h.01"  /><path d="M16 6h.01"  /><path d="M12 6h.01"  /><path d="M12 10h.01"  /><path d="M12 14h.01"  /><path d="M16 10h.01"  /><path d="M16 14h.01"  /><path d="M8 10h.01"  /><path d="M8 14h.01"  />
        </svg>
    }
}

}
pub use r#building::Building;
mod r#umbrella {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Umbrella)]
pub fn r#umbrella(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M22 12a9.92 9.92 0 0 0-3.24-6.41 10.12 10.12 0 0 0-13.52 0A9.92 9.92 0 0 0 2 12Z"  /><path d="M12 12v8a2 2 0 0 0 4 0"  /><line y2="3" y1="2" x2="12" x1="12"  />
        </svg>
    }
}

}
pub use r#umbrella::Umbrella;
mod r#volume_x {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(VolumeX)]
pub fn r#volume_x(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"  /><line y2="15" x2="16" x1="22" y1="9"  /><line x2="22" y2="15" y1="9" x1="16"  />
        </svg>
    }
}

}
pub use r#volume_x::VolumeX;
mod r#webcam {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Webcam)]
pub fn r#webcam(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="10" r="8" cx="12"  /><circle r="3" cx="12" cy="10"  /><path d="M7 22h10"  /><path d="M12 22v-4"  />
        </svg>
    }
}

}
pub use r#webcam::Webcam;
mod r#heart {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Heart)]
pub fn r#heart(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20.42 4.58a5.4 5.4 0 0 0-7.65 0l-.77.78-.77-.78a5.4 5.4 0 0 0-7.65 0C1.46 6.7 1.33 10.28 4 13l8 8 8-8c2.67-2.72 2.54-6.3.42-8.42z"  />
        </svg>
    }
}

}
pub use r#heart::Heart;
mod r#network {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Network)]
pub fn r#network(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="2" width="6" height="6" x="9"  /><rect height="6" width="6" x="16" y="16"  /><rect width="6" x="2" height="6" y="16"  /><path d="M5 16v-4h14v4"  /><path d="M12 12V8"  />
        </svg>
    }
}

}
pub use r#network::Network;
mod r#heart_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(HeartOff)]
pub fn r#heart_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4.12 4.107a5.4 5.4 0 0 0-.538.473C1.46 6.7 1.33 10.28 4 13l8 8 4.5-4.5"  /><path d="M19.328 13.672 20 13c2.67-2.72 2.54-6.3.42-8.42a5.4 5.4 0 0 0-7.65 0l-.77.78-.77-.78a5.4 5.4 0 0 0-2.386-1.393"  /><line x2="22" y1="2" y2="22" x1="2"  />
        </svg>
    }
}

}
pub use r#heart_off::HeartOff;
mod r#trello {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Trello)]
pub fn r#trello(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect height="18" rx="2" x="3" y="3" ry="2" width="18"  /><rect x="7" height="9" width="3" y="7"  /><rect y="7" height="5" width="3" x="14"  />
        </svg>
    }
}

}
pub use r#trello::Trello;
mod r#cake {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Cake)]
pub fn r#cake(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20 21v-8a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8"  /><path d="M4 16s.5-1 2-1 2.5 2 4 2 2.5-2 4-2 2.5 2 4 2 2-1 2-1"  /><path d="M2 21h20"  /><path d="M7 8v2"  /><path d="M12 8v2"  /><path d="M17 8v2"  /><path d="M7 4h.01"  /><path d="M12 4h.01"  /><path d="M17 4h.01"  />
        </svg>
    }
}

}
pub use r#cake::Cake;
mod r#cross {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Cross)]
pub fn r#cross(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M11 2a2 2 0 0 0-2 2v5H4a2 2 0 0 0-2 2v2c0 1.1.9 2 2 2h5v5c0 1.1.9 2 2 2h2a2 2 0 0 0 2-2v-5h5a2 2 0 0 0 2-2v-2a2 2 0 0 0-2-2h-5V4a2 2 0 0 0-2-2h-2z"  />
        </svg>
    }
}

}
pub use r#cross::Cross;
mod r#archive_restore {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArchiveRestore)]
pub fn r#archive_restore(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="2" height="5" rx="2" y="4" width="20"  /><path d="M12 13v7"  /><path d="m9 16 3-3 3 3"  /><path d="M4 9v9a2 2 0 0 0 2 2h2"  /><path d="M20 9v9a2 2 0 0 1-2 2h-2"  />
        </svg>
    }
}

}
pub use r#archive_restore::ArchiveRestore;
mod r#brush {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Brush)]
pub fn r#brush(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m9.06 11.9 8.07-8.06a2.85 2.85 0 1 1 4.03 4.03l-8.06 8.08"  /><path d="M7.07 14.94c-1.66 0-3 1.35-3 3.02 0 1.33-2.5 1.52-2 2.02 1.08 1.1 2.49 2.02 4 2.02 2.2 0 4-1.8 4-4.04a3.01 3.01 0 0 0-3-3.02z"  />
        </svg>
    }
}

}
pub use r#brush::Brush;
mod r#file {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(File)]
pub fn r#file(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  />
        </svg>
    }
}

}
pub use r#file::File;
mod r#recycle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Recycle)]
pub fn r#recycle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M7 19H4.815a1.83 1.83 0 0 1-1.57-.881 1.785 1.785 0 0 1-.004-1.784L7.196 9.5"  /><path d="M11 19h8.203a1.83 1.83 0 0 0 1.556-.89 1.784 1.784 0 0 0 0-1.775l-1.226-2.12"  /><path d="m14 16-3 3 3 3"  /><path d="M8.293 13.596 7.196 9.5 3.1 10.598"  /><path d="m9.344 5.811 1.093-1.892A1.83 1.83 0 0 1 11.985 3a1.784 1.784 0 0 1 1.546.888l3.943 6.843"  /><path d="m13.378 9.633 4.096 1.098 1.097-4.096"  />
        </svg>
    }
}

}
pub use r#recycle::Recycle;
mod r#terminal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Terminal)]
pub fn r#terminal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="4 17 10 11 4 5"  /><line y1="19" y2="19" x2="20" x1="12"  />
        </svg>
    }
}

}
pub use r#terminal::Terminal;
mod r#at_sign {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AtSign)]
pub fn r#at_sign(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" r="4" cx="12"  /><path d="M16 8v5a3 3 0 0 0 6 0v-1a10 10 0 1 0-3.92 7.94"  />
        </svg>
    }
}

}
pub use r#at_sign::AtSign;
mod r#dice_5 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Dice5)]
pub fn r#dice_5(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="3" height="18" rx="2" y="3" ry="2" width="18"  /><path d="M16 8h.01"  /><path d="M8 8h.01"  /><path d="M8 16h.01"  /><path d="M16 16h.01"  /><path d="M12 12h.01"  />
        </svg>
    }
}

}
pub use r#dice_5::Dice5;
mod r#layers {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Layers)]
pub fn r#layers(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="12 2 2 7 12 12 22 7 12 2"  /><polyline points="2 17 12 22 22 17"  /><polyline points="2 12 12 17 22 12"  />
        </svg>
    }
}

}
pub use r#layers::Layers;
mod r#shield_alert {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ShieldAlert)]
pub fn r#shield_alert(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"  /><path d="M12 8v4"  /><path d="M12 16h.01"  />
        </svg>
    }
}

}
pub use r#shield_alert::ShieldAlert;
mod r#flask_conical {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FlaskConical)]
pub fn r#flask_conical(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10 2v8L4.72 20.55a1 1 0 0 0 .9 1.45h12.76a1 1 0 0 0 .9-1.45L14 10V2"  /><path d="M8.5 2h7"  /><path d="M7 16h10"  />
        </svg>
    }
}

}
pub use r#flask_conical::FlaskConical;
mod r#arrow_up {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowUp)]
pub fn r#arrow_up(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y1="19" y2="5" x1="12" x2="12"  /><polyline points="5 12 12 5 19 12"  />
        </svg>
    }
}

}
pub use r#arrow_up::ArrowUp;
mod r#train {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Train)]
pub fn r#train(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect rx="2" height="16" width="16" x="4" y="3"  /><path d="M4 11h16"  /><path d="M12 3v8"  /><path d="m8 19-2 3"  /><path d="m18 22-2-3"  /><path d="M8 15h0"  /><path d="M16 15h0"  />
        </svg>
    }
}

}
pub use r#train::Train;
mod r#user {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(User)]
pub fn r#user(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"  /><circle cy="7" r="4" cx="12"  />
        </svg>
    }
}

}
pub use r#user::User;
mod r#scan {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Scan)]
pub fn r#scan(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 7V5a2 2 0 0 1 2-2h2"  /><path d="M17 3h2a2 2 0 0 1 2 2v2"  /><path d="M21 17v2a2 2 0 0 1-2 2h-2"  /><path d="M7 21H5a2 2 0 0 1-2-2v-2"  />
        </svg>
    }
}

}
pub use r#scan::Scan;
mod r#calendar_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CalendarOff)]
pub fn r#calendar_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4.18 4.18A2 2 0 0 0 3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 1.82-1.18"  /><path d="M21 15.5V6a2 2 0 0 0-2-2H9.5"  /><path d="M16 2v4"  /><path d="M3 10h7"  /><path d="M21 10h-5.5"  /><line x1="2" y1="2" y2="22" x2="22"  />
        </svg>
    }
}

}
pub use r#calendar_off::CalendarOff;
mod r#calendar_search {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CalendarSearch)]
pub fn r#calendar_search(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 12V6a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v14c0 1.1.9 2 2 2h7.5"  /><path d="M16 2v4"  /><path d="M8 2v4"  /><path d="M3 10h18"  /><path d="M18 21a3 3 0 1 0 0-6 3 3 0 0 0 0 6v0Z"  /><path d="m22 22-1.5-1.5"  />
        </svg>
    }
}

}
pub use r#calendar_search::CalendarSearch;
mod r#sprout {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Sprout)]
pub fn r#sprout(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M7 20h10"  /><path d="M10 20c5.5-2.5.8-6.4 3-10"  /><path d="M9.5 9.4c1.1.8 1.8 2.2 2.3 3.7-2 .4-3.5.4-4.8-.3-1.2-.6-2.3-1.9-3-4.2 2.8-.5 4.4 0 5.5.8z"  /><path d="M14.1 6a7 7 0 0 0-1.1 4c1.9-.1 3.3-.6 4.3-1.4 1-1 1.6-2.3 1.7-4.6-2.7.1-4 1-4.9 2z"  />
        </svg>
    }
}

}
pub use r#sprout::Sprout;
mod r#eraser {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Eraser)]
pub fn r#eraser(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m7 21-4.3-4.3c-1-1-1-2.5 0-3.4l9.6-9.6c1-1 2.5-1 3.4 0l5.6 5.6c1 1 1 2.5 0 3.4L13 21"  /><path d="M22 21H7"  /><path d="m5 11 9 9"  />
        </svg>
    }
}

}
pub use r#eraser::Eraser;
mod r#toy_brick {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ToyBrick)]
pub fn r#toy_brick(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect rx="1" x="3" width="18" y="8" height="12"  /><path d="M10 8V5c0-.6-.4-1-1-1H6a1 1 0 0 0-1 1v3"  /><path d="M19 8V5c0-.6-.4-1-1-1h-3a1 1 0 0 0-1 1v3"  />
        </svg>
    }
}

}
pub use r#toy_brick::ToyBrick;
mod r#align_horizontal_justify_center {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignHorizontalJustifyCenter)]
pub fn r#align_horizontal_justify_center(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect height="14" rx="2" x="2" y="5" width="6"  /><rect width="6" y="7" x="16" height="10" rx="2"  /><path d="M12 2v20"  />
        </svg>
    }
}

}
pub use r#align_horizontal_justify_center::AlignHorizontalJustifyCenter;
mod r#bookmark_minus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BookmarkMinus)]
pub fn r#bookmark_minus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"  /><line y2="10" x2="9" x1="15" y1="10"  />
        </svg>
    }
}

}
pub use r#bookmark_minus::BookmarkMinus;
mod r#align_horizontal_space_between {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignHorizontalSpaceBetween)]
pub fn r#align_horizontal_space_between(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect rx="2" x="3" y="5" width="6" height="14"  /><rect x="15" width="6" height="10" y="7" rx="2"  /><path d="M3 2v20"  /><path d="M21 2v20"  />
        </svg>
    }
}

}
pub use r#align_horizontal_space_between::AlignHorizontalSpaceBetween;
mod r#file_clock {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileClock)]
pub fn r#file_clock(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16 22h2c.5 0 1-.2 1.4-.6.4-.4.6-.9.6-1.4V7.5L14.5 2H6c-.5 0-1 .2-1.4.6C4.2 3 4 3.5 4 4v3"  /><polyline points="14 2 14 8 20 8"  /><circle cy="16" r="6" cx="8"  /><path d="M9.5 17.5 8 16.25V14"  />
        </svg>
    }
}

}
pub use r#file_clock::FileClock;
mod r#zap {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Zap)]
pub fn r#zap(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"  />
        </svg>
    }
}

}
pub use r#zap::Zap;
mod r#unlink_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Unlink2)]
pub fn r#unlink_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M15 7h2a5 5 0 0 1 0 10h-2m-6 0H7A5 5 0 0 1 7 7h2"  />
        </svg>
    }
}

}
pub use r#unlink_2::Unlink2;
mod r#sun_dim {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SunDim)]
pub fn r#sun_dim(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8z"  /><path d="M12 4h.01"  /><path d="M20 12h.01"  /><path d="M12 20h.01"  /><path d="M4 12h.01"  /><path d="M17.657 6.343h.01"  /><path d="M17.657 17.657h.01"  /><path d="M6.343 17.657h.01"  /><path d="M6.343 6.343h.01"  />
        </svg>
    }
}

}
pub use r#sun_dim::SunDim;
mod r#list_music {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ListMusic)]
pub fn r#list_music(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 15V6"  /><path d="M18.5 18a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5Z"  /><path d="M12 12H3"  /><path d="M16 6H3"  /><path d="M12 18H3"  />
        </svg>
    }
}

}
pub use r#list_music::ListMusic;
mod r#palmtree {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Palmtree)]
pub fn r#palmtree(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M13 8c0-2.76-2.46-5-5.5-5S2 5.24 2 8h2l1-1 1 1h4"  /><path d="M13 7.14A5.82 5.82 0 0 1 16.5 6c3.04 0 5.5 2.24 5.5 5h-3l-1-1-1 1h-3"  /><path d="M5.89 9.71c-2.15 2.15-2.3 5.47-.35 7.43l4.24-4.25.7-.7.71-.71 2.12-2.12c-1.95-1.96-5.27-1.8-7.42.35z"  /><path d="M11 15.5c.5 2.5-.17 4.5-1 6.5h4c2-5.5-.5-12-1-14"  />
        </svg>
    }
}

}
pub use r#palmtree::Palmtree;
mod r#list_plus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ListPlus)]
pub fn r#list_plus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M11 12H3"  /><path d="M16 6H3"  /><path d="M16 18H3"  /><path d="M18 9v6"  /><path d="M21 12h-6"  />
        </svg>
    }
}

}
pub use r#list_plus::ListPlus;
mod r#stethoscope {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Stethoscope)]
pub fn r#stethoscope(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4.8 2.3A.3.3 0 1 0 5 2H4a2 2 0 0 0-2 2v5a6 6 0 0 0 6 6v0a6 6 0 0 0 6-6V4a2 2 0 0 0-2-2h-1a.2.2 0 1 0 .3.3"  /><path d="M8 15v1a6 6 0 0 0 6 6v0a6 6 0 0 0 6-6v-4"  /><circle cx="20" cy="10" r="2"  />
        </svg>
    }
}

}
pub use r#stethoscope::Stethoscope;
mod r#cloud_moon {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CloudMoon)]
pub fn r#cloud_moon(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M13 22H7a5 5 0 1 1 4.9-6H13a3 3 0 0 1 0 6Z"  /><path d="M10.083 9A6.002 6.002 0 0 1 16 4a4.243 4.243 0 0 0 6 6c0 2.22-1.206 4.16-3 5.197"  />
        </svg>
    }
}

}
pub use r#cloud_moon::CloudMoon;
mod r#flower_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Flower2)]
pub fn r#flower_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 5a3 3 0 1 1 3 3m-3-3a3 3 0 1 0-3 3m3-3v1M9 8a3 3 0 1 0 3 3M9 8h1m5 0a3 3 0 1 1-3 3m3-3h-1m-2 3v-1"  /><circle cy="8" r="2" cx="12"  /><path d="M12 10v12"  /><path d="M12 22c4.2 0 7-1.667 7-5-4.2 0-7 1.667-7 5Z"  /><path d="M12 22c-4.2 0-7-1.667-7-5 4.2 0 7 1.667 7 5Z"  />
        </svg>
    }
}

}
pub use r#flower_2::Flower2;
mod r#newspaper {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Newspaper)]
pub fn r#newspaper(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 22h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v16a2 2 0 0 1-2 2Zm0 0a2 2 0 0 1-2-2v-9c0-1.1.9-2 2-2h2"  /><path d="M18 14h-8"  /><path d="M15 18h-5"  /><path d="M10 6h8v4h-8V6Z"  />
        </svg>
    }
}

}
pub use r#newspaper::Newspaper;
mod r#separator_vertical {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SeparatorVertical)]
pub fn r#separator_vertical(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x2="12" y2="21" x1="12" y1="3"  /><polyline points="8 8 4 12 8 16"  /><polyline points="16 16 20 12 16 8"  />
        </svg>
    }
}

}
pub use r#separator_vertical::SeparatorVertical;
mod r#alert_circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlertCircle)]
pub fn r#alert_circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" r="10" cy="12"  /><line x2="12" y1="8" y2="12" x1="12"  /><line x2="12.01" y2="16" y1="16" x1="12"  />
        </svg>
    }
}

}
pub use r#alert_circle::AlertCircle;
mod r#alert_octagon {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlertOctagon)]
pub fn r#alert_octagon(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="7.86 2 16.14 2 22 7.86 22 16.14 16.14 22 7.86 22 2 16.14 2 7.86 7.86 2"  /><line y1="8" y2="12" x1="12" x2="12"  /><line y1="16" y2="16" x2="12.01" x1="12"  />
        </svg>
    }
}

}
pub use r#alert_octagon::AlertOctagon;
mod r#separator_horizontal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SeparatorHorizontal)]
pub fn r#separator_horizontal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y1="12" y2="12" x1="3" x2="21"  /><polyline points="8 8 12 4 16 8"  /><polyline points="16 16 12 20 8 16"  />
        </svg>
    }
}

}
pub use r#separator_horizontal::SeparatorHorizontal;
mod r#triangle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Triangle)]
pub fn r#triangle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"  />
        </svg>
    }
}

}
pub use r#triangle::Triangle;
mod r#package {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Package)]
pub fn r#package(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x2="7.5" y1="9.4" x1="16.5" y2="4.21"  /><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"  /><polyline points="3.29 7 12 12 20.71 7"  /><line y1="22" y2="12" x2="12" x1="12"  />
        </svg>
    }
}

}
pub use r#package::Package;
mod r#file_audio_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileAudio2)]
pub fn r#file_audio_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 22h14a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v2"  /><polyline points="14 2 14 8 20 8"  /><path d="M2 17v-3a4 4 0 0 1 8 0v3"  /><circle cx="9" cy="17" r="1"  /><circle cx="3" cy="17" r="1"  />
        </svg>
    }
}

}
pub use r#file_audio_2::FileAudio2;
mod r#landmark {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Landmark)]
pub fn r#landmark(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x1="3" y1="22" y2="22" x2="21"  /><line y1="18" y2="11" x1="6" x2="6"  /><line y1="18" y2="11" x1="10" x2="10"  /><line y2="11" x2="14" x1="14" y1="18"  /><line x2="18" x1="18" y1="18" y2="11"  /><polygon points="12 2 20 7 4 7"  />
        </svg>
    }
}

}
pub use r#landmark::Landmark;
mod r#arrow_left_right {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowLeftRight)]
pub fn r#arrow_left_right(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="17 11 21 7 17 3"  /><line x2="9" y2="7" x1="21" y1="7"  /><polyline points="7 21 3 17 7 13"  /><line x2="3" y2="17" y1="17" x1="15"  />
        </svg>
    }
}

}
pub use r#arrow_left_right::ArrowLeftRight;
mod r#bug {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Bug)]
pub fn r#bug(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect width="8" y="6" rx="4" height="14" x="8"  /><path d="m19 7-3 2"  /><path d="m5 7 3 2"  /><path d="m19 19-3-2"  /><path d="m5 19 3-2"  /><path d="M20 13h-4"  /><path d="M4 13h4"  /><path d="m10 4 1 2"  /><path d="m14 4-1 2"  />
        </svg>
    }
}

}
pub use r#bug::Bug;
mod r#sun_moon {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SunMoon)]
pub fn r#sun_moon(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8z"  /><path d="M12 8a2.828 2.828 0 1 0 4 4"  /><path d="M12 2v2"  /><path d="M12 20v2"  /><path d="m4.93 4.93 1.41 1.41"  /><path d="m17.66 17.66 1.41 1.41"  /><path d="M2 12h2"  /><path d="M20 12h2"  /><path d="m6.34 17.66-1.41 1.41"  /><path d="m19.07 4.93-1.41 1.41"  />
        </svg>
    }
}

}
pub use r#sun_moon::SunMoon;
mod r#film {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Film)]
pub fn r#film(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect height="20" width="20" x="2" ry="2.18" rx="2.18" y="2"  /><line x2="7" y2="22" x1="7" y1="2"  /><line y2="22" y1="2" x1="17" x2="17"  /><line y2="12" x2="22" y1="12" x1="2"  /><line y1="7" x2="7" x1="2" y2="7"  /><line y1="17" y2="17" x1="2" x2="7"  /><line y1="17" x2="22" x1="17" y2="17"  /><line x2="22" x1="17" y1="7" y2="7"  />
        </svg>
    }
}

}
pub use r#film::Film;
mod r#meh {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Meh)]
pub fn r#meh(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" r="10" cx="12"  /><line y2="15" x2="16" x1="8" y1="15"  /><line y1="9" y2="9" x1="9" x2="9.01"  /><line x1="15" x2="15.01" y2="9" y1="9"  />
        </svg>
    }
}

}
pub use r#meh::Meh;
mod r#video_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(VideoOff)]
pub fn r#video_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10.66 6H14a2 2 0 0 1 2 2v2.34l1 1L22 8v8"  /><path d="M16 16a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h2l10 10Z"  /><line x1="2" y1="2" x2="22" y2="22"  />
        </svg>
    }
}

}
pub use r#video_off::VideoOff;
mod r#chevrons_left_right {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ChevronsLeftRight)]
pub fn r#chevrons_left_right(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m9 7-5 5 5 5"  /><path d="m15 7 5 5-5 5"  />
        </svg>
    }
}

}
pub use r#chevrons_left_right::ChevronsLeftRight;
mod r#file_badge_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileBadge2)]
pub fn r#file_badge_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><path d="M12 13a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"  /><path d="m14 12.5 1 5.5-3-1-3 1 1-5.5"  />
        </svg>
    }
}

}
pub use r#file_badge_2::FileBadge2;
mod r#x_square {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(XSquare)]
pub fn r#x_square(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect width="18" ry="2" height="18" rx="2" y="3" x="3"  /><line x2="15" y2="15" y1="9" x1="9"  /><line x1="15" x2="9" y1="9" y2="15"  />
        </svg>
    }
}

}
pub use r#x_square::XSquare;
mod r#cookie {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Cookie)]
pub fn r#cookie(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 2a10 10 0 1 0 10 10 4 4 0 0 1-5-5 4 4 0 0 1-5-5"  /><path d="M8.5 8.5v.01"  /><path d="M16 15.5v.01"  /><path d="M12 12v.01"  /><path d="M11 17v.01"  /><path d="M7 14v.01"  />
        </svg>
    }
}

}
pub use r#cookie::Cookie;
mod r#git_merge {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(GitMerge)]
pub fn r#git_merge(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="18" cy="18" r="3"  /><circle cy="6" cx="6" r="3"  /><path d="M6 21V9a9 9 0 0 0 9 9"  />
        </svg>
    }
}

}
pub use r#git_merge::GitMerge;
mod r#tv_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Tv2)]
pub fn r#tv_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M7 21h10"  /><rect y="3" width="20" height="14" x="2" rx="2"  />
        </svg>
    }
}

}
pub use r#tv_2::Tv2;
mod r#strikethrough {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Strikethrough)]
pub fn r#strikethrough(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16 4H9a3 3 0 0 0-2.83 4"  /><path d="M14 12a4 4 0 0 1 0 8H6"  /><line x2="20" y1="12" x1="4" y2="12"  />
        </svg>
    }
}

}
pub use r#strikethrough::Strikethrough;
mod r#tv {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Tv)]
pub fn r#tv(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="2" rx="2" width="20" y="7" height="15" ry="2"  /><polyline points="17 2 12 7 7 2"  />
        </svg>
    }
}

}
pub use r#tv::Tv;
mod r#file_volume_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileVolume2)]
pub fn r#file_volume_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><path d="M11.5 13.5c.32.4.5.94.5 1.5s-.18 1.1-.5 1.5"  /><path d="M15 12c.64.8 1 1.87 1 3s-.36 2.2-1 3"  /><path d="M8 15h.01"  />
        </svg>
    }
}

}
pub use r#file_volume_2::FileVolume2;
mod r#phone_call {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PhoneCall)]
pub fn r#phone_call(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"  /><path d="M14.05 2a9 9 0 0 1 8 7.94"  /><path d="M14.05 6A5 5 0 0 1 18 10"  />
        </svg>
    }
}

}
pub use r#phone_call::PhoneCall;
mod r#tree_deciduous {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(TreeDeciduous)]
pub fn r#tree_deciduous(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M8 19h8a4 4 0 0 0 3.8-2.8 4 4 0 0 0-1.6-4.5c1-1.1 1-2.7.4-4-.7-1.2-2.2-2-3.6-1.7a3 3 0 0 0-3-3 3 3 0 0 0-3 3c-1.4-.2-2.9.5-3.6 1.7-.7 1.3-.5 2.9.4 4a4 4 0 0 0-1.6 4.5A4 4 0 0 0 8 19Z"  /><path d="M12 19v3"  />
        </svg>
    }
}

}
pub use r#tree_deciduous::TreeDeciduous;
mod r#lamp {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Lamp)]
pub fn r#lamp(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M8 2h8l4 10H4L8 2Z"  /><path d="M12 12v6"  /><path d="M8 22v-2c0-1.1.9-2 2-2h4a2 2 0 0 1 2 2v2H8Z"  />
        </svg>
    }
}

}
pub use r#lamp::Lamp;
mod r#function_square {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FunctionSquare)]
pub fn r#function_square(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect width="18" y="3" height="18" rx="2" x="3" ry="2"  /><path d="M9 17c2 0 2.8-1 2.8-2.8V10c0-2 1-3.3 3.2-3"  /><path d="M9 11.2h5.7"  />
        </svg>
    }
}

}
pub use r#function_square::FunctionSquare;
mod r#folder_cog {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderCog)]
pub fn r#folder_cog(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10.5 20H4a2 2 0 0 1-2-2V5c0-1.1.9-2 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H20a2 2 0 0 1 2 2v3"  /><circle r="3" cy="18" cx="18"  /><path d="M18 14v1"  /><path d="M18 21v1"  /><path d="M22 18h-1"  /><path d="M15 18h-1"  /><path d="m21 15-.88.88"  /><path d="M15.88 20.12 15 21"  /><path d="m21 21-.88-.88"  /><path d="M15.88 15.88 15 15"  />
        </svg>
    }
}

}
pub use r#folder_cog::FolderCog;
mod r#lightbulb {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Lightbulb)]
pub fn r#lightbulb(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="18" x1="9" y1="18" x2="15"  /><line x2="14" y1="22" x1="10" y2="22"  /><path d="M15.09 14c.18-.98.65-1.74 1.41-2.5A4.65 4.65 0 0 0 18 8 6 6 0 0 0 6 8c0 1 .23 2.23 1.5 3.5A4.61 4.61 0 0 1 8.91 14"  />
        </svg>
    }
}

}
pub use r#lightbulb::Lightbulb;
mod r#siren {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Siren)]
pub fn r#siren(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M7 12a5 5 0 0 1 5-5v0a5 5 0 0 1 5 5v6H7v-6Z"  /><path d="M5 20a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v2H5v-2Z"  /><path d="M21 12h1"  /><path d="M18.5 4.5 18 5"  /><path d="M2 12h1"  /><path d="M12 2v1"  /><path d="m4.929 4.929.707.707"  /><path d="M12 12v6"  />
        </svg>
    }
}

}
pub use r#siren::Siren;
mod r#file_bar_chart_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileBarChart2)]
pub fn r#file_bar_chart_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><path d="M12 18v-6"  /><path d="M8 18v-1"  /><path d="M16 18v-3"  />
        </svg>
    }
}

}
pub use r#file_bar_chart_2::FileBarChart2;
mod r#unlink {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Unlink)]
pub fn r#unlink(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m18.84 12.25 1.72-1.71h-.02a5.004 5.004 0 0 0-.12-7.07 5.006 5.006 0 0 0-6.95 0l-1.72 1.71"  /><path d="m5.17 11.75-1.71 1.71a5.004 5.004 0 0 0 .12 7.07 5.006 5.006 0 0 0 6.95 0l1.71-1.71"  /><line y1="2" y2="5" x2="8" x1="8"  /><line x1="2" y2="8" x2="5" y1="8"  /><line x1="16" y1="19" y2="22" x2="16"  /><line y2="16" x1="19" x2="22" y1="16"  />
        </svg>
    }
}

}
pub use r#unlink::Unlink;
mod r#cloud_drizzle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CloudDrizzle)]
pub fn r#cloud_drizzle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242"  /><path d="M8 19v1"  /><path d="M8 14v1"  /><path d="M16 19v1"  /><path d="M16 14v1"  /><path d="M12 21v1"  /><path d="M12 16v1"  />
        </svg>
    }
}

}
pub use r#cloud_drizzle::CloudDrizzle;
mod r#rotate_ccw {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(RotateCcw)]
pub fn r#rotate_ccw(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 2v6h6"  /><path d="M3 13a9 9 0 1 0 3-7.7L3 8"  />
        </svg>
    }
}

}
pub use r#rotate_ccw::RotateCcw;
mod r#clock_12 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clock12)]
pub fn r#clock_12(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" cx="12" r="10"  /><polyline points="12 6 12 12"  />
        </svg>
    }
}

}
pub use r#clock_12::Clock12;
mod r#folder {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Folder)]
pub fn r#folder(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"  />
        </svg>
    }
}

}
pub use r#folder::Folder;
mod r#bike {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Bike)]
pub fn r#bike(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="3.5" cx="5.5" cy="17.5"  /><circle cy="17.5" cx="18.5" r="3.5"  /><path d="M15 6a1 1 0 1 0 0-2 1 1 0 0 0 0 2zm-3 11.5V14l-3-3 4-3 2 3h2"  />
        </svg>
    }
}

}
pub use r#bike::Bike;
mod r#wifi_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(WifiOff)]
pub fn r#wifi_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y1="2" y2="22" x2="22" x1="2"  /><path d="M8.5 16.5a5 5 0 0 1 7 0"  /><path d="M2 8.82a15 15 0 0 1 4.17-2.65"  /><path d="M10.66 5c4.01-.36 8.14.9 11.34 3.76"  /><path d="M16.85 11.25a10 10 0 0 1 2.22 1.68"  /><path d="M5 13a10 10 0 0 1 5.24-2.76"  /><line y1="20" x1="12" y2="20" x2="12.01"  />
        </svg>
    }
}

}
pub use r#wifi_off::WifiOff;
mod r#package_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Package2)]
pub fn r#package_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 9h18v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V9Z"  /><path d="m3 9 2.45-4.9A2 2 0 0 1 7.24 3h9.52a2 2 0 0 1 1.8 1.1L21 9"  /><path d="M12 3v6"  />
        </svg>
    }
}

}
pub use r#package_2::Package2;
mod r#edit_3 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Edit3)]
pub fn r#edit_3(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 20h9"  /><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"  />
        </svg>
    }
}

}
pub use r#edit_3::Edit3;
mod r#pipette {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Pipette)]
pub fn r#pipette(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m2 22 1-1h3l9-9"  /><path d="M3 21v-3l9-9"  /><path d="m15 6 3.4-3.4a2.1 2.1 0 1 1 3 3L18 9l.4.4a2.1 2.1 0 1 1-3 3l-3.8-3.8a2.1 2.1 0 1 1 3-3l.4.4Z"  />
        </svg>
    }
}

}
pub use r#pipette::Pipette;
mod r#timer_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(TimerOff)]
pub fn r#timer_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10 2h4"  /><path d="M4.6 11a8 8 0 0 0 1.7 8.7 8 8 0 0 0 8.7 1.7"  /><path d="M7.4 7.4a8 8 0 0 1 10.3 1 8 8 0 0 1 .9 10.2"  /><path d="m2 2 20 20"  /><path d="M12 12v-2"  />
        </svg>
    }
}

}
pub use r#timer_off::TimerOff;
mod r#factory {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Factory)]
pub fn r#factory(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 20a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8l-7 5V8l-7 5V4a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2Z"  /><path d="M17 18h1"  /><path d="M12 18h1"  /><path d="M7 18h1"  />
        </svg>
    }
}

}
pub use r#factory::Factory;
mod r#chrome {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Chrome)]
pub fn r#chrome(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="12" r="10"  /><circle cx="12" r="4" cy="12"  /><line x2="12" y1="8" y2="8" x1="21.17"  /><line x2="8.54" x1="3.95" y1="6.06" y2="14"  /><line y2="14" y1="21.94" x2="15.46" x1="10.88"  />
        </svg>
    }
}

}
pub use r#chrome::Chrome;
mod r#file_digit {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileDigit)]
pub fn r#file_digit(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 22h14a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v4"  /><polyline points="14 2 14 8 20 8"  /><path d="M10 12h2v6"  /><rect x="2" width="4" height="6" y="12"  /><path d="M10 18h4"  />
        </svg>
    }
}

}
pub use r#file_digit::FileDigit;
mod r#package_plus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PackagePlus)]
pub fn r#package_plus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16 16h6"  /><path d="M19 13v6"  /><path d="M21 10V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l2-1.14"  /><path d="M16.5 9.4 7.55 4.24"  /><polyline points="3.29 7 12 12 20.71 7"  /><line y1="22" x2="12" y2="12" x1="12"  />
        </svg>
    }
}

}
pub use r#package_plus::PackagePlus;
mod r#file_video {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileVideo)]
pub fn r#file_video(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><path d="m10 11 5 3-5 3v-6Z"  />
        </svg>
    }
}

}
pub use r#file_video::FileVideo;
mod r#users {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Users)]
pub fn r#users(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"  /><circle r="4" cy="7" cx="9"  /><path d="M22 21v-2a4 4 0 0 0-3-3.87"  /><path d="M16 3.13a4 4 0 0 1 0 7.75"  />
        </svg>
    }
}

}
pub use r#users::Users;
mod r#slack {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Slack)]
pub fn r#slack(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="2" rx="1.5" width="3" height="8" x="13"  /><path d="M19 8.5V10h1.5A1.5 1.5 0 1 0 19 8.5"  /><rect y="14" rx="1.5" width="3" x="8" height="8"  /><path d="M5 15.5V14H3.5A1.5 1.5 0 1 0 5 15.5"  /><rect x="14" rx="1.5" height="3" y="13" width="8"  /><path d="M15.5 19H14v1.5a1.5 1.5 0 1 0 1.5-1.5"  /><rect height="3" rx="1.5" y="8" width="8" x="2"  /><path d="M8.5 5H10V3.5A1.5 1.5 0 1 0 8.5 5"  />
        </svg>
    }
}

}
pub use r#slack::Slack;
mod r#feather {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Feather)]
pub fn r#feather(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20.24 12.24a6 6 0 0 0-8.49-8.49L5 10.5V19h8.5z"  /><line y2="22" x2="2" y1="8" x1="16"  /><line y2="15" y1="15" x2="9" x1="17.5"  />
        </svg>
    }
}

}
pub use r#feather::Feather;
mod r#mic_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Mic2)]
pub fn r#mic_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m12 8-9.04 9.06a2.82 2.82 0 1 0 3.98 3.98L16 12"  /><circle cy="7" cx="17" r="5"  />
        </svg>
    }
}

}
pub use r#mic_2::Mic2;
mod r#copyleft {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Copyleft)]
pub fn r#copyleft(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cx="12" cy="12"  /><path d="M9 9.35a4 4 0 1 1 0 5.3"  />
        </svg>
    }
}

}
pub use r#copyleft::Copyleft;
mod r#folder_minus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderMinus)]
pub fn r#folder_minus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"  /><line x2="15" y2="13" y1="13" x1="9"  />
        </svg>
    }
}

}
pub use r#folder_minus::FolderMinus;
mod r#file_symlink {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileSymlink)]
pub fn r#file_symlink(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 22h14a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v7"  /><polyline points="14 2 14 8 20 8"  /><path d="m10 18 3-3-3-3"  /><path d="M4 18v-1a2 2 0 0 1 2-2h6"  />
        </svg>
    }
}

}
pub use r#file_symlink::FileSymlink;
mod r#utensils_crossed {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(UtensilsCrossed)]
pub fn r#utensils_crossed(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m16 2-2.3 2.3a3 3 0 0 0 0 4.2l1.8 1.8a3 3 0 0 0 4.2 0L22 8"  /><path d="M15 15 3.3 3.3a4.2 4.2 0 0 0 0 6l7.3 7.3c.7.7 2 .7 2.8 0L15 15Zm0 0 7 7"  /><path d="m2.1 21.8 6.4-6.3"  /><path d="m19 5-7 7"  />
        </svg>
    }
}

}
pub use r#utensils_crossed::UtensilsCrossed;
mod r#refresh_cw {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(RefreshCw)]
pub fn r#refresh_cw(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 2v6h-6"  /><path d="M3 12a9 9 0 0 1 15-6.7L21 8"  /><path d="M3 22v-6h6"  /><path d="M21 12a9 9 0 0 1-15 6.7L3 16"  />
        </svg>
    }
}

}
pub use r#refresh_cw::RefreshCw;
mod r#chevrons_down {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ChevronsDown)]
pub fn r#chevrons_down(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="7 13 12 18 17 13"  /><polyline points="7 6 12 11 17 6"  />
        </svg>
    }
}

}
pub use r#chevrons_down::ChevronsDown;
mod r#bomb {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Bomb)]
pub fn r#bomb(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="9" cx="11" cy="13"  /><path d="m19.5 9.5 1.8-1.8a2.4 2.4 0 0 0 0-3.4l-1.6-1.6a2.41 2.41 0 0 0-3.4 0l-1.8 1.8"  /><path d="m22 2-1.5 1.5"  />
        </svg>
    }
}

}
pub use r#bomb::Bomb;
mod r#command {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Command)]
pub fn r#command(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18 3a3 3 0 0 0-3 3v12a3 3 0 0 0 3 3 3 3 0 0 0 3-3 3 3 0 0 0-3-3H6a3 3 0 0 0-3 3 3 3 0 0 0 3 3 3 3 0 0 0 3-3V6a3 3 0 0 0-3-3 3 3 0 0 0-3 3 3 3 0 0 0 3 3h12a3 3 0 0 0 3-3 3 3 0 0 0-3-3z"  />
        </svg>
    }
}

}
pub use r#command::Command;
mod r#minimize {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Minimize)]
pub fn r#minimize(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M8 3v3a2 2 0 0 1-2 2H3"  /><path d="M21 8h-3a2 2 0 0 1-2-2V3"  /><path d="M3 16h3a2 2 0 0 1 2 2v3"  /><path d="M16 21v-3a2 2 0 0 1 2-2h3"  />
        </svg>
    }
}

}
pub use r#minimize::Minimize;
mod r#radio_receiver {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(RadioReceiver)]
pub fn r#radio_receiver(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M5 16v2"  /><path d="M19 16v2"  /><rect x="2" y="8" height="8" width="20" rx="2"  /><path d="M18 12h0"  />
        </svg>
    }
}

}
pub use r#radio_receiver::RadioReceiver;
mod r#file_badge {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileBadge)]
pub fn r#file_badge(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 7V4a2 2 0 0 1 2-2h8.5L20 7.5V20a2 2 0 0 1-2 2h-6"  /><polyline points="14 2 14 8 20 8"  /><path d="M5 17a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"  /><path d="M7 16.5 8 22l-3-1-3 1 1-5.5"  />
        </svg>
    }
}

}
pub use r#file_badge::FileBadge;
mod r#album {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Album)]
pub fn r#album(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect height="18" rx="2" ry="2" x="3" y="3" width="18"  /><polyline points="11 3 11 11 14 8 17 11 17 3"  />
        </svg>
    }
}

}
pub use r#album::Album;
mod r#drumstick {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Drumstick)]
pub fn r#drumstick(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M15.45 15.4c-2.13.65-4.3.32-5.7-1.1-2.29-2.27-1.76-6.5 1.17-9.42 2.93-2.93 7.15-3.46 9.43-1.18 1.41 1.41 1.74 3.57 1.1 5.71-1.4-.51-3.26-.02-4.64 1.36-1.38 1.38-1.87 3.23-1.36 4.63z"  /><path d="m11.25 15.6-2.16 2.16a2.5 2.5 0 1 1-4.56 1.73 2.49 2.49 0 0 1-1.41-4.24 2.5 2.5 0 0 1 3.14-.32l2.16-2.16"  />
        </svg>
    }
}

}
pub use r#drumstick::Drumstick;
mod r#file_key_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileKey2)]
pub fn r#file_key_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 10V4a2 2 0 0 1 2-2h8.5L20 7.5V20a2 2 0 0 1-2 2H4"  /><polyline points="14 2 14 8 20 8"  /><circle r="2" cx="4" cy="16"  /><path d="m10 10-4.5 4.5"  /><path d="m9 11 1 1"  />
        </svg>
    }
}

}
pub use r#file_key_2::FileKey2;
mod r#grape {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Grape)]
pub fn r#grape(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M22 5V2l-5.89 5.89"  /><circle r="3" cy="15.89" cx="16.6"  /><circle r="3" cx="8.11" cy="7.4"  /><circle r="3" cx="12.35" cy="11.65"  /><circle r="3" cx="13.91" cy="5.85"  /><circle r="3" cx="18.15" cy="10.09"  /><circle cx="6.56" cy="13.2" r="3"  /><circle cy="17.44" cx="10.8" r="3"  /><circle r="3" cy="19" cx="5"  />
        </svg>
    }
}

}
pub use r#grape::Grape;
mod r#home {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Home)]
pub fn r#home(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"  /><polyline points="9 22 9 12 15 12 15 22"  />
        </svg>
    }
}

}
pub use r#home::Home;
mod r#pointer {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Pointer)]
pub fn r#pointer(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M22 14a8 8 0 0 1-8 8"  /><path d="M18 11v-1a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v0"  /><path d="M14 10V9a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v1"  /><path d="M10 9.5V4a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v10"  /><path d="M18 11a2 2 0 1 1 4 0v3a8 8 0 0 1-8 8h-2c-2.8 0-4.5-.86-5.99-2.34l-3.6-3.6a2 2 0 0 1 2.83-2.82L7 15"  />
        </svg>
    }
}

}
pub use r#pointer::Pointer;
mod r#columns {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Columns)]
pub fn r#columns(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="3" width="18" rx="2" ry="2" y="3" height="18"  /><line y1="3" y2="21" x2="12" x1="12"  />
        </svg>
    }
}

}
pub use r#columns::Columns;
mod r#settings_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Settings2)]
pub fn r#settings_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20 7h-9"  /><path d="M14 17H5"  /><circle cy="17" r="3" cx="17"  /><circle cx="7" cy="7" r="3"  />
        </svg>
    }
}

}
pub use r#settings_2::Settings2;
mod r#signal_zero {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SignalZero)]
pub fn r#signal_zero(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 20h.01"  /><path d="M7 20v-4"  />
        </svg>
    }
}

}
pub use r#signal_zero::SignalZero;
mod r#thermometer {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Thermometer)]
pub fn r#thermometer(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14 4v10.54a4 4 0 1 1-4 0V4a2 2 0 0 1 4 0Z"  />
        </svg>
    }
}

}
pub use r#thermometer::Thermometer;
mod r#watch {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Watch)]
pub fn r#watch(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" r="6" cy="12"  /><polyline points="12 10 12 12 13 13"  /><path d="m16.13 7.66-.81-4.05a2 2 0 0 0-2-1.61h-2.68a2 2 0 0 0-2 1.61l-.78 4.05"  /><path d="m7.88 16.36.8 4a2 2 0 0 0 2 1.61h2.72a2 2 0 0 0 2-1.61l.81-4.05"  />
        </svg>
    }
}

}
pub use r#watch::Watch;
mod r#wind {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Wind)]
pub fn r#wind(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M17.7 7.7a2.5 2.5 0 1 1 1.8 4.3H2"  /><path d="M9.6 4.6A2 2 0 1 1 11 8H2"  /><path d="M12.6 19.4A2 2 0 1 0 14 16H2"  />
        </svg>
    }
}

}
pub use r#wind::Wind;
mod r#corner_left_down {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CornerLeftDown)]
pub fn r#corner_left_down(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="14 15 9 20 4 15"  /><path d="M20 4h-7a4 4 0 0 0-4 4v12"  />
        </svg>
    }
}

}
pub use r#corner_left_down::CornerLeftDown;
mod r#calendar_clock {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CalendarClock)]
pub fn r#calendar_clock(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 7.5V6a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h3.5"  /><path d="M16 2v4"  /><path d="M8 2v4"  /><path d="M3 10h5"  /><path d="M17.5 17.5 16 16.25V14"  /><path d="M22 16a6 6 0 1 1-12 0 6 6 0 0 1 12 0Z"  />
        </svg>
    }
}

}
pub use r#calendar_clock::CalendarClock;
mod r#calendar_x {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CalendarX)]
pub fn r#calendar_x(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="3" height="18" rx="2" width="18" y="4" ry="2"  /><line y1="2" x1="16" y2="6" x2="16"  /><line x1="8" x2="8" y1="2" y2="6"  /><line y1="10" y2="10" x2="21" x1="3"  /><line y1="14" x1="10" x2="14" y2="18"  /><line x2="10" y2="18" x1="14" y1="14"  />
        </svg>
    }
}

}
pub use r#calendar_x::CalendarX;
mod r#thermometer_snowflake {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ThermometerSnowflake)]
pub fn r#thermometer_snowflake(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 12h10"  /><path d="M9 4v16"  /><path d="m3 9 3 3-3 3"  /><path d="M12 6 9 9 6 6"  /><path d="m6 18 3-3 1.5 1.5"  /><path d="M20 4v10.54a4 4 0 1 1-4 0V4a2 2 0 0 1 4 0Z"  />
        </svg>
    }
}

}
pub use r#thermometer_snowflake::ThermometerSnowflake;
mod r#thumbs_down {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ThumbsDown)]
pub fn r#thumbs_down(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M17 14V2"  /><path d="M9 18.12 10 14H4.17a2 2 0 0 1-1.92-2.56l2.33-8A2 2 0 0 1 6.5 2H20a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-2.76a2 2 0 0 0-1.79 1.11L12 22h0a3.13 3.13 0 0 1-3-3.88Z"  />
        </svg>
    }
}

}
pub use r#thumbs_down::ThumbsDown;
mod r#stretch_horizontal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(StretchHorizontal)]
pub fn r#stretch_horizontal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect height="6" x="2" width="20" y="4" rx="2"  /><rect height="6" width="20" rx="2" y="14" x="2"  />
        </svg>
    }
}

}
pub use r#stretch_horizontal::StretchHorizontal;
mod r#layout_list {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LayoutList)]
pub fn r#layout_list(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="3" y="14" width="7" height="7"  /><rect y="3" x="3" height="7" width="7"  /><line y2="4" x1="14" x2="21" y1="4"  /><line y2="9" x1="14" x2="21" y1="9"  /><line y2="15" x1="14" x2="21" y1="15"  /><line x1="14" x2="21" y2="20" y1="20"  />
        </svg>
    }
}

}
pub use r#layout_list::LayoutList;
mod r#more_vertical {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MoreVertical)]
pub fn r#more_vertical(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" r="1" cx="12"  /><circle cx="12" cy="5" r="1"  /><circle cy="19" r="1" cx="12"  />
        </svg>
    }
}

}
pub use r#more_vertical::MoreVertical;
mod r#align_center {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignCenter)]
pub fn r#align_center(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x2="3" y2="6" y1="6" x1="21"  /><line y1="12" x2="7" y2="12" x1="17"  /><line y1="18" x2="5" y2="18" x1="19"  />
        </svg>
    }
}

}
pub use r#align_center::AlignCenter;
mod r#phone_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PhoneOff)]
pub fn r#phone_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10.68 13.31a16 16 0 0 0 3.41 2.6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7 2 2 0 0 1 1.72 2v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.42 19.42 0 0 1-3.33-2.67m-2.67-3.34a19.79 19.79 0 0 1-3.07-8.63A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91"  /><line y2="22" x2="2" y1="2" x1="22"  />
        </svg>
    }
}

}
pub use r#phone_off::PhoneOff;
mod r#award {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Award)]
pub fn r#award(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="8" r="6"  /><path d="M15.477 12.89 17 22l-5-3-5 3 1.523-9.11"  />
        </svg>
    }
}

}
pub use r#award::Award;
mod r#file_diff {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileDiff)]
pub fn r#file_diff(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><path d="M12 13V7"  /><path d="M9 10h6"  /><path d="M9 17h6"  />
        </svg>
    }
}

}
pub use r#file_diff::FileDiff;
mod r#tag {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Tag)]
pub fn r#tag(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 12V2h10l9.44 9.44a2 2 0 0 1 0 2.82l-7.18 7.18a2 2 0 0 1-2.82 0L2 12Z"  /><path d="M7 7h.01"  />
        </svg>
    }
}

}
pub use r#tag::Tag;
mod r#clipboard_copy {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ClipboardCopy)]
pub fn r#clipboard_copy(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="2" height="4" ry="1" x="8" width="8" rx="1"  /><path d="M8 4H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2"  /><path d="M16 4h2a2 2 0 0 1 2 2v4"  /><path d="M21 14H11"  /><path d="m15 10-4 4 4 4"  />
        </svg>
    }
}

}
pub use r#clipboard_copy::ClipboardCopy;
mod r#timer {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Timer)]
pub fn r#timer(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y1="2" x1="10" x2="14" y2="2"  /><line x1="12" x2="15" y2="11" y1="14"  /><circle cy="14" cx="12" r="8"  />
        </svg>
    }
}

}
pub use r#timer::Timer;
mod r#arrow_down_right {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowDownRight)]
pub fn r#arrow_down_right(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x2="17" x1="7" y2="17" y1="7"  /><polyline points="17 7 17 17 7 17"  />
        </svg>
    }
}

}
pub use r#arrow_down_right::ArrowDownRight;
mod r#chevron_last {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ChevronLast)]
pub fn r#chevron_last(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="7 18 13 12 7 6"  /><path d="M17 6v12"  />
        </svg>
    }
}

}
pub use r#chevron_last::ChevronLast;
mod r#server_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ServerOff)]
pub fn r#server_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M7 2h13a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2h-5"  /><path d="M10 10 2.5 2.5C2 2 2 2.5 2 5v3a2 2 0 0 0 2 2h6z"  /><path d="M22 17v-1a2 2 0 0 0-2-2h-1"  /><path d="M4 14a2 2 0 0 0-2 2v4a2 2 0 0 0 2 2h16.5l1-.5.5.5-8-8H4z"  /><path d="M6 18h.01"  /><path d="m2 2 20 20"  />
        </svg>
    }
}

}
pub use r#server_off::ServerOff;
mod r#image_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ImageOff)]
pub fn r#image_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="22" x1="2" y1="2" x2="22"  /><path d="M10.41 10.41a2 2 0 1 1-2.83-2.83"  /><line x1="13.5" y1="13.5" y2="21" x2="6"  /><line y1="12" x2="21" y2="15" x1="18"  /><path d="M3.59 3.59A1.99 1.99 0 0 0 3 5v14a2 2 0 0 0 2 2h14c.55 0 1.052-.22 1.41-.59"  /><path d="M21 15V5a2 2 0 0 0-2-2H9"  />
        </svg>
    }
}

}
pub use r#image_off::ImageOff;
mod r#upload {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Upload)]
pub fn r#upload(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"  /><polyline points="17 8 12 3 7 8"  /><line y1="3" x2="12" x1="12" y2="15"  />
        </svg>
    }
}

}
pub use r#upload::Upload;
mod r#grab {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Grab)]
pub fn r#grab(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18 11.5V9a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v1.4"  /><path d="M14 10V8a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v2"  /><path d="M10 9.9V9a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v5"  /><path d="M6 14v0a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v0"  /><path d="M18 11v0a2 2 0 1 1 4 0v3a8 8 0 0 1-8 8h-4a8 8 0 0 1-8-8 2 2 0 1 1 4 0"  />
        </svg>
    }
}

}
pub use r#grab::Grab;
mod r#cloud_snow {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CloudSnow)]
pub fn r#cloud_snow(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242"  /><path d="M8 15h.01"  /><path d="M8 19h.01"  /><path d="M12 17h.01"  /><path d="M12 21h.01"  /><path d="M16 15h.01"  /><path d="M16 19h.01"  />
        </svg>
    }
}

}
pub use r#cloud_snow::CloudSnow;
mod r#navigation_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(NavigationOff)]
pub fn r#navigation_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M8.43 8.43 3 11l8 2 2 8 2.57-5.43"  /><path d="M17.39 11.73 22 2l-9.73 4.61"  /><line x1="2" x2="22" y1="2" y2="22"  />
        </svg>
    }
}

}
pub use r#navigation_off::NavigationOff;
mod r#flag_triangle_right {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FlagTriangleRight)]
pub fn r#flag_triangle_right(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M7 22V2l10 5-10 5"  />
        </svg>
    }
}

}
pub use r#flag_triangle_right::FlagTriangleRight;
mod r#pocket {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Pocket)]
pub fn r#pocket(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 3h16a2 2 0 0 1 2 2v6a10 10 0 0 1-10 10A10 10 0 0 1 2 11V5a2 2 0 0 1 2-2z"  /><polyline points="8 10 12 14 16 10"  />
        </svg>
    }
}

}
pub use r#pocket::Pocket;
mod r#heart_handshake {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(HeartHandshake)]
pub fn r#heart_handshake(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20.42 4.58a5.4 5.4 0 0 0-7.65 0l-.77.78-.77-.78a5.4 5.4 0 0 0-7.65 0C1.46 6.7 1.33 10.28 4 13l8 8 8-8c2.67-2.72 2.54-6.3.42-8.42z"  /><path d="M12 5.36 8.87 8.5a2.13 2.13 0 0 0 0 3h0a2.13 2.13 0 0 0 3 0l2.26-2.21a3 3 0 0 1 4.22 0l2.4 2.4"  /><path d="m18 15-2-2"  /><path d="m15 18-2-2"  />
        </svg>
    }
}

}
pub use r#heart_handshake::HeartHandshake;
mod r#lightbulb_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LightbulbOff)]
pub fn r#lightbulb_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9 18h6"  /><path d="M10 22h4"  /><path d="m2 2 20 20"  /><path d="M9 2.804A6 6 0 0 1 18 8a4.65 4.65 0 0 1-1.03 3"  /><path d="M8.91 14a4.61 4.61 0 0 0-1.41-2.5C6.23 10.23 6 9 6 8a6 6 0 0 1 .084-1"  />
        </svg>
    }
}

}
pub use r#lightbulb_off::LightbulbOff;
mod r#trash_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Trash2)]
pub fn r#trash_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 6h18"  /><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"  /><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"  /><line x2="10" y1="11" y2="17" x1="10"  /><line x2="14" x1="14" y2="17" y1="11"  />
        </svg>
    }
}

}
pub use r#trash_2::Trash2;
mod r#type {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Type)]
pub fn r#type(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="4 7 4 4 20 4 20 7"  /><line x2="15" y1="20" y2="20" x1="9"  /><line x1="12" y2="20" y1="4" x2="12"  />
        </svg>
    }
}

}
pub use r#type::Type;
mod r#unlock {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Unlock)]
pub fn r#unlock(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect width="18" ry="2" height="11" rx="2" x="3" y="11"  /><path d="M7 11V7a5 5 0 0 1 9.9-1"  />
        </svg>
    }
}

}
pub use r#unlock::Unlock;
mod r#shuffle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Shuffle)]
pub fn r#shuffle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="16 3 21 3 21 8"  /><line y1="20" x1="4" y2="3" x2="21"  /><polyline points="21 16 21 21 16 21"  /><line y1="15" x1="15" x2="21" y2="21"  /><line y2="9" y1="4" x1="4" x2="9"  />
        </svg>
    }
}

}
pub use r#shuffle::Shuffle;
mod r#sidebar {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Sidebar)]
pub fn r#sidebar(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="3" rx="2" height="18" width="18" ry="2" y="3"  /><line x1="9" y2="21" y1="3" x2="9"  />
        </svg>
    }
}

}
pub use r#sidebar::Sidebar;
mod r#move_diagonal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MoveDiagonal)]
pub fn r#move_diagonal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="13 5 19 5 19 11"  /><polyline points="11 19 5 19 5 13"  /><line y1="5" x2="5" y2="19" x1="19"  />
        </svg>
    }
}

}
pub use r#move_diagonal::MoveDiagonal;
mod r#smartphone_charging {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SmartphoneCharging)]
pub fn r#smartphone_charging(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="5" y="2" width="14" rx="2" ry="2" height="20"  /><path d="M12.667 8 10 12h4l-2.667 4"  />
        </svg>
    }
}

}
pub use r#smartphone_charging::SmartphoneCharging;
mod r#thermometer_sun {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ThermometerSun)]
pub fn r#thermometer_sun(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 9a4 4 0 0 0-2 7.5"  /><path d="M12 3v2"  /><path d="m6.6 18.4-1.4 1.4"  /><path d="M20 4v10.54a4 4 0 1 1-4 0V4a2 2 0 0 1 4 0Z"  /><path d="M4 13H2"  /><path d="M6.34 7.34 4.93 5.93"  />
        </svg>
    }
}

}
pub use r#thermometer_sun::ThermometerSun;
mod r#compass {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Compass)]
pub fn r#compass(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="12" r="10"  /><polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76"  />
        </svg>
    }
}

}
pub use r#compass::Compass;
mod r#help_circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(HelpCircle)]
pub fn r#help_circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="12" r="10"  /><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"  /><line x1="12" x2="12.01" y1="17" y2="17"  />
        </svg>
    }
}

}
pub use r#help_circle::HelpCircle;
mod r#bookmark_plus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BookmarkPlus)]
pub fn r#bookmark_plus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"  /><line x2="12" y1="7" y2="13" x1="12"  /><line y1="10" x2="9" y2="10" x1="15"  />
        </svg>
    }
}

}
pub use r#bookmark_plus::BookmarkPlus;
mod r#video {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Video)]
pub fn r#video(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m22 8-6 4 6 4V8Z"  /><rect y="6" width="14" rx="2" x="2" height="12" ry="2"  />
        </svg>
    }
}

}
pub use r#video::Video;
mod r#subscript {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Subscript)]
pub fn r#subscript(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m4 5 8 8"  /><path d="m12 5-8 8"  /><path d="M20 19h-4c0-1.5.44-2 1.5-2.5S20 15.33 20 14c0-.47-.17-.93-.48-1.29a2.11 2.11 0 0 0-2.62-.44c-.42.24-.74.62-.9 1.07"  />
        </svg>
    }
}

}
pub use r#subscript::Subscript;
mod r#timer_reset {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(TimerReset)]
pub fn r#timer_reset(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10 2h4"  /><path d="M12 14v-4"  /><path d="M4 13a8 8 0 0 1 8-7 8 8 0 1 1-5.3 14L4 17.6"  /><path d="M9 17H4v5"  />
        </svg>
    }
}

}
pub use r#timer_reset::TimerReset;
mod r#arrow_left_circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowLeftCircle)]
pub fn r#arrow_left_circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cy="12" cx="12"  /><polyline points="12 8 8 12 12 16"  /><line x1="16" y1="12" y2="12" x2="8"  />
        </svg>
    }
}

}
pub use r#arrow_left_circle::ArrowLeftCircle;
mod r#infinity {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Infinity)]
pub fn r#infinity(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18.178 8c5.096 0 5.096 8 0 8-5.095 0-7.133-8-12.739-8-4.585 0-4.585 8 0 8 5.606 0 7.644-8 12.74-8z"  />
        </svg>
    }
}

}
pub use r#infinity::Infinity;
mod r#lamp_wall_down {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LampWallDown)]
pub fn r#lamp_wall_down(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M11 13h6l3 7H8l3-7Z"  /><path d="M14 13V8a2 2 0 0 0-2-2H8"  /><path d="M4 9h2a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2H4v6Z"  />
        </svg>
    }
}

}
pub use r#lamp_wall_down::LampWallDown;
mod r#monitor {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Monitor)]
pub fn r#monitor(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect ry="2" rx="2" x="2" y="3" width="20" height="14"  /><line y1="21" y2="21" x1="8" x2="16"  /><line y2="21" x1="12" x2="12" y1="17"  />
        </svg>
    }
}

}
pub use r#monitor::Monitor;
mod r#file_cog_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileCog2)]
pub fn r#file_cog_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><circle cy="15" cx="12" r="2"  /><path d="M12 12v1"  /><path d="M12 17v1"  /><path d="m14.6 13.5-.87.5"  /><path d="m10.27 16-.87.5"  /><path d="m14.6 16.5-.87-.5"  /><path d="m10.27 14-.87-.5"  />
        </svg>
    }
}

}
pub use r#file_cog_2::FileCog2;
mod r#gitlab {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Gitlab)]
pub fn r#gitlab(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m22 13.29-3.33-10a.42.42 0 0 0-.14-.18.38.38 0 0 0-.22-.11.39.39 0 0 0-.23.07.42.42 0 0 0-.14.18l-2.26 6.67H8.32L6.1 3.26a.42.42 0 0 0-.1-.18.38.38 0 0 0-.26-.08.39.39 0 0 0-.23.07.42.42 0 0 0-.14.18L2 13.29a.74.74 0 0 0 .27.83L12 21l9.69-6.88a.71.71 0 0 0 .31-.83Z"  />
        </svg>
    }
}

}
pub use r#gitlab::Gitlab;
mod r#folder_closed {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderClosed)]
pub fn r#folder_closed(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"  /><path d="M2 10h20"  />
        </svg>
    }
}

}
pub use r#folder_closed::FolderClosed;
mod r#keyboard {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Keyboard)]
pub fn r#keyboard(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect ry="2" y="4" x="2" width="20" height="16" rx="2"  /><path d="M6 8h.001"  /><path d="M10 8h.001"  /><path d="M14 8h.001"  /><path d="M18 8h.001"  /><path d="M8 12h.001"  /><path d="M12 12h.001"  /><path d="M16 12h.001"  /><path d="M7 16h10"  />
        </svg>
    }
}

}
pub use r#keyboard::Keyboard;
mod r#folder_open {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderOpen)]
pub fn r#folder_open(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m6 14 1.45-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.55 6a2 2 0 0 1-1.94 1.5H4a2 2 0 0 1-2-2V5c0-1.1.9-2 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H18a2 2 0 0 1 2 2v2"  />
        </svg>
    }
}

}
pub use r#folder_open::FolderOpen;
mod r#arrow_down_left {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowDownLeft)]
pub fn r#arrow_down_left(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x2="7" y1="7" y2="17" x1="17"  /><polyline points="17 17 7 17 7 7"  />
        </svg>
    }
}

}
pub use r#arrow_down_left::ArrowDownLeft;
mod r#eye {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Eye)]
pub fn r#eye(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"  /><circle cy="12" r="3" cx="12"  />
        </svg>
    }
}

}
pub use r#eye::Eye;
mod r#arrow_big_up {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowBigUp)]
pub fn r#arrow_big_up(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9 21V10H5l7-7 7 7h-4v11z"  />
        </svg>
    }
}

}
pub use r#arrow_big_up::ArrowBigUp;
mod r#folder_cog_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderCog2)]
pub fn r#folder_cog_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"  /><circle cy="13" cx="12" r="2"  /><path d="M12 10v1"  /><path d="M12 15v1"  /><path d="m14.6 11.5-.87.5"  /><path d="m10.27 14-.87.5"  /><path d="m14.6 14.5-.87-.5"  /><path d="m10.27 12-.87-.5"  />
        </svg>
    }
}

}
pub use r#folder_cog_2::FolderCog2;
mod r#instagram {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Instagram)]
pub fn r#instagram(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="2" width="20" height="20" rx="5" ry="5" y="2"  /><path d="M16 11.37A4 4 0 1 1 12.63 8 4 4 0 0 1 16 11.37z"  /><line x1="17.5" y1="6.5" y2="6.5" x2="17.51"  />
        </svg>
    }
}

}
pub use r#instagram::Instagram;
mod r#layout_grid {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LayoutGrid)]
pub fn r#layout_grid(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="3" width="7" height="7" y="3"  /><rect width="7" height="7" x="14" y="3"  /><rect x="14" width="7" y="14" height="7"  /><rect width="7" height="7" x="3" y="14"  />
        </svg>
    }
}

}
pub use r#layout_grid::LayoutGrid;
mod r#chevron_right {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ChevronRight)]
pub fn r#chevron_right(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="9 18 15 12 9 6"  />
        </svg>
    }
}

}
pub use r#chevron_right::ChevronRight;
mod r#globe_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Globe2)]
pub fn r#globe_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M15 21v-4a2 2 0 0 1 2-2h4"  /><path d="M7 4v2a3 3 0 0 0 3 2h0a2 2 0 0 1 2 2 2 2 0 0 0 4 0 2 2 0 0 1 2-2h3"  /><path d="M3 11h2a2 2 0 0 1 2 2v1a2 2 0 0 0 2 2 2 2 0 0 1 2 2v4"  /><circle cy="12" r="10" cx="12"  />
        </svg>
    }
}

}
pub use r#globe_2::Globe2;
mod r#line_chart {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LineChart)]
pub fn r#line_chart(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 3v18h18"  /><path d="m19 9-5 5-4-4-3 3"  />
        </svg>
    }
}

}
pub use r#line_chart::LineChart;
mod r#package_check {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PackageCheck)]
pub fn r#package_check(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m16 16 2 2 4-4"  /><path d="M21 10V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l2-1.14"  /><path d="M16.5 9.4 7.55 4.24"  /><polyline points="3.29 7 12 12 20.71 7"  /><line y2="12" y1="22" x1="12" x2="12"  />
        </svg>
    }
}

}
pub use r#package_check::PackageCheck;
mod r#toggle_right {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ToggleRight)]
pub fn r#toggle_right(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="1" height="14" width="22" y="5" rx="7" ry="7"  /><circle r="3" cy="12" cx="16"  />
        </svg>
    }
}

}
pub use r#toggle_right::ToggleRight;
mod r#circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Circle)]
pub fn r#circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="12" r="10"  />
        </svg>
    }
}

}
pub use r#circle::Circle;
mod r#baby {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Baby)]
pub fn r#baby(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9 12h0"  /><path d="M15 12h0"  /><path d="M10 16c.5.3 1.2.5 2 .5s1.5-.2 2-.5"  /><path d="M19 6.3a9 9 0 0 1 1.8 3.9 2 2 0 0 1 0 3.6 9 9 0 0 1-17.6 0 2 2 0 0 1 0-3.6A9 9 0 0 1 12 3c2 0 3.5 1.1 3.5 2.5s-.9 2.5-2 2.5c-.8 0-1.5-.4-1.5-1"  />
        </svg>
    }
}

}
pub use r#baby::Baby;
mod r#dice_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Dice2)]
pub fn r#dice_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect width="18" ry="2" height="18" x="3" y="3" rx="2"  /><path d="M15 9h.01"  /><path d="M9 15h.01"  />
        </svg>
    }
}

}
pub use r#dice_2::Dice2;
mod r#figma {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Figma)]
pub fn r#figma(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M5 5.5A3.5 3.5 0 0 1 8.5 2H12v7H8.5A3.5 3.5 0 0 1 5 5.5z"  /><path d="M12 2h3.5a3.5 3.5 0 1 1 0 7H12V2z"  /><path d="M12 12.5a3.5 3.5 0 1 1 7 0 3.5 3.5 0 1 1-7 0z"  /><path d="M5 19.5A3.5 3.5 0 0 1 8.5 16H12v3.5a3.5 3.5 0 1 1-7 0z"  /><path d="M5 12.5A3.5 3.5 0 0 1 8.5 9H12v7H8.5A3.5 3.5 0 0 1 5 12.5z"  />
        </svg>
    }
}

}
pub use r#figma::Figma;
mod r#cloud_sun {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CloudSun)]
pub fn r#cloud_sun(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 2v2"  /><path d="m4.93 4.93 1.41 1.41"  /><path d="M20 12h2"  /><path d="m19.07 4.93-1.41 1.41"  /><path d="M15.947 12.65a4 4 0 0 0-5.925-4.128"  /><path d="M13 22H7a5 5 0 1 1 4.9-6H13a3 3 0 0 1 0 6Z"  />
        </svg>
    }
}

}
pub use r#cloud_sun::CloudSun;
mod r#pause {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Pause)]
pub fn r#pause(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="6" y="4" height="16" width="4"  /><rect y="4" width="4" x="14" height="16"  />
        </svg>
    }
}

}
pub use r#pause::Pause;
mod r#disc {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Disc)]
pub fn r#disc(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" r="10" cy="12"  /><circle cy="12" r="3" cx="12"  />
        </svg>
    }
}

}
pub use r#disc::Disc;
mod r#file_warning {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileWarning)]
pub fn r#file_warning(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><path d="M12 9v4"  /><path d="M12 17h.01"  />
        </svg>
    }
}

}
pub use r#file_warning::FileWarning;
mod r#printer {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Printer)]
pub fn r#printer(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="6 9 6 2 18 2 18 9"  /><path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"  /><rect width="12" x="6" y="14" height="8"  />
        </svg>
    }
}

}
pub use r#printer::Printer;
mod r#package_open {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PackageOpen)]
pub fn r#package_open(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20.91 8.84 8.56 2.23a1.93 1.93 0 0 0-1.81 0L3.1 4.13a2.12 2.12 0 0 0-.05 3.69l12.22 6.93a2 2 0 0 0 1.94 0L21 12.51a2.12 2.12 0 0 0-.09-3.67Z"  /><path d="m3.09 8.84 12.35-6.61a1.93 1.93 0 0 1 1.81 0l3.65 1.9a2.12 2.12 0 0 1 .1 3.69L8.73 14.75a2 2 0 0 1-1.94 0L3 12.51a2.12 2.12 0 0 1 .09-3.67Z"  /><line y2="13" x1="12" x2="12" y1="22"  /><path d="M20 13.5v3.37a2.06 2.06 0 0 1-1.11 1.83l-6 3.08a1.93 1.93 0 0 1-1.78 0l-6-3.08A2.06 2.06 0 0 1 4 16.87V13.5"  />
        </svg>
    }
}

}
pub use r#package_open::PackageOpen;
mod r#cigarette_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CigaretteOff)]
pub fn r#cigarette_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x1="2" y1="2" x2="22" y2="22"  /><path d="M12 12H2v4h14"  /><path d="M22 12v4"  /><path d="M18 12h-.5"  /><path d="M7 12v4"  /><path d="M18 8c0-2.5-2-2.5-2-5"  /><path d="M22 8c0-2.5-2-2.5-2-5"  />
        </svg>
    }
}

}
pub use r#cigarette_off::CigaretteOff;
mod r#paintbrush_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Paintbrush2)]
pub fn r#paintbrush_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14 19.9V16h3a2 2 0 0 0 2-2v-2H5v2c0 1.1.9 2 2 2h3v3.9a2 2 0 1 0 4 0Z"  /><path d="M6 12V2h12v10"  /><path d="M14 2v4"  /><path d="M10 2v2"  />
        </svg>
    }
}

}
pub use r#paintbrush_2::Paintbrush2;
mod r#signal_medium {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SignalMedium)]
pub fn r#signal_medium(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 20h.01"  /><path d="M7 20v-4"  /><path d="M12 20v-8"  />
        </svg>
    }
}

}
pub use r#signal_medium::SignalMedium;
mod r#clapperboard {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clapperboard)]
pub fn r#clapperboard(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 11v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8H4Z"  /><path d="m4 11-.88-2.87a2 2 0 0 1 1.33-2.5l11.48-3.5a2 2 0 0 1 2.5 1.32l.87 2.87L4 11.01Z"  /><path d="m6.6 4.99 3.38 4.2"  /><path d="m11.86 3.38 3.38 4.2"  />
        </svg>
    }
}

}
pub use r#clapperboard::Clapperboard;
mod r#twitch {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Twitch)]
pub fn r#twitch(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 2H3v16h5v4l4-4h5l4-4V2zm-10 9V7m5 4V7"  />
        </svg>
    }
}

}
pub use r#twitch::Twitch;
mod r#list_checks {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ListChecks)]
pub fn r#list_checks(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="6" y1="6" x1="10" x2="21"  /><line x2="21" x1="10" y1="12" y2="12"  /><line y1="18" x1="10" y2="18" x2="21"  /><polyline points="3 6 4 7 6 5"  /><polyline points="3 12 4 13 6 11"  /><polyline points="3 18 4 19 6 17"  />
        </svg>
    }
}

}
pub use r#list_checks::ListChecks;
mod r#file_bar_chart {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileBarChart)]
pub fn r#file_bar_chart(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><path d="M12 18v-4"  /><path d="M8 18v-2"  /><path d="M16 18v-6"  />
        </svg>
    }
}

}
pub use r#file_bar_chart::FileBarChart;
mod r#navigation_2_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Navigation2Off)]
pub fn r#navigation_2_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9.31 9.31 5 21l7-4 7 4-1.17-3.17"  /><path d="M14.53 8.88 12 2l-1.17 3.17"  /><line y1="2" x2="22" y2="22" x1="2"  />
        </svg>
    }
}

}
pub use r#navigation_2_off::Navigation2Off;
mod r#user_check {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(UserCheck)]
pub fn r#user_check(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"  /><circle r="4" cx="9" cy="7"  /><polyline points="16 11 18 13 22 9"  />
        </svg>
    }
}

}
pub use r#user_check::UserCheck;
mod r#locate_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LocateOff)]
pub fn r#locate_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="12" x1="2" x2="5" y1="12"  /><line x1="19" x2="22" y2="12" y1="12"  /><line y1="2" x1="12" x2="12" y2="5"  /><line x1="12" x2="12" y1="19" y2="22"  /><path d="M7.11 7.11C5.83 8.39 5 10.1 5 12c0 3.87 3.13 7 7 7 1.9 0 3.61-.83 4.89-2.11"  /><path d="M18.71 13.96c.19-.63.29-1.29.29-1.96 0-3.87-3.13-7-7-7-.67 0-1.33.1-1.96.29"  /><line x2="22" y2="22" x1="2" y1="2"  />
        </svg>
    }
}

}
pub use r#locate_off::LocateOff;
mod r#smile_plus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SmilePlus)]
pub fn r#smile_plus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M22 11v1a10 10 0 1 1-9-10"  /><path d="M8 14s1.5 2 4 2 4-2 4-2"  /><line x2="9.01" y2="9" y1="9" x1="9"  /><line x2="15.01" y1="9" x1="15" y2="9"  /><path d="M16 5h6"  /><path d="M19 2v6"  />
        </svg>
    }
}

}
pub use r#smile_plus::SmilePlus;
mod r#sliders_horizontal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SlidersHorizontal)]
pub fn r#sliders_horizontal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y1="4" y2="4" x1="21" x2="14"  /><line x2="3" y2="4" x1="10" y1="4"  /><line x2="12" y1="12" x1="21" y2="12"  /><line y1="12" x2="3" x1="8" y2="12"  /><line y1="20" y2="20" x2="16" x1="21"  /><line y2="20" x1="12" x2="3" y1="20"  /><line x1="14" x2="14" y1="2" y2="6"  /><line y2="14" x2="8" y1="10" x1="8"  /><line x2="16" y2="22" y1="18" x1="16"  />
        </svg>
    }
}

}
pub use r#sliders_horizontal::SlidersHorizontal;
mod r#file_scan {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileScan)]
pub fn r#file_scan(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20 10V7.5L14.5 2H6a2 2 0 0 0-2 2v16c0 1.1.9 2 2 2h4.5"  /><polyline points="14 2 14 8 20 8"  /><path d="M16 22a2 2 0 0 1-2-2"  /><path d="M20 22a2 2 0 0 0 2-2"  /><path d="M20 14a2 2 0 0 1 2 2"  /><path d="M16 14a2 2 0 0 0-2 2"  />
        </svg>
    }
}

}
pub use r#file_scan::FileScan;
mod r#file_search {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileSearch)]
pub fn r#file_search(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 22h14a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v3"  /><polyline points="14 2 14 8 20 8"  /><path d="M5 17a3 3 0 1 0 0-6 3 3 0 0 0 0 6z"  /><path d="m9 18-1.5-1.5"  />
        </svg>
    }
}

}
pub use r#file_search::FileSearch;
mod r#arrow_down {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowDown)]
pub fn r#arrow_down(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y1="5" x2="12" x1="12" y2="19"  /><polyline points="19 12 12 19 5 12"  />
        </svg>
    }
}

}
pub use r#arrow_down::ArrowDown;
mod r#folder_check {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderCheck)]
pub fn r#folder_check(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"  /><path d="m9 13 2 2 4-4"  />
        </svg>
    }
}

}
pub use r#folder_check::FolderCheck;
mod r#repeat_1 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Repeat1)]
pub fn r#repeat_1(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m17 2 4 4-4 4"  /><path d="M3 11v-1a4 4 0 0 1 4-4h14"  /><path d="m7 22-4-4 4-4"  /><path d="M21 13v1a4 4 0 0 1-4 4H3"  /><path d="M11 10h1v4"  />
        </svg>
    }
}

}
pub use r#repeat_1::Repeat1;
mod r#file_lock {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileLock)]
pub fn r#file_lock(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><rect rx="1" y="12" height="6" x="8" width="8"  /><path d="M15 12v-2a3 3 0 1 0-6 0v2"  />
        </svg>
    }
}

}
pub use r#file_lock::FileLock;
mod r#file_image {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileImage)]
pub fn r#file_image(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><circle cx="10" cy="13" r="2"  /><path d="m20 17-1.09-1.09a2 2 0 0 0-2.82 0L10 22"  />
        </svg>
    }
}

}
pub use r#file_image::FileImage;
mod r#file_minus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileMinus)]
pub fn r#file_minus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><line x2="15" x1="9" y1="15" y2="15"  />
        </svg>
    }
}

}
pub use r#file_minus::FileMinus;
mod r#octagon {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Octagon)]
pub fn r#octagon(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="7.86 2 16.14 2 22 7.86 22 16.14 16.14 22 7.86 22 2 16.14 2 7.86 7.86 2"  />
        </svg>
    }
}

}
pub use r#octagon::Octagon;
mod r#text_cursor {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(TextCursor)]
pub fn r#text_cursor(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M17 22h-1a4 4 0 0 1-4-4V6a4 4 0 0 1 4-4h1"  /><path d="M7 22h1a4 4 0 0 0 4-4v-1"  /><path d="M7 2h1a4 4 0 0 1 4 4v1"  />
        </svg>
    }
}

}
pub use r#text_cursor::TextCursor;
mod r#activity {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Activity)]
pub fn r#activity(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"  />
        </svg>
    }
}

}
pub use r#activity::Activity;
mod r#check_square {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CheckSquare)]
pub fn r#check_square(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="9 11 12 14 22 4"  /><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"  />
        </svg>
    }
}

}
pub use r#check_square::CheckSquare;
mod r#align_horizontal_space_around {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignHorizontalSpaceAround)]
pub fn r#align_horizontal_space_around(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect rx="2" width="6" height="10" x="9" y="7"  /><path d="M4 22V2"  /><path d="M20 22V2"  />
        </svg>
    }
}

}
pub use r#align_horizontal_space_around::AlignHorizontalSpaceAround;
mod r#map_pin_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MapPinOff)]
pub fn r#map_pin_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M5.43 5.43A8.06 8.06 0 0 0 4 10c0 6 8 12 8 12a29.94 29.94 0 0 0 5-5"  /><path d="M19.18 13.52A8.66 8.66 0 0 0 20 10a8 8 0 0 0-8-8 7.88 7.88 0 0 0-3.52.82"  /><path d="M9.13 9.13A2.78 2.78 0 0 0 9 10a3 3 0 0 0 3 3 2.78 2.78 0 0 0 .87-.13"  /><path d="M14.9 9.25a3 3 0 0 0-2.15-2.16"  /><line x2="22" y2="22" x1="2" y1="2"  />
        </svg>
    }
}

}
pub use r#map_pin_off::MapPinOff;
mod r#locate {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Locate)]
pub fn r#locate(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x2="5" x1="2" y1="12" y2="12"  /><line x2="22" x1="19" y1="12" y2="12"  /><line y2="5" x1="12" x2="12" y1="2"  /><line y1="19" x1="12" y2="22" x2="12"  /><circle cy="12" r="7" cx="12"  />
        </svg>
    }
}

}
pub use r#locate::Locate;
mod r#file_pie_chart {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FilePieChart)]
pub fn r#file_pie_chart(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16 22h2a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v3"  /><polyline points="14 2 14 8 20 8"  /><path d="M4.04 11.71a5.84 5.84 0 1 0 8.2 8.29"  /><path d="M13.83 16A5.83 5.83 0 0 0 8 10.17V16h5.83Z"  />
        </svg>
    }
}

}
pub use r#file_pie_chart::FilePieChart;
mod r#align_center_horizontal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignCenterHorizontal)]
pub fn r#align_center_horizontal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 12h20"  /><path d="M10 16v4a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-4"  /><path d="M10 8V4a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v4"  /><path d="M20 16v1a2 2 0 0 1-2 2h-2a2 2 0 0 1-2-2v-1"  /><path d="M14 8V7c0-1.1.9-2 2-2h2a2 2 0 0 1 2 2v1"  />
        </svg>
    }
}

}
pub use r#align_center_horizontal::AlignCenterHorizontal;
mod r#pizza {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Pizza)]
pub fn r#pizza(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M15 11h.01"  /><path d="M11 15h.01"  /><path d="M16 16h.01"  /><path d="m2 16 20 6-6-20c-3.36.9-6.42 2.67-8.88 5.12A19.876 19.876 0 0 0 2 16Z"  /><path d="M17 6c-6.29 1.47-9.43 5.13-11 11"  />
        </svg>
    }
}

}
pub use r#pizza::Pizza;
mod r#superscript {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Superscript)]
pub fn r#superscript(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m4 19 8-8"  /><path d="m12 19-8-8"  /><path d="M20 12h-4c0-1.5.442-2 1.5-2.5S20 8.334 20 7.002c0-.472-.17-.93-.484-1.29a2.105 2.105 0 0 0-2.617-.436c-.42.239-.738.614-.899 1.06"  />
        </svg>
    }
}

}
pub use r#superscript::Superscript;
mod r#cloud_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CloudOff)]
pub fn r#cloud_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m2 2 20 20"  /><path d="M5.782 5.782A7 7 0 0 0 9 19h8.5a4.5 4.5 0 0 0 1.307-.193"  /><path d="M21.532 16.5A4.5 4.5 0 0 0 17.5 10h-1.79A7.008 7.008 0 0 0 10 5.07"  />
        </svg>
    }
}

}
pub use r#cloud_off::CloudOff;
mod r#user_plus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(UserPlus)]
pub fn r#user_plus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"  /><circle cx="9" cy="7" r="4"  /><line x2="19" y2="14" y1="8" x1="19"  /><line x1="22" y1="11" y2="11" x2="16"  />
        </svg>
    }
}

}
pub use r#user_plus::UserPlus;
mod r#list_start {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ListStart)]
pub fn r#list_start(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16 12H3"  /><path d="M16 18H3"  /><path d="M10 6H3"  /><path d="M21 18V8a2 2 0 0 0-2-2h-5"  /><path d="m16 8-2-2 2-2"  />
        </svg>
    }
}

}
pub use r#list_start::ListStart;
mod r#download_cloud {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(DownloadCloud)]
pub fn r#download_cloud(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242"  /><path d="M12 12v9"  /><path d="m8 17 4 4 4-4"  />
        </svg>
    }
}

}
pub use r#download_cloud::DownloadCloud;
mod r#shield {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Shield)]
pub fn r#shield(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"  />
        </svg>
    }
}

}
pub use r#shield::Shield;
mod r#vibrate_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(VibrateOff)]
pub fn r#vibrate_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m2 8 2 2-2 2 2 2-2 2"  /><path d="m22 8-2 2 2 2-2 2 2 2"  /><path d="M8 8v10c0 .55.45 1 1 1h6c.55 0 1-.45 1-1v-2"  /><path d="M16 10.34V6c0-.55-.45-1-1-1h-4.34"  /><line y1="2" y2="22" x2="22" x1="2"  />
        </svg>
    }
}

}
pub use r#vibrate_off::VibrateOff;
mod r#chevrons_down_up {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ChevronsDownUp)]
pub fn r#chevrons_down_up(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m7 20 5-5 5 5"  /><path d="m7 4 5 5 5-5"  />
        </svg>
    }
}

}
pub use r#chevrons_down_up::ChevronsDownUp;
mod r#graduation_cap {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(GraduationCap)]
pub fn r#graduation_cap(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M22 10v6M2 10l10-5 10 5-10 5z"  /><path d="M6 12v5c3 3 9 3 12 0v-5"  />
        </svg>
    }
}

}
pub use r#graduation_cap::GraduationCap;
mod r#bar_chart_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BarChart2)]
pub fn r#bar_chart_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x1="18" y2="10" y1="20" x2="18"  /><line x2="12" y2="4" x1="12" y1="20"  /><line x1="6" y1="20" x2="6" y2="14"  />
        </svg>
    }
}

}
pub use r#bar_chart_2::BarChart2;
mod r#box {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Box)]
pub fn r#box(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"  /><polyline points="3.29 7 12 12 20.71 7"  /><line x2="12" y1="22" y2="12" x1="12"  />
        </svg>
    }
}

}
pub use r#box::Box;
mod r#align_end_vertical {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignEndVertical)]
pub fn r#align_end_vertical(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="2" height="6" rx="2" y="4" width="16"  /><rect width="9" y="14" height="6" rx="2" x="9"  /><path d="M22 22V2"  />
        </svg>
    }
}

}
pub use r#align_end_vertical::AlignEndVertical;
mod r#clock_9 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clock9)]
pub fn r#clock_9(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" cx="12" r="10"  /><polyline points="12 6 12 12 7.5 12"  />
        </svg>
    }
}

}
pub use r#clock_9::Clock9;
mod r#gavel {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Gavel)]
pub fn r#gavel(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m14 13-7.5 7.5c-.83.83-2.17.83-3 0 0 0 0 0 0 0a2.12 2.12 0 0 1 0-3L11 10"  /><path d="m16 16 6-6"  /><path d="m8 8 6-6"  /><path d="m9 7 8 8"  /><path d="m21 11-8-8"  />
        </svg>
    }
}

}
pub use r#gavel::Gavel;
mod r#hand {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Hand)]
pub fn r#hand(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18 11V6a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v0"  /><path d="M14 10V4a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v2"  /><path d="M10 10.5V6a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v8"  /><path d="M18 8a2 2 0 1 1 4 0v6a8 8 0 0 1-8 8h-2c-2.8 0-4.5-.86-5.99-2.34l-3.6-3.6a2 2 0 0 1 2.83-2.82L7 15"  />
        </svg>
    }
}

}
pub use r#hand::Hand;
mod r#layout {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Layout)]
pub fn r#layout(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"  /><line y1="9" x1="3" y2="9" x2="21"  /><line y1="21" x2="9" y2="9" x1="9"  />
        </svg>
    }
}

}
pub use r#layout::Layout;
mod r#pound_sterling {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PoundSterling)]
pub fn r#pound_sterling(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18 7c0-5.333-8-5.333-8 0"  /><path d="M10 7v14"  /><path d="M6 21h12"  /><path d="M6 13h10"  />
        </svg>
    }
}

}
pub use r#pound_sterling::PoundSterling;
mod r#pen_tool {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PenTool)]
pub fn r#pen_tool(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m12 19 7-7 3 3-7 7-3-3z"  /><path d="m18 13-1.5-7.5L2 2l3.5 14.5L13 18l5-5z"  /><path d="m2 2 7.586 7.586"  /><circle cx="11" cy="11" r="2"  />
        </svg>
    }
}

}
pub use r#pen_tool::PenTool;
mod r#scale {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Scale)]
pub fn r#scale(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m16 16 3-8 3 8c-.87.65-1.92 1-3 1s-2.13-.35-3-1Z"  /><path d="m2 16 3-8 3 8c-.87.65-1.92 1-3 1s-2.13-.35-3-1Z"  /><path d="M7 21h10"  /><path d="M12 3v18"  /><path d="M3 7h2c2 0 5-1 7-2 2 1 5 2 7 2h2"  />
        </svg>
    }
}

}
pub use r#scale::Scale;
mod r#file_question {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileQuestion)]
pub fn r#file_question(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><path d="M10 10.3c.2-.4.5-.8.9-1a2.1 2.1 0 0 1 2.6.4c.3.4.5.8.5 1.3 0 1.3-2 2-2 2"  /><path d="M12 17h.01"  />
        </svg>
    }
}

}
pub use r#file_question::FileQuestion;
mod r#shovel {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Shovel)]
pub fn r#shovel(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 22v-5l5-5 5 5-5 5z"  /><path d="M9.5 14.5 16 8"  /><path d="m17 2 5 5-.5.5a3.53 3.53 0 0 1-5 0s0 0 0 0a3.53 3.53 0 0 1 0-5L17 2"  />
        </svg>
    }
}

}
pub use r#shovel::Shovel;
mod r#table_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Table2)]
pub fn r#table_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9 3H5a2 2 0 0 0-2 2v4m6-6h10a2 2 0 0 1 2 2v4M9 3v18m0 0h10a2 2 0 0 0 2-2V9M9 21H5a2 2 0 0 1-2-2V9m0 0h18"  />
        </svg>
    }
}

}
pub use r#table_2::Table2;
mod r#beaker {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Beaker)]
pub fn r#beaker(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4.5 3h15"  /><path d="M6 3v16a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V3"  /><path d="M6 14h12"  />
        </svg>
    }
}

}
pub use r#beaker::Beaker;
mod r#clock_1 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clock1)]
pub fn r#clock_1(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cx="12" cy="12"  /><polyline points="12 6 12 12 14.5 8"  />
        </svg>
    }
}

}
pub use r#clock_1::Clock1;
mod r#arrow_up_left {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowUpLeft)]
pub fn r#arrow_up_left(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="7" y1="17" x2="7" x1="17"  /><polyline points="7 17 7 7 17 7"  />
        </svg>
    }
}

}
pub use r#arrow_up_left::ArrowUpLeft;
mod r#hourglass {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Hourglass)]
pub fn r#hourglass(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M5 22h14"  /><path d="M5 2h14"  /><path d="M17 22v-4.172a2 2 0 0 0-.586-1.414L12 12l-4.414 4.414A2 2 0 0 0 7 17.828V22"  /><path d="M7 2v4.172a2 2 0 0 0 .586 1.414L12 12l4.414-4.414A2 2 0 0 0 17 6.172V2"  />
        </svg>
    }
}

}
pub use r#hourglass::Hourglass;
mod r#smartphone {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Smartphone)]
pub fn r#smartphone(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="2" height="20" x="5" rx="2" width="14" ry="2"  /><path d="M12 18h.01"  />
        </svg>
    }
}

}
pub use r#smartphone::Smartphone;
mod r#wand {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Wand)]
pub fn r#wand(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M15 4V2"  /><path d="M15 16v-2"  /><path d="M8 9h2"  /><path d="M20 9h2"  /><path d="M17.8 11.8 19 13"  /><path d="M15 9h0"  /><path d="M17.8 6.2 19 5"  /><path d="m3 21 9-9"  /><path d="M12.2 6.2 11 5"  />
        </svg>
    }
}

}
pub use r#wand::Wand;
mod r#bell_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BellOff)]
pub fn r#bell_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M13.73 21a2 2 0 0 1-3.46 0"  /><path d="M18.63 13A17.888 17.888 0 0 1 18 8"  /><path d="M6.26 6.26A5.86 5.86 0 0 0 6 8c0 7-3 9-3 9h14"  /><path d="M18 8a6 6 0 0 0-9.33-5"  /><path d="m2 2 20 20"  />
        </svg>
    }
}

}
pub use r#bell_off::BellOff;
mod r#hand_metal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(HandMetal)]
pub fn r#hand_metal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18 12.5V10a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v1.4"  /><path d="M14 11V9a2 2 0 1 0-4 0v2"  /><path d="M10 10.5V5a2 2 0 1 0-4 0v9"  /><path d="m7 15-1.76-1.76a2 2 0 0 0-2.83 2.82l3.6 3.6C7.5 21.14 9.2 22 12 22h2a8 8 0 0 0 8-8V7a2 2 0 1 0-4 0v5"  />
        </svg>
    }
}

}
pub use r#hand_metal::HandMetal;
mod r#corner_up_left {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CornerUpLeft)]
pub fn r#corner_up_left(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="9 14 4 9 9 4"  /><path d="M20 20v-7a4 4 0 0 0-4-4H4"  />
        </svg>
    }
}

}
pub use r#corner_up_left::CornerUpLeft;
mod r#piggy_bank {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PiggyBank)]
pub fn r#piggy_bank(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M19 5c-1.5 0-2.8 1.4-3 2-3.5-1.5-11-.3-11 5 0 1.8 0 3 2 4.5V20h4v-2h3v2h4v-4c1-.5 1.7-1 2-2h2v-4h-2c0-1-.5-1.5-1-2h0V5z"  /><path d="M2 9v1c0 1.1.9 2 2 2h1"  /><path d="M16 11h0"  />
        </svg>
    }
}

}
pub use r#piggy_bank::PiggyBank;
mod r#carrot {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Carrot)]
pub fn r#carrot(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2.27 21.7s9.87-3.5 12.73-6.36a4.5 4.5 0 0 0-6.36-6.37C5.77 11.84 2.27 21.7 2.27 21.7zM8.64 14l-2.05-2.04M15.34 15l-2.46-2.46"  /><path d="M22 9s-1.33-2-3.5-2C16.86 7 15 9 15 9s1.33 2 3.5 2S22 9 22 9z"  /><path d="M15 2s-2 1.33-2 3.5S15 9 15 9s2-1.84 2-3.5C17 3.33 15 2 15 2z"  />
        </svg>
    }
}

}
pub use r#carrot::Carrot;
mod r#chevrons_right_left {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ChevronsRightLeft)]
pub fn r#chevrons_right_left(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m20 17-5-5 5-5"  /><path d="m4 17 5-5-5-5"  />
        </svg>
    }
}

}
pub use r#chevrons_right_left::ChevronsRightLeft;
mod r#clock_11 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clock11)]
pub fn r#clock_11(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="12" r="10"  /><polyline points="12 6 12 12 9.5 8"  />
        </svg>
    }
}

}
pub use r#clock_11::Clock11;
mod r#folder_up {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderUp)]
pub fn r#folder_up(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"  /><path d="M12 10v6"  /><path d="m9 13 3-3 3 3"  />
        </svg>
    }
}

}
pub use r#folder_up::FolderUp;
mod r#cloudy {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Cloudy)]
pub fn r#cloudy(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M17.5 21H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z"  /><path d="M22 10a3 3 0 0 0-3-3h-2.207a5.502 5.502 0 0 0-10.702.5"  />
        </svg>
    }
}

}
pub use r#cloudy::Cloudy;
mod r#paintbrush {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Paintbrush)]
pub fn r#paintbrush(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18.37 2.63 14 7l-1.59-1.59a2 2 0 0 0-2.82 0L8 7l9 9 1.59-1.59a2 2 0 0 0 0-2.82L17 10l4.37-4.37a2.12 2.12 0 1 0-3-3Z"  /><path d="M9 8c-2 3-4 3.5-7 4l8 10c2-1 6-5 6-7"  /><path d="M14.5 17.5 4.5 15"  />
        </svg>
    }
}

}
pub use r#paintbrush::Paintbrush;
mod r#scaling {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Scaling)]
pub fn r#scaling(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 3 9 15"  /><path d="M12 3H3v18h18v-9"  /><path d="M16 3h5v5"  /><path d="M14 15H9v-5"  />
        </svg>
    }
}

}
pub use r#scaling::Scaling;
mod r#shopping_bag {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ShoppingBag)]
pub fn r#shopping_bag(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M6 2 3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4z"  /><line x1="3" y2="6" y1="6" x2="21"  /><path d="M16 10a4 4 0 0 1-8 0"  />
        </svg>
    }
}

}
pub use r#shopping_bag::ShoppingBag;
mod r#sun {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Sun)]
pub fn r#sun(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" r="4" cy="12"  /><path d="M12 2v2"  /><path d="M12 20v2"  /><path d="m4.93 4.93 1.41 1.41"  /><path d="m17.66 17.66 1.41 1.41"  /><path d="M2 12h2"  /><path d="M20 12h2"  /><path d="m6.34 17.66-1.41 1.41"  /><path d="m19.07 4.93-1.41 1.41"  />
        </svg>
    }
}

}
pub use r#sun::Sun;
mod r#circle_dot {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CircleDot)]
pub fn r#circle_dot(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" r="10" cx="12"  /><circle r="1" cx="12" cy="12"  />
        </svg>
    }
}

}
pub use r#circle_dot::CircleDot;
mod r#link {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Link)]
pub fn r#link(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"  /><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"  />
        </svg>
    }
}

}
pub use r#link::Link;
mod r#calculator {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Calculator)]
pub fn r#calculator(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="4" rx="2" y="2" width="16" height="20"  /><line x1="8" y2="6" y1="6" x2="16"  /><line y2="18" y1="14" x1="16" x2="16"  /><path d="M16 10h.01"  /><path d="M12 10h.01"  /><path d="M8 10h.01"  /><path d="M12 14h.01"  /><path d="M8 14h.01"  /><path d="M12 18h.01"  /><path d="M8 18h.01"  />
        </svg>
    }
}

}
pub use r#calculator::Calculator;
mod r#heart_crack {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(HeartCrack)]
pub fn r#heart_crack(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20.42 4.58a5.4 5.4 0 0 0-7.65 0l-.77.78-.77-.78a5.4 5.4 0 0 0-7.65 0C1.46 6.7 1.33 10.28 4 13l8 8 8-8c2.67-2.72 2.54-6.3.42-8.42z"  /><path d="m12 13-1-1 2-2-3-2.5 2.77-2.92"  />
        </svg>
    }
}

}
pub use r#heart_crack::HeartCrack;
mod r#calendar_check {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CalendarCheck)]
pub fn r#calendar_check(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="3" y="4" height="18" ry="2" width="18" rx="2"  /><line y1="2" x1="16" x2="16" y2="6"  /><line x1="8" y1="2" x2="8" y2="6"  /><line y2="10" x1="3" y1="10" x2="21"  /><path d="m9 16 2 2 4-4"  />
        </svg>
    }
}

}
pub use r#calendar_check::CalendarCheck;
mod r#trending_up {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(TrendingUp)]
pub fn r#trending_up(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="22 7 13.5 15.5 8.5 10.5 2 17"  /><polyline points="16 7 22 7 22 13"  />
        </svg>
    }
}

}
pub use r#trending_up::TrendingUp;
mod r#wallet {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Wallet)]
pub fn r#wallet(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20 12V8H6a2 2 0 0 1-2-2c0-1.1.9-2 2-2h12v4"  /><path d="M4 6v12c0 1.1.9 2 2 2h14v-4"  /><path d="M18 12a2 2 0 0 0-2 2c0 1.1.9 2 2 2h4v-4h-4z"  />
        </svg>
    }
}

}
pub use r#wallet::Wallet;
mod r#waves {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Waves)]
pub fn r#waves(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 6c.6.5 1.2 1 2.5 1C7 7 7 5 9.5 5c2.6 0 2.4 2 5 2 2.5 0 2.5-2 5-2 1.3 0 1.9.5 2.5 1"  /><path d="M2 12c.6.5 1.2 1 2.5 1 2.5 0 2.5-2 5-2 2.6 0 2.4 2 5 2 2.5 0 2.5-2 5-2 1.3 0 1.9.5 2.5 1"  /><path d="M2 18c.6.5 1.2 1 2.5 1 2.5 0 2.5-2 5-2 2.6 0 2.4 2 5 2 2.5 0 2.5-2 5-2 1.3 0 1.9.5 2.5 1"  />
        </svg>
    }
}

}
pub use r#waves::Waves;
mod r#file_audio {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileAudio)]
pub fn r#file_audio(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M17.5 22h.5c.5 0 1-.2 1.4-.6.4-.4.6-.9.6-1.4V7.5L14.5 2H6c-.5 0-1 .2-1.4.6C4.2 3 4 3.5 4 4v3"  /><polyline points="14 2 14 8 20 8"  /><path d="M10 20v-1a2 2 0 1 1 4 0v1a2 2 0 1 1-4 0Z"  /><path d="M6 20v-1a2 2 0 1 0-4 0v1a2 2 0 1 0 4 0Z"  /><path d="M2 19v-3a6 6 0 0 1 12 0v3"  />
        </svg>
    }
}

}
pub use r#file_audio::FileAudio;
mod r#server_cog {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ServerCog)]
pub fn r#server_cog(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M5 10H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2h-1"  /><path d="M5 14H4a2 2 0 0 0-2 2v4a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-4a2 2 0 0 0-2-2h-1"  /><path d="M6 6h.01"  /><path d="M6 18h.01"  /><circle cx="12" cy="12" r="3"  /><path d="M12 8v1"  /><path d="M12 15v1"  /><path d="M16 12h-1"  /><path d="M9 12H8"  /><path d="m15 9-.88.88"  /><path d="M9.88 14.12 9 15"  /><path d="m15 15-.88-.88"  /><path d="M9.88 9.88 9 9"  />
        </svg>
    }
}

}
pub use r#server_cog::ServerCog;
mod r#file_terminal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileTerminal)]
pub fn r#file_terminal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><path d="m8 16 2-2-2-2"  /><path d="M12 18h4"  />
        </svg>
    }
}

}
pub use r#file_terminal::FileTerminal;
mod r#briefcase {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Briefcase)]
pub fn r#briefcase(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="7" rx="2" x="2" height="14" ry="2" width="20"  /><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"  />
        </svg>
    }
}

}
pub use r#briefcase::Briefcase;
mod r#cherry {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Cherry)]
pub fn r#cherry(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 17a5 5 0 0 0 10 0c0-2.76-2.5-5-5-3-2.5-2-5 .24-5 3Z"  /><path d="M12 17a5 5 0 0 0 10 0c0-2.76-2.5-5-5-3-2.5-2-5 .24-5 3Z"  /><path d="M7 14c3.22-2.91 4.29-8.75 5-12 1.66 2.38 4.94 9 5 12"  /><path d="M22 9c-4.29 0-7.14-2.33-10-7 5.71 0 10 4.67 10 7Z"  />
        </svg>
    }
}

}
pub use r#cherry::Cherry;
mod r#sheet {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Sheet)]
pub fn r#sheet(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect width="18" y="3" height="18" rx="2" x="3" ry="2"  /><line x2="21" x1="3" y1="9" y2="9"  /><line x1="3" y2="15" y1="15" x2="21"  /><line x2="9" x1="9" y1="9" y2="21"  /><line y1="9" x2="15" x1="15" y2="21"  />
        </svg>
    }
}

}
pub use r#sheet::Sheet;
mod r#file_box {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileBox)]
pub fn r#file_box(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 22H18a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v4"  /><polyline points="14 2 14 8 20 8"  /><path d="M2.97 13.12c-.6.36-.97 1.02-.97 1.74v3.28c0 .72.37 1.38.97 1.74l3 1.83c.63.39 1.43.39 2.06 0l3-1.83c.6-.36.97-1.02.97-1.74v-3.28c0-.72-.37-1.38-.97-1.74l-3-1.83a1.97 1.97 0 0 0-2.06 0l-3 1.83Z"  /><path d="m7 17-4.74-2.85"  /><path d="m7 17 4.74-2.85"  /><path d="M7 17v5"  />
        </svg>
    }
}

}
pub use r#file_box::FileBox;
mod r#undo_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Undo2)]
pub fn r#undo_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9 14 4 9l5-5"  /><path d="M4 9h10.5a5.5 5.5 0 0 1 5.5 5.5v0a5.5 5.5 0 0 1-5.5 5.5H11"  />
        </svg>
    }
}

}
pub use r#undo_2::Undo2;
mod r#egg_fried {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(EggFried)]
pub fn r#egg_fried(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="11.5" cy="12.5" r="3.5"  /><path d="M3 8c0-3.5 2.5-6 6.5-6 5 0 4.83 3 7.5 5s5 2 5 6c0 4.5-2.5 6.5-7 6.5-2.5 0-2.5 2.5-6 2.5s-7-2-7-5.5c0-3 1.5-3 1.5-5C3.5 10 3 9 3 8Z"  />
        </svg>
    }
}

}
pub use r#egg_fried::EggFried;
mod r#puzzle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Puzzle)]
pub fn r#puzzle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M19.439 7.85c-.049.322.059.648.289.878l1.568 1.568c.47.47.706 1.087.706 1.704s-.235 1.233-.706 1.704l-1.611 1.611a.98.98 0 0 1-.837.276c-.47-.07-.802-.48-.968-.925a2.501 2.501 0 1 0-3.214 3.214c.446.166.855.497.925.968a.979.979 0 0 1-.276.837l-1.61 1.61a2.404 2.404 0 0 1-1.705.707 2.402 2.402 0 0 1-1.704-.706l-1.568-1.568a1.026 1.026 0 0 0-.877-.29c-.493.074-.84.504-1.02.968a2.5 2.5 0 1 1-3.237-3.237c.464-.18.894-.527.967-1.02a1.026 1.026 0 0 0-.289-.877l-1.568-1.568A2.402 2.402 0 0 1 1.998 12c0-.617.236-1.234.706-1.704L4.23 8.77c.24-.24.581-.353.917-.303.515.077.877.528 1.073 1.01a2.5 2.5 0 1 0 3.259-3.259c-.482-.196-.933-.558-1.01-1.073-.05-.336.062-.676.303-.917l1.525-1.525A2.402 2.402 0 0 1 12 1.998c.617 0 1.234.236 1.704.706l1.568 1.568c.23.23.556.338.877.29.493-.074.84-.504 1.02-.968a2.5 2.5 0 1 1 3.237 3.237c-.464.18-.894.527-.967 1.02Z"  />
        </svg>
    }
}

}
pub use r#puzzle::Puzzle;
mod r#external_link {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ExternalLink)]
pub fn r#external_link(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"  /><polyline points="15 3 21 3 21 9"  /><line y1="14" x2="21" x1="10" y2="3"  />
        </svg>
    }
}

}
pub use r#external_link::ExternalLink;
mod r#lasso_select {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LassoSelect)]
pub fn r#lasso_select(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M7 22a5 5 0 0 1-2-4"  /><path d="M7 16.93c.96.43 1.96.74 2.99.91"  /><path d="M3.34 14A6.8 6.8 0 0 1 2 10c0-4.42 4.48-8 10-8s10 3.58 10 8a7.19 7.19 0 0 1-.33 2"  /><path d="M5 18a2 2 0 1 0 0-4 2 2 0 0 0 0 4z"  /><path d="M14.33 22h-.09a.35.35 0 0 1-.24-.32v-10a.34.34 0 0 1 .33-.34c.08 0 .15.03.21.08l7.34 6a.33.33 0 0 1-.21.59h-4.49l-2.57 3.85a.35.35 0 0 1-.28.14v0z"  />
        </svg>
    }
}

}
pub use r#lasso_select::LassoSelect;
mod r#credit_card {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CreditCard)]
pub fn r#credit_card(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect width="20" y="5" height="14" x="2" rx="2"  /><line y1="10" x1="2" x2="22" y2="10"  />
        </svg>
    }
}

}
pub use r#credit_card::CreditCard;
mod r#log_out {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LogOut)]
pub fn r#log_out(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"  /><polyline points="16 17 21 12 16 7"  /><line x1="21" y1="12" x2="9" y2="12"  />
        </svg>
    }
}

}
pub use r#log_out::LogOut;
mod r#frame {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Frame)]
pub fn r#frame(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="6" x1="22" y1="6" x2="2"  /><line x1="22" y1="18" y2="18" x2="2"  /><line x1="6" y1="2" x2="6" y2="22"  /><line x2="18" x1="18" y1="2" y2="22"  />
        </svg>
    }
}

}
pub use r#frame::Frame;
mod r#history {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(History)]
pub fn r#history(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 3v5h5"  /><path d="M3.05 13A9 9 0 1 0 6 5.3L3 8"  /><path d="M12 7v5l4 2"  />
        </svg>
    }
}

}
pub use r#history::History;
mod r#minus_circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MinusCircle)]
pub fn r#minus_circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="12" r="10"  /><line y2="12" y1="12" x1="8" x2="16"  />
        </svg>
    }
}

}
pub use r#minus_circle::MinusCircle;
mod r#folder_output {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderOutput)]
pub fn r#folder_output(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M2 7.5V5c0-1.1.9-2 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H20a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H2"  /><path d="M2 13h10"  /><path d="m5 10-3 3 3 3"  />
        </svg>
    }
}

}
pub use r#folder_output::FolderOutput;
mod r#text_cursor_input {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(TextCursorInput)]
pub fn r#text_cursor_input(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M13 20h-1a3 3 0 0 1-3-3V7a3 3 0 0 1 3-3h1"  /><path d="M5 4h1a3 3 0 0 1 3 3v10a3 3 0 0 1-3 3H5"  /><path d="M13.1 7.9h6.8A2.18 2.18 0 0 1 22 10v4a2.11 2.11 0 0 1-2.1 2.1h-6.8"  /><path d="M4.8 16.1h-.7A2.18 2.18 0 0 1 2 14v-4a2.18 2.18 0 0 1 2.1-2.1h.7"  />
        </svg>
    }
}

}
pub use r#text_cursor_input::TextCursorInput;
mod r#mouse {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Mouse)]
pub fn r#mouse(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="3" x="6" height="18" rx="6" width="12"  /><path d="M12 7v4"  />
        </svg>
    }
}

}
pub use r#mouse::Mouse;
mod r#rewind {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Rewind)]
pub fn r#rewind(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="11 19 2 12 11 5 11 19"  /><polygon points="22 19 13 12 22 5 22 19"  />
        </svg>
    }
}

}
pub use r#rewind::Rewind;
mod r#trending_down {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(TrendingDown)]
pub fn r#trending_down(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="22 17 13.5 8.5 8.5 13.5 2 7"  /><polyline points="16 17 22 17 22 11"  />
        </svg>
    }
}

}
pub use r#trending_down::TrendingDown;
mod r#folder_edit {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderEdit)]
pub fn r#folder_edit(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M8.42 10.61a2.1 2.1 0 1 1 2.97 2.97L5.95 19 2 20l.99-3.95 5.43-5.44Z"  /><path d="M2 11.5V5c0-1.1.9-2 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H20a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2h-9.5"  />
        </svg>
    }
}

}
pub use r#folder_edit::FolderEdit;
mod r#clipboard_list {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ClipboardList)]
pub fn r#clipboard_list(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="8" y="2" ry="1" width="8" height="4" rx="1"  /><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"  /><path d="M12 11h4"  /><path d="M12 16h4"  /><path d="M8 11h.01"  /><path d="M8 16h.01"  />
        </svg>
    }
}

}
pub use r#clipboard_list::ClipboardList;
mod r#play_circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PlayCircle)]
pub fn r#play_circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cx="12" cy="12"  /><polygon points="10 8 16 12 10 16 10 8"  />
        </svg>
    }
}

}
pub use r#play_circle::PlayCircle;
mod r#cog {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Cog)]
pub fn r#cog(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 20a8 8 0 1 0 0-16 8 8 0 0 0 0 16Z"  /><path d="M12 14a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z"  /><path d="M12 2v2"  /><path d="M12 22v-2"  /><path d="m17 20.66-1-1.73"  /><path d="M11 10.27 7 3.34"  /><path d="m20.66 17-1.73-1"  /><path d="m3.34 7 1.73 1"  /><path d="M14 12h8"  /><path d="M2 12h2"  /><path d="m20.66 7-1.73 1"  /><path d="m3.34 17 1.73-1"  /><path d="m17 3.34-1 1.73"  /><path d="m11 13.73-4 6.93"  />
        </svg>
    }
}

}
pub use r#cog::Cog;
mod r#bold {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Bold)]
pub fn r#bold(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"  /><path d="M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"  />
        </svg>
    }
}

}
pub use r#bold::Bold;
mod r#hexagon {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Hexagon)]
pub fn r#hexagon(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"  />
        </svg>
    }
}

}
pub use r#hexagon::Hexagon;
mod r#file_x_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileX2)]
pub fn r#file_x_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 22h14a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v4"  /><path d="M14 2v6h6"  /><path d="m3 12.5 5 5"  /><path d="m8 12.5-5 5"  />
        </svg>
    }
}

}
pub use r#file_x_2::FileX2;
mod r#alarm_minus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlarmMinus)]
pub fn r#alarm_minus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 21a8 8 0 1 0 0-16 8 8 0 0 0 0 16z"  /><path d="M5 3 2 6"  /><path d="m22 6-3-3"  /><path d="m6 19-2 2"  /><path d="m18 19 2 2"  /><path d="M9 13h6"  />
        </svg>
    }
}

}
pub use r#alarm_minus::AlarmMinus;
mod r#file_check {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileCheck)]
pub fn r#file_check(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"  /><polyline points="14 2 14 8 20 8"  /><path d="m9 15 2 2 4-4"  />
        </svg>
    }
}

}
pub use r#file_check::FileCheck;
mod r#dollar_sign {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(DollarSign)]
pub fn r#dollar_sign(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x2="12" y2="22" x1="12" y1="2"  /><path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"  />
        </svg>
    }
}

}
pub use r#dollar_sign::DollarSign;
mod r#align_end_horizontal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignEndHorizontal)]
pub fn r#align_end_horizontal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect width="6" y="2" rx="2" x="4" height="16"  /><rect x="14" y="9" rx="2" height="9" width="6"  /><path d="M22 22H2"  />
        </svg>
    }
}

}
pub use r#align_end_horizontal::AlignEndHorizontal;
mod r#coffee {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Coffee)]
pub fn r#coffee(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M17 8h1a4 4 0 1 1 0 8h-1"  /><path d="M3 8h14v9a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4Z"  /><line y2="4" x1="6" y1="2" x2="6"  /><line x2="10" y2="4" y1="2" x1="10"  /><line x1="14" y2="4" x2="14" y1="2"  />
        </svg>
    }
}

}
pub use r#coffee::Coffee;
mod r#more_horizontal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MoreHorizontal)]
pub fn r#more_horizontal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="1" cx="12" cy="12"  /><circle r="1" cx="19" cy="12"  /><circle cy="12" cx="5" r="1"  />
        </svg>
    }
}

}
pub use r#more_horizontal::MoreHorizontal;
mod r#tent {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Tent)]
pub fn r#tent(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M19 20 10 4"  /><path d="m5 20 9-16"  /><path d="M3 20h18"  /><path d="m12 15-3 5"  /><path d="m12 15 3 5"  />
        </svg>
    }
}

}
pub use r#tent::Tent;
mod r#play {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Play)]
pub fn r#play(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="5 3 19 12 5 21 5 3"  />
        </svg>
    }
}

}
pub use r#play::Play;
mod r#archive {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Archive)]
pub fn r#archive(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="2" rx="2" y="4" width="20" height="5"  /><path d="M4 9v9a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9"  /><path d="M10 13h4"  />
        </svg>
    }
}

}
pub use r#archive::Archive;
mod r#martini {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Martini)]
pub fn r#martini(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M8 22h8"  /><path d="M12 11v11"  /><path d="m19 3-7 8-7-8Z"  />
        </svg>
    }
}

}
pub use r#martini::Martini;
mod r#chef_hat {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ChefHat)]
pub fn r#chef_hat(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M6 13.87A4 4 0 0 1 7.41 6a5.11 5.11 0 0 1 1.05-1.54 5 5 0 0 1 7.08 0A5.11 5.11 0 0 1 16.59 6 4 4 0 0 1 18 13.87V21H6Z"  /><line y2="17" x1="6" x2="18" y1="17"  />
        </svg>
    }
}

}
pub use r#chef_hat::ChefHat;
mod r#dice_6 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Dice6)]
pub fn r#dice_6(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect height="18" rx="2" x="3" y="3" width="18" ry="2"  /><path d="M16 8h.01"  /><path d="M16 12h.01"  /><path d="M16 16h.01"  /><path d="M8 8h.01"  /><path d="M8 12h.01"  /><path d="M8 16h.01"  />
        </svg>
    }
}

}
pub use r#dice_6::Dice6;
mod r#git_pull_request {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(GitPullRequest)]
pub fn r#git_pull_request(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="18" r="3" cy="18"  /><circle r="3" cx="6" cy="6"  /><path d="M13 6h3a2 2 0 0 1 2 2v7"  /><line x1="6" x2="6" y2="21" y1="9"  />
        </svg>
    }
}

}
pub use r#git_pull_request::GitPullRequest;
mod r#indent {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Indent)]
pub fn r#indent(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="3 8 7 12 3 16"  /><line y2="12" x1="21" x2="11" y1="12"  /><line x1="21" y1="6" x2="11" y2="6"  /><line y1="18" y2="18" x1="21" x2="11"  />
        </svg>
    }
}

}
pub use r#indent::Indent;
mod r#inspect {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Inspect)]
pub fn r#inspect(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 11V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h6"  /><path d="m12 12 4 10 1.7-4.3L22 16Z"  />
        </svg>
    }
}

}
pub use r#inspect::Inspect;
mod r#folder_archive {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderArchive)]
pub fn r#folder_archive(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M22 20V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2h6"  /><circle cy="19" cx="16" r="2"  /><path d="M16 11v-1"  /><path d="M16 17v-2"  />
        </svg>
    }
}

}
pub use r#folder_archive::FolderArchive;
mod r#lamp_floor {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LampFloor)]
pub fn r#lamp_floor(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9 2h6l3 7H6l3-7Z"  /><path d="M12 9v13"  /><path d="M9 22h6"  />
        </svg>
    }
}

}
pub use r#lamp_floor::LampFloor;
mod r#fast_forward {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FastForward)]
pub fn r#fast_forward(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="13 19 22 12 13 5 13 19"  /><polygon points="2 19 11 12 2 5 2 19"  />
        </svg>
    }
}

}
pub use r#fast_forward::FastForward;
mod r#minimize_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Minimize2)]
pub fn r#minimize_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="4 14 10 14 10 20"  /><polyline points="20 10 14 10 14 4"  /><line x1="14" x2="21" y1="10" y2="3"  /><line y1="21" x1="3" x2="10" y2="14"  />
        </svg>
    }
}

}
pub use r#minimize_2::Minimize2;
mod r#maximize {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Maximize)]
pub fn r#maximize(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M8 3H5a2 2 0 0 0-2 2v3"  /><path d="M21 8V5a2 2 0 0 0-2-2h-3"  /><path d="M3 16v3a2 2 0 0 0 2 2h3"  /><path d="M16 21h3a2 2 0 0 0 2-2v-3"  />
        </svg>
    }
}

}
pub use r#maximize::Maximize;
mod r#monitor_speaker {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MonitorSpeaker)]
pub fn r#monitor_speaker(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M5.5 20H8"  /><path d="M17 9h.01"  /><rect x="12" width="10" height="16" rx="2" y="4"  /><path d="M8 6H4a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2h4"  /><circle cx="17" cy="15" r="1"  />
        </svg>
    }
}

}
pub use r#monitor_speaker::MonitorSpeaker;
mod r#battery_medium {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BatteryMedium)]
pub fn r#battery_medium(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect height="10" ry="2" rx="2" x="2" width="16" y="7"  /><line y2="13" x2="22" x1="22" y1="11"  /><line x2="6" y1="11" x1="6" y2="13"  /><line x1="10" y1="11" y2="13" x2="10"  />
        </svg>
    }
}

}
pub use r#battery_medium::BatteryMedium;
mod r#megaphone {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Megaphone)]
pub fn r#megaphone(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m3 11 18-5v12L3 14v-3z"  /><path d="M11.6 16.8a3 3 0 1 1-5.8-1.6"  />
        </svg>
    }
}

}
pub use r#megaphone::Megaphone;
mod r#arrow_down_circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ArrowDownCircle)]
pub fn r#arrow_down_circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cy="12" cx="12"  /><polyline points="8 12 12 16 16 12"  /><line x1="12" x2="12" y2="16" y1="8"  />
        </svg>
    }
}

}
pub use r#arrow_down_circle::ArrowDownCircle;
mod r#shield_close {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ShieldClose)]
pub fn r#shield_close(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"  /><line x1="9.5" y2="14" y1="9" x2="14.5"  /><line x2="9.5" x1="14.5" y1="9" y2="14"  />
        </svg>
    }
}

}
pub use r#shield_close::ShieldClose;
mod r#syringe {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Syringe)]
pub fn r#syringe(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m18 2 4 4"  /><path d="m17 7 3-3"  /><path d="M19 9 8.7 19.3c-1 1-2.5 1-3.4 0l-.6-.6c-1-1-1-2.5 0-3.4L15 5"  /><path d="m9 11 4 4"  /><path d="m5 19-3 3"  /><path d="m14 4 6 6"  />
        </svg>
    }
}

}
pub use r#syringe::Syringe;
mod r#calendar_minus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CalendarMinus)]
pub fn r#calendar_minus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 13V6a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h8"  /><line y1="2" x1="16" x2="16" y2="6"  /><line y2="6" y1="2" x1="8" x2="8"  /><line x1="3" x2="21" y2="10" y1="10"  /><line y2="19" x1="16" y1="19" x2="22"  />
        </svg>
    }
}

}
pub use r#calendar_minus::CalendarMinus;
mod r#clock_10 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Clock10)]
pub fn r#clock_10(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="10" cy="12" cx="12"  /><polyline points="12 6 12 12 8 10"  />
        </svg>
    }
}

}
pub use r#clock_10::Clock10;
mod r#scissors {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Scissors)]
pub fn r#scissors(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="3" cx="6" cy="6"  /><circle cy="18" r="3" cx="6"  /><line x2="8.12" y2="15.88" x1="20" y1="4"  /><line x2="20" x1="14.47" y1="14.48" y2="20"  /><line x2="12" x1="8.12" y1="8.12" y2="12"  />
        </svg>
    }
}

}
pub use r#scissors::Scissors;
mod r#link_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Link2)]
pub fn r#link_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9 17H7A5 5 0 0 1 7 7h2"  /><path d="M15 7h2a5 5 0 1 1 0 10h-2"  /><line x2="16" y2="12" x1="8" y1="12"  />
        </svg>
    }
}

}
pub use r#link_2::Link2;
mod r#code_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Code2)]
pub fn r#code_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m18 16 4-4-4-4"  /><path d="m6 8-4 4 4 4"  /><path d="m14.5 4-5 16"  />
        </svg>
    }
}

}
pub use r#code_2::Code2;
mod r#import {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Import)]
pub fn r#import(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 3v12"  /><path d="m8 11 4 4 4-4"  /><path d="M8 5H4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-4"  />
        </svg>
    }
}

}
pub use r#import::Import;
mod r#align_vertical_space_between {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignVerticalSpaceBetween)]
pub fn r#align_vertical_space_between(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="5" y="15" width="14" height="6" rx="2"  /><rect width="10" y="3" height="6" rx="2" x="7"  /><path d="M2 21h20"  /><path d="M2 3h20"  />
        </svg>
    }
}

}
pub use r#align_vertical_space_between::AlignVerticalSpaceBetween;
mod r#asterisk {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Asterisk)]
pub fn r#asterisk(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 6v12"  /><path d="M17.196 9 6.804 15"  /><path d="m6.804 9 10.392 6"  />
        </svg>
    }
}

}
pub use r#asterisk::Asterisk;
mod r#flashlight {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Flashlight)]
pub fn r#flashlight(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18 6c0 2-2 2-2 4v10a2 2 0 0 1-2 2h-4a2 2 0 0 1-2-2V10c0-2-2-2-2-4V2h12z"  /><line x1="6" y1="6" y2="6" x2="18"  /><line y2="12" x1="12" x2="12" y1="12"  />
        </svg>
    }
}

}
pub use r#flashlight::Flashlight;
mod r#move {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Move)]
pub fn r#move(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="5 9 2 12 5 15"  /><polyline points="9 5 12 2 15 5"  /><polyline points="15 19 12 22 9 19"  /><polyline points="19 9 22 12 19 15"  /><line y2="12" x2="22" x1="2" y1="12"  /><line y1="2" x1="12" x2="12" y2="22"  />
        </svg>
    }
}

}
pub use r#move::Move;
mod r#cloud_rain_wind {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CloudRainWind)]
pub fn r#cloud_rain_wind(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242"  /><path d="m9.2 22 3-7"  /><path d="m9 13-3 7"  /><path d="m17 13-3 7"  />
        </svg>
    }
}

}
pub use r#cloud_rain_wind::CloudRainWind;
mod r#trash {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Trash)]
pub fn r#trash(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 6h18"  /><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"  /><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"  />
        </svg>
    }
}

}
pub use r#trash::Trash;
mod r#file_video_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileVideo2)]
pub fn r#file_video_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 8V4a2 2 0 0 1 2-2h8.5L20 7.5V20a2 2 0 0 1-2 2H4"  /><polyline points="14 2 14 8 20 8"  /><path d="m10 15.5 4 2.5v-6l-4 2.5"  /><rect width="8" y="12" height="6" x="2" rx="1"  />
        </svg>
    }
}

}
pub use r#file_video_2::FileVideo2;
mod r#cloud_fog {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CloudFog)]
pub fn r#cloud_fog(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242"  /><path d="M16 17H7"  /><path d="M17 21H9"  />
        </svg>
    }
}

}
pub use r#cloud_fog::CloudFog;
mod r#file_plus_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FilePlus2)]
pub fn r#file_plus_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 22h14a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v4"  /><polyline points="14 2 14 8 20 8"  /><path d="M3 15h6"  /><path d="M6 12v6"  />
        </svg>
    }
}

}
pub use r#file_plus_2::FilePlus2;
mod r#languages {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Languages)]
pub fn r#languages(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m5 8 6 6"  /><path d="m4 14 6-6 2-3"  /><path d="M2 5h12"  /><path d="M7 2h1"  /><path d="m22 22-5-10-5 10"  /><path d="M14 18h6"  />
        </svg>
    }
}

}
pub use r#languages::Languages;
mod r#dice_4 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Dice4)]
pub fn r#dice_4(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect width="18" height="18" x="3" ry="2" rx="2" y="3"  /><path d="M16 8h.01"  /><path d="M8 8h.01"  /><path d="M8 16h.01"  /><path d="M16 16h.01"  />
        </svg>
    }
}

}
pub use r#dice_4::Dice4;
mod r#bell_plus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BellPlus)]
pub fn r#bell_plus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18.387 12C19.198 15.799 21 17 21 17H3s3-2 3-9a6 6 0 0 1 7-5.916"  /><path d="M13.73 21a2 2 0 0 1-3.46 0"  /><path d="M18 2v6"  /><path d="M21 5h-6"  />
        </svg>
    }
}

}
pub use r#bell_plus::BellPlus;
mod r#bitcoin {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Bitcoin)]
pub fn r#bitcoin(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M11.767 19.089c4.924.868 6.14-6.025 1.216-6.894m-1.216 6.894L5.86 18.047m5.908 1.042-.347 1.97m1.563-8.864c4.924.869 6.14-6.025 1.215-6.893m-1.215 6.893-3.94-.694m5.155-6.2L8.29 4.26m5.908 1.042.348-1.97M7.48 20.364l3.126-17.727"  />
        </svg>
    }
}

}
pub use r#bitcoin::Bitcoin;
mod r#calendar_plus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CalendarPlus)]
pub fn r#calendar_plus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 13V6a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h8"  /><line y1="2" y2="6" x1="16" x2="16"  /><line y1="2" x2="8" x1="8" y2="6"  /><line y2="10" x1="3" y1="10" x2="21"  /><line y1="16" x1="19" x2="19" y2="22"  /><line x1="16" x2="22" y1="19" y2="19"  />
        </svg>
    }
}

}
pub use r#calendar_plus::CalendarPlus;
mod r#eye_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(EyeOff)]
pub fn r#eye_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9.88 9.88a3 3 0 1 0 4.24 4.24"  /><path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"  /><path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"  /><line x2="22" y2="22" x1="2" y1="2"  />
        </svg>
    }
}

}
pub use r#eye_off::EyeOff;
mod r#frown {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Frown)]
pub fn r#frown(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" r="10" cx="12"  /><path d="M16 16s-1.5-2-4-2-4 2-4 2"  /><line y1="9" x2="9.01" y2="9" x1="9"  /><line y2="9" x1="15" x2="15.01" y1="9"  />
        </svg>
    }
}

}
pub use r#frown::Frown;
mod r#git_branch {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(GitBranch)]
pub fn r#git_branch(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y1="3" x1="6" y2="15" x2="6"  /><circle cx="18" cy="6" r="3"  /><circle cx="6" r="3" cy="18"  /><path d="M18 9a9 9 0 0 1-9 9"  />
        </svg>
    }
}

}
pub use r#git_branch::GitBranch;
mod r#image_plus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ImagePlus)]
pub fn r#image_plus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h7"  /><line x2="22" y2="5" y1="5" x1="16"  /><line x1="19" y1="2" x2="19" y2="8"  /><circle cy="9" r="2" cx="9"  /><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"  />
        </svg>
    }
}

}
pub use r#image_plus::ImagePlus;
mod r#cup_soda {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CupSoda)]
pub fn r#cup_soda(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m6 8 1.75 12.28a2 2 0 0 0 2 1.72h4.54a2 2 0 0 0 2-1.72L18 8"  /><path d="M5 8h14"  /><path d="M7 15a6.47 6.47 0 0 1 5 0 6.47 6.47 0 0 0 5 0"  /><path d="m12 8 1-6h2"  />
        </svg>
    }
}

}
pub use r#cup_soda::CupSoda;
mod r#list_ordered {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ListOrdered)]
pub fn r#list_ordered(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x1="10" y1="6" x2="21" y2="6"  /><line x2="21" x1="10" y2="12" y1="12"  /><line y1="18" y2="18" x1="10" x2="21"  /><path d="M4 6h1v4"  /><path d="M4 10h2"  /><path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1"  />
        </svg>
    }
}

}
pub use r#list_ordered::ListOrdered;
mod r#loader {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Loader)]
pub fn r#loader(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="6" x2="12" x1="12" y1="2"  /><line y1="18" y2="22" x1="12" x2="12"  /><line x2="7.76" y1="4.93" x1="4.93" y2="7.76"  /><line y2="19.07" x1="16.24" x2="19.07" y1="16.24"  /><line x1="2" y1="12" x2="6" y2="12"  /><line y1="12" y2="12" x1="18" x2="22"  /><line x2="7.76" x1="4.93" y2="16.24" y1="19.07"  /><line x2="19.07" x1="16.24" y1="7.76" y2="4.93"  />
        </svg>
    }
}

}
pub use r#loader::Loader;
mod r#bluetooth {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Bluetooth)]
pub fn r#bluetooth(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m7 7 10 10-5 5V2l5 5L7 17"  />
        </svg>
    }
}

}
pub use r#bluetooth::Bluetooth;
mod r#armchair {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Armchair)]
pub fn r#armchair(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M19 9V6a2 2 0 0 0-2-2H7a2 2 0 0 0-2 2v3"  /><path d="M3 11v5a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-5a2 2 0 0 0-4 0v2H7v-2a2 2 0 0 0-4 0Z"  /><path d="M5 18v2"  /><path d="M19 18v2"  />
        </svg>
    }
}

}
pub use r#armchair::Armchair;
mod r#minus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Minus)]
pub fn r#minus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line y2="12" x1="5" y1="12" x2="19"  />
        </svg>
    }
}

}
pub use r#minus::Minus;
mod r#regex {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Regex)]
pub fn r#regex(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M17 3v10"  /><path d="m12.67 5.5 8.66 5"  /><path d="m12.67 10.5 8.66-5"  /><path d="M9 17a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v2a2 2 0 0 0 2 2h2a2 2 0 0 0 2-2v-2z"  />
        </svg>
    }
}

}
pub use r#regex::Regex;
mod r#github {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Github)]
pub fn r#github(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"  /><path d="M9 18c-4.51 2-5-2-7-2"  />
        </svg>
    }
}

}
pub use r#github::Github;
mod r#trees {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Trees)]
pub fn r#trees(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10 10v.2A3 3 0 0 1 8.9 16v0H5v0h0a3 3 0 0 1-1-5.8V10a3 3 0 0 1 6 0Z"  /><path d="M7 16v6"  /><path d="M13 19v3"  /><path d="M12 19h8.3a1 1 0 0 0 .7-1.7L18 14h.3a1 1 0 0 0 .7-1.7L16 9h.2a1 1 0 0 0 .8-1.7L13 3l-1.4 1.5"  />
        </svg>
    }
}

}
pub use r#trees::Trees;
mod r#x_octagon {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(XOctagon)]
pub fn r#x_octagon(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polygon points="7.86 2 16.14 2 22 7.86 22 16.14 16.14 22 7.86 22 2 16.14 2 7.86 7.86 2"  /><line y1="9" x1="15" x2="9" y2="15"  /><line y2="15" x2="15" x1="9" y1="9"  />
        </svg>
    }
}

}
pub use r#x_octagon::XOctagon;
mod r#folder_tree {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderTree)]
pub fn r#folder_tree(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M13 10h7a1 1 0 0 0 1-1V6a1 1 0 0 0-1-1h-2.5a1 1 0 0 1-.8-.4l-.9-1.2A1 1 0 0 0 15 3h-2a1 1 0 0 0-1 1v5a1 1 0 0 0 1 1Z"  /><path d="M13 21h7a1 1 0 0 0 1-1v-3a1 1 0 0 0-1-1h-2.88a1 1 0 0 1-.9-.55l-.44-.9a1 1 0 0 0-.9-.55H13a1 1 0 0 0-1 1v5a1 1 0 0 0 1 1Z"  /><path d="M3 3v2c0 1.1.9 2 2 2h3"  /><path d="M3 3v13c0 1.1.9 2 2 2h3"  />
        </svg>
    }
}

}
pub use r#folder_tree::FolderTree;
mod r#edit {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Edit)]
pub fn r#edit(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"  /><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"  />
        </svg>
    }
}

}
pub use r#edit::Edit;
mod r#code {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Code)]
pub fn r#code(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="16 18 22 12 16 6"  /><polyline points="8 6 2 12 8 18"  />
        </svg>
    }
}

}
pub use r#code::Code;
mod r#file_cog {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileCog)]
pub fn r#file_cog(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 6V4a2 2 0 0 1 2-2h8.5L20 7.5V20a2 2 0 0 1-2 2H4"  /><polyline points="14 2 14 8 20 8"  /><circle r="3" cy="14" cx="6"  /><path d="M6 10v1"  /><path d="M6 17v1"  /><path d="M10 14H9"  /><path d="M3 14H2"  /><path d="m9 11-.88.88"  /><path d="M3.88 16.12 3 17"  /><path d="m9 17-.88-.88"  /><path d="M3.88 11.88 3 11"  />
        </svg>
    }
}

}
pub use r#file_cog::FileCog;
mod r#flag {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Flag)]
pub fn r#flag(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z"  /><line y2="15" x2="4" y1="22" x1="4"  />
        </svg>
    }
}

}
pub use r#flag::Flag;
mod r#package_minus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PackageMinus)]
pub fn r#package_minus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16 16h6"  /><path d="M21 10V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l2-1.14"  /><path d="M16.5 9.4 7.55 4.24"  /><polyline points="3.29 7 12 12 20.71 7"  /><line y1="22" y2="12" x2="12" x1="12"  />
        </svg>
    }
}

}
pub use r#package_minus::PackageMinus;
mod r#message_circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(MessageCircle)]
pub fn r#message_circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"  />
        </svg>
    }
}

}
pub use r#message_circle::MessageCircle;
mod r#dribbble {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Dribbble)]
pub fn r#dribbble(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="12" r="10"  /><path d="M19.13 5.09C15.22 9.14 10 10.44 2.25 10.94"  /><path d="M21.75 12.84c-6.62-1.41-12.14 1-16.38 6.32"  /><path d="M8.56 2.75c4.37 6 6 9.42 8 17.72"  />
        </svg>
    }
}

}
pub use r#dribbble::Dribbble;
mod r#fuel {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Fuel)]
pub fn r#fuel(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x1="3" y2="22" y1="22" x2="15"  /><line x2="14" y1="9" x1="4" y2="9"  /><path d="M14 22V4a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v18"  /><path d="M14 13h2a2 2 0 0 1 2 2v2a2 2 0 0 0 2 2h0a2 2 0 0 0 2-2V9.83a2 2 0 0 0-.59-1.42L18 5"  />
        </svg>
    }
}

}
pub use r#fuel::Fuel;
mod r#rotate_3_d {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Rotate3D)]
pub fn r#rotate_3_d(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M16.466 7.5C15.643 4.237 13.952 2 12 2 9.239 2 7 6.477 7 12s2.239 10 5 10c.342 0 .677-.069 1-.2"  /><path d="m15.194 13.707 3.814 1.86-1.86 3.814"  /><path d="M19 15.57c-1.804.885-4.274 1.43-7 1.43-5.523 0-10-2.239-10-5s4.477-5 10-5c4.838 0 8.873 1.718 9.8 4"  />
        </svg>
    }
}

}
pub use r#rotate_3_d::Rotate3D;
mod r#battery {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Battery)]
pub fn r#battery(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect width="16" rx="2" y="7" ry="2" height="10" x="2"  /><line x2="22" y1="11" y2="13" x1="22"  />
        </svg>
    }
}

}
pub use r#battery::Battery;
mod r#shopping_cart {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ShoppingCart)]
pub fn r#shopping_cart(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="21" cx="8" r="1"  /><circle cy="21" cx="19" r="1"  /><path d="M2.05 2.05h2l2.66 12.42a2 2 0 0 0 2 1.58h9.78a2 2 0 0 0 1.95-1.57l1.65-7.43H5.12"  />
        </svg>
    }
}

}
pub use r#shopping_cart::ShoppingCart;
mod r#info {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Info)]
pub fn r#info(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" r="10" cx="12"  /><line y1="16" x2="12" x1="12" y2="12"  /><line x1="12" x2="12.01" y1="8" y2="8"  />
        </svg>
    }
}

}
pub use r#info::Info;
mod r#clipboard_type {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ClipboardType)]
pub fn r#clipboard_type(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="8" y="2" width="8" rx="1" height="4" ry="1"  /><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"  /><path d="M9 12v-1h6v1"  /><path d="M11 17h2"  /><path d="M12 11v6"  />
        </svg>
    }
}

}
pub use r#clipboard_type::ClipboardType;
mod r#align_vertical_justify_start {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignVerticalJustifyStart)]
pub fn r#align_vertical_justify_start(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="5" y="16" height="6" rx="2" width="14"  /><rect y="6" x="7" height="6" width="10" rx="2"  /><path d="M2 2h20"  />
        </svg>
    }
}

}
pub use r#align_vertical_justify_start::AlignVerticalJustifyStart;
mod r#layout_dashboard {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LayoutDashboard)]
pub fn r#layout_dashboard(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect height="9" y="3" width="7" x="3"  /><rect width="7" height="5" x="14" y="3"  /><rect height="9" y="12" width="7" x="14"  /><rect x="3" width="7" height="5" y="16"  />
        </svg>
    }
}

}
pub use r#layout_dashboard::LayoutDashboard;
mod r#corner_down_right {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CornerDownRight)]
pub fn r#corner_down_right(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="15 10 20 15 15 20"  /><path d="M4 4v7a4 4 0 0 0 4 4h12"  />
        </svg>
    }
}

}
pub use r#corner_down_right::CornerDownRight;
mod r#link_2_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Link2Off)]
pub fn r#link_2_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M9 17H7A5 5 0 0 1 7 7"  /><path d="M15 7h2a5 5 0 0 1 4 8"  /><line x2="12" y1="12" y2="12" x1="8"  /><line y1="2" y2="22" x2="22" x1="2"  />
        </svg>
    }
}

}
pub use r#link_2_off::Link2Off;
mod r#alarm_check {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlarmCheck)]
pub fn r#alarm_check(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 21a8 8 0 1 0 0-16 8 8 0 0 0 0 16z"  /><path d="M5 3 2 6"  /><path d="m22 6-3-3"  /><path d="m6 19-2 2"  /><path d="m18 19 2 2"  /><path d="m9 13 2 2 4-4"  />
        </svg>
    }
}

}
pub use r#alarm_check::AlarmCheck;
mod r#file_code {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FileCode)]
pub fn r#file_code(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 22h14a2 2 0 0 0 2-2V7.5L14.5 2H6a2 2 0 0 0-2 2v4"  /><polyline points="14 2 14 8 20 8"  /><path d="m9 18 3-3-3-3"  /><path d="m5 12-3 3 3 3"  />
        </svg>
    }
}

}
pub use r#file_code::FileCode;
mod r#flag_triangle_left {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FlagTriangleLeft)]
pub fn r#flag_triangle_left(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M17 22V2L7 7l10 5"  />
        </svg>
    }
}

}
pub use r#flag_triangle_left::FlagTriangleLeft;
mod r#mail {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Mail)]
pub fn r#mail(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"  /><polyline points="22,6 12,13 2,6"  />
        </svg>
    }
}

}
pub use r#mail::Mail;
mod r#cloud_moon_rain {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CloudMoonRain)]
pub fn r#cloud_moon_rain(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10.083 9A6.002 6.002 0 0 1 16 4a4.243 4.243 0 0 0 6 6c0 2.22-1.206 4.16-3 5.197"  /><path d="M3 20a5 5 0 1 1 8.9-4H13a3 3 0 0 1 2 5.24"  /><path d="M11 20v2"  /><path d="M7 19v2"  />
        </svg>
    }
}

}
pub use r#cloud_moon_rain::CloudMoonRain;
mod r#folder_search {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderSearch)]
pub fn r#folder_search(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M11 20H4a2 2 0 0 1-2-2V5c0-1.1.9-2 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H20a2 2 0 0 1 2 2v4"  /><circle cy="17" r="3" cx="17"  /><path d="m21 21-1.5-1.5"  />
        </svg>
    }
}

}
pub use r#folder_search::FolderSearch;
mod r#gamepad_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Gamepad2)]
pub fn r#gamepad_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <line x2="10" x1="6" y2="11" y1="11"  /><line x2="8" y1="9" y2="13" x1="8"  /><line x2="15.01" y2="12" y1="12" x1="15"  /><line x2="18.01" y1="10" y2="10" x1="18"  /><path d="M17.32 5H6.68a4 4 0 0 0-3.978 3.59c-.006.052-.01.101-.017.152C2.604 9.416 2 14.456 2 16a3 3 0 0 0 3 3c1 0 1.5-.5 2-1l1.414-1.414A2 2 0 0 1 9.828 16h4.344a2 2 0 0 1 1.414.586L17 18c.5.5 1 1 2 1a3 3 0 0 0 3-3c0-1.545-.604-6.584-.685-7.258-.007-.05-.011-.1-.017-.151A4 4 0 0 0 17.32 5z"  />
        </svg>
    }
}

}
pub use r#gamepad_2::Gamepad2;
mod r#binary {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Binary)]
pub fn r#binary(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M6 20h4"  /><path d="M14 10h4"  /><path d="M6 14h2v6"  /><path d="M14 4h2v6"  /><rect x="6" y="4" width="4" height="6"  /><rect width="4" height="6" x="14" y="14"  />
        </svg>
    }
}

}
pub use r#binary::Binary;
mod r#bar_chart_horizontal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BarChartHorizontal)]
pub fn r#bar_chart_horizontal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 3v18h18"  /><path d="M7 16h8"  /><path d="M7 11h12"  /><path d="M7 6h3"  />
        </svg>
    }
}

}
pub use r#bar_chart_horizontal::BarChartHorizontal;
mod r#chevrons_left {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ChevronsLeft)]
pub fn r#chevrons_left(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="11 17 6 12 11 7"  /><polyline points="18 17 13 12 18 7"  />
        </svg>
    }
}

}
pub use r#chevrons_left::ChevronsLeft;
mod r#framer {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Framer)]
pub fn r#framer(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M5 16V9h14V2H5l14 14h-7m-7 0 7 7v-7m-7 0h7"  />
        </svg>
    }
}

}
pub use r#framer::Framer;
mod r#globe {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Globe)]
pub fn r#globe(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" r="10" cy="12"  /><line y2="12" x1="2" y1="12" x2="22"  /><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"  />
        </svg>
    }
}

}
pub use r#globe::Globe;
mod r#layout_template {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(LayoutTemplate)]
pub fn r#layout_template(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21 3H3v7h18V3z"  /><path d="M21 14h-5v7h5v-7z"  /><path d="M12 14H3v7h9v-7z"  />
        </svg>
    }
}

}
pub use r#layout_template::LayoutTemplate;
mod r#music_2 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Music2)]
pub fn r#music_2(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle r="4" cy="18" cx="8"  /><path d="M12 18V2l7 4"  />
        </svg>
    }
}

}
pub use r#music_2::Music2;
mod r#ruler {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Ruler)]
pub fn r#ruler(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M21.3 8.7 8.7 21.3c-1 1-2.5 1-3.4 0l-2.6-2.6c-1-1-1-2.5 0-3.4L15.3 2.7c1-1 2.5-1 3.4 0l2.6 2.6c1 1 1 2.5 0 3.4Z"  /><path d="m7.5 10.5 2 2"  /><path d="m10.5 7.5 2 2"  /><path d="m13.5 4.5 2 2"  /><path d="m4.5 13.5 2 2"  />
        </svg>
    }
}

}
pub use r#ruler::Ruler;
mod r#laptop {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Laptop)]
pub fn r#laptop(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20 16V7a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v9m16 0H4m16 0 1.28 2.55a1 1 0 0 1-.9 1.45H3.62a1 1 0 0 1-.9-1.45L4 16"  />
        </svg>
    }
}

}
pub use r#laptop::Laptop;
mod r#divide_square {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(DivideSquare)]
pub fn r#divide_square(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect y="3" x="3" width="18" ry="2" rx="2" height="18"  /><line x1="8" y2="12" x2="16" y1="12"  /><line y1="16" y2="16" x2="12" x1="12"  /><line x1="12" y1="8" x2="12" y2="8"  />
        </svg>
    }
}

}
pub use r#divide_square::DivideSquare;
mod r#facebook {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Facebook)]
pub fn r#facebook(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M18 2h-3a5 5 0 0 0-5 5v3H7v4h3v8h4v-8h3l1-4h-4V7a1 1 0 0 1 1-1h3z"  />
        </svg>
    }
}

}
pub use r#facebook::Facebook;
mod r#grip_horizontal {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(GripHorizontal)]
pub fn r#grip_horizontal(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="9" r="1" cx="12"  /><circle cx="19" cy="9" r="1"  /><circle cx="5" cy="9" r="1"  /><circle cx="12" r="1" cy="15"  /><circle r="1" cx="19" cy="15"  /><circle r="1" cy="15" cx="5"  />
        </svg>
    }
}

}
pub use r#grip_horizontal::GripHorizontal;
mod r#check_circle {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(CheckCircle)]
pub fn r#check_circle(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"  /><polyline points="22 4 12 14.01 9 11.01"  />
        </svg>
    }
}

}
pub use r#check_circle::CheckCircle;
mod r#egg {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Egg)]
pub fn r#egg(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 22c6.23-.05 7.87-5.57 7.5-10-.36-4.34-3.95-9.96-7.5-10-3.55.04-7.14 5.66-7.5 10-.37 4.43 1.27 9.95 7.5 10z"  />
        </svg>
    }
}

}
pub use r#egg::Egg;
mod r#folder_lock {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(FolderLock)]
pub fn r#folder_lock(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10 20H4a2 2 0 0 1-2-2V5c0-1.1.9-2 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H20a2 2 0 0 1 2 2v2.5"  /><rect width="8" x="14" height="5" rx="1" y="17"  /><path d="M20 17v-2a2 2 0 1 0-4 0v2"  />
        </svg>
    }
}

}
pub use r#folder_lock::FolderLock;
mod r#list_video {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ListVideo)]
pub fn r#list_video(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12 12H3"  /><path d="M16 6H3"  /><path d="M12 18H3"  /><path d="m16 12 5 3-5 3v-6Z"  />
        </svg>
    }
}

}
pub use r#list_video::ListVideo;
mod r#phone_outgoing {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(PhoneOutgoing)]
pub fn r#phone_outgoing(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="22 8 22 2 16 2"  /><line x1="16" y1="8" x2="22" y2="2"  /><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"  />
        </svg>
    }
}

}
pub use r#phone_outgoing::PhoneOutgoing;
mod r#bell_minus {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(BellMinus)]
pub fn r#bell_minus(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M13.73 21a2 2 0 0 1-3.46 0"  /><path d="M21 5h-6"  /><path d="M18.021 9C18.29 15.193 21 17 21 17H3s3-2 3-9a6 6 0 0 1 7-5.916"  />
        </svg>
    }
}

}
pub use r#bell_minus::BellMinus;
mod r#bookmark {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Bookmark)]
pub fn r#bookmark(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"  />
        </svg>
    }
}

}
pub use r#bookmark::Bookmark;
mod r#dice_3 {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Dice3)]
pub fn r#dice_3(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect x="3" height="18" rx="2" ry="2" y="3" width="18"  /><path d="M16 8h.01"  /><path d="M12 12h.01"  /><path d="M8 16h.01"  />
        </svg>
    }
}

}
pub use r#dice_3::Dice3;
mod r#chevrons_right {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ChevronsRight)]
pub fn r#chevrons_right(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <polyline points="13 17 18 12 13 7"  /><polyline points="6 17 11 12 6 7"  />
        </svg>
    }
}

}
pub use r#chevrons_right::ChevronsRight;
mod r#star_off {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(StarOff)]
pub fn r#star_off(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M8.34 8.34 2 9.27l5 4.87L5.82 21 12 17.77 18.18 21l-.59-3.43"  /><path d="M18.42 12.76 22 9.27l-6.91-1L12 2l-1.44 2.91"  /><line y1="2" x1="2" x2="22" y2="22"  />
        </svg>
    }
}

}
pub use r#star_off::StarOff;
mod r#swiss_franc {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SwissFranc)]
pub fn r#swiss_franc(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10 21V3h8"  /><path d="M6 16h9"  /><path d="M10 9.5h7"  />
        </svg>
    }
}

}
pub use r#swiss_franc::SwissFranc;
mod r#underline {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Underline)]
pub fn r#underline(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M6 4v6a6 6 0 0 0 12 0V4"  /><line y1="20" x2="20" x1="4" y2="20"  />
        </svg>
    }
}

}
pub use r#underline::Underline;
mod r#voicemail {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Voicemail)]
pub fn r#voicemail(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cy="12" r="4" cx="6"  /><circle r="4" cx="18" cy="12"  /><line x1="6" y1="16" x2="18" y2="16"  />
        </svg>
    }
}

}
pub use r#voicemail::Voicemail;
mod r#cloud {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Cloud)]
pub fn r#cloud(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z"  />
        </svg>
    }
}

}
pub use r#cloud::Cloud;
mod r#sidebar_open {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(SidebarOpen)]
pub fn r#sidebar_open(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect ry="2" y="3" rx="2" x="3" height="18" width="18"  /><path d="M9 3v18"  /><path d="m14 9 3 3-3 3"  />
        </svg>
    }
}

}
pub use r#sidebar_open::SidebarOpen;
mod r#align_vertical_distribute_end {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(AlignVerticalDistributeEnd)]
pub fn r#align_vertical_distribute_end(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <rect height="6" width="14" rx="2" y="14" x="5"  /><rect rx="2" x="7" height="6" y="4" width="10"  /><path d="M2 20h20"  /><path d="M2 10h20"  />
        </svg>
    }
}

}
pub use r#align_vertical_distribute_end::AlignVerticalDistributeEnd;
mod r#sofa {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Sofa)]
pub fn r#sofa(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M20 9V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v3"  /><path d="M2 11v5a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-5a2 2 0 0 0-4 0v2H6v-2a2 2 0 0 0-4 0Z"  /><path d="M4 18v2"  /><path d="M20 18v2"  /><path d="M12 4v9"  />
        </svg>
    }
}

}
pub use r#sofa::Sofa;
mod r#contrast {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Contrast)]
pub fn r#contrast(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <circle cx="12" cy="12" r="10"  /><path d="M12 18a6 6 0 0 0 0-12v12z"  />
        </svg>
    }
}

}
pub use r#contrast::Contrast;
mod r#settings {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Settings)]
pub fn r#settings(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"  /><circle cy="12" r="3" cx="12"  />
        </svg>
    }
}

}
pub use r#settings::Settings;
mod r#thumbs_up {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(ThumbsUp)]
pub fn r#thumbs_up(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M7 10v12"  /><path d="M15 5.88 14 10h5.83a2 2 0 0 1 1.92 2.56l-2.33 8A2 2 0 0 1 17.5 22H4a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2h2.76a2 2 0 0 0 1.79-1.11L12 2h0a3.13 3.13 0 0 1 3 3.88Z"  />
        </svg>
    }
}

}
pub use r#thumbs_up::ThumbsUp;
mod r#truck {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Truck)]
pub fn r#truck(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M10 17h4V5H2v12h3"  /><path d="M20 17h2v-3.34a4 4 0 0 0-1.17-2.83L19 9h-5"  /><path d="M14 17h1"  /><circle r="2.5" cy="17.5" cx="7.5"  /><circle cx="17.5" cy="17.5" r="2.5"  />
        </svg>
    }
}

}
pub use r#truck::Truck;
mod r#undo {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Undo)]
pub fn r#undo(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M3 7v6h6"  /><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"  />
        </svg>
    }
}

}
pub use r#undo::Undo;
mod r#folders {
use yew::{function_component, html, Html};

use crate::IconProps;

#[function_component(Folders)]
pub fn r#folders(
    &IconProps {
        class,
        size,
        fill,
        color,
        stroke_width,
        stroke_linecap,
        stroke_linejoin,
    }: &IconProps,
) -> Html {
    html! {
        <svg
            {class}
            width={size}
            height={size}
            viewBox="0 0 24 24"
            {fill}
            stroke={color}
            stroke-width={stroke_width}
            stroke-linecap={stroke_linecap}
            stroke-linejoin={stroke_linejoin}
        >
            <path d="M8 17h12a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3.93a2 2 0 0 1-1.66-.9l-.82-1.2a2 2 0 0 0-1.66-.9H8a2 2 0 0 0-2 2v9c0 1.1.9 2 2 2Z"  /><path d="M2 8v11c0 1.1.9 2 2 2h14"  />
        </svg>
    }
}

}
pub use r#folders::Folders;