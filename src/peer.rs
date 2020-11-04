use web_sys::Blob;
use yew::prelude::*;

use crate::socket_agent::*;
use crate::structures::*;

use gloo::events::EventListener;
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
    link: ComponentLink<Self>,

    audioref: NodeRef,
    audiocache: Vec<AudioChunk>,
    audlistener: Option<EventListener>,
    peer: Player,
    tippy: Option<Tippy>,
}

pub enum Msg {
    Ignore,

    AudEnded,
    ReceivedAudio(String, AudioChunk),
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
        let agent = SocketAgent::bridge(_link.callback(|data| match data {
            AgentOutput::SocketMessage(msg) => match msg {
                SocketMessage::AudioChat(id, chnk) => Msg::ReceivedAudio(id, chnk),
                _ => Msg::Ignore,
            },
            _ => Msg::Ignore,
        }));
        // agent.send(AgentInput::LobbyInput(LobbyInputs::RequestLobby));
        Self {
            _socket_agent: agent,
            link: _link,
            tippy: None,
            peer: _props.peer,
            state: _props.state,
            audiocache: vec![],
            audlistener: None,
            audioref: NodeRef::default(),
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        match _msg {
            Msg::Ignore => false,
            Msg::AudEnded => {
                if let Some(ad) = self.audiocache.first() {
                    let ad = ad.clone();
                    self.audiocache.remove(0);
                    let ublob = ad.to_blob();
                    match ublob {
                        Ok(blob) => {
                            log::info!(
                                "Reassembled blobtype {:#?} size {:#?}",
                                blob.type_(),
                                blob.size()
                            );
                            let url = web_sys::Url::create_object_url_with_blob(&blob);
                            match url {
                                Ok(url) => {
                                    let audel: web_sys::HtmlAudioElement =
                                        self.audioref.cast().expect("Not audioelement");
                                    audel.set_src(&url);
                                    audel.play();
                                }
                                Err(err) => log::warn!("Cant create blob url {:#?}", err),
                            }
                        }
                        Err(err) => log::warn!("Cant create to blob {:#?}", err),
                    }
                }
                false
            }
            Msg::ReceivedAudio(id, chnk) => {
                self.audiocache.push(chnk);
                if self.audiocache.len() > 3 {
                    self.audiocache.remove(0);
                }

                let audel: web_sys::HtmlAudioElement =
                    self.audioref.cast().expect("Not audioelement");
                if let None = self.audlistener {
                    let link_clone = self.link.clone();
                    let listener = EventListener::new(&audel, "ended", move |ev| {
                        link_clone.send_message(Msg::AudEnded);
                    });
                    self.audlistener = Some(listener);
                    self.link.send_message(Msg::AudEnded);
                } else if audel.paused() {
                    self.link.send_message(Msg::AudEnded);
                }
                false
            }
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

                <audio id="auid" ref=self.audioref.clone() />
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
impl AudioChunk {
    fn to_u8_array(&self) -> JsValue {
        let uint = js_sys::Uint8Array::from(self.data.as_slice());
        JsValue::from(uint)
    }
    fn to_blob(&self) -> Result<Blob, JsValue> {
        let arr = JsValue::from(js_sys::Array::of1(&self.to_u8_array()));
        let mut bag = web_sys::BlobPropertyBag::new();
        bag.type_(&self.type_);
        let blob = Blob::new_with_u8_array_sequence_and_options(&arr, &bag);

        blob
    }
}
