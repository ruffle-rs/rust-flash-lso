use crate::component_string_input::StringInput;
use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;
use yew::prelude::*;
use yew::{Component, Html, Properties};

pub struct NumberInput<T: 'static + Clone + Display + PartialEq + FromStr> {
    pd: PhantomData<T>,
}

#[derive(PartialEq, Clone, Properties)]
pub struct Props<T: Clone + PartialEq> {
    pub onchange: Callback<T>,
    pub value: T,

    #[prop_or(None)]
    pub onfocus: Option<Callback<()>>,
    #[prop_or(None)]
    pub onblur: Option<Callback<()>>,
}

pub enum Msg {
    Value(String),
    Focus,
    UnFocus,
}

impl<T: 'static + Clone + Display + PartialEq + FromStr> Component for NumberInput<T> {
    type Message = Msg;
    type Properties = Props<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            pd: Default::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Value(s) => {
                if let Ok(f) = s.parse::<T>() {
                    ctx.props().onchange.emit(f);
                }
                true
            }
            Msg::Focus => {
                if let Some(onfocus) = &ctx.props().onfocus {
                    onfocus.emit(());
                }
                true
            }
            Msg::UnFocus => {
                if let Some(onblur) = &ctx.props().onblur {
                    onblur.emit(());
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <StringInput
                onchange={ctx.link().callback(Msg::Value)}
                onblur={ctx.link().callback(move |_fe| Msg::UnFocus)}
                onfocus={ctx.link().callback(move |_fe| Msg::Focus)}
                value={format!("{}", ctx.props().value)}/>
        }
    }
}
