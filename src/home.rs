use yew::prelude::*;

use crate::avatar::avatar;
use crate::socket_agent::{
    AgentInput, AgentOutput, LobbyOutputs, SocketAgent, SocketInputs, SocketOutputs, TransferData,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref SIGNAL_URL: String = String::from("wss://signalws.herokuapp.com");
}

pub struct Home {
    name: String,
    room_id: String,
    link: ComponentLink<Self>,
    is_connecting: bool,
    socket_agent: Box<dyn yew::Bridge<SocketAgent>>,
}

pub enum Msg {
    Connected,
    Disconnected,
    ErrorConnecting,
    Connect,
    Ignore,
    LobbyJoined(crate::lobby::Lobby),
    NameChange(String),
    RoomIdChange(String),
}

impl Component for Home {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let agent = SocketAgent::bridge(_link.callback(|data| match data {
            AgentOutput::SocketOutput(out) => match out {
                SocketOutputs::Connected => Msg::Connected,
                SocketOutputs::Disconnected => Msg::Disconnected,
                SocketOutputs::ErrorConnecting => Msg::ErrorConnecting,
                SocketOutputs::SocketMessage(_) => Msg::Ignore,
            },
            AgentOutput::LobbyOutput(out) => match out {
                LobbyOutputs::Connected(lob) => Msg::LobbyJoined(lob),
                LobbyOutputs::RequestResult(_) => Msg::Ignore,
                _ => Msg::Ignore,
            },
        }));
        Home {
            name: "".to_string(),
            room_id: "".to_string(),
            link: _link,
            socket_agent: agent,
            is_connecting: false,
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
                        .send(AgentInput::SocketInput(SocketInputs::Connect(
                            SIGNAL_URL.to_string(),
                            self.name.clone(),
                        )));
                    true
                }
            }
            Msg::Connected => {
                self.socket_agent
                    .send(AgentInput::SocketInput(SocketInputs::SendData(
                        TransferData {
                            command: "J".to_string(),
                            id: Some(self.room_id.clone()),
                            data: None,
                        },
                    )));
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
                    crate::app::AppRoute::Room(lob.roomid),
                ));
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
            <section class="section">
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
                            <div class="control is-expanded">
                                <input oninput=self.link.callback(|msg:InputData|Msg::RoomIdChange(msg.value)) class="input" type="text" placeholder="Enter Room Id to join"/>
                            </div>
                            <div class="control">
                                <a onclick=self.link.callback(|_|Msg::Connect) class=format!("button is-info {}",if self.is_connecting{"is-loading"}else{""})>
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
            </section>
            </>
        }
    }
}
