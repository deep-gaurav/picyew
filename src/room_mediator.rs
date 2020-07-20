use yew::prelude::*;

use crate::lobby::Lobby;
use crate::room::Room;
use crate::gameroom::Game;
use crate::socket_agent::*;

#[derive(Clone,Debug)]
pub enum Page{
    Lobby,
    Game,
}

pub struct RoomMediator {
    _socket_agent: Box<dyn yew::Bridge<SocketAgent>>,
    lobby: Option<Lobby>,
    props:Props
}

pub enum Msg {
    Ignore,
    ReceiveLobby(Option<Lobby>),
}

#[derive(Properties, Clone, Debug)]
pub struct Props {
    pub roomid: String,
    pub page:Page
}

impl Component for RoomMediator {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let mut agent = SocketAgent::bridge(_link.callback(|data| match data {
            AgentOutput::SocketOutput(_out) => Msg::Ignore,
            AgentOutput::LobbyOutput(out) => match out {
                LobbyOutputs::RequestResult(lobby) => Msg::ReceiveLobby(lobby),
                _ => Msg::Ignore,
            },
        }));
        agent.send(AgentInput::LobbyInput(LobbyInputs::RequestLobby));
        Self {
            _socket_agent: agent,
            lobby: None,
            props:_props
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        match _msg {
            Msg::Ignore => false,
            Msg::ReceiveLobby(lobby) => {
                log::debug!("Received lobby from agent {:#?}", lobby);
                match lobby {
                    Some(lobby) => {
                        self.lobby = Some(lobby);
                        true
                    }
                    None => {
                        crate::app::go_to_route(yew_router::route::Route::from(
                            crate::app::AppRoute::Home,
                        ));
                        true
                    }
                }
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        match &self.lobby {
            None => {
                html! {
                    <div class="container">
                        <div class="button is-loading"/>
                    </div>
                }
            }
            Some(lobby) => {
                match self.props.page{
                    Page::Lobby=>html!{<Room lobby=lobby/>},
                    Page::Game=>html!{<Game lobby=lobby />}
                }
            }
        }
    }
}
