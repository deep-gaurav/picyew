use yew::prelude::*;

use crate::chat_history::ChatHistory;
use crate::lobby::*;
use crate::peer::PeerWidget;
use crate::socket_agent::*;

use crate::app::{go_to_route,AppRoute};

pub struct Room {
    _socket_agent: Box<dyn yew::Bridge<SocketAgent>>,
    link:ComponentLink<Self>,
    lobby: Lobby,
}

pub enum Msg {
    Ignore,
    Refresh(Lobby),
    StartGame,
    Sealed,
}

#[derive(Properties, Clone, Debug)]
pub struct Props {
    pub lobby: Lobby,
}

impl Component for Room {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let mut agent = SocketAgent::bridge(_link.callback(|data| match data {
            AgentOutput::LobbyOutput(data) => match data {
                LobbyOutputs::LobbyRefresh(lobby) => Msg::Refresh(lobby),
                LobbyOutputs::Sealed => {
                    Msg::Sealed
                }
                _ => Msg::Ignore,
            },
            AgentOutput::SocketOutput(_) => Msg::Ignore,
        }));
        agent.send(AgentInput::LobbyInput(LobbyInputs::RequestLobby));
        Self {
            _socket_agent: agent,
            lobby: _props.lobby,
            link:_link
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        match _msg {
            Msg::Ignore => false,
            Msg::Refresh(lobby) => {
                self.lobby = lobby;
                true
            }
            Msg::StartGame => {
                self._socket_agent.send(AgentInput::LobbyInput(LobbyInputs::SealLobby));
                false
            }
            Msg::Sealed => {
                crate::app::go_to_route(yew_router::route::Route::from(
                    crate::app::AppRoute::Game(self.lobby.roomid.clone()),
                ));
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <div class="section">
                    <div class="container">
                        <h1 class="title has-text-centered">
                            {format!("Room {}",self.lobby.roomid)}
                        </h1>
                    </div>
                <div class="my-2">
                <div class="columns  is-mobile">
                {
                    for self.lobby.peers.iter().map(|p|html!{
                        <div class="column">
                        <PeerWidget key=format!("{:#?}",p) peer=p.1.clone()/>
                        </div>
                    })
                }
                </div>
                {
                    if self.lobby.peers.iter().find(|p|p.0<&self.lobby.selfid).is_none(){
                        html!{
                            <div class="container has-text-centered">
                                <button class="button is-primary" onclick=self.link.callback(
                                    |_|Msg::StartGame
                                )>{"Start"}</button>
                            </div>
                        }
                    }else{
                        html!{

                        }
                    }
                }
                </div>
                    <ChatHistory lobby=self.lobby.clone()/>
                </div>
            </>
        }
    }
}
