use yew::prelude::*;

use crate::avatar::getavatarcolor;
use crate::structures::*;
use crate::socket_agent::*;

pub struct ChatHistory {
    _socket_agent: Box<dyn yew::Bridge<SocketAgent>>,
    history: Html,
    link: ComponentLink<Self>,
    inputref: NodeRef,

    chataudio: NodeRef,
    successaudio: NodeRef,
}

pub enum Msg {
    Ignore,
    SendChat,
    AddChat(String,String),

}

#[derive(Properties, Clone, Debug)]
pub struct Props {
    pub lobby: Lobby,
}

impl Component for ChatHistory {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let agent = SocketAgent::bridge(_link.callback(|data| match data {
            AgentOutput::SocketMessage(msg)=> match msg{
                SocketMessage::Chat(name,chat)=>{
                    Msg::AddChat(name,chat)
                }
                _=>Msg::Ignore
            }
            _ => Msg::Ignore,
        }));
        Self {
            _socket_agent: agent,
            history: html! {},
            link: _link,
            inputref: NodeRef::default(),

            chataudio: NodeRef::default(),
            successaudio: NodeRef::default(),
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        match _msg {
            Msg::Ignore => false,
            Msg::AddChat(name,chat)=>{
                let chatel:web_sys::HtmlAudioElement = self.chataudio.cast().expect("Not audioelement");
                chatel.play();
                self.history = html! {
                    <>
                        <span style=format!("color:{}",getavatarcolor(&name))>
                            {
                                name+" "
                            }
                        </span>
                        <span>
                            {
                                chat
                            }
                        </span>
                        <br/>
                        {
                            self.history.clone()
                        }
                    </>
                };
                true
            }
            Msg::SendChat => {
                use web_sys::HtmlInputElement;
                let inputel: HtmlInputElement = self.inputref.cast().expect("Not htmlinputelement");
                let message = inputel.value();
                self._socket_agent.send(
                    AgentInput::Send(
                        PlayerMessage::Chat(message)
                    )
                );
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
                <audio ref=self.chataudio.clone() hidden=true src="/sounds/Sharp.ogg" />
                <div class="has-text-centered">
                <div class="box" style="display:inline-block;height:50vh;overflow:auto;">

                    <form onsubmit=self.link.callback(|f:FocusEvent|{f.prevent_default();Msg::SendChat})>
                    <div class="field has-addons">
                        <div class="control">
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
                    <div class="has-text-justified">
                        {
                            self.history.clone()
                        }
                    </div>
                </div>
                </div>
            </>
        }
    }
}
