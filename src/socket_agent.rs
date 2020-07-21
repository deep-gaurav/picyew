use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MessageEvent;
use web_sys::WebSocket;
use yew::agent::Agent;
use yew::agent::Context;
use yew::agent::HandlerId;
use yew::prelude::*;
use yew::worker::AgentLink;

use serde::{Deserialize, Serialize};
use web_sys::*;

use crate::structures::*;


pub enum AgentInput {
    Connect(String),
    Send(PlayerMessage)
}

#[derive(Clone, Debug)]
pub enum AgentOutput {
    SocketMessage(SocketMessage),
    SocketConnected,
    SocketDisconnected,
    SocketErrorConnecting
}

pub struct SocketAgent {
    link: AgentLink<Self>,
    subscribers: Vec<HandlerId>,
    socket: Option<WebSocket>,
    updatecallback: Callback<(WebSocket, String)>,
}



pub enum Msg {
    Connected((WebSocket, String)),
    Disconnected,
    ErrorConnecting,
    SocketMessage(SocketMessage),

    // PeerConnect(u32),
    // PeerDisconnect(u32),

    SendSocketMessage(PlayerMessage),
    Ignore,
}


impl Agent for SocketAgent {
    type Reach = Context<Self>;
    type Message = Msg;
    type Input = AgentInput;
    type Output = AgentOutput;

    fn create(link: AgentLink<Self>) -> Self {
        log::info!("Creating agent");
        SocketAgent {
            updatecallback: link.callback(|sock| Msg::Connected(sock)),
            link,
            socket: None,
            subscribers: vec![],
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.push(id);
    }

    fn disconnected(&mut self, _id: HandlerId) {
        if let Some(idx) = self.subscribers.iter().position(|id| id == &_id) {
            self.subscribers.remove(idx);
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::Connected(socket) => {
                let linkclone = self.link.clone();
                let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                    // handle message
                    
                    let uint8array = js_sys::Uint8Array::new(&e.data());
                    let vec =uint8array.to_vec();
                    match bincode::deserialize(&vec[..]){
                        Ok(msg)=>{
                            linkclone.send_message(Msg::SocketMessage(msg))
                        }
                        Err(er)=>{
                            log::error!("Message received not Socket Message {:#?} {:#?}",er,vec);
                            log::error!("Is text {}",e.data().is_string());
                            log::error!("Data {:#?}",js_sys::Uint8Array::new(&e.data()).to_vec());
                            log::error!("Data {:#?}",js_sys::Uint16Array::new(&e.data()).to_vec());
                        }
                    }
                })
                    as Box<dyn FnMut(MessageEvent)>);
                socket
                    .0
                    .set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

                onmessage_callback.forget();
                self.socket = Some(socket.0);
                for subs in self.subscribers.iter() {
                    self.link.respond(
                        subs.clone(),
                        AgentOutput::SocketConnected,
                    )
                }
            }
            Msg::SocketMessage(msg) => {
                // log::debug!("socket message {:#?}", msg);
                // self.handle_socket_msg(&msg);
                self.broadcast(AgentOutput::SocketMessage(msg));
            }
            Msg::Disconnected => {
                log::warn!("Disconnected from socket");
                self.socket = None;
                self.broadcast(AgentOutput::SocketDisconnected);
            }

            Msg::SendSocketMessage(data) => {
                self.send_socket_message(&data);
            }
            Msg::ErrorConnecting=>{
                self.broadcast(AgentOutput::SocketErrorConnecting);
            }

            Msg::Ignore => {}
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            AgentInput::Connect(url)=>{
                self.connect_to_socket(url);
            }
            AgentInput::Send(msg)=>{
                self.send_socket_message(&msg);
            }
        }
    }
}

impl SocketAgent {
    fn broadcast(&self, output: AgentOutput) {
        // log::debug!("broadcast {:#?}", output);
        for subs in self.subscribers.iter() {
            self.link.respond(subs.clone(), output.clone());
        }
    }
    fn connect_to_socket(&mut self,url:String){
        let subscribers = self.subscribers.clone();
        let linkclone = self.link.clone();

        let onerror_callback = Closure::wrap(Box::new(move |_| {
            for subs in subscribers.clone() {
                linkclone.clone().send_message(
                    Msg::ErrorConnecting
                );
            }
        })
            as Box<dyn FnMut(JsValue)>);

        let ws = WebSocket::new(&format!("{}", url));
        match ws {
            Ok(ws) => {
                let wss = ws.clone();
                let updatecallback = self.updatecallback.clone();
                let onopen_callback = Closure::wrap(Box::new(move |_| {
                    updatecallback.emit((wss.clone(), url.clone()));
                })
                    as Box<dyn FnMut(JsValue)>);
                ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

                ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
                onopen_callback.forget();

                ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
                onerror_callback.forget();
                let linkclone = self.link.clone();
                let onclose_callback = Closure::wrap(Box::new(move |_| {
                    linkclone.send_message(Msg::Disconnected);
                })
                    as Box<dyn FnMut(JsValue)>);

                ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
                onclose_callback.forget();
            }
            Err(e) => {
                log::debug!("Cannot connect {:#?}", e);
                self.broadcast(AgentOutput::SocketErrorConnecting);
            }
        }
    }

    fn send_socket_message(&self, data: &PlayerMessage) {
        match &self.socket {
            Some(socket) => {
                match bincode::serialize(&data){
                    Ok(bytes)=>{
                        if let Err(er)=socket.send_with_u8_array(
                            &bytes[..]
                        ){
                            log::warn!("Cant send message {:#?}",er);
                        }
                    }
                    Err(er)=>{
                        log::error!("Cant serialize to bincode data {:#?}",data);
                    }
                }
            }
            None => log::error!("Trying to send data without connection {:#?}", data),
        }
    }
}
