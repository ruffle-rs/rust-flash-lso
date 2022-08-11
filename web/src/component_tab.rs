use yew::prelude::*;
use yew::{Children, Component, Html, Properties};

pub struct Tab {}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub label: String,
    pub loading: bool,
    pub children: Children,
}

impl Component for Tab {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
             <>
             { for ctx.props().children.iter()}
             </>
        }
    }
}
