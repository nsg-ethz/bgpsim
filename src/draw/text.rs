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

        let bg_class = ctx
            .props()
            .bg_class
            .clone()
            .unwrap_or_else(|| classes!("fill-gray-50", "stroke-0"));
        let text_class = ctx.props().text_class.clone().unwrap_or_default();
        html! {
            <>
                <rect x={p_box.x()} y={p_box.y()} width={box_w} height={box_h} class={bg_class} {rx} />
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
            let text_elem = self.text_ref.cast::<SvgGraphicsElement>().unwrap();
            let bbox = text_elem.get_b_box().unwrap();
            ctx.link().send_message(Msg::UpdateSize(bbox));
        } else {
            self.rerender = true;
        }
    }
}
