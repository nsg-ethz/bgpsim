use strum_macros::EnumIter;
use yew::{classes, Classes};

pub mod arrows;
pub mod bgp_session;
pub mod canvas;
pub mod link;
pub mod link_weight;
pub mod next_hop;
pub mod router;
pub mod text;

#[derive(Clone, Copy, PartialEq, Eq, EnumIter, Debug)]
pub enum SvgColor {
    BlueLight,
    PurpleLight,
    GreenLight,
    RedLight,
    YellowLight,
    BlueDark,
    PurpleDark,
    GreenDark,
    RedDark,
    YellowDark,
    Light,
    Dark,
}

impl Default for SvgColor {
    fn default() -> Self {
        SvgColor::BlueLight
    }
}

impl SvgColor {
    pub fn classes(&self) -> Classes {
        match self {
            SvgColor::BlueLight => classes! {"text-blue-500", "hover:text-blue-700"},
            SvgColor::PurpleLight => classes! {"text-purple-500", "hover:text-purple-700"},
            SvgColor::GreenLight => classes! {"text-green-500", "hover:text-green-700"},
            SvgColor::RedLight => classes! {"text-red-500", "hover:text-red-700"},
            SvgColor::YellowLight => classes! {"text-yellow-500", "hover:text-yellow-700"},
            SvgColor::BlueDark => classes! {"text-blue-800", "hover:text-blue-700"},
            SvgColor::PurpleDark => classes! {"text-purple-800", "hover:text-purple-700"},
            SvgColor::GreenDark => classes! {"text-green-800", "hover:text-green-700"},
            SvgColor::RedDark => classes! {"text-red-800", "hover:text-red-700"},
            SvgColor::YellowDark => classes! {"text-yellow-800", "hover:text-yellow-700"},
            SvgColor::Light => classes! {"text-gray-300", "hover:text-gray-500"},
            SvgColor::Dark => classes! {"text-gray-800", "hover:text-gray-500"},
        }
    }

    pub fn arrow_tip(&self) -> &'static str {
        match self {
            SvgColor::BlueLight => "arrow-tip-blue-500",
            SvgColor::PurpleLight => "arrow-tip-purple-500",
            SvgColor::GreenLight => "arrow-tip-green-500",
            SvgColor::RedLight => "arrow-tip-red-500",
            SvgColor::YellowLight => "arrow-tip-yellow-500",
            SvgColor::BlueDark => "arrow-tip-blue-800",
            SvgColor::PurpleDark => "arrow-tip-purple-800",
            SvgColor::GreenDark => "arrow-tip-green-800",
            SvgColor::RedDark => "arrow-tip-red-800",
            SvgColor::YellowDark => "arrow-tip-yellow-800",
            SvgColor::Light => "arrow-tip-gray-300",
            SvgColor::Dark => "arrow-tip-gray-800",
        }
    }
}
