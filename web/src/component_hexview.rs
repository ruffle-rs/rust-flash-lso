use crate::component_number_input::NumberInput;
use crate::web_expect::WebSafeExpect;
use yew::prelude::*;
use yew::{Component, Html, Properties};

pub struct HexView {
    selected: Option<usize>,
}

#[derive(PartialEq, Properties, Clone)]
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

    fn create(_ctx: &Context<Self>) -> Self {
        Self { selected: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Edit(data, index) => {
                let mut new_data = ctx.props().bytes.clone();
                new_data[index] = data;
                ctx.props().onchange.emit(new_data);
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
                ctx.props()
                    .onremove
                    .emit(self.selected.web_expect("Nothing selected"));
                true
            }
            Msg::Add => {
                ctx.props().onadd.emit(());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
            <table class="table table-striped">
              <tbody>
               { self.table_body(ctx) }
              </tbody>
            </table>
                <span onclick={ctx.link().callback(move |_| Msg::Add)} class="btn btn-primary">{"Add"}</span>
                {self.remove_button(ctx)}
            </>
        }
    }
}

const CHUNK_SIZE: usize = 8;

impl HexView {
    fn remove_button(&self, ctx: &Context<Self>) -> Html {
        if self.selected.is_some() {
            html! {
                <span onclick={ctx.link().callback(move |_| Msg::Remove)} class="ml-2 btn btn-danger">{"Remove"}</span>
            }
        } else {
            html! {}
        }
    }

    fn table_body(&self, ctx: &Context<Self>) -> Html {
        let chunks: Vec<&[u8]> = ctx.props().bytes.chunks(CHUNK_SIZE).collect();

        html! {
            <>
            {for chunks.iter().enumerate().map(|(chunk_index, chunk)| html! {
                    <tr>
                        { for chunk.iter().enumerate().map(move |(subchunk_index, v)| html! {
                            <td>
                                <NumberInput<u8>
                                    value={*v}
                                    onchange={ctx.link().callback(move |data| Msg::Edit(data, chunk_index*CHUNK_SIZE + subchunk_index))}
                                    onfocus={ctx.link().callback(move |_| Msg::Focus(chunk_index*CHUNK_SIZE + subchunk_index))}
                                        onblur={ctx.link().callback(move |_| Msg::Blur)}/>
                            </td>
                        })}
                    </tr>
               })}
            </>
        }
    }
}
