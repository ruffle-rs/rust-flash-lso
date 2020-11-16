use yew::prelude::*;
use yew::{Component, ComponentLink, Html, Properties};
use crate::component_string_input::StringInput;

pub struct NumberInput {
    link: ComponentLink<Self>,
    pub(crate) props: Props,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub onchange: Callback<f64>,
    pub value: f64,
}

pub enum Msg {
    Value(String)
}

impl Component for NumberInput {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::Value(s) => {
                if let Ok(f) = s.parse::<f64>() {
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
