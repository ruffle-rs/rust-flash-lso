use std::string::ToString;
use yew::prelude::*;

use flash_lso::extra::flex;
use flash_lso::read::Reader;
use flash_lso::types::{Attribute, Element, Lso, Value};

use crate::blob_bindgen::Blob;
use crate::component_hexview::HexView;
use crate::component_modal::modal::Modal;
use crate::component_modal::ModalContainer;
use crate::component_number_input::NumberInput;
use crate::component_string_input::StringInput;
use crate::component_tab::Tab;
use crate::component_tabs::Tabs;
use crate::component_treenode::TreeNode;
use crate::uintarray_bindgen::Uint8Array;
use crate::url_bindgen::URL;
use crate::web_expect::WebSafeExpect;
use crate::EditableValue;
use crate::TreeNodePath;
use flash_lso::write::write_to_bytes;
use std::ops::Deref;
use wasm_bindgen::JsCast;
use web_sys::File;
use web_sys::{EventTarget, HtmlInputElement};
use yew::events::Event;

pub struct LoadedFile {
    pub file_name: String,
    pub file: Option<Lso>,
}

impl LoadedFile {
    pub fn empty_from_file(file: &File) -> Self {
        LoadedFile {
            file: None,
            file_name: file.name(),
        }
    }
}

pub struct Model {
    tasks: Vec<gloo_file::callbacks::FileReader>,
    files: Vec<LoadedFile>,
    current_selection: Option<EditableValue>,
    current_tab: Option<usize>,
    error_messages: Vec<String>,
    search: String,
}

#[derive(Default, Debug)]
pub struct FileData {
    /// The name of the file
    pub name: String,
    /// The bytes in the file
    pub content: Vec<u8>,
}

#[derive(Debug)]
pub enum Msg {
    Files(Vec<File>),
    Loaded(usize, FileData),
    Selection(EditableValue),
    Edited(Value),
    TabSelected(usize),
    CloseTab(usize),
    CloseModal(usize),
    RootSelected,
    SearchQuery(String),
    ElementChange(Element),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            tasks: vec![],
            files: vec![],
            current_selection: None,
            current_tab: None,
            error_messages: Vec::new(),
            search: "".to_string(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::info!("MODEL msg={:?}", msg);
        match msg {
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let index = self.files.len();
                    self.files.push(LoadedFile::empty_from_file(&file));

                    let file_name = file.name();
                    let cb = ctx.link().callback(move |data: Vec<u8>| {
                        Msg::Loaded(
                            index,
                            FileData {
                                name: file_name.clone(),
                                content: data,
                            },
                        )
                    });

                    let task = gloo_file::callbacks::read_as_bytes(
                        &gloo_file::Blob::from(file),
                        move |res| {
                            if let Ok(res) = res {
                                cb.emit(res);
                            }
                        },
                    );
                    self.tasks.push(task);
                }
            }
            Msg::Loaded(index, file) => {
                let mut parser = Reader::default();
                flex::read::register_decoders(&mut parser.amf3_decoder);

                match parser.parse(&file.content) {
                    Ok(sol) => {
                        self.files
                            .get_mut(index)
                            .web_expect(&format!("No loading file at index {}", index))
                            .file = Some(sol);

                        if self.current_tab.is_none() {
                            self.current_tab = Some(0);
                        }
                    }
                    Err(e) => {
                        log::warn!("Got error {:?}", e);
                        self.error_messages
                            .push(format!("Failed to load '{}'", file.name));
                        self.files.remove(index);
                    }
                }
            }
            Msg::Selection(val) => {
                if self.current_selection.as_ref().map(|ev| ev.value.clone())
                    == Some(val.value.clone())
                {
                    self.current_selection = None;
                } else {
                    self.current_selection = Some(val);
                }
            }
            Msg::Edited(val) => {
                self.current_selection
                    .as_ref()
                    .web_expect("Unable to get current selection")
                    .callback
                    .emit(val.clone());
                self.current_selection
                    .as_mut()
                    .web_expect("Unable to get mut current selection")
                    .value = val;
            }
            Msg::TabSelected(index) => self.current_tab = Some(index),
            Msg::CloseTab(index) => {
                if let Some(sel) = self.current_tab {
                    if sel > 0 {
                        if sel >= index {
                            self.current_tab = Some(sel - 1);
                        }
                    } else {
                        self.current_tab = None;
                    }
                }
                self.files.remove(index);
            }
            Msg::CloseModal(index) => {
                self.error_messages.remove(index);
            }
            Msg::RootSelected => {
                self.current_selection = None;
            }
            Msg::SearchQuery(s) => {
                self.search = s;
            }
            Msg::ElementChange(el) => {
                if let Some(tab_index) = self.current_tab {
                    if let Some(file) = self.files.get_mut(tab_index) {
                        if let Some(ref mut file) = &mut file.file {
                            let old_element = file.body.iter().position(|e| e.name == el.name);
                            if let Some(index) = old_element {
                                file.body[index] = el.clone();
                                log::info!("Set {} to {:?}", index, el);
                            }
                        }
                    }
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                { self.navbar(ctx) }
                { self.error_modal(ctx) }

                <Tabs selected={self.current_tab} ontabselect={ctx.link().callback(Msg::TabSelected)} ontabremove={ctx.link().callback(Msg::CloseTab)}>
                    { for self.files.iter().enumerate().map(|(i,f)| html_nested! {
                    <Tab label={f.file_name.clone()} loading={f.file.is_none()}>
                        { if let Some(file) = &f.file {
                            self.view_file(ctx, i, file)
                        } else {
                            html! {}
                        }}
                    </Tab>
                }) }
                </Tabs>
            </div>
        }
    }
}

impl Model {
    fn error_modal(&self, ctx: &Context<Self>) -> Html {
        html! {
            <ModalContainer onclose={ctx.link().callback(Msg::CloseModal)}>
                { for self.error_messages.iter().map(|e| html_nested! {
                    <Modal title={"Loading Failed"} content={e.clone()}/>
                })}
            </ModalContainer>
        }
    }

    fn value_details(&self, val: EditableValue, ctx: &Context<Self>) -> Html {
        match val.value {
            Value::Object(children, Some(def)) => {
                let def_clone = def.clone();
                let dynamic_icon = if def.attributes.contains(Attribute::Dynamic) {
                    "icon/check.svg"
                } else {
                    "icon/x.svg"
                };
                let external_icon = if def.attributes.contains(Attribute::External) {
                    "icon/check.svg"
                } else {
                    "icon/x.svg"
                };

                let static_props_details = if def.static_properties.is_empty() {
                    html! {}
                } else {
                    html! {
                    <table class="table table-striped">
                            <thead>
                                <tr>
                                    <th>{"Static Properties"}</th>
                                </tr>
                            </thead>
                            { for def_clone.static_properties.iter().map(|p| html! {
                                <tr>
                                    <td>{p}</td>
                                </tr>
                            })}
                        </table>
                    }
                };

                html! {
                    <>
                      <div class="input-group mb-2">
                        <div class="input-group-prepend">
                          <div class="input-group-text">{"Name"}</div>
                        </div>
                        <input
                        onchange={ctx.link().batch_callback(move |e: Event| {
                            let target: Option<EventTarget> = e.target();
                            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

                            input.map(|input| {
                                let mut new_def = def.clone();
                                new_def.name = input.value();
                                Msg::Edited(Value::Object(children.clone(), Some(new_def)))
                            })
                        })}  value={def.name.clone()} class="form-control" type="text"/>
                      </div>

                      <ul class="list-group list-group-horizontal mt-2 mb-2">
                          <li class="list-group-item"><img alt={"Dynamic icon"} src={dynamic_icon} style={"width: 32; height: 32;"} class={"mr-2"}/>{"Dynamic"}</li>
                          <li class="list-group-item"><img alt={"External icon"} src={external_icon} style={"width: 32; height: 32;"} class={"mr-2"}/>{"External"}</li>
                      </ul>
                        { static_props_details }
                    </>
                }
            }
            Value::VectorObject(elements, name, fixed_length) => {
                let elements_clone_2 = elements.clone();
                html! {
                    <>
                    <StringInput onchange={ctx.link().callback(move |new_name| Msg::Edited(Value::VectorObject(elements.clone(), new_name, fixed_length)))} value={name.clone()}/>
                    <div class="custom-control custom-switch">
                      <input type={"checkbox"} class={"custom-control-input"} id={"customSwitch1"} checked={fixed_length} onclick={ctx.link().callback(move |_| {
                        Msg::Edited(Value::VectorObject(elements_clone_2.clone(), name.clone(), !fixed_length))
                      })}/>
                      <label class={"custom-control-label"} for={"customSwitch1"}>{"Fixed Length"}</label>
                    </div>
                    </>
                }
            }
            Value::Number(n) => html! {
                <NumberInput<f64> onchange={ctx.link().callback(move |data| Msg::Edited(Value::Number(data)))} value={n}/>
            },
            Value::Integer(n) => html! {
                <NumberInput<i32> onchange={ctx.link().callback(move |data| Msg::Edited(Value::Integer(data)))} value={n}/>
            },
            Value::ByteArray(n) => {
                let n_clone = n.clone();
                html! {
                <>
                    <HexView
                        bytes={n.clone()}
                        onchange={ctx.link().callback(move |data| Msg::Edited(Value::ByteArray(data)))}
                        onadd={ctx.link().callback(move |_| {
                            let mut e = n.clone();
                            e.push(0);
                            Msg::Edited(Value::ByteArray(e))
                        })}
                        onremove={ctx.link().callback(move |index| {
                            let mut e = n_clone.clone();
                            e.remove(index);
                            Msg::Edited(Value::ByteArray(e))
                        })}/>
                  </>
                }
            }
            Value::String(s) => html! {
                <StringInput onchange={ctx.link().callback(move |s| Msg::Edited(Value::String(s)))} value={s}/>
            },
            Value::Bool(b) => html! {
                <div class="custom-control custom-switch">
                  <input type={"checkbox"} class={"custom-control-input"} id={"customSwitch1"} checked={b} onclick={ctx.link().callback(move |_| {
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
                    <input onchange={ctx.link().batch_callback(move |e: Event| {
                            let target: Option<EventTarget> = e.target();
                            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

                            input.map(|input| {
                            if let Ok(x) = input.value().parse::<f64>() {
                                     Msg::Edited(Value::Date(x, tz))
                                 } else {
                                     Msg::Edited(Value::Date(x, tz))
                                }
                            })
                        })} value={format!("{}", x)} class="form-control" type="number"/>
                  </div>

                  { if tz.is_some() { html!{
                  <div class="input-group mb-2">
                    <div class="input-group-prepend">
                      <div class="input-group-text">{"Timezone"}</div>
                    </div>
                    <input
                      onchange={ctx.link().batch_callback(move |e: Event| {
                            let target: Option<EventTarget> = e.target();
                            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

                            input.map(|input| {
                            if let Ok(tz) = input.value().parse::<u16>() {
                                     Msg::Edited(Value::Date(x, Some(tz)))
                                 } else {
                                     Msg::Edited(Value::Date(x, tz))
                                }
                            })
                        })}
                      value={format!("{}", tz.web_expect("Unable to get timezone"))} class="form-control" type="number"/>
                  </div>
                  }} else {html!{}}}
                </>
            },
            Value::XML(content, string) => html! {
                <StringInput onchange={ctx.link().callback(move |s| Msg::Edited(Value::XML(s, string)))} value={content}/>
            },
            Value::VectorInt(elements, fixed_length) => {
                let elements_clone = elements.clone();
                let elements_clone3 = elements.clone();
                html! {
                    <>
                        <div class="custom-control custom-switch mb-2">
                          <input type={"checkbox"} class={"custom-control-input"} id={"vectorIntFixed"} checked={fixed_length} onclick={ctx.link().callback(move |_| {
                            Msg::Edited(Value::VectorInt(elements_clone.clone(), !fixed_length))
                          })}/>
                          <label class={"custom-control-label"} for={"vectorIntFixed"}>{"Fixed Length"}</label>
                        </div>

                        <table class="table table-striped">
                            <thead>
                                <tr>
                                    <th>{"#"}</th>
                                    <th>{"Value"}</th>
                                    <th></th>
                                    <th></th>
                                </tr>
                            </thead>
                            <tbody>
                            { for elements.iter().enumerate().map(|(i, e)| {
                                let elements_clone4 = elements_clone3.clone();
                                let elements_clone5 = elements_clone3.clone();
                                html! {
                                <tr>
                                    <td>{i}</td>
                                    <td>
                                        <input onchange={ctx.link().batch_callback(move |e: Event| {
                                        let target: Option<EventTarget> = e.target();
                                        let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

                                        input.map(|input| {
                                            if let Ok(data) = input.value().parse::<i32>() {
                                                let mut new_elements = elements_clone5.clone();
                                                new_elements[i] = data;
                                                Msg::Edited(Value::VectorInt(new_elements, fixed_length))
                                             } else {
                                                 Msg::Edited(Value::VectorInt(elements_clone5.clone(), fixed_length))
                                            }
                                        })
                                    })} value={format!("{}", e)} class="form-control" type="text"/>
                                    </td>
                                    <td></td>
                                    <td>
                                    <span onclick={ctx.link().callback(move |_| {
                                        let mut e = elements_clone4.clone();
                                        e.remove(i);
                                        Msg::Edited(Value::VectorInt(e, fixed_length))
                                      })} class="btn btn-link">
                                            <img alt={"Remove"} src={"icon/x.svg"} style={"width: 32; height: 32;"} class={"mr-2"}/>
                                        </span>
                                    </td>
                                </tr>
                            }
                            })}
                            </tbody>
                        </table>
                        <span onclick={ctx.link().callback(move |_| {
                            let mut e = elements_clone3.clone();
                            e.push(0);
                            Msg::Edited(Value::VectorInt(e, fixed_length))
                          })} class="btn btn-primary">{"Add"}</span>
                    </>
                }
            }
            Value::VectorUInt(elements, fixed_length) => {
                let elements_clone = elements.clone();
                let elements_clone3 = elements.clone();
                html! {
                    <>
                        <div class="custom-control custom-switch mb-2">
                          <input type={"checkbox"} class={"custom-control-input"} id={"vectorIntFixed"} checked={fixed_length} onclick={ctx.link().callback(move |_| {
                            Msg::Edited(Value::VectorUInt(elements_clone.clone(), !fixed_length))
                          })}/>
                          <label class={"custom-control-label"} for={"vectorIntFixed"}>{"Fixed Length"}</label>
                        </div>

                        <table class="table table-striped">
                            <thead>
                                <tr>
                                    <th>{"#"}</th>
                                    <th>{"Value"}</th>
                                    <th></th>
                                    <th></th>
                                </tr>
                            </thead>
                            <tbody>
                            { for elements.iter().enumerate().map(|(i, e)| {
                                let elements_clone4 = elements_clone3.clone();
                                let elements_clone5 = elements_clone3.clone();
                                html! {
                                <tr>
                                    <td>{i}</td>
                                    <td>
                                     <input onchange={ctx.link().batch_callback(move |e: Event| {
                                        let target: Option<EventTarget> = e.target();
                                        let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

                                        input.map(|input| {
                                            if let Ok(data) = input.value().parse::<u32>() {
                                                let mut new_elements = elements_clone5.clone();
                                                new_elements[i] = data;
                                                Msg::Edited(Value::VectorUInt(new_elements, fixed_length))
                                             } else {
                                                 Msg::Edited(Value::VectorUInt(elements_clone5.clone(), fixed_length))
                                            }
                                        })
                                    })} value={format!("{}", e)} class="form-control" type="text"/>
                                    </td>
                                    <td></td>
                                    <td>
                                    <span onclick={ctx.link().callback(move |_| {
                                        let mut e = elements_clone4.clone();
                                        e.remove(i);
                                        Msg::Edited(Value::VectorUInt(e, fixed_length))
                                      })} class="btn btn-link">
                                            <img alt={"Remove"} src={"icon/x.svg"} style={"width: 32; height: 32;"} class={"mr-2"}/>
                                        </span>
                                    </td>
                                </tr>
                            }
                            })}
                            </tbody>
                        </table>
                        <span onclick={ctx.link().callback(move |_| {
                            let mut e = elements_clone3.clone();
                            e.push(0);
                            Msg::Edited(Value::VectorUInt(e, fixed_length))
                          })} class="btn btn-primary">{"Add"}</span>
                    </>
                }
            }
            Value::VectorDouble(elements, fixed_length) => {
                let elements_clone = elements.clone();
                let elements_clone3 = elements.clone();
                html! {
                    <>
                        <div class="custom-control custom-switch mb-2">
                          <input type={"checkbox"} class={"custom-control-input"} id={"vectorIntFixed"} checked={fixed_length} onclick={ctx.link().callback(move |_| {
                            Msg::Edited(Value::VectorDouble(elements_clone.clone(), !fixed_length))
                          })}/>
                          <label class={"custom-control-label"} for={"vectorIntFixed"}>{"Fixed Length"}</label>
                        </div>

                        <table class="table table-striped">
                            <thead>
                                <tr>
                                    <th>{"#"}</th>
                                    <th>{"Value"}</th>
                                    <th></th>
                                    <th></th>
                                </tr>
                            </thead>
                            <tbody>
                            { for elements.iter().enumerate().map(|(i, e)| {
                                let elements_clone4 = elements_clone3.clone();
                                let elements_clone5 = elements_clone3.clone();
                                html! {
                                <tr>
                                    <td>{i}</td>
                                    <td>
                                        <input onchange={ctx.link().batch_callback(move |e: Event| {
                                        let target: Option<EventTarget> = e.target();
                                        let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

                                        input.map(|input| {
                                            if let Ok(data) = input.value().parse::<f64>() {
                                                let mut new_elements = elements_clone5.clone();
                                                new_elements[i] = data;
                                                Msg::Edited(Value::VectorDouble(new_elements, fixed_length))
                                             } else {
                                                 Msg::Edited(Value::VectorDouble(elements_clone5.clone(), fixed_length))
                                            }
                                        })
                                    })} value={format!("{}", e)} class="form-control" type="text"/>
                                    </td>
                                    <td></td>
                                    <td>
                                    <span onclick={ctx.link().callback(move |_| {
                                        let mut e = elements_clone4.clone();
                                        e.remove(i);
                                        Msg::Edited(Value::VectorDouble(e, fixed_length))
                                      })} class="btn btn-link">
                                            <img alt={"Remove"} src={"icon/x.svg"} style={"width: 32; height: 32;"} class={"mr-2"}/>
                                        </span>
                                    </td>
                                </tr>
                            }
                            })}
                            </tbody>
                        </table>
                        <span onclick={ctx.link().callback(move |_| {
                            let mut e = elements_clone3.clone();
                            e.push(0.0);
                            Msg::Edited(Value::VectorDouble(e, fixed_length))
                          })} class="btn btn-primary">{"Add"}</span>
                    </>
                }
            }
            // Value::AMF3(e) => self.value_details(e.clone()),
            _ => html! {},
        }
    }

    fn navbar(&self, ctx: &Context<Self>) -> Html {
        html! {
            <nav class="navbar navbar-expand-lg">
                <ul class="navbar-nav mr-auto">
                    <li class="nav-item">
                        <div class="btn-group mr-2" role="group">
                            <label for="files" class="btn btn-primary">{"Open"}</label>
                            { self.save_button(ctx) }
                        </div>
                    </li>
                    <input id="files" style="visibility:hidden;" type="file" onchange={ctx.link().batch_callback(|e: Event| {
                        let target: Option<EventTarget> = e.target();
                        let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

                        input.map(|input| {
                            let mut result = Vec::new();

                            if let Some(files_list) = input.files() {
                                for ii in 0..files_list.length() {
                                    let file = files_list.item(ii).web_expect("filelist::item");
                                    result.push(file);
                                }
                            }

                            Msg::Files(result)
                        })
                    })}/>
                </ul>
            </nav>
        }
    }

    fn save_button(&self, _ctx: &Context<Self>) -> Html {
        if let Some(tab_index) = self.current_tab {
            let mut lso = self.files[tab_index]
                .file
                .clone()
                .web_expect("Failed to get file");
            let bytes = write_to_bytes(&mut lso).web_expect("Failed to write lso to bytes");

            let options: js_sys::Object = js_sys::Object::new();

            let arr: Uint8Array = Uint8Array::new(bytes.len() as u32);
            for (i, b) in bytes.iter().enumerate() {
                arr.set(i as u32, (*b).into());
            }

            let arr2: js_sys::Array = js_sys::Array::new_with_length(1);
            arr2.set(0, arr.into());

            let blob = Blob::new(arr2, options.into());
            let url = URL::createObjectURL(&blob);

            html! {
                <a href={url} download={"save.sol"} class="btn btn-primary" style="height: 38px">{"Save"}</a>
            }
        } else {
            html! {}
        }
    }

    fn view_file(&self, ctx: &Context<Self>, _index: usize, data: &Lso) -> Html {
        let root_class = "text-white bg-primary rounded-pill pl-2 pr-2";

        html! {
            <div class="container-fluid">
                <div class="row">
                    <div class="col-5">
                        <StringInput value={self.search.clone()} onchange={ctx.link().callback(Msg::SearchQuery)} class="mt-2 col-md-4" placeholder="Search..."/>

                        <div id="tree" class="mt-2">
                            <span onclick={ctx.link().callback(|_| Msg::RootSelected)}>
                                <img alt={"File"} src={"icon/file.svg"} style={"width: 32; height: 32;"} class="mr-2"/>
                            </span>
                            <span
                                class={root_class}
                                onclick={ctx.link().callback(move |_| Msg::RootSelected)}>{ "/" }</span>
                            <ul>
                                { for data.body.iter().map(|e| html! {
                                    <TreeNode element_callback={ctx.link().callback(Msg::ElementChange)} filter={self.search.clone()} selection={self.current_selection.clone()} parent_path={TreeNodePath::root()} name={e.name.clone()} value={e.value.deref().clone()} parent_callback={ctx.link().callback(Msg::Selection)}></TreeNode>
                                })}
                            </ul>
                        </div>
                    </div>
                    <div class="col-7">
                        {
                            if let Some(selection) = &self.current_selection {
                                let details_content = self.value_details(selection.clone(), ctx);
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
                                };

                                html! {
                                    <>
                                    <ul class="list-group list-group-horizontal mt-2 mb-2">
                                      <li class="list-group-item">{value_type}</li>
                                      <li class="list-group-item">{self.current_selection.clone().map(|cs| cs.path.string()).unwrap_or_else(|| "/".to_string())}</li>
                                    </ul>
                                    {{details_content}}
                                    </>
                                }
                            } else {
                                html! {
                                    <>
                                    <ul class="list-group list-group-horizontal mt-2">
                                      <li class="list-group-item"><img alt={"File format"} src={"icon/database.svg"} style={"width: 32; height: 32;"} class={"mr-2"}/>{data.header.length}</li>
                                      <li class="list-group-item">{data.header.format_version}</li>
                                    </ul>
                                    <p>{"Select an item for more details"}</p>
                                    </>
                             }
                            }
                        }
                    </div>
                </div>
            </div>
        }
    }
}
