use yew::prelude::*;

use crate::socket_agent::*;
use crate::structures::*;

use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    pub type Tippy;

    #[wasm_bindgen(js_namespace = window)]
    pub fn tippy(selector: &str) -> Tippy;

    #[wasm_bindgen(method)]
    pub fn show(this: &Tippy);

    #[wasm_bindgen(method)]
    pub fn hide(this: &Tippy);

    #[wasm_bindgen(method)]
    pub fn setContent(this: &Tippy, content: &str);

}

pub struct PeerWidget {
    _socket_agent: Box<dyn yew::Bridge<SocketAgent>>,
    state: State,
    peer: Player,
    tippy: Option<Tippy>,
}

pub enum Msg {
    Ignore,
}

#[derive(Properties, Clone, Debug)]
pub struct Props {
    pub peer: Player,
    pub state: State,
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
            tippy: None,
            peer: _props.peer,
            state: _props.state,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        match _msg {
            Msg::Ignore => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        self.state = _props.state;
        self.peer = _props.peer;
        true
    }

    fn rendered(&mut self, _first_render: bool) {}

    fn view(&self) -> Html {
        use crate::avatar::avatar;
        let score = {
            match &self.state {
                State::Lobby(_) => html! {},
                State::Game(leader, score, data) => {
                    let score = score.scores.get(&self.peer.id).unwrap_or(&0).to_string();
                    html! {
                        score
                    }
                }
            }
        };
        let color = {
            match &self.state {
                State::Lobby(_) => "transparent",
                State::Game(leader, _, data) => {
                    if &self.peer.id == leader {
                        "blue"
                    } else if data.guessed.contains(&self.peer.id) {
                        "green"
                    } else {
                        "black"
                    }
                }
            }
        };
        html! {
            <>
                <div class="container has-text-centered">
                    <div id=&self.peer.id style=format!("display:inline-block;border-width:5px;border-style:solid;border-radius:50%;border-color:{}",color)>
                    {
                        avatar(&self.peer.name)
                    }
                    </div>
                    <div>
                    {
                        &self.peer.name
                    }
                    </div>
                    <div>
                    {
                        score
                    }
                    </div>
                </div>
            </>
        }
    }
}
