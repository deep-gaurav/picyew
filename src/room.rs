use yew::prelude::*;

use crate::chat_history::ChatHistory;
use crate::peer::PeerWidget;
use crate::socket_agent::*;
use crate::structures::*;

use crate::app::{go_to_route,AppRoute};

pub struct Room {
    _socket_agent: Box<dyn yew::Bridge<SocketAgent>>,
    link:ComponentLink<Self>,
    lobby: Lobby,
    selfid: String,
    gamestartcb: Callback<Lobby>
}

pub enum Msg {
    Ignore,
    Refresh(Lobby),
    StartGame,
    GameStarted(State),

    PlayerJoined(Player),
    PlayerDisconnected(Player),
    
    LeaderChange(State)
    // Chat(String,String)
}

#[derive(Properties, Clone, Debug)]
pub struct Props {
    pub lobby: Lobby,
    pub selfid: String,
    pub gamestartcb: Callback<Lobby>
}

impl Component for Room {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let agent = SocketAgent::bridge(_link.callback(|data| match data {
           
           AgentOutput::SocketMessage(msg)=>{
               match msg{
                   SocketMessage::PlayerJoined(pl)=>{
                       Msg::PlayerJoined(pl)
                   }
                   SocketMessage::PlayerDisconnected(player)=>{
                       Msg::PlayerDisconnected(player)
                   }
                   SocketMessage::LeaderChange(leader)=>{
                       Msg::LeaderChange(leader)
                   }
                   SocketMessage::GameStart(state)=>{
                       Msg::GameStarted(state)
                   }
                   _=>{
                    //    log::warn!("Unexpected socket message {:#?}",msg);
                       Msg::Ignore
                   }
               }
           }
            _ => Msg::Ignore
        }));
        Self {
            _socket_agent: agent,
            lobby: _props.lobby,
            link:_link,
            selfid: _props.selfid,
            gamestartcb: _props.gamestartcb
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
                self._socket_agent.send(
                    AgentInput::Send(PlayerMessage::StartGame)
                );
                false
            }
            Msg::GameStarted(state) => {
                // crate::app::go_to_route(yew_router::route::Route::from(
                //     crate::app::AppRoute::Game(self.lobby.id.clone()),
                // ));
                self.lobby.state = state;
                self.gamestartcb.emit(self.lobby.clone());
                true
            }
            Msg::PlayerJoined(player)=>{
                self.lobby.players.insert(player.id.clone(), player);
                true
            }
            Msg::PlayerDisconnected(player)=>{
                if let Some(pl)=self.lobby.players.remove(&player.id){
                    log::debug!("Player Removed {:#?}",pl);
                }
                true
            }
            Msg::LeaderChange(leader)=>{
                self.lobby.state = leader;
                true
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
                            {format!("Room {}",self.lobby.id)}
                        </h1>
                    </div>
                <div class="my-2">
                <div class="columns  is-mobile">
                {
                    for self.lobby.players.iter().map(|p|html!{
                        <div class="column">
                        <PeerWidget key=format!("{:#?}",p) peer=p.1.clone()/>
                        </div>
                    })
                }
                </div>
                {
                    if self.selfid==self.lobby.state.leader(){
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
