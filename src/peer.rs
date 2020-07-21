use yew::prelude::*;

use crate::structures::*;
use crate::socket_agent::*;

pub struct PeerWidget {
    _socket_agent: Box<dyn yew::Bridge<SocketAgent>>,
    peer: Player,
}

pub enum Msg {
    Ignore,
}

#[derive(Properties, Clone, Debug)]
pub struct Props {
    pub peer: Player,
}

impl Component for PeerWidget {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let mut agent = SocketAgent::bridge(_link.callback(|data| match data {
            _ => Msg::Ignore,
        }));
        // agent.send(AgentInput::LobbyInput(LobbyInputs::RequestLobby));
        Self {
            _socket_agent: agent,
            peer: _props.peer,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        match _msg {
            Msg::Ignore => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        use crate::avatar::avatar;
        html! {
            <>
                <div class="container has-text-centered">
                    {
                        avatar(&self.peer.name)
                    }
                    {
                        &self.peer.name
                    }
                </div>
            </>
        }
    }
}
