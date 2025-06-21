use crate::component_tab::Tab;
use yew::prelude::*;
use yew::virtual_dom::VChild;
use yew::{ChildrenWithProps, Component, Html, Properties};

pub struct Tabs {}

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

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::info!("TAB msg={msg:?}");
        match msg {
            Msg::Selected(pos) => {
                ctx.props().ontabselect.emit(pos);
                true
            }
            Msg::Removed(pos) => {
                ctx.props().ontabremove.emit(pos);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
             <>
                 <ul class="nav nav-tabs">
                      { for ctx.props().children.iter().enumerate().map(|(i, e)| html! {
                         <li class="nav-item" role="tablist">
                            <span class={format!("nav-link {}", if ctx.props().selected == Some(i) {"active"} else {""})} role="tab" onclick={ctx.link().callback(move |_| Msg::Selected(i))}>
                                <a href="#" style="vertical-align: middle;">{&e.props.label}</a>{ self.tab_details(ctx, e, i) }
                             </span>
                         </li>
                      })}
                 </ul>

                 <div class="tab-content">
                      { for ctx.props().children.iter().enumerate().map(|(i, e)| html! {
                         <div class={format!("tab-pane fade {}", if ctx.props().selected == Some(i) {"show active"} else {""})} role="tabpanel">
                             {e}
                         </div>
                      })}
                 </div>
             </>
        }
    }
}

impl Tabs {
    fn tab_details(&self, ctx: &Context<Self>, tab: VChild<Tab>, index: usize) -> Html {
        if tab.props.loading {
            self.loading_spinner()
        } else {
            self.remove_button(ctx, index)
        }
    }

    fn remove_button(&self, ctx: &Context<Self>, index: usize) -> Html {
        html! {
            <span onclick={ctx.link().callback(move |_| Msg::Removed(index))}><img alt={"Close"} src={"icon/x.svg"} style={"width: 24px; height: 24px;"} class={"mr-2"}/></span>
        }
    }

    fn loading_spinner(&self) -> Html {
        html! {
            <span>{"Loading"}</span>
        }
    }
}
