use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::{Callback, Component, Context, Html, Properties, events::Event, html};

pub struct StringInput {}

#[derive(PartialEq, Properties)]
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
    Focus,
    UnFocus,
}

impl Component for StringInput {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        StringInput {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Value(s) => {
                ctx.props().onchange.emit(s);
                true
            }
            Msg::Focus => {
                if let Some(onfocus) = &ctx.props().onfocus {
                    onfocus.emit(());
                }
                true
            }
            Msg::UnFocus => {
                if let Some(onblur) = &ctx.props().onblur {
                    onblur.emit(());
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <input
                onchange={ctx.link().batch_callback(|e: Event| {
                    let target: Option<EventTarget> = e.target();
                    let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

                    input.map(|input| Msg::Value(input.value()))
                })}
                placeholder={ctx.props().placeholder.clone()}
                onblur={ctx.link().callback(move |_fe| Msg::UnFocus)}
                onfocus={ctx.link().callback(move |_fe| Msg::Focus)}
                value={ctx.props().value.clone()}
                class={format!("form-control {}", ctx.props().class)}/>
        }
    }
}
