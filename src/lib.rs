#![recursion_limit = "1024"]
mod app;
mod avatar;
mod chat_history;
mod draw_widget;
mod home;
mod peer;
mod room;
mod room_mediator;
mod socket_agent;
mod gameroom;
mod structures;
mod notification_agent;
mod notification_widget;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C"{
    pub fn atob(inp:&str)->String;
    pub fn btoa(inp:&str)->String;
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<app::App>();

    Ok(())
}
