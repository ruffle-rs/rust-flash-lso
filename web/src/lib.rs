#![recursion_limit = "256"]

use std::string::ToString;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};

use flash_lso::flex;
use flash_lso::types::{Attribute, Element, Sol, SolElement, SolValue};
use flash_lso::LSODeserializer;

pub mod component_tab;
pub mod component_tabs;
pub mod jquery_bindgen;

use crate::component_tab::Tab;
use crate::component_tabs::Tabs;
use std::ops::Deref;

struct Model {
    link: ComponentLink<Self>,
    reader: ReaderService,
    tasks: Vec<ReaderTask>,
    files: Vec<Sol>,
    current_selection: Option<Element>,
}

enum Msg {
    Files(Vec<File>),
    Loaded(FileData),
    Selection(Element),
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
            current_selection: None,
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
                let mut parser = LSODeserializer::default();
                flex::decode::register_decoders(&mut parser.amf3_decoder);

                let sol = parser.parse_full(&file.content).unwrap().1;
                self.files.push(sol);
            }
            Msg::Selection(val) => self.current_selection = Some(val),
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
                { self.navbar() }
                <Tabs>
                    { for self.files.iter().enumerate().map(|(i,f)| html_nested! {
                    <Tab label={&f.header.name}>
                        { self.view_file(i,f)}
                    </Tab>
                }) }
                </Tabs>
            </div>
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        // jquery_bindgen::jquery("#tree").jstree();
    }
}

impl Model {
    fn value_details(&self, val: Element) -> Html {
        match val.borrow_mut().deref() {
            SolValue::Object(_, Some(def)) => html! {
                <>
                    <p>{"name"}</p>
                    <p>{&def.name}</p>
                    <p>{"is dynamic"}</p>
                    <p>{def.attributes.contains(Attribute::DYNAMIC)}</p>
                    <p>{"is external"}</p>
                    <p>{def.attributes.contains(Attribute::EXTERNAL)}</p>
                    <p>{"static properties"}</p>
                    <ul>
                        { for def.static_properties.iter().map(|p| html! {
                            <li>{p}</li>
                        })}
                    </ul>
                </>
            },
            SolValue::TypedObject(name, _) => html! {
                <>
                <p>{"name"}</p>
                <p>{name}</p>
                </>
            },
            SolValue::VectorObject(_, name, _) => html! {
                <>
                <p>{"name"}</p>
                <p>{name}</p>
                </>
            },
            SolValue::Number(n) => html! {
                <p>{n}</p>
            },
            SolValue::Integer(n) => html! {
                <p>{n}</p>
            },
            SolValue::ByteArray(n) => html! {
                <p>{format!("{:?}", n.as_slice())}</p>
            },
            SolValue::String(s) => html! {
                <p>{s}</p>
            },
            SolValue::Bool(b) => html! {
                <p>{ if *b {"true"} else {"false"} }</p>
            },
            SolValue::Null => html! {
                <p>{ "null" }</p>
            },
            SolValue::Undefined => html! {
                <p>{ "undefined" }</p>
            },
            SolValue::Unsupported => html! {
                <p>{ "unsupported" }</p>
            },
            SolValue::Date(x, tz) => html! {
                <>
                <p>{ "epoch" }</p>
                <p>{ x }</p>
                <p>{ "timezone" }</p>
                <p>{ format!("{:?}", tz) }</p>
                </>
            },
            SolValue::XML(content, string) => html! {
                <p>{ content }</p>
            },
            _ => html! {},
        }
    }

    fn navbar(&self) -> Html {
        html! {
            <nav class="navbar navbar-expand-lg">
                <ul class="navbar-nav mr-auto">
                    <li class="nav-item">
                        <label for="files" class="btn btn-primary">{"Open"}</label>
                        <input id="files" class="btn btn-default" style="visibility:hidden;" type="file" onchange=self.link.callback(move |value| {
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
                    </li>
                </ul>
            </nav>
        }
    }

    fn view_array_element(&self, index: usize, data: &Element) -> Html {
        html! {
            <div>
                <p>{index}</p>
                { self.view_sol_value(data.clone()) }
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

    fn view_sol_value(&self, data: Element) -> Html {
        match data.borrow_mut().deref() {
            SolValue::Object(elements, _class_def) => html! {
                <ul>
                    { for elements.iter().map(|e| self.view_sol_element(Box::from(e.clone())))}
                </ul>
            },
            SolValue::TypedObject(_name, elements) => html! {
                <ul>
                    { for elements.iter().map(|e| self.view_sol_element(Box::from(e.clone())))}
                </ul>
            },
            SolValue::StrictArray(x) => html! {
                <ul>
                    { for x.iter().enumerate().map(|(i, v)| self.view_array_element(i, v))}
                </ul>
            },
            SolValue::ECMAArray(dense, assoc, _size) => html! {
                    <ul>
                        { for dense.iter().enumerate().map(|(i, v)| self.view_array_element(i, v))}
                        { for assoc.iter().map(|e| self.view_sol_element(Box::from(e.clone())))}
                    </ul>
            },
            SolValue::VectorInt(x, _fixed_len) => html! {
                <ul>
                    { for x.iter().enumerate().map(|(i, v)| self.view_array_index(i) )}
                </ul>
            },
            SolValue::VectorUInt(x, _fixed_len) => html! {
                <ul>
                    { for x.iter().enumerate().map(|(i, v)| self.view_array_index(i) )}
                </ul>
            },
            SolValue::VectorDouble(x, _fixed_len) => html! {
                <ul>
                    { for x.iter().enumerate().map(|(i, v)| self.view_array_index(i) )}
                </ul>
            },
            SolValue::VectorObject(children, _name, _fixed_len) => html! {
                <ul>
                    { for children.iter().enumerate().map(|(i, v)| self.view_array_element(i, v))}
                </ul>
            },
            SolValue::Dictionary(children, _) => html! {
                <ul>
                    { for children.iter().map(|(k, v)| html! {
                            <>
                            <li><span >{ "key" }</span></li>
                            <li><span >{ "value" }</span></li>
                            </>
                        })}
                </ul>
            },
            SolValue::Custom(el, el2, class_def) => html! {
                <ul>
                    <li>
                        {"Custom elements"}
                        <ul>
                            { for el.iter().map(|e| self.view_sol_element(Box::from(e.clone())))}
                        </ul>
                    </li>
                    <li>
                        {"Standard elements"}
                        <ul>
                            { for el2.iter().map(|e| self.view_sol_element(Box::from(e.clone())))}
                        </ul>
                    </li>
                </ul>
            },
            _ => html! {},
        }
    }

    fn view_sol_element(&self, data: Box<SolElement>) -> Html {
        let name = data.name.clone();
        let value = data.value.clone();
        let value_clone = data.value.clone();
        html! {
            <li>
                <span onclick=self.link.callback(move |_| Msg::Selection(value_clone.clone()))>{ name }</span>
                {self.view_sol_value(value)}
            </li>
        }
    }

    fn view_file(&self, index: usize, data: &Sol) -> Html {
        html! {


            <div class="container-fluid">
                <div class="row">
                    <div class="col-4">
                        <p>{ &format!("Name: {}", data.header.name) }</p>
                        <p>{ &format!("Size: {} bytes", data.header.length) }</p>
                        <p>{ &format!("Version: {}", data.header.format_version) }</p>
                        <div id="tree">
                            <ul>
                                { for data.body.iter().map(|e| self.view_sol_element(Box::from(e.clone())))}
                            </ul>
                        </div>
                    </div>
                    <div class="col-8">
                        {
                            if let Some(selection) = &self.current_selection {
                                self.value_details(selection.clone())
                            } else {
                                html! { <p>{"Select an item"}</p> }
                            }
                        }
                    </div>
                </div>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<Model>::new().mount_to_body();
}
