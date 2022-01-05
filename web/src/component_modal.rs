use crate::jquery_bindgen::jquery;
use wasm_bindgen::JsValue;
use yew::prelude::*;

pub struct ModalContainer {
    link: ComponentLink<Self>,
    pub(crate) props: Props,
}

#[derive(Properties, Clone, PartialEq)]
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

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            //TODO: should this be passed to host as well, to be able to remove dismissed messages
            Msg::Close(index) => {
                let id = format!("#modal-{}", index);
                jquery(&id).modal(&JsValue::from("hide"));
                self.props.onclose.emit(index);
            }
        }
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
             { for self.props.children.iter().enumerate().map(|(i, mut modal)| {
                modal.props.id = format!("modal-{}", i);
                modal.props.onclosed = Some(self.link.callback(move |_| {
                    Msg::Close(i)
                }));
                modal
             })}
             </>
        }
    }

    /// When a <ModalContainer/> is rendered it displays all of its child modals
    fn rendered(&mut self, _first_render: bool) {
        let ids = self
            .props
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
    use yewtil::NeqAssign;

    pub enum Msg {
        Closed,
    }

    pub struct Modal {
        link: ComponentLink<Self>,
        pub(crate) props: Props,
    }

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

        fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
            Self { link, props }
        }

        fn update(&mut self, msg: Self::Message) -> bool {
            match msg {
                Msg::Closed => {
                    if let Some(callback) = &self.props.onclosed {
                        callback.emit(())
                    }
                }
            }
            false
        }

        fn change(&mut self, props: Self::Properties) -> bool {
            self.props.neq_assign(props)
        }

        //TODO: currently dismissing using anything other than the close button will cause the modal to re-appear when a new one is created
        //TODO: only fix seems to be to not use a js modal but rather a custom one
        fn view(&self) -> Html {
            html! {
                <div class="modal fade" tabindex="-1" role="dialog" id=self.props.id.clone()>
                  <div class="modal-dialog" role="document">
                    <div class="modal-content">
                      <div class="modal-header">
                        <h5 class="modal-title" id="exampleModalLabel">{&self.props.title}</h5>
                        <button type="button" class="close" data-dismiss="modal" aria-label="Close">
                          <span aria-hidden="true">{"x"}</span>
                        </button>
                      </div>
                      <div class="modal-body">
                        {&self.props.content}
                      </div>
                      <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" onclick=self.link.callback(|_| Msg::Closed)>{"Close"}</button>
                      </div>
                    </div>
                  </div>
                </div>
            }
        }
    }
}
