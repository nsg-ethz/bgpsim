use netsim::types::{Prefix, RouterId};

#[derive(Clone, Default)]
pub struct State {
    selected: Selected,
    hover: Hover,
    layer: Layer,
    prefix: Option<Prefix>,
}

impl State {
    pub fn selected(&self) -> Selected {
        self.selected
    }

    pub fn hover(&self) -> Hover {
        self.hover
    }

    pub fn layer(&self) -> Layer {
        self.layer
    }

    pub fn prefix(&self) -> Option<Prefix> {
        self.prefix
    }

    pub fn set_selected(&mut self, selected: Selected) {
        self.selected = selected
    }

    pub fn set_hover(&mut self, hover: Hover) {
        self.hover = hover
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.layer = layer;
        self.selected = match (self.layer, self.selected) {
            (Layer::FwState, Selected::None)
            | (Layer::FwState, Selected::Router(_))
            | (Layer::Igp, Selected::None)
            | (Layer::Igp, Selected::Router(_))
            | (Layer::Bgp, Selected::None)
            | (Layer::Bgp, Selected::Router(_)) => self.selected,
        };
    }

    pub fn set_prefix(&mut self, prefix: Option<Prefix>) {
        self.prefix = prefix
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Selected {
    None,
    Router(RouterId),
}

impl Default for Selected {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy)]
pub enum Hover {
    None,
    Router(RouterId),
    BgpSession(RouterId, RouterId),
    NextHop(RouterId, RouterId),
}

impl Default for Hover {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    FwState,
    Igp,
    Bgp,
}

impl std::fmt::Display for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layer::FwState => f.write_str("Forwarding"),
            Layer::Igp => f.write_str("IGP"),
            Layer::Bgp => f.write_str("BGP"),
        }
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self::FwState
    }
}
