use yew::prelude::*;

use crate::lobby::*;
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
    link:ComponentLink<Self>,
}

pub enum Msg {
    Ignore,
    Refresh(Lobby),
    NewTurn(u32),
    Message(u32,TransferData)
}

#[derive(Properties, Clone, Debug)]
pub struct Props {
    pub lobby: Lobby,
}

impl Component for Game {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let mut agent = SocketAgent::bridge(_link.callback(|data| match data {
            AgentOutput::LobbyOutput(data)=>{
                match data{
                    LobbyOutputs::LobbyRefresh(lobby)=>Msg::Refresh(lobby),
                    LobbyOutputs::TurnChaned(old,new)=>{
                        Msg::NewTurn(new)
                    }
                    LobbyOutputs::PeerMessage(id,msg,_lobby)=>Msg::Message(id,msg),
                    _=>Msg::Ignore
                }
            }
            _=>Msg::Ignore
        }));
        agent.send(AgentInput::LobbyInput(LobbyInputs::RequestLobby));
        Self {
            _socket_agent: agent,
            lobby:_props.lobby,
            link:_link
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        match _msg {
            Msg::Ignore => false,
            Msg::Refresh(lobby)=>{
                self.lobby=lobby;
                self.refresh();
                true
            }
            Msg::NewTurn(turn)=>{
                self.new_turn(&turn);
                true
            }
            Msg::Message(id,msg)=>{
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="section py-2">
            <div class="">
                <div class="container">
                    <h1 class="title has-text-centered">
                        {format!("Room {}",self.lobby.roomid)}
                    </h1>
                </div>
            </div>
            <div class="columns  is-mobile mt-3">
            {
                for self.lobby.peers.iter().map(|p|html!{
                    <div class="column">
                    <PeerWidget key=format!("{:#?}",p) peer=p.1.clone()/>
                    </div>
                })
            }
            </div>
            <div class="container my-2 has-text-centered" style="letter-spacing:2px;">
                {
                    if self.lobby.selfid==self.lobby.turn{
                        
                        self.lobby.word.clone()
                        
                    }else{
                        if self.lobby.peers.get(&self.lobby.selfid).and_then(|f|Some(f.guessed)).unwrap_or_default(){
                            "You Guessed it !".to_string()
                        }else{
                            WORD_HIDE_REGEX.replace_all(&self.lobby.word,"_").to_string()
                        }
                    }
                }
            </div>
            <div class="columns">
                <div class="column  is-three-quarters-widescreen">
                    <div key=self.lobby.turn style="">
                        <DrawWidget draw=self.lobby.turn==self.lobby.selfid/>
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
        if turn==&self.lobby.selfid{
            use crate::data::WORDS;
            let randomword = &WORDS[
                (js_sys::Math::random()*WORDS.len() as f64)as usize
            ];
            self._socket_agent.send(AgentInput::LobbyInput(
                LobbyInputs::ChangeWord(randomword.clone())
            ));
            // self.link.send_message(Msg::Refresh(self.lobby.clone()));
        }
    }

    fn refresh(&mut self){
        if !self.lobby.peers.iter().any(|f|f.0==&self.lobby.turn) || (self.has_everyone_guessed() && self.lobby.turn==self.lobby.selfid)
        {
            self.decide_turn();
        }
    }

    fn decide_turn(&mut self){
        
        let mut keys_iter = self.lobby.peers.keys().sorted();
        while let Some(peer)=keys_iter.next(){
            if peer == &self.lobby.turn{
                if let Some(peer)=keys_iter.next(){
                    self._socket_agent.send(
                        AgentInput::LobbyInput(LobbyInputs::ChangeTurn(*peer))
                    );
                    return;
                }
            }
        }
        let mut keys_iter = self.lobby.peers.keys().sorted();
        if let Some(peer)=keys_iter.next(){
            self._socket_agent.send(
                AgentInput::LobbyInput(LobbyInputs::ChangeTurn(*peer))
            );
        }
    }
    fn has_everyone_guessed(&self)-> bool{
        let guessed;
        if self.lobby.peers.len()<2{
            guessed=self.lobby.peers.get(&self.lobby.selfid).and_then(|f|Some(f.guessed)).unwrap_or_default()
        }else{
            guessed=true;
        }
        for p in &self.lobby.peers{
            
            if !p.1.guessed{
                if !(p.0 == &self.lobby.turn){
                    log::info!("{} has not guessed",p.1.name);
                    return false;
                }
            }
            
        }
        if guessed{
            log::info!("Everyone has guessed");
        }else{
            log::info!("Not everyone has guessed");
        }
        guessed
    }
}