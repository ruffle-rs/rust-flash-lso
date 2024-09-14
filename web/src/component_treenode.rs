use crate::{EditableValue, TreeNodePath};
use flash_lso::types::{Element, Value};
use std::ops::Deref;
use std::rc::Rc;
use yew::prelude::*;
use yew::{Component, Html, Properties};

#[derive(Debug)]
pub enum Msg {
    Selection(EditableValue),
    Toggle,
    Edited(Value),
    ElementChange(Element),
    CustomElementChange(Element),
    CustomElementChangeStandard(Element),
}

pub struct TreeNode {
    expanded: bool,
    value: Value,
}

#[derive(PartialEq, Properties, Clone)]
pub struct Props {
    pub parent_path: TreeNodePath,
    pub name: String,
    pub value: Value,
    pub parent_callback: Callback<EditableValue>,
    pub selection: Option<EditableValue>,
    pub filter: String,
    #[prop_or(None)]
    pub element_callback: Option<Callback<Element>>,
}

impl Component for TreeNode {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let value = ctx.props().value.clone();
        Self {
            expanded: false,
            value,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::info!("<TreeNode>@{}, MSG: {:?}", self.path(ctx).string(), msg);
        match msg {
            Msg::Selection(val) => {
                ctx.props().parent_callback.emit(val);
                true
            }
            Msg::Toggle => {
                self.expanded = !self.expanded;
                true
            }
            Msg::Edited(v) => {
                self.value = v.clone();
                if let Some(x) = &ctx.props().element_callback {
                    x.emit(Element::new(ctx.props().name.clone(), Rc::new(v)));
                }
                true
            }
            Msg::ElementChange(el) => {
                match &mut self.value {
                    Value::Object(_, old_el, _) => {
                        let index = old_el.iter().position(|e| e.name == el.name);
                        if let Some(index) = index {
                            old_el[index] = el;
                        }
                    }
                    _ => {
                        log::warn!("Unknown element change");
                    }
                }

                true
            }
            Msg::CustomElementChange(el) => {
                match &mut self.value {
                    Value::Custom(a, _b, _) => {
                        let index = a.iter().position(|e| e.name == el.name);
                        if let Some(index) = index {
                            a[index] = el;
                        }
                    }
                    _ => {
                        log::warn!("Unknown element change for custom element");
                    }
                }

                true
            }
            Msg::CustomElementChangeStandard(el) => {
                match &mut self.value {
                    Value::Custom(_a, b, _) => {
                        let index = b.iter().position(|e| e.name == el.name);
                        if let Some(index) = index {
                            b[index] = el;
                        }
                    }
                    _ => {
                        log::warn!("Unknown element change for custom element standard");
                    }
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let name = ctx.props().name.clone();
        let value = self.value.clone();

        let icon = if TreeNode::has_children(&value) {
            if self.expanded {
                "icon/folder-minus.svg"
            } else {
                "icon/folder-plus.svg"
            }
        } else {
            "icon/file-text.svg"
        };

        let classes = if self.selected(ctx) {
            "text-white bg-primary rounded-pill pl-2 pr-2 user-select-none"
        } else {
            "pl-2 pr-2 user-select-none"
        };

        let callback = ctx.link().callback(Msg::Edited);
        let v = self.value.clone();
        let path = self.path(ctx);

        if !self.is_visible(ctx) {
            return html! {};
        }

        html! {
             <div>
                <span onclick={ctx.link().callback(|_| Msg::Toggle)}>
                    <img alt={"Toggle"} src={icon} style={"width: 32; height: 32;"} class={"mr-2"}/>
                </span>
                <span
                    class={classes}
                    onclick={ctx.link().callback(move |_| Msg::Selection(EditableValue {
                        value: v.clone(),
                        callback: callback.clone(),
                        path: path.clone(),
                    }))}>{ name }</span>
                { if self.expanded {
                    self.view_sol_value(ctx, Rc::new(self.value.clone()))
                } else {
                    html!{}
                }}
             </div>
        }
    }
}

impl TreeNode {
    pub fn is_visible(&self, ctx: &Context<Self>) -> bool {
        // Visible if no filter or if we are included in filter, also we must be visible if we have visible children
        let has_visible_children = match &ctx.props().value {
            Value::Object(_, ele, _) => ele.iter().any(|e| e.name.contains(&ctx.props().filter)),
            Value::ECMAArray(e1, e2, _) => {
                e2.iter().any(|e| e.name.contains(&ctx.props().filter))
                    || e1
                        .iter()
                        .enumerate()
                        .any(|(i, _e)| format!("{}", i).contains(&ctx.props().filter))
            }
            Value::StrictArray(e1) => e1
                .iter()
                .enumerate()
                .any(|(i, _e)| format!("{}", i).contains(&ctx.props().filter)),
            Value::VectorObject(e1, _, _) => e1
                .iter()
                .enumerate()
                .any(|(i, _e)| format!("{}", i).contains(&ctx.props().filter)),
            Value::Custom(e1, e2, _) => {
                e1.iter().any(|e| e.name.contains(&ctx.props().filter))
                    || e2.iter().any(|e| e.name.contains(&ctx.props().filter))
            }
            _ => false,
        };

        ctx.props().filter.is_empty()
            || ctx.props().name.contains(&ctx.props().filter)
            || (TreeNode::has_children(&ctx.props().value) && has_visible_children)
    }

    pub fn path(&self, ctx: &Context<Self>) -> TreeNodePath {
        ctx.props().parent_path.join(ctx.props().name.clone())
    }

    pub fn selected(&self, ctx: &Context<Self>) -> bool {
        let selected_path = ctx.props().selection.clone().map(|s| s.path);
        selected_path.map_or(false, |tnp| tnp.contains(self.path(ctx)))
    }

    pub fn has_children(data: &Value) -> bool {
        matches!(
            data,
            Value::Object(_, _, _)
                | Value::StrictArray(_)
                | Value::ECMAArray(_, _, _)
                | Value::VectorObject(_, _, _)
                | Value::AMF3(_)
                | Value::Dictionary(_, _)
                | Value::Custom(_, _, _)
        )
    }

    pub fn view_array_element(&self, ctx: &Context<Self>, index: usize, data: &Rc<Value>) -> Html {
        html! {
            <div>
                <TreeNode filter={ctx.props().filter.clone()} selection={ctx.props().selection.clone()} parent_path={self.path(ctx)} name={format!("{}", index)} value={data.deref().clone()} parent_callback={ctx.link().callback(Msg::Selection)}></TreeNode>
            </div>
        }
    }

    pub fn view_sol_value(&self, ctx: &Context<Self>, data: Rc<Value>) -> Html {
        match data.deref() {
            Value::AMF3(e) => self.view_sol_value(ctx, e.clone()),
            Value::Object(_, elements, _class_def) => html! {
                <ul>
                    { for elements.iter().map(|e| html! {
                        <TreeNode element_callback={ctx.link().callback(Msg::ElementChange)} filter={ctx.props().filter.clone()} selection={ctx.props().selection.clone()} parent_path={self.path(ctx)} name={e.name.clone()} value={e.value.deref().clone()} parent_callback={ctx.link().callback(Msg::Selection)}></TreeNode>
                    })}
                </ul>
            },
            Value::StrictArray(x) => html! {
                <ul>
                    { for x.iter().enumerate().map(|(i, v)| self.view_array_element(ctx, i, v))}
                </ul>
            },
            Value::ECMAArray(dense, assoc, _size) => html! {
                    <ul>
                       { for dense.iter().enumerate().map(|(i, v)| self.view_array_element(ctx, i, v))}
                        { for assoc.iter().map(|e| html! {
                            <TreeNode filter={ctx.props().filter.clone()} selection={ctx.props().selection.clone()} parent_path={self.path(ctx)} name={e.name.clone()} value={e.value.deref().clone()} parent_callback={ctx.link().callback(Msg::Selection)}></TreeNode>
                        })}
                    </ul>
            },
            Value::VectorObject(children, _name, _fixed_len) => html! {
                <ul>
                   { for children.iter().enumerate().map(|(i, v)| self.view_array_element(ctx, i, v))}
                </ul>
            },
            Value::Dictionary(children, _) => html! {
                <ul>
                    { for children.iter().map(|(k, v)| html! {
                            <>
                            <li>
                                <TreeNode filter={ctx.props().filter.clone()} selection={ctx.props().selection.clone()} parent_path={self.path(ctx)} name="key" value={k.deref().clone()} parent_callback={ctx.link().callback(Msg::Selection)}></TreeNode>
                            </li>
                            <li>
                                <TreeNode filter={ctx.props().filter.clone()} selection={ctx.props().selection.clone()} parent_path={self.path(ctx)} name="value" value={v.deref().clone()} parent_callback={ctx.link().callback(Msg::Selection)}></TreeNode>
                            </li>
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
                                <TreeNode element_callback={ctx.link().callback(Msg::CustomElementChange)} filter={ctx.props().filter.clone()} selection={ctx.props().selection.clone()} parent_path={self.path(ctx)} name={e.name.clone()} value={e.value.deref().clone()} parent_callback={ctx.link().callback(Msg::Selection)}></TreeNode>
                            })}
                        </ul>
                    </li>
                    <li>
                        {"Standard elements"}
                        <ul>
                           { for el2.iter().map(|e| html! {
                                <TreeNode element_callback={ctx.link().callback(Msg::CustomElementChangeStandard)} filter={ctx.props().filter.clone()} selection={ctx.props().selection.clone()} parent_path={self.path(ctx)} name={e.name.clone()} value={e.value.deref().clone()} parent_callback={ctx.link().callback(Msg::Selection)}></TreeNode>
                            })}
                        </ul>
                    </li>
                </ul>
            },
            _ => html! {},
        }
    }
}
