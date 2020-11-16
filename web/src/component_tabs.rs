use crate::component_tab::Tab;
use yew::prelude::*;
use yew::virtual_dom::VChild;
use yew::{ChildrenWithProps, Component, ComponentLink, Html, Properties};

pub struct Tabs {
    link: ComponentLink<Self>,
    props: Props,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub ontabselect: Callback<usize>,
    pub ontabremove: Callback<usize>,
    pub children: ChildrenWithProps<Tab>,
    pub selected: Option<usize>,
}

#[derive(Debug)]
pub enum Msg {
    Selected(usize),
    Removed(usize),
}

impl Component for Tabs {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        log::info!("TAB msg={:?}", msg);
        match msg {
            Msg::Selected(pos) => {
                self.props.ontabselect.emit(pos);
                true
            }
            Msg::Removed(pos) => {
                self.props.ontabremove.emit(pos);
                true
            }
        }
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
                            <span class={format!("nav-link {}", if self.props.selected == Some(i) {"active"} else {""})} role="tab" onclick=self.link.callback(move |_| Msg::Selected(i))>
                                <a style="vertical-align: middle;">{&e.props.label}</a>{ self.tab_details(e, i) }
                             </span>
                         </li>
                      })}
                 </ul>

                 <div class="tab-content">
                      { for self.props.children.iter().enumerate().map(|(i, e)| html! {
                         <div class={format!("tab-pane fade {}", if self.props.selected == Some(i) {"show active"} else {""})} role="tabpanel">
                             {e}
                         </div>
                      })}
                 </div>
             </>
        }
    }
}

impl Tabs {
    fn tab_details(&self, tab: VChild<Tab>, index: usize) -> Html {
        if tab.props.loading {
            self.loading_spinner()
        } else {
            self.remove_button(index)
        }
    }

    fn remove_button(&self, index: usize) -> Html {
        html! {
            <span onclick=self.link.callback(move |_| Msg::Removed(index))><img src={"icon/x.svg"} style={"width: 24px; height: 24px;"} class={"mr-2"}/></span>
        }
    }

    fn loading_spinner(&self) -> Html {
        html! {
            <span>{"Loading"}</span>
        }
    }
}
