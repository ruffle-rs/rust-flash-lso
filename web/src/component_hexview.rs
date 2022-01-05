use crate::component_number_input::NumberInput;
use crate::web_expect::WebSafeExpect;
use yew::prelude::*;
use yew::{Component, ComponentLink, Html, Properties};
use yewtil::NeqAssign;

pub struct HexView {
    link: ComponentLink<Self>,
    props: Props,
    selected: Option<usize>,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub bytes: Vec<u8>,
    pub onchange: Callback<Vec<u8>>,

    pub onadd: Callback<()>,
    pub onremove: Callback<usize>,
}

pub enum Msg {
    Edit(u8, usize),
    Focus(usize),
    Blur,
    Remove,
    Add,
}

impl Component for HexView {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            selected: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::Edit(data, index) => {
                let mut new_data = self.props.bytes.clone();
                new_data[index] = data;
                self.props.onchange.emit(new_data);
                true
            }
            Msg::Focus(index) => {
                self.selected = Some(index);
                true
            }
            Msg::Blur => {
                // self.selected = None;
                true
            }
            Msg::Remove => {
                self.props
                    .onremove
                    .emit(self.selected.web_expect("Nothing selected"));
                true
            }
            Msg::Add => {
                self.props.onadd.emit(());
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! {
            <>
            <table class="table table-striped">
              <tbody>
               { self.table_body() }
              </tbody>
            </table>
                <span onclick=self.link.callback(move |_| Msg::Add) class="btn btn-primary">{"Add"}</span>
                {self.remove_button()}
            </>
        }
    }
}

const CHUNK_SIZE: usize = 8;

impl HexView {
    fn remove_button(&self) -> Html {
        if self.selected.is_some() {
            return html! {
                <span onclick=self.link.callback(move |_| Msg::Remove) class="ml-2 btn btn-danger">{"Remove"}</span>
            };
        } else {
            return html! {};
        }
    }

    fn table_body(&self) -> Html {
        let chunks: Vec<&[u8]> = self.props.bytes.chunks(CHUNK_SIZE).collect();

        html! {
            <>
            {for chunks.iter().enumerate().map(|(chunk_index, chunk)| html! {
                    <tr>
                        { for chunk.iter().enumerate().map(move |(subchunk_index, v)| html! {
                            <td>
                                <NumberInput<u8>
                                    value=v.clone()
                                    onchange=self.link.callback(move |data| Msg::Edit(data, chunk_index*CHUNK_SIZE + subchunk_index))
                                    onfocus=self.link.callback(move |_| Msg::Focus(chunk_index*CHUNK_SIZE + subchunk_index))
                                    onblur=self.link.callback(move |_| Msg::Blur)/>
                            </td>
                        })}
                    </tr>
               })}
            </>
        }
    }
}
