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
pub(crate) mod uintarray_bindgen;
pub(crate) mod url_bindgen;
pub(crate) mod web_expect;
pub(crate) mod style;

use crate::component_model::Model;

#[derive(Clone, Debug)]
pub struct EditableValue {
    pub value: Value,
    pub callback: Callback<Value>,
}

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    App::<Model>::new().mount_to_body();
}

//TODO fix saving
// context menu
// Fix selection
// Searching
