use yew::prelude::*;
use yew::{Component, ComponentLink, Html, Properties};
use yewtil::NeqAssign;

pub struct StringInput {
    link: ComponentLink<Self>,
    pub(crate) props: Props,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub onchange: Callback<String>,
    pub value: String,

    #[prop_or("".to_string())]
    pub placeholder: String,

    #[prop_or("".to_string())]
    pub class: String,

    #[prop_or(None)]
    pub onfocus: Option<Callback<()>>,
    #[prop_or(None)]
    pub onblur: Option<Callback<()>>,
}

pub enum Msg {
    Value(String),
    Ignored,
    Focus,
    UnFocus,
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
            _ => false,
        }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! {
            <input
                onchange=self.link.callback(move |cd| {
                    if let ChangeData::Value(s) = cd {
                        Msg::Value(s)
                    } else {
                        Msg::Ignored
                    }
                })
                placeholder=self.props.placeholder.clone()
                onblur=self.link.callback(move |_fe| Msg::UnFocus)
                onfocus=self.link.callback(move |_fe| Msg::Focus)
                value=self.props.value.clone()
                class=format!("form-control {}", self.props.class)/>
        }
    }
}
