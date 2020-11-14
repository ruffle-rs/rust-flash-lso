#![recursion_limit = "1024"]

use std::string::ToString;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};

use flash_lso::flex;
use flash_lso::types::{Attribute, Element, Sol, Value};
use flash_lso::LSODeserializer;

mod blob_bindgen;
mod component_hexview;
pub mod component_tab;
pub mod component_tabs;
pub mod component_treenode;
pub mod jquery_bindgen;
mod uintarray_bindgen;
mod url_bindgen;

use crate::component_hexview::HexView;
use crate::component_tab::Tab;
use crate::component_tabs::Tabs;
use crate::component_treenode::TreeNode;
use flash_lso::encoder::write_to_bytes;
use std::ops::Deref;

#[derive(Clone)]
pub struct EditableValue {
    value: Value,
    callback: Callback<Value>,
}

struct Model {
    link: ComponentLink<Self>,
    reader: ReaderService,
    tasks: Vec<ReaderTask>,
    files: Vec<Sol>,
    current_selection: Option<EditableValue>,
}

enum Msg {
    Files(Vec<File>),
    Loaded(FileData),
    Selection(EditableValue),
    Edited(Value),
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

                let sol = parser.parse(&file.content).unwrap().1;
                self.files.push(sol);
            }
            Msg::Selection(val) => self.current_selection = Some(val),
            Msg::Edited(val) => self.current_selection.as_ref().unwrap().callback.emit(val),
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
    fn value_details(&self, val: EditableValue) -> Html {
        match val.value {
            Value::Object(children, Some(def)) => {
                let def_clone = def.clone();
                let dynamic_icon = if def.attributes.contains(Attribute::DYNAMIC) {
                    "icon/check.svg"
                } else {
                    "icon/x.svg"
                };
                let external_icon = if def.attributes.contains(Attribute::EXTERNAL) {
                    "icon/check.svg"
                } else {
                    "icon/x.svg"
                };

                return html! {
                    <>
                      <div class="input-group mb-2">
                        <div class="input-group-prepend">
                          <div class="input-group-text">{"Name"}</div>
                        </div>
                        <input onchange={ self.link.callback(move |cd| {
                            if let ChangeData::Value(s) = cd {
                                let mut new_def = def.clone();
                                new_def.name = s;
                                Msg::Edited(Value::Object(children.clone(), Some(new_def)))
                            } else {
                                Msg::Edited(Value::Object(children.clone(), Some(def.clone())))
                            }
                        })} value={def.name.clone()} class="form-control" type="text" id="inlineFormInputGroup"/>
                      </div>

                      <ul class="list-group list-group-horizontal mt-2 mb-2">
                          <li class="list-group-item"><img src={dynamic_icon} style={"width: 32; height: 32;"} class={"mr-2"}/>{"Dynamic"}</li>
                          <li class="list-group-item"><img src={external_icon} style={"width: 32; height: 32;"} class={"mr-2"}/>{"External"}</li>
                      </ul>
                        <p>{"static properties"}</p>
                        <table class="table table-striped">
                            { for def_clone.static_properties.iter().map(|p| html! {
                                <tr>
                                    <td>{p}</td>
                                </tr>
                            })}
                        </table>
                    </>
                };
            }
            Value::VectorObject(_, name, _) => html! {
                <>
                <p>{"name"}</p>
                <p>{name}</p>
                </>
            },
            Value::Number(n) => html! {
                <input onchange={ self.link.callback(move |cd| {
                    if let ChangeData::Value(s) = cd {
                        if let Ok(data) = s.parse::<f64>() {
                            Msg::Edited(Value::Number(data))
                        } else {
                            Msg::Edited(Value::Number(n))
                        }
                    } else {
                        Msg::Edited(Value::Number(n))
                    }
                })} value={n} class="form-control"/>
            },
            Value::Integer(n) => html! {
                <input onchange={ self.link.callback(move |cd| {
                    if let ChangeData::Value(s) = cd {
                        if let Ok(data) = s.parse::<i32>() {
                            Msg::Edited(Value::Integer(data))
                        } else {
                            Msg::Edited(Value::Integer(n))
                        }
                    } else {
                        Msg::Edited(Value::Integer(n))
                    }
                })} value={n} class="form-control"/>
            },
            Value::ByteArray(n) => html! {
                <HexView bytes={n}/>
            },
            Value::String(s) => html! {
                <input onchange={ self.link.callback(move |cd| {
                    if let ChangeData::Value(s) = cd {
                        Msg::Edited(Value::String(s.clone()))
                    } else {
                        Msg::Edited(Value::String(s.clone()))
                    }
                })} value={s.clone()} class="form-control"/>
            },
            Value::Bool(b) => html! {
                <div class="custom-control custom-switch">
                  <input type={"checkbox"} class={"custom-control-input"} id={"customSwitch1"} checked={b} onclick={self.link.callback(move |_| {
                    Msg::Edited(Value::Bool(!b))
                  })}/>
                  <label class={"custom-control-label"} for={"customSwitch1"}>{"State"}</label>
                </div>
            },
            Value::Date(x, tz) => html! {
                <>
                <div class="input-group mb-2">
                    <div class="input-group-prepend">
                      <div class="input-group-text">{"Epoch"}</div>
                    </div>
                    <input onchange={ self.link.callback(move |cd| {
                        if let ChangeData::Value(s) = cd {
                            if let Ok(x) = s.parse::<f64>() {
                                Msg::Edited(Value::Date(x, tz))
                            } else {
                                Msg::Edited(Value::Date(x, tz))
                            }
                        } else {
                            Msg::Edited(Value::Date(x, tz))
                        }
                    })} value={x} class="form-control" type="number"/>
                  </div>

                  { if tz.is_some() { html!{
                  <div class="input-group mb-2">
                    <div class="input-group-prepend">
                      <div class="input-group-text">{"Timezone"}</div>
                    </div>
                    <input onchange={ self.link.callback(move |cd| {
                        if let ChangeData::Value(s) = cd {
                            if let Ok(tz) = s.parse::<u16>() {
                                Msg::Edited(Value::Date(x, Some(tz)))
                            } else {
                                Msg::Edited(Value::Date(x, tz))
                            }
                        } else {
                            Msg::Edited(Value::Date(x, tz))
                        }
                    })} value={tz.unwrap()} class="form-control" type="number"/>
                  </div>
                  }} else {html!{}}}
                </>
            },
            Value::XML(content, string) => html! {
                <input onchange={ self.link.callback(move |cd| {
                    if let ChangeData::Value(s) = cd {
                        Msg::Edited(Value::XML(s.clone(), string))
                    } else {
                        Msg::Edited(Value::XML(content.clone(), string))
                    }
                })} value={content.clone()} class="form-control"/>
            },
            // Value::AMF3(e) => self.value_details(e.clone()),
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

    fn test(&self, index: usize) -> Html {
        let bytes = write_to_bytes(&self.files[index]);

        let options: js_sys::Object = js_sys::Object::new();

        let arr: uintarray_bindgen::Uint8Array =
            uintarray_bindgen::Uint8Array::new(bytes.len() as u32);
        for (i, b) in bytes.iter().enumerate() {
            arr.set(i as u32, (*b).into());
        }

        let arr2: js_sys::Array = js_sys::Array::new_with_length(1);
        arr2.set(0, arr.into());

        let blob = blob_bindgen::Blob::new(arr2, options.into());
        let url = url_bindgen::URL::createObjectURL(&blob);

        html! {
            <a href={url} download={"save.sol"} class="btn btn-primary">{"Save"}</a>
        }
    }

    fn view_file(&self, index: usize, data: &Sol) -> Html {
        html! {
            <div class="container-fluid">
                <div class="row">
                    <div class="col-5">
                        <ul class="list-group list-group-horizontal mt-2">
                          <li class="list-group-item"><img src={"icon/database.svg"} style={"width: 32; height: 32;"} class={"mr-2"}/>{data.header.length}</li>
                          <li class="list-group-item">{data.header.format_version}</li>
                        </ul>
                        { self.test(index) }

                        <div id="tree">
                            <span><img src={"icon/file.svg"} style={"width: 32; height: 32;"} class={"mr-2"}/>{"/"}</span>
                            <ul>
                                { for data.body.iter().map(|e| html! {
                                    <TreeNode name={e.name.clone()} value={e.value.deref().clone()} parent_callback={self.link.callback(|val| Msg::Selection(val))}></TreeNode>
                                })}
                            </ul>
                        </div>
                    </div>
                    <div class="col-7">
                        {
                            if let Some(selection) = &self.current_selection {
                                let details_content = self.value_details(selection.clone());
                                let value_type = match &selection.value {
                                    Value::Number(_) => "Number".to_string(),
                                    Value::Bool(_) => "Boolean".to_string(),
                                    Value::String(_) => "String".to_string(),
                                    Value::Object(_, _) => "Object".to_string(),
                                    Value::Null => "Null".to_string(),
                                    Value::Undefined => "Undefined".to_string(),
                                    Value::ECMAArray(_, _, _) => "ECMAArray".to_string(),
                                    Value::StrictArray(_) => "StrictArray".to_string(),
                                    Value::Date(_, _) => "Date".to_string(),
                                    Value::Unsupported => "Unsupported".to_string(),
                                    Value::XML(_, _) => "XML".to_string(),
                                    Value::AMF3(_) => "AMF3<TODO>".to_string(),
                                    Value::Integer(_) => "Integer".to_string(),
                                    Value::ByteArray(_) => "ByteArray".to_string(),
                                    Value::VectorInt(_, _) => "Vector<Int>".to_string(),
                                    Value::VectorUInt(_, _) => "Vector<UInt>".to_string(),
                                    Value::VectorDouble(_, _) => "Vector<Double>".to_string(),
                                    Value::VectorObject(_, _, _) => "Vector<Object>".to_string(),
                                    Value::Dictionary(_, _) => "Dictionary".to_string(),
                                    Value::Custom(_, _, cd) => {
                                        if let Some(cd) = cd {
                                            format!("Custom<{}>", cd.name)
                                        } else {
                                            "Custom<Unknown>".to_string()
                                        }
                                    },
                                     _ => "Unknown".to_string()
                                };

                                html! {
                                    <>
                                    <ul class="list-group list-group-horizontal mt-2 mb-2">
                                      <li class="list-group-item">{value_type}</li>
                                    </ul>
                                    {{details_content}}
                                    </>
                                }
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
