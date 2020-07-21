use yew::prelude::*;

use crate::structures::*;
use crate::socket_agent::*;
use crate::draw_widget::DrawWidget;
use crate::chat_history::ChatHistory;
use crate::peer::PeerWidget;
use itertools::Itertools;

use lazy_static::lazy_static;

use regex::Regex;
lazy_static! {
    pub static ref WORD_HIDE_REGEX:Regex=Regex::new(r"\S").unwrap();    
}

pub struct Game {
    _socket_agent: Box<dyn yew::Bridge<SocketAgent>>,
    lobby:Lobby,
    selfid:String,
    link:ComponentLink<Self>,
}

pub enum Msg {
    Ignore,
    PlayerJoin(Player),
    PlayerDisconnect(Player),
    LeaderChange(State)
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
            AgentOutput::SocketMessage(msg)=>{
                match msg{
                    SocketMessage::PlayerJoined(p)=>{
                        Msg::PlayerJoin(p)
                    }
                    SocketMessage::PlayerDisconnected(p)=>{
                        Msg::PlayerDisconnect(p)
                    }
                    SocketMessage::LeaderChange(leader)=>{
                        Msg::LeaderChange(leader)
                    }
                    _=>Msg::Ignore
                }
            }
            _=>Msg::Ignore
        }));
        Self {
            _socket_agent: agent,
            lobby:_props.lobby,
            link:_link,
            selfid: _props.selfid
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        match _msg {
            Msg::Ignore => false,
            Msg::LeaderChange(leader)=>{
                self.lobby.state = leader;
                true
            }
            Msg::PlayerJoin(p)=>{
                self.lobby.players.insert(p.id.clone(), p);
                true
            }
            Msg::PlayerDisconnect(p)=>{
                self.lobby.players.remove(&p.id);
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
            match &self.lobby.state{
                State::Game(_,pt)=>pt.drawing.clone(),
                State::Lobby(_)=>vec![]
            }
        };
        let word = {
            match &self.lobby.state{
                State::Game(_,pt)=>pt.word.clone(),
                State::Lobby(_)=>String::default()
            }
        }; 
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
                    <PeerWidget key=format!("{:#?}",p) peer=p.1.clone()/>
                    </div>
                })
            }
            </div>
            <div class="container my-2 has-text-centered" style="letter-spacing:2px;">
                {
                    if self.selfid==leader{
                        
                        word.clone()
                        
                    }else{
                        WORD_HIDE_REGEX.replace_all(&word.clone(),"_").to_string()   
                    }
                    
                }
            </div>
            <div class="columns">
                <div class="column  is-three-quarters-widescreen">
                    <div key=leader.clone() style="">
                        <DrawWidget draw=&leader==&self.selfid initialpoints=points />
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
impl Game{

    fn new_turn(&mut self,turn:&u32){
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

    fn refresh(&mut self){
        // if !self.lobby.peers.iter().any(|f|f.0==&self.lobby.turn) || (self.has_everyone_guessed() && self.lobby.turn==self.lobby.selfid)
        // {
        //     self.decide_turn();
        // }
    }

    fn decide_turn(&mut self){
        
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
    fn has_everyone_guessed(&self)-> bool{
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