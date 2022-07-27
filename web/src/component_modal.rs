use crate::jquery_bindgen::jquery;
use wasm_bindgen::JsValue;
use yew::prelude::*;

pub struct ModalContainer {}

#[derive(PartialEq, Properties, Clone)]
pub struct Props {
    pub children: ChildrenWithProps<modal::Modal>,
    pub onclose: Callback<usize>,
}

pub enum Msg {
    Close(usize),
}

impl Component for ModalContainer {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            //TODO: should this be passed to host as well, to be able to remove dismissed messages
            Msg::Close(index) => {
                let id = format!("#modal-{}", index);
                jquery(&id).modal(&JsValue::from("hide"));
                ctx.props().onclose.emit(index);
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
             <>
             { for ctx.props().children.iter().enumerate().map(|(i, mut modal)| {
                 //TODO: fix
                // modal.props.id = format!("modal-{}", i);
                // modal.props.onclosed = Some(ctx.link().callback(move |_| {
                //     Msg::Close(i)
                // }));
                modal
             })}
             </>
        }
    }

    /// When a <ModalContainer/> is rendered it displays all of its child modals
    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        let ids = ctx
            .props()
            .children
            .iter()
            .enumerate()
            .map(|(i, _)| format!("#modal-{}", i));

        for id in ids {
            let o: js_sys::Object = js_sys::Object::new();
            jquery(&id).modal(&o);
        }
    }
}

pub mod modal {
    use yew::prelude::*;

    pub enum Msg {
        Closed,
    }

    pub struct Modal {}

    #[derive(Properties, Clone, PartialEq)]
    pub struct Props {
        pub content: String,
        pub title: String,

        // Props filled by container
        #[prop_or(None)]
        pub onclosed: Option<Callback<()>>,
        #[prop_or("".to_string())]
        pub id: String,
    }

    impl Component for Modal {
        type Message = Msg;
        type Properties = Props;

        fn create(_ctx: &Context<Self>) -> Self {
            Self {}
        }

        fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
            match msg {
                Msg::Closed => {
                    if let Some(callback) = &ctx.props().onclosed {
                        callback.emit(())
                    }
                }
            }
            false
        }

        //TODO: currently dismissing using anything other than the close button will cause the modal to re-appear when a new one is created
        //TODO: only fix seems to be to not use a js modal but rather a custom one
        fn view(&self, ctx: &Context<Self>) -> Html {
            html! {
                <div class="modal fade" tabindex="-1" role="dialog" id={ctx.props().id.clone()}>
                  <div class="modal-dialog" role="document">
                    <div class="modal-content">
                      <div class="modal-header">
                        <h5 class="modal-title" id="exampleModalLabel">{&ctx.props().title}</h5>
                        <button type="button" class="close" data-dismiss="modal" aria-label="Close">
                          <span aria-hidden="true">{"x"}</span>
                        </button>
                      </div>
                      <div class="modal-body">
                        {&ctx.props().content}
                      </div>
                      <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::Closed)}>{"Close"}</button>
                      </div>
                    </div>
                  </div>
                </div>
            }
        }
    }
}
