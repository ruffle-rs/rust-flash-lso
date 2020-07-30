#![recursion_limit = "256"]

use std::convert::TryInto;
use std::string::ToString;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew::services::reader::{File, FileChunk, FileData, ReaderService, ReaderTask};
use yew::web_sys::Element;

use flash_lso::types::{Sol, SolElement, SolValue};
use flash_lso::LSODeserializer;

struct Model {
    link: ComponentLink<Self>,
    reader: ReaderService,
    tasks: Vec<ReaderTask>,
    files: Vec<Sol>,
}

enum Msg {
    Files(Vec<File>),
    Loaded(FileData),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            reader: ReaderService::new(),
            tasks: vec![],
            files: vec![],
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let task = {
                        let callback = self.link.callback(Msg::Loaded);
                        self.reader.read_file(file, callback).unwrap()
                    };
                    self.tasks.push(task);
                }
            }
            Msg::Loaded(file) => {
                let sol = LSODeserializer::default()
                    .parse_full(&file.content)
                    .unwrap()
                    .1;
                self.files.push(sol);
            }
            _ => return false,
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <input type="file" onchange=self.link.callback(move |value| {
                            let mut result = Vec::new();
                            if let ChangeData::Files(files) = value {
                                let files = js_sys::try_iter(&files)
                                    .unwrap()
                                    .unwrap()
                                    .into_iter()
                                    .map(|v| File::from(v.unwrap()));
                                result.extend(files);
                            }
                            Msg::Files(result)
                        })/>

                 <ul>
                    { for self.files.iter().map(|f| self.view_file(f)) }
                </ul>
            </div>
        }
    }
}

impl Model {
    fn view_array_element(&self, index: usize, data: &SolValue) -> Html {
        html! {
            <div>
                <p>{index}</p>
                { self.view_sol_value(data) }
            </div>
        }
    }

    fn view_array_index(&self, index: usize) -> Html {
        html! {
            <div>
                <p>{index}</p>
            </div>
        }
    }

    fn view_sol_value(&self, data: &SolValue) -> Html {
        match data {
            SolValue::Object(elements, class_def) => {
                html! {
                    <ul>
                        { for elements.iter().map(|e| self.view_sol_element(e))}
                    </ul>
                }
            }
            SolValue::StrictArray(x) => {
                html! {
                    <ul>
                        { for x.iter().enumerate().map(|(i, v)| self.view_array_element(i, v))}
                    </ul>
                }
            }
            SolValue::ECMAArray(dense, assoc, size) => {
                html! {
                    <ul>
                        { for dense.iter().enumerate().map(|(i, v)| self.view_array_element(i, v))}
                        { for assoc.iter().map(|e| self.view_sol_element(e))}
                    </ul>
                }
            }
            SolValue::VectorInt(x, _fixed_len) => {
                html! {
                <ul>
                    { for x.iter().enumerate().map(|(i, v)| self.view_array_index(i) )}
                </ul>
                }
            }
            SolValue::VectorUInt(x, _fixed_len) => {
                html! {
                <ul>
                    { for x.iter().enumerate().map(|(i, v)| self.view_array_index(i) )}
                </ul>
                }
            }
            SolValue::VectorDouble(x, _fixed_len) => {
                html! {
                <ul>
                    { for x.iter().enumerate().map(|(i, v)| self.view_array_index(i) )}
                </ul>
                }
            }
            SolValue::VectorObject(children, name, _fixed_len) => {
                html! {
                    <ul>
                        { for children.iter().enumerate().map(|(i, v)| self.view_array_element(i, v))}
                    </ul>
                }
            }
            _ => html! { <div></div> },
        }
    }

    fn view_sol_element(&self, data: &SolElement) -> Html {
        html! {
            <li>
                <p>{ &data.name }</p>
                {self.view_sol_value(&data.value)}
            </li>
        }
    }

    fn view_file(&self, data: &Sol) -> Html {
        html! {
            <div>
                <p>{ &format!("Name: {}", data.header.name) }</p>
                <p>{ &format!("Size: {} bytes", data.header.length) }</p>
                <p>{ &format!("Version: {}", data.header.format_version) }</p>
                <ul>
                    { for data.body.iter().map(|e| self.view_sol_element(e))}
                </ul>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<Model>::new().mount_to_body();
}
