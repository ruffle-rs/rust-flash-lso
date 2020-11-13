use crate::{EditableValue, Model};
use flash_lso::types::{Element, Value};
use std::ops::Deref;
use std::rc::Rc;
use yew::prelude::*;
use yew::{Children, Component, ComponentLink, Html, Properties};

pub enum Msg {
    Selection(EditableValue),
    Toggle,
    Edited(Value),
}

pub struct TreeNode {
    props: Props,
    link: ComponentLink<Self>,
    expanded: bool,
    value: Value,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub name: String,
    pub value: Value,
    // pub element: Element,
    pub parent_callback: Callback<EditableValue>,
}

impl Component for TreeNode {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let value = props.value.clone();
        Self {
            props,
            link,
            expanded: false,
            value,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::Selection(val) => {
                self.props.parent_callback.emit(val);
                true
            }
            Msg::Toggle => {
                self.expanded = !self.expanded;
                true
            }
            Msg::Edited(v) => {
                self.value = v;
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        //TODO: this will no longer update the value e.g. if the type of a child is changed
        if props != self.props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let name = self.props.name.clone();
        let value = self.value.clone();
        let value_clone = self.value.clone();

        let icon = if TreeNode::has_children(&value) {
            if self.expanded {
                "icon/folder-minus.svg"
            } else {
                "icon/folder-plus.svg"
            }
        } else {
            "icon/file.svg"
        };

        let callback = self.link.callback(|val| Msg::Edited(val));
        let v = self.value.clone();

        html! {
             <div>
                <span onclick=self.link.callback(|_| Msg::Toggle)>
                    <img src={icon} style={"width: 32; height: 32;"} class={"mr-2"}/>
                </span>
                <span onclick=self.link.callback(move |_| Msg::Selection(EditableValue {
            value: v.clone(),
            callback: callback.clone()
        }))>{ name }</span>
                { if self.expanded {
                    self.view_sol_value(Rc::new(self.value.clone()))
                } else {
                    html!{}
                }}
             </div>
        }
    }
}

impl TreeNode {
    pub fn has_children(data: &Value) -> bool {
        match data {
            Value::Object(_, _) => true,
            Value::StrictArray(_) => true,
            Value::ECMAArray(_, _, _) => true,
            Value::VectorObject(_, _, _) => true,
            Value::VectorDouble(_, _) => true,
            Value::VectorUInt(_, _) => true,
            Value::VectorInt(_, _) => true,
            Value::AMF3(_) => true,
            Value::Dictionary(_, _) => true,
            Value::Custom(_, _, _) => true,
            _ => false,
        }
    }

    pub fn view_array_index(&self, index: usize) -> Html {
        html! {
            <div>
                <p>{index}</p>
            </div>
        }
    }

    pub fn view_array_element(&self, index: usize, data: &Rc<Value>) -> Html {
        html! {
            <div>
                <TreeNode name={format!("{}", index)} value={data.deref().clone()} parent_callback={self.link.callback(|val| Msg::Selection(val))}></TreeNode>
            </div>
        }
    }

    pub fn view_sol_value(&self, data: Rc<Value>) -> Html {
        match data.deref() {
            Value::AMF3(e) => self.view_sol_value(e.clone()),
            Value::Object(elements, _class_def) => html! {
                <ul>
                    { for elements.iter().map(|e| html! {
                        <TreeNode name={e.name.clone()} value={e.value.deref().clone()} parent_callback={self.link.callback(|val| Msg::Selection(val))}></TreeNode>
                    })}
                </ul>
            },
            Value::StrictArray(x) => html! {
                <ul>
                    { for x.iter().enumerate().map(|(i, v)| self.view_array_element(i, v))}
                </ul>
            },
            Value::ECMAArray(dense, assoc, _size) => html! {
                    <ul>
                       { for dense.iter().enumerate().map(|(i, v)| self.view_array_element(i, v))}
                        { for assoc.iter().map(|e| html! {
                            <TreeNode name={e.name.clone()} value={e.value.deref().clone()} parent_callback={self.link.callback(|val| Msg::Selection(val))}></TreeNode>
                        })}
                    </ul>
            },
            Value::VectorInt(x, _fixed_len) => html! {
                <ul>
                   { for x.iter().enumerate().map(|(i, v)| self.view_array_index(i) )}
                </ul>
            },
            Value::VectorUInt(x, _fixed_len) => html! {
                <ul>
                   { for x.iter().enumerate().map(|(i, v)| self.view_array_index(i) )}
                </ul>
            },
            Value::VectorDouble(x, _fixed_len) => html! {
                <ul>
                   { for x.iter().enumerate().map(|(i, v)| self.view_array_index(i) )}
                </ul>
            },
            Value::VectorObject(children, _name, _fixed_len) => html! {
                <ul>
                   { for children.iter().enumerate().map(|(i, v)| self.view_array_element(i, v))}
                </ul>
            },
            Value::Dictionary(children, _) => html! {
                <ul>
                    { for children.iter().map(|(k, v)| html! {
                            <>
                            <li><span >{ "key" }</span></li>
                            <li><span >{ "value" }</span></li>
                            </>
                        })}
                </ul>
            },
            Value::Custom(el, el2, _class_def) => html! {
                <ul>
                    <li>
                        {"Custom elements"}
                        <ul>
                            { for el.iter().map(|e| html! {
                                <TreeNode name={e.name.clone()} value={e.value.deref().clone()} parent_callback={self.link.callback(|val| Msg::Selection(val))}></TreeNode>
                            })}
                        </ul>
                    </li>
                    <li>
                        {"Standard elements"}
                        <ul>
                           { for el2.iter().map(|e| html! {
                                <TreeNode name={e.name.clone()} value={e.value.deref().clone()} parent_callback={self.link.callback(|val| Msg::Selection(val))}></TreeNode>
                            })}
                        </ul>
                    </li>
                </ul>
            },
            _ => html! {},
        }
    }
}
