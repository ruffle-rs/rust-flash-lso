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
    pub ontabselect: Callback<usize>,
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
            Msg::Selected(pos) => {
                self.selected = pos;
                self.props.ontabselect.emit(pos);
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if self.props != props {
            // If we have just added the first tab
            if self.props.children.is_empty() && !props.children.is_empty() {
                self.update(Msg::Selected(0));
            }
            // If we have just removed the current tab
            //TODO: this wont work if we can remove any tab, selection will need to be tracked by parent
            if self.props.children.len() > props.children.len() {
                self.update(Msg::Selected(self.selected - 1));
            }
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
