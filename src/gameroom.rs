use yew::prelude::*;

use crate::chat_history::ChatHistory;
use crate::draw_widget::DrawWidget;
use crate::notification_agent::*;
use crate::peer::PeerWidget;
use crate::socket_agent::*;
use crate::structures::*;

use lazy_static::lazy_static;

use regex::Regex;
lazy_static! {
    pub static ref WORD_HIDE_REGEX: Regex = Regex::new(r"\S").unwrap();
}

pub struct Game {
    _socket_agent: Box<dyn yew::Bridge<SocketAgent>>,
    notif_agent: Box<dyn yew::Bridge<NotificationAgent>>,
    lobby: Lobby,
    selfid: String,
    link: ComponentLink<Self>,
}

pub enum Msg {
    Ignore,
    PlayerJoin(Player),
    PlayerDisconnect(Player),
    LeaderChange(State),
    ChooseWord(String),
}

#[derive(Properties, Clone, Debug)]
pub struct Props {
    pub lobby: Lobby,
    pub selfid: String,
}

impl Component for Game {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let mut agent = SocketAgent::bridge(_link.callback(|data| match data {
            AgentOutput::SocketMessage(msg) => {
                match msg {
                    SocketMessage::PlayerJoined(p) => Msg::PlayerJoin(p),
                    SocketMessage::PlayerDisconnected(p) => Msg::PlayerDisconnect(p),
                    SocketMessage::LeaderChange(leader) => Msg::LeaderChange(leader),
                    SocketMessage::TimeUpdate(state) => {
                        // log::debug!("time update {:#?}",state);
                        Msg::LeaderChange(state)
                    }
                    SocketMessage::ScoreChange(state) => Msg::LeaderChange(state),
                    _ => Msg::Ignore,
                }
            }
            _ => Msg::Ignore,
        }));
        let notif_agent = NotificationAgent::bridge(_link.callback(|_| Msg::Ignore));
        Self {
            _socket_agent: agent,
            notif_agent,
            lobby: _props.lobby,
            link: _link,
            selfid: _props.selfid,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        match _msg {
            Msg::Ignore => false,
            Msg::LeaderChange(leader) => {
                self.lobby.state = leader;
                true
            }
            Msg::PlayerJoin(p) => {
                self.lobby.players.insert(p.id.clone(), p);
                true
            }
            Msg::PlayerDisconnect(p) => {
                self.lobby.players.remove(&p.id);
                true
            }
            Msg::ChooseWord(word) => {
                self._socket_agent
                    .send(AgentInput::Send(PlayerMessage::WordChosen(word)));
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let leader = self.lobby.state.leader().to_string();

        let points = {
            match &self.lobby.state {
                State::Game(_, _, pt) => pt.drawing.clone(),
                State::Lobby(_) => vec![],
            }
        };
        let time = {
            match &self.lobby.state {
                State::Lobby(_) => 0,
                State::Game(_, _, data) => data.time,
            }
        };

        let wordc = {
            match &self.lobby.state {
                State::Game(leader, _, pt) => match &pt.word {
                    WordState::ChoseWords(words) => {
                        if &self.selfid == leader {
                            html! {
                                <div class="card">
                                    <div class="card-heading">
                                        <div class="card-header-title is-centered">
                                            {
                                                "Choose Word"
                                            }
                                        </div>
                                    </div>
                                    <div class="card-content">
                                        <div class="container has-text-centered">
                                            <div class="columns">
                                                {
                                                    for words.iter().map(|word|{
                                                        let wordclone=word.clone();
                                                        html!{
                                                            <div class="column">
                                                                <button class="button is-normal is-outlined" onclick=self.link.callback(
                                                                    move |_|Msg::ChooseWord(wordclone.clone())
                                                                )>
                                                                    {
                                                                        word.clone()
                                                                    }
                                                                </button>
                                                            </div>
                                                        }
                                                        }
                                                    )
                                                }
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }
                        } else {
                            let p = self
                                .lobby
                                .players
                                .get(leader)
                                .and_then(|p| Some(p.name.clone()));
                            html! {
                                <div class="container has-text-centered my-2" style="letter-spacing:2px;">
                                    {
                                        p.unwrap_or_default()+" is choosing the word"
                                    }
                                </div>
                            }
                        }
                    }
                    WordState::Word(word) => {
                        html! {
                            <div class="container my-2 has-text-centered" style="letter-spacing:2px;">
                                {
                                    if &self.selfid==leader{

                                        word.clone()

                                    }else{
                                        WORD_HIDE_REGEX.replace_all(&word.clone(),"_").to_string()
                                    }

                                }
                            </div>
                        }
                    }
                },
                State::Lobby(_) => html! {},
            }
        };

        let draw = {
            match &self.lobby.state {
                State::Lobby(_) => false,
                State::Game(leader, _, data) => {
                    if leader == &self.selfid {
                        match &data.word {
                            WordState::ChoseWords(_) => false,
                            WordState::Word(_) => true,
                        }
                    } else {
                        false
                    }
                }
            }
        };
        let state = self.lobby.state.clone();
        html! {
            <div class="section py-2">
            <div class="">
                <div class="container">
                    <h1 class="title has-text-centered">
                        {format!("Room {}",self.lobby.id)}
                    </h1>
                </div>
            </div>
            <div class="columns  is-mobile mt-3">
            {
                for self.lobby.players.iter().map(|p|html!{
                    <div class="column">
                    <PeerWidget key=format!("{:#?}",p) state=state.clone() peer=p.1.clone()/>
                    </div>
                })
            }
            </div>

            <div class="container has-text-centered my-2" style="letter-spacing:2px;">
                <span class="icon">
                    <svg style="width:24px;height:24px" viewBox="0 0 24 24">
                        <path fill="currentColor" d="M19.03 7.39L20.45 5.97C20 5.46 19.55 5 19.04 4.56L17.62 6C16.07 4.74 14.12 4 12 4C7.03 4 3 8.03 3 13S7.03 22 12 22C17 22 21 17.97 21 13C21 10.88 20.26 8.93 19.03 7.39M13 14H11V7H13V14M15 1H9V3H15V1Z" />
                    </svg>
                </span>
                <span>
                {
                    time.to_string()
                }
                </span>
            </div>

            {
                wordc
            }
            <div class="columns">
                <div class="column  is-three-quarters-widescreen">
                    <div key=leader.clone()+&draw.to_string() style="">
                        <DrawWidget draw=draw initialpoints=points />
                    </div>
                </div>

                <div class="column">
                    <ChatHistory lobby=self.lobby.clone()/>
                </div>
            </div>
            </div>
        }
    }
}
impl Game {
    fn new_turn(&mut self, turn: &u32) {
        // if turn==&self.lobby.selfid{
        //     use crate::data::WORDS;
        //     let randomword = &WORDS[
        //         (js_sys::Math::random()*WORDS.len() as f64)as usize
        //     ];
        //     self._socket_agent.send(AgentInput::LobbyInput(
        //         LobbyInputs::ChangeWord(randomword.clone())
        //     ));
        //     // self.link.send_message(Msg::Refresh(self.lobby.clone()));
        // }
    }

    fn refresh(&mut self) {
        // if !self.lobby.peers.iter().any(|f|f.0==&self.lobby.turn) || (self.has_everyone_guessed() && self.lobby.turn==self.lobby.selfid)
        // {
        //     self.decide_turn();
        // }
    }

    fn decide_turn(&mut self) {

        // let mut keys_iter = self.lobby.peers.keys().sorted();
        // while let Some(peer)=keys_iter.next(){
        //     if peer == &self.lobby.turn{
        //         if let Some(peer)=keys_iter.next(){
        //             self._socket_agent.send(
        //                 AgentInput::LobbyInput(LobbyInputs::ChangeTurn(*peer))
        //             );
        //             return;
        //         }
        //     }
        // }
        // let mut keys_iter = self.lobby.peers.keys().sorted();
        // if let Some(peer)=keys_iter.next(){
        //     self._socket_agent.send(
        //         AgentInput::LobbyInput(LobbyInputs::ChangeTurn(*peer))
        //     );
        // }
    }
    fn has_everyone_guessed(&self) -> bool {
        // let guessed;
        // if self.lobby.peers.len()<2{
        //     guessed=self.lobby.peers.get(&self.lobby.selfid).and_then(|f|Some(f.guessed)).unwrap_or_default()
        // }else{
        //     guessed=true;
        // }
        // for p in &self.lobby.peers{

        //     if !p.1.guessed{
        //         if !(p.0 == &self.lobby.turn){
        //             log::info!("{} has not guessed",p.1.name);
        //             return false;
        //         }
        //     }

        // }
        // if guessed{
        //     log::info!("Everyone has guessed");
        // }else{
        //     log::info!("Not everyone has guessed");
        // }
        // guessed
        false
    }
}
