// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022-2023 Tibor Schneider <sctibor@ethz.ch>
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

use std::{fmt::Display, marker::PhantomData};
use web_sys::{SvgGraphicsElement, SvgRect};

use yew::prelude::*;

use crate::point::Point;

pub struct Text<T> {
    phantom: PhantomData<T>,
    width: f64,
    height: f64,
    offset: Point,
    text_ref: NodeRef,
    rerender: bool,
}

pub enum Msg {
    UpdateSize(SvgRect),
}

#[derive(Properties, PartialEq)]
pub struct Properties<T>
where
    T: PartialEq,
{
    pub p: Point,
    pub text: T,
    pub text_class: Option<Classes>,
    pub bg_class: Option<Classes>,
    pub padding: Option<f64>,
    pub padding_x: Option<f64>,
    pub rounded_corners: Option<f64>,
    pub onclick: Option<Callback<MouseEvent>>,
}

impl<T> Component for Text<T>
where
    T: Display + PartialEq + 'static,
{
    type Message = Msg;
    type Properties = Properties<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Text {
            width: 0.0,
            height: 0.0,
            offset: Default::default(),
            phantom: PhantomData,
            text_ref: Default::default(),
            rerender: true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let p = ctx.props().p + self.offset;
        let padding = ctx.props().padding.unwrap_or(1.0);
        let padding_x = ctx.props().padding_x.unwrap_or(padding);
        let rx = ctx.props().rounded_corners.unwrap_or(0.0).to_string();
        let p_box = p - Point::new(padding_x, padding + self.height / 2.0);
        let box_w = (self.width + 2.0 * padding_x).to_string();
        let box_h = (self.height + 2.0 * padding).to_string();

        let mut bg_class = ctx
            .props()
            .bg_class
            .clone()
            .unwrap_or_else(|| classes!("fill-base-2", "stroke-0"));
        bg_class.push("cursor-pointer");
        let mut text_class = ctx.props().text_class.clone().unwrap_or_default();
        text_class.push("stroke-main");
        text_class.push("pointer-events-none");
        let onclick = ctx.props().onclick.clone();
        html! {
            <>
                <rect x={p_box.x()} y={p_box.y()} width={box_w} height={box_h} class={bg_class} {rx} {onclick} />
                <text class={text_class} x={p.x()} y={p.y()} ref={self.text_ref.clone()} dominant-baseline="central">{ ctx.props().text.to_string() }</text>
            </>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateSize(bbox) => {
                let width = bbox.width() as f64;
                let height = bbox.height() as f64;
                if (width, height) != (self.width, self.height) {
                    self.width = width;
                    self.height = height;
                    self.rerender = false;
                    self.offset = Point::new(-self.width / 2.0, 0.0);
                    true
                } else {
                    false
                }
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _: bool) {
        if self.rerender {
            if let Some(bbox) = self
                .text_ref
                .cast::<SvgGraphicsElement>()
                .and_then(|e| e.get_b_box().ok())
            {
                ctx.link().send_message(Msg::UpdateSize(bbox));
            } else {
                log::error!("Could not get the bounding box of the text!")
            }
        } else {
            self.rerender = true;
        }
    }
}
