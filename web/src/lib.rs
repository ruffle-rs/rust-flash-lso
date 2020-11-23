#![recursion_limit = "1024"]

use wasm_bindgen::prelude::*;
use yew::prelude::*;

use flash_lso::types::Value;

pub(crate) mod blob_bindgen;
pub(crate) mod component_hexview;
pub(crate) mod component_modal;
pub(crate) mod component_model;
pub(crate) mod component_number_input;
pub(crate) mod component_string_input;
pub(crate) mod component_tab;
pub(crate) mod component_tabs;
pub(crate) mod component_treenode;
pub(crate) mod jquery_bindgen;
pub(crate) mod style;
pub(crate) mod uintarray_bindgen;
pub(crate) mod url_bindgen;
pub(crate) mod web_expect;

use crate::component_model::Model;

#[derive(Clone, Debug, PartialEq)]
pub struct EditableValue {
    pub value: Value,
    pub callback: Callback<Value>,
    pub path: TreeNodePath,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeNodePath(Vec<String>);
impl TreeNodePath {
    pub fn root() -> Self {
        Self {
            0: vec!["/".to_string()],
        }
    }

    pub fn join(&self, child: String) -> Self {
        Self {
            0: self.0.iter().chain(&[child]).cloned().collect(),
        }
    }

    pub fn contains(&self, other: Self) -> bool {
        log::warn!("Does {:?} contain {:?}", self, other);

        if other.0.len() > self.0.len() {
            return false;
        }

        self.0[..other.0.len()] == other.0
    }

    pub fn string(&self) -> String {
        self.0.join("::")
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    App::<Model>::new().mount_to_body();
}

//TODO fix saving
// context menu
