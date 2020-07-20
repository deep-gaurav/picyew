use yew::prelude::*;

use crate::avatar::getavatarcolor;
use crate::lobby::*;
use crate::socket_agent::*;

pub struct ChatHistory {
    _socket_agent: Box<dyn yew::Bridge<SocketAgent>>,
    history: Html,
    lobby: Lobby,
    link: ComponentLink<Self>,
    inputref: NodeRef,
}

pub enum Msg {
    Ignore,
    AddToHistory(Html),
    SetLobby(Lobby),
    SendChat,
}

#[derive(Properties, Clone, Debug)]
pub struct Props {
    pub lobby: Lobby,
}

impl Component for ChatHistory {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let mut agent = SocketAgent::bridge(_link.callback(|data| match data {
            AgentOutput::LobbyOutput(data) => match data {
                LobbyOutputs::PeerMessage(id, msg, lobby) => {
                    if msg.command.starts_with("C: ") {
                        let peername = lobby
                            .peers
                            .get(&id)
                            .and_then(|f| Some(f.name.clone()))
                            .unwrap_or_default();
                        
                        let msg = msg.data.unwrap_or_default();
                        if msg.eq_ignore_ascii_case(&lobby.word){
                            Msg::AddToHistory(
                                html! {
                                    <>
                                        <span style=format!("color:{}",getavatarcolor(&peername))>
                                        {
                                            peername+" "
                                        }
                                        </span>
                                        <span>
                                            {
                                                "Guessed The word!"
                                            }
                                        </span>
                                        <br/>
                                    </>
                                }
                            )
                        }
                        else{

                            Msg::AddToHistory(
                                html! {
                                <>
                                <span style=format!("color:{}",getavatarcolor(&peername))>
                                    {
                                        peername+" "
                                    }
                                </span>
                                <span>
                                    {
                                        msg
                                    }
                                </span>
                                <br/>
    
                            </>
                            })
                        }
                    } else {
                        Msg::Ignore
                    }
                }
                LobbyOutputs::LobbyRefresh(lobby)=>Msg::SetLobby(lobby),
                _ => Msg::Ignore,
            },
            _ => Msg::Ignore,
        }));
        agent.send(AgentInput::LobbyInput(LobbyInputs::RequestLobby));
        Self {
            _socket_agent: agent,
            history: html! {},
            link: _link,
            lobby: _props.lobby,
            inputref: NodeRef::default(),
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        match _msg {
            Msg::Ignore => false,
            Msg::AddToHistory(html) => {
                self.history = html! {
                    <>
                        {html}
                        {self.history.clone()}
                    </>
                };
                true
            }
            Msg::SetLobby(lobby)=>{
                self.lobby=lobby;
                true
            }
            Msg::SendChat => {
                use web_sys::HtmlInputElement;
                let inputel: HtmlInputElement = self.inputref.cast().expect("Not htmlinputelement");
                let message = inputel.value();
                self._socket_agent
                    .send(AgentInput::LobbyInput(LobbyInputs::PeerBroadcastMessage(
                        TransferData {
                            command: "C".to_string(),
                            id: Some(self.lobby.selfid.to_string()),
                            data: Some(message.clone()),
                        },
                    )));
                let selfpeername = self
                    .lobby
                    .peers
                    .get(&self.lobby.selfid)
                    .and_then(|f| Some(f.name.clone()))
                    .unwrap_or_default();
                self.history = html! {
                    <>
                        <span style=format!("color:{}",getavatarcolor(&selfpeername))>
                            {
                                selfpeername+" "
                            }
                        </span>
                        <span>
                            {
                                if message.eq_ignore_ascii_case(&self.lobby.word){
                                    "Guessed the word!!"
                                }else{
                                    &message
                                }
                            }
                        </span>
                        <br/>
                        {
                            self.history.clone()
                        }
                    </>
                };
                inputel.set_value("");
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
                <div class="box" style="height:50vh;overflow:auto;">

                    <form onsubmit=self.link.callback(|f:FocusEvent|{f.prevent_default();Msg::SendChat})>
                    <div class="field has-addons">
                        <div class="control is-expanded">
                            <input onsubmit=self.link.callback(|_|Msg::SendChat) ref=self.inputref.clone() class="input" type="text" placeholder="Type to Chat"/>
                        </div>
                        <div class="control">
                            <a onclick=self.link.callback(|_|Msg::SendChat) class="button is-primary">
                            {
                                "Send"
                            }
                            </a>
                        </div>
                    </div>
                    </form>
                    <div>
                        {
                            self.history.clone()
                        }
                    </div>
                </div>
            </>
        }
    }
}
