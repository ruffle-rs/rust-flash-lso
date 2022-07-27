use crate::component_string_input::StringInput;
use std::fmt::Display;
use std::str::FromStr;
use yew::prelude::*;
use yew::{Component, ComponentLink, Html, Properties};
use yewtil::NeqAssign;

pub struct NumberInput<T: 'static + Clone + Display + PartialEq + FromStr> {
    link: ComponentLink<Self>,
    pub(crate) props: Props<T>,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props<T: Clone> {
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
            Msg::Focus => {
                if let Some(onfocus) = &self.props.onfocus {
                    onfocus.emit(());
                }
                true
            }
            Msg::UnFocus => {
                if let Some(onblur) = &self.props.onblur {
                    onblur.emit(());
                }
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! {
            <StringInput
                onchange=self.link.callback(Msg::Value)
                onblur=self.link.callback(move |_fe| Msg::UnFocus)
                onfocus=self.link.callback(move |_fe| Msg::Focus)
                value={format!("{}", self.props.value)}/>
        }
    }
}
