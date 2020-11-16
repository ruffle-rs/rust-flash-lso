use yew::prelude::*;
use yew::{Children, Component, ComponentLink, Html, Properties};

pub struct Tab {
    pub(crate) props: Props,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub label: String,
    pub loading: bool,
    pub children: Children,
}

impl Component for Tab {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        false
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
             <>
             { for self.props.children.iter()}
             </>
        }
    }
}
