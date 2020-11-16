use yew::prelude::*;
use yew::{Component, ComponentLink, Html, Properties};
use crate::component_string_input::StringInput;
use std::fmt::Display;
use std::str::FromStr;

pub struct NumberInput<T: 'static + Clone + Display + PartialEq + FromStr> {
    link: ComponentLink<Self>,
    pub(crate) props: Props<T>,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props<T: Clone> {
    pub onchange: Callback<T>,
    pub value: T,
}

pub enum Msg {
    Value(String)
}

impl<T: 'static + Clone + Display + PartialEq + FromStr> Component for NumberInput<T> {
    type Message = Msg;
    type Properties = Props<T>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::Value(s) => {
                if let Ok(f) = s.parse::<T>() {
                    self.props.onchange.emit(f);
                }
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if props != self.props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        html! {
            <StringInput onchange=self.link.callback(move |s| Msg::Value(s)) value={format!("{}", self.props.value)}/>
        }
    }
}
