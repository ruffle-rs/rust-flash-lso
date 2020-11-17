use crate::component_number_input::NumberInput;
use yew::prelude::*;
use yew::{Component, ComponentLink, Html, Properties};

pub struct HexView {
    link: ComponentLink<Self>,
    props: Props,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub bytes: Vec<u8>,
    pub onchange: Callback<Vec<u8>>,
}

pub enum Msg {
    Edit(u8, usize),
}

impl Component for HexView {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::Edit(data, index) => {
                let mut new_data = self.props.bytes.clone();
                new_data[index] = data;
                self.props.onchange.emit(new_data);
                true
            }
        }
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
            <table class="table table-striped">
              <tbody>
               { self.table_body() }
              </tbody>
            </table>
        }
    }
}

const CHUNK_SIZE: usize = 8;

impl HexView {
    pub fn table_body(&self) -> Html {
        let chunks: Vec<&[u8]> = self.props.bytes.chunks(CHUNK_SIZE).collect();

        html! {
            <>
            {for chunks.iter().enumerate().map(|(chunk_index, chunk)| html! {
                    <tr>
                        { for chunk.iter().enumerate().map(move |(subchunk_index, v)| html! {
                            <td>
                                <NumberInput<u8> value={v} onchange=self.link.callback(move |data| Msg::Edit(data, chunk_index*CHUNK_SIZE + subchunk_index))/>
                            </td>
                        })}
                    </tr>
               })}
            </>
        }
    }
}
