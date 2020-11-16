use yew::prelude::*;
use yew::{Component, ComponentLink, Html, Properties};

pub struct StringInput {
    link: ComponentLink<Self>,
    pub(crate) props: Props,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub onchange: Callback<String>,
    pub value: String,
}

pub enum Msg {
    Value(String),
    Ignored,
}

impl Component for StringInput {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::Value(s) => {
                self.props.onchange.emit(s);
                true
            }
            _ => false,
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
            <input onchange={ self.link.callback(move |cd| {
                    if let ChangeData::Value(s) = cd {
                        Msg::Value(s)
                    } else {
                        Msg::Ignored
                    }
                })} value={&self.props.value} class="form-control"/>
        }
    }
}
