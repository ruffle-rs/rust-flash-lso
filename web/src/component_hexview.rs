use yew::prelude::*;
use yew::{Component, ComponentLink, Html, Properties};

pub struct HexView {
    props: Props,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub bytes: Vec<u8>,
}

impl Component for HexView {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
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
            <table class="table table-striped">
              <tbody>
               { self.table_body() }
              </tbody>
            </table>
        }
    }
}
impl HexView {
    pub fn table_body(&self) -> Html {
        let chunks: Vec<&[u8]> = self.props.bytes.chunks(8).collect();

        html! {
            <>
            {for chunks.iter().map(|chunk| html! {
                    <tr>
                        { for chunk.iter().map(|v| html! { <td>{v}</td>})}
                    </tr>
               })}
            </>
        }
    }
}
