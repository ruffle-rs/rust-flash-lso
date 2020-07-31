use crate::component_tab::Tab;
use yew::prelude::*;
use yew::{ChildrenWithProps, Component, ComponentLink, Html, Properties};

pub struct Tabs {
    link: ComponentLink<Self>,
    selected: usize,
    props: Props,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub children: ChildrenWithProps<Tab>,
}

pub enum Msg {
    Selected(usize),
}

impl Component for Tabs {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            selected: 0,
            props,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::Selected(pos) => self.selected = pos,
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        html! {
             <>
                 <ul class="nav nav-tabs">
                      { for self.props.children.iter().enumerate().map(|(i, e)| html! {
                         <li class="nav-item" role="tablist">
                             <a class={format!("nav-link {}", if self.selected == i {"active"} else {""})} role="tab" onclick=self.link.callback(move |_| Msg::Selected(i))>{&e.props.label}</a>
                         </li>
                      })}
                 </ul>

                 <div class="tab-content">
                      { for self.props.children.iter().enumerate().map(|(i, e)| html! {
                         <div class={format!("tab-pane fade {}", if self.selected == i {"show active"} else {""})} role="tabpanel">
                             {e}
                         </div>
                      })}
                 </div>
             </>
        }
    }
}
