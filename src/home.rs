use yew::prelude::*;

use crate::avatar::avatar;
use crate::socket_agent::{AgentInput, AgentOutput, SocketAgent};
use crate::structures::*;
use lazy_static::lazy_static;

use wasm_bindgen::*;

lazy_static! {
    static ref SIGNAL_URL: String = String::from("wss://signalws.herokuapp.com");
}

pub struct Home {
    name: String,
    room_id: String,
    link: ComponentLink<Self>,
    is_connecting: bool,
    socket_agent: Box<dyn yew::Bridge<SocketAgent>>,
    props: Props,
}

#[derive(Debug, Properties, Clone)]
pub struct Props {
    pub lobbyjoinedcb: Callback<(String, Lobby)>,
    pub prefillroomid: String,
}

use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    pub fn get_uid() -> String;
}

pub enum Msg {
    Connected,
    Disconnected,
    ErrorConnecting,
    Connect,
    Ignore,
    LobbyJoined(Lobby),
    NameChange(String),
    RoomIdChange(String),
}

impl Component for Home {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let agent = SocketAgent::bridge(_link.callback(|data| match data {
            AgentOutput::SocketConnected => Msg::Connected,

            AgentOutput::SocketMessage(msg) => match msg {
                SocketMessage::LobbyJoined(lobby) => Msg::LobbyJoined(lobby),
                SocketMessage::Close(_) => Msg::Disconnected,
                _ => Msg::Ignore,
            },
            AgentOutput::SocketDisconnected => Msg::Disconnected,
            AgentOutput::SocketErrorConnecting => Msg::ErrorConnecting,
        }));
        Home {
            name: "".to_string(),
            room_id: _props.prefillroomid.clone(),
            link: _link,
            socket_agent: agent,
            is_connecting: false,
            props: _props,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        match _msg {
            Msg::NameChange(name) => {
                self.name = name;
                true
            }
            Msg::RoomIdChange(id) => {
                self.room_id = id;
                true
            }
            Msg::Connect => {
                if self.name.is_empty() {
                    false
                } else {
                    self.is_connecting = true;
                    self.socket_agent
                        .send(AgentInput::Connect(SIGNAL_URL.to_string()));
                    true
                }
            }
            Msg::Connected => {
                let uid = unsafe { get_uid() };
                log::info!("uid is {:#?}", uid);
                self.socket_agent
                    .send(AgentInput::Send(PlayerMessage::Initialize(
                        uid,
                        self.name.to_string(),
                    )));
                if self.room_id.is_empty() {
                    self.socket_agent
                        .send(AgentInput::Send(PlayerMessage::CreateLobby));
                } else {
                    self.socket_agent
                        .send(AgentInput::Send(PlayerMessage::JoinLobby(
                            self.room_id.clone(),
                        )));
                }
                false
            }
            Msg::Disconnected => {
                self.is_connecting = false;
                true
            }
            Msg::ErrorConnecting => {
                self.is_connecting = false;
                true
            }
            Msg::LobbyJoined(lob) => {
                crate::app::go_to_route(yew_router::route::Route::from(
                    crate::app::AppRoute::Room(lob.id.clone()),
                ));
                let uid = unsafe { get_uid() };
                self.props.lobbyjoinedcb.emit((uid, lob));
                true
            }
            Msg::Ignore => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
            <section class="section">
                <div class="container">
                    <h1 class="title has-text-centered">
                        {"Pictionary"}
                    </h1>
                </div>

            </section>
            <section class="section has-text-centered">
                <div class="container" style="display:inline-flex;">
                <div class="box">
                    {
                        avatar(&self.name)
                    }

                    <div class="container mt-2">
                        <fieldset disabled=self.is_connecting>
                        <div class="field">
                            <div class="control">
                                <input oninput=self.link.callback(|msg:InputData|Msg::NameChange(msg.value)) class="input" type="text" placeholder="Enter Name"/>
                            </div>
                        </div>
                        </fieldset>
                    </div>
                    <div class="container mt-2">
                        <fieldset disabled=self.name.is_empty() || self.is_connecting>
                        <div class="field has-addons">
                            <div class="control ">
                                <input value=self.room_id.clone() oninput=self.link.callback(|msg:InputData|Msg::RoomIdChange(msg.value)) class="input" type="text" placeholder="Enter Room Id to join"/>
                            </div>
                            <div class="control">
                                <a key=self.is_connecting.to_string() onclick=self.link.callback(|_|Msg::Connect) class=format!("button is-outlined is-primary {}",if self.is_connecting{"is-loading"}else{""})>
                                    {
                                        if(self.room_id.is_empty()){
                                            "Create"
                                        }else{
                                            "Join"
                                        }
                                    }
                                </a>
                            </div>
                        </div>
                        </fieldset>
                    </div>
                </div>
                </div>
            </section>
            </>
        }
    }
}
