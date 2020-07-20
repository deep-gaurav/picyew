use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MessageEvent;
use web_sys::WebSocket;
use yew::agent::Agent;
use yew::agent::Context;
use yew::agent::HandlerId;
use yew::prelude::*;
use yew::worker::AgentLink;

use serde::{Deserialize, Serialize};
use web_sys::*;

use crate::lobby::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferData {
    pub command: String,
    pub id: Option<String>,
    pub data: Option<String>,
}

pub enum AgentInput {
    SocketInput(SocketInputs),
    LobbyInput(LobbyInputs),
}

#[derive(Clone, Debug)]
pub enum AgentOutput {
    SocketOutput(SocketOutputs),
    LobbyOutput(LobbyOutputs),
}

pub struct SocketAgent {
    link: AgentLink<Self>,
    subscribers: Vec<HandlerId>,
    socket: Option<WebSocket>,
    updatecallback: Callback<(WebSocket, String)>,
    lobby: Option<Lobby>,
    name: Option<String>,
}

pub enum SocketInputs {
    Connect(String, String),
    SendData(TransferData),
}

#[derive(Clone, Debug)]
pub enum SocketOutputs {
    Connected,
    ErrorConnecting,
    Disconnected,
    SocketMessage(TransferData),
}

pub enum LobbyInputs {
    RequestLobby,
    PeerMessage(u32, TransferData),
    PeerBroadcastMessage(TransferData),
    PeerBroadcastBinaryMessage(Vec<u8>),
    SealLobby,
    ChangeTurn(u32),
    ChangeWord(String),
}

#[derive(Clone, Debug)]
pub enum LobbyOutputs {
    Connected(Lobby),
    PeerMessage(u32, TransferData, Lobby),
    PeerBinaryMessage(u32,Vec<u8>),

    LobbyRefresh(Lobby),

    RequestResult(Option<Lobby>),
    TurnChaned(u32,u32),
    WordChanged(String),
    Sealed
}

pub enum Msg {
    Connected((WebSocket, String)),
    Disconnected,
    SocketMessage(TransferData),

    PeerConnect(u32),
    PeerDataChannel(u32, RtcDataChannel),
    PeerDataChannelOpened(u32),
    PeerDataChannelMessage(u32, TransferData),
    PeerDataChannelBinaryMessage(u32,Vec<u8>),
    PeerDisconnect(u32),

    SendSocketMessage(TransferData),
    Ignore,
}

fn parse_string_todata(response: &str) -> TransferData {
    let tdata = TransferData {
        command: response[0..3].to_string(),
        id: {
            if let Some(first) = response.split("\n").next() {
                if first.len() > 3 {
                    Some(first[3..].to_string())
                } else {
                    None
                }
            } else {
                None
            }
        },
        data: response
            .split("\n")
            .nth(1)
            .and_then(|f| Some(f.to_string())),
    };
    tdata
}

impl Agent for SocketAgent {
    type Reach = Context<Self>;
    type Message = Msg;
    type Input = AgentInput;
    type Output = AgentOutput;

    fn create(link: AgentLink<Self>) -> Self {
        log::info!("Creating agent");
        SocketAgent {
            updatecallback: link.callback(|sock| Msg::Connected(sock)),
            link,
            socket: None,
            subscribers: vec![],
            lobby: None,
            name: None,
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.push(id);
    }

    fn disconnected(&mut self, _id: HandlerId) {
        if let Some(idx) = self.subscribers.iter().position(|id| id == &_id) {
            self.subscribers.remove(idx);
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::Connected(socket) => {
                let linkclone = self.link.clone();
                let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                    // handle message
                    let response = e
                        .data()
                        .as_string()
                        .expect("Can't convert received data to a string");
                    log::debug!("msg: {:#?}", response);
                    let tdata = parse_string_todata(&response);
                    linkclone.send_message(Msg::SocketMessage(tdata));
                })
                    as Box<dyn FnMut(MessageEvent)>);
                socket
                    .0
                    .set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

                onmessage_callback.forget();
                self.socket = Some(socket.0);
                for subs in self.subscribers.iter() {
                    self.link.respond(
                        subs.clone(),
                        AgentOutput::SocketOutput(SocketOutputs::Connected),
                    )
                }
            }
            Msg::SocketMessage(msg) => {
                log::debug!("socket message {:#?}", msg);
                self.handle_socket_msg(&msg);
                for subs in self.subscribers.iter() {
                    self.link.respond(
                        subs.clone(),
                        AgentOutput::SocketOutput(SocketOutputs::SocketMessage(msg.clone())),
                    );
                }
            }
            Msg::Disconnected => {
                log::warn!("Disconnected from socket");
                self.socket = None;
                for subs in self.subscribers.iter() {
                    self.link.respond(
                        subs.clone(),
                        AgentOutput::SocketOutput(SocketOutputs::Disconnected),
                    );
                }
            }

            Msg::PeerConnect(id) => {
                self.peer_connect(id);
            }
            Msg::PeerDisconnect(id) => self.peer_disconnect(id),
            Msg::SendSocketMessage(data) => {
                self.send_socket_message(data);
            }
            Msg::PeerDataChannel(id, channel) => {
                self.set_data_channel(id, channel);
            }
            Msg::PeerDataChannelMessage(id, msg) => {
                self.handle_peer_msg(id, msg);
            }
            Msg::PeerDataChannelBinaryMessage(id,msg)=>{
                self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::PeerBinaryMessage(id,msg)))
            }
            Msg::PeerDataChannelOpened(id) => {
                self.send_peer_msg(
                    &id,
                    &TransferData {
                        command: "N".to_string(),
                        id: self.name.clone(),
                        data: None,
                    },
                );
            }

            Msg::Ignore => {}
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            Self::Input::SocketInput(input) => match input {
                SocketInputs::Connect(url, name) => {
                    self.name = Some(name);
                    let subscribers = self.subscribers.clone();
                    let linkclone = self.link.clone();

                    let onerror_callback = Closure::wrap(Box::new(move |_| {
                        for subs in subscribers.clone() {
                            linkclone.clone().respond(
                                subs,
                                AgentOutput::SocketOutput(SocketOutputs::ErrorConnecting),
                            )
                        }
                    })
                        as Box<dyn FnMut(JsValue)>);

                    let ws = WebSocket::new(&format!("{}", url));
                    match ws {
                        Ok(ws) => {
                            let wss = ws.clone();
                            let updatecallback = self.updatecallback.clone();
                            let onopen_callback = Closure::wrap(Box::new(move |_| {
                                updatecallback.emit((wss.clone(), url.clone()));
                            })
                                as Box<dyn FnMut(JsValue)>);

                            ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
                            onopen_callback.forget();

                            ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
                            onerror_callback.forget();
                            let linkclone = self.link.clone();
                            let onclose_callback = Closure::wrap(Box::new(move |_| {
                                linkclone.send_message(Msg::Disconnected);
                            })
                                as Box<dyn FnMut(JsValue)>);

                            ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
                            onclose_callback.forget();
                        }
                        Err(e) => {
                            log::debug!("Cannot connect {:#?}", e);
                            for subs in &self.subscribers {
                                self.link.respond(
                                    subs.clone(),
                                    AgentOutput::SocketOutput(SocketOutputs::ErrorConnecting),
                                )
                            }
                        }
                    }
                }
                SocketInputs::SendData(data) => {
                    self.send_socket_message(data);
                }
            },
            Self::Input::LobbyInput(input) => match input {
                LobbyInputs::RequestLobby => self.link.respond(
                    _id,
                    AgentOutput::LobbyOutput(LobbyOutputs::RequestResult(self.lobby.clone())),
                ),
                LobbyInputs::PeerMessage(id, msg) => self.send_peer_msg(&id, &msg),
                LobbyInputs::PeerBroadcastMessage(msg) => self.peer_broadcast(msg),
                LobbyInputs::PeerBroadcastBinaryMessage(msg)=> self.peer_broadcast_binary(msg),
                LobbyInputs::SealLobby => self.seal_lobby(),
                LobbyInputs::ChangeTurn(turn) => self.change_turn(&turn, true),
                LobbyInputs::ChangeWord(word) => self.change_word(word,true)
            },
        }
    }
}

impl SocketAgent {
    fn broadcast(&self, output: AgentOutput) {
        // log::debug!("broadcast {:#?}", output);
        for subs in self.subscribers.iter() {
            self.link.respond(subs.clone(), output.clone());
        }
    }

    fn handle_socket_msg(&mut self, msg: &TransferData) {
        if msg.command.starts_with("I: ") {
            if let Some(lobby) = &mut self.lobby {
                if let Ok(selfid) = msg.id.clone().unwrap_or_default().trim().parse() {
                    lobby.selfid = selfid;
                    lobby.peers.insert(
                        lobby.selfid,
                        Peer {
                            id: selfid,
                            is_self: true,
                            name: self.name.clone().unwrap_or_default(),
                            is_ready: true,
                            connection: None,
                            ..Peer::default()
                        },
                    );
                }
                let lob = lobby.clone();
                self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::Connected(lob)));
            } else {
                if let Ok(selfid) = msg.id.clone().unwrap_or_default().trim().parse() {
                    let mut peers = std::collections::HashMap::new();
                    peers.insert(
                        selfid,
                        Peer {
                            id: selfid,
                            is_self: true,
                            name: self.name.clone().unwrap_or_default(),
                            is_ready: true,
                            connection: None,
                            ..Peer::default()
                        },
                    );
                    self.lobby = Some(Lobby {
                        selfid,
                        roomid: "".to_string(),
                        word:"".to_string(),
                        peers,
                        turn:0,
                        sealed:false
                    });
                }
            }
        } else if msg.command.starts_with("J: ") {
            if let Some(lobby) = &mut self.lobby {
                if let Some(roomid) = &msg.id {
                    lobby.roomid = roomid.clone();
                }
                let lob = lobby.clone();
                self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::Connected(lob)));
            } else {
                if let Some(roomid) = &msg.id {
                    self.lobby = Some(Lobby {
                        turn: 0,
                        selfid: 0,
                        word: "".to_string(),
                        roomid: roomid.clone(),
                        peers: std::collections::HashMap::new(),
                        sealed: false
                    });
                }
            }
        } else if msg.command.starts_with("N: ") {
            match &mut self.lobby {
                Some(lobby) => match msg.id.clone().unwrap_or_default().parse::<u32>() {
                    Ok(id) => {
                        lobby.peers.insert(
                            id,
                            Peer {
                                id,
                                is_self: false,
                                is_ready: false,
                                name: "".to_string(),
                                connection: None,
                                ..Peer::default()
                            },
                        );
                        self.link.send_message(Msg::PeerConnect(id));
                    }
                    Err(_err) => {
                        log::error!("Peer id not int {:#?}", _err);
                    }
                },
                None => {
                    log::error!("Peer connected but No Lobby");
                }
            }
        } else if msg.command.starts_with("O: ") {
            if let Some(lobby) = &mut self.lobby {
                if let Some(offer) = &msg.data {
                    if let Ok(id) = msg.id.clone().unwrap_or_default().parse::<u32>() {
                        if let Some(peer) = lobby.peers.get_mut(&id) {
                            if let Some(conn) = &mut peer.connection {
                                if let Ok(offer) = yew::utils::window().atob(offer) {
                                    let offer = js_sys::JSON::parse(&offer);
                                    if let Ok(offer) = offer {
                                        let connectionclone = conn.clone();
                                        let peerid = peer.id.clone();
                                        let offerclone = offer.clone();
                                        let future = async move {
                                            use wasm_bindgen_futures::*;
                                            let fut = JsFuture::from(
                                                connectionclone.set_remote_description(
                                                    offerclone.unchecked_ref(),
                                                ),
                                            );
                                            match fut.await {
                                                Ok(_) => {
                                                    let answer = connectionclone.create_answer();

                                                    let answerpromise =
                                                        JsFuture::from(answer).await;
                                                    match answerpromise {
                                                        Ok(answer) => {
                                                            let fut = JsFuture::from(
                                                                connectionclone
                                                                    .set_local_description(
                                                                        answer.unchecked_ref(),
                                                                    ),
                                                            )
                                                            .await;
                                                            match fut{
                                                                Ok(_)=>{

                                                                    Msg::SendSocketMessage(TransferData {
                                                                        command: "A".to_string(),
                                                                        id: Some(peerid.to_string()),
                                                                        data: Some(
                                                                            yew::utils::window()
                                                                                .btoa(&String::from(
                                                                                    js_sys::JSON::stringify(&answer).unwrap(),
                                                                                ))
                                                                                .unwrap(),
                                                                        ),
                                                                    })
                                                                }
                                                                Err(err)=>{
                                                                    log::error!("Cannot set answer to local {:#?}",err);
                                                                    Msg::Ignore
                                                                }
                                                            }
                                                        }
                                                        Err(err) => {
                                                            log::error!(
                                                                "Cant create answer {:#?}",
                                                                err
                                                            );
                                                            Msg::Ignore
                                                        }
                                                    }
                                                }
                                                Err(err) => {
                                                    log::error!("Cant set remote desc {:#?}", err);
                                                    Msg::Ignore
                                                }
                                            }
                                        };
                                        crate::peer_handler::send_future(self.link.clone(), future);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else if msg.command.starts_with("A: ") {
            if let Some(lobby) = &mut self.lobby {
                if let Some(answer) = &msg.data {
                    if let Ok(id) = msg.id.clone().unwrap_or_default().parse::<u32>() {
                        if let Some(peer) = lobby.peers.get_mut(&id) {
                            if let Some(conn) = &mut peer.connection {
                                if let Ok(answer) = yew::utils::window().atob(answer) {
                                    let answer = js_sys::JSON::parse(&answer);
                                    if let Ok(answer) = answer {
                                        let connectionclone = conn.clone();
                                        let answerclone = answer.clone();
                                        let future = async move {
                                            use wasm_bindgen_futures::*;
                                            let fut = JsFuture::from(
                                                connectionclone.set_remote_description(
                                                    answerclone.unchecked_ref(),
                                                ),
                                            );
                                            match fut.await {
                                                Ok(_) => Msg::Ignore,
                                                Err(err) => {
                                                    log::error!("Cant set remote desc {:#?}", err);
                                                    Msg::Ignore
                                                }
                                            }
                                        };
                                        crate::peer_handler::send_future(self.link.clone(), future);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else if msg.command.starts_with("C: ") {
            if let Some(lobby) = &mut self.lobby {
                if let Some(candidate) = &msg.data {
                    if let Ok(id) = msg.id.clone().unwrap_or_default().parse::<u32>() {
                        if let Some(peer) = lobby.peers.get_mut(&id) {
                            if let Some(conn) = &mut peer.connection {
                                if let Ok(candidate) = yew::utils::window().atob(candidate) {
                                    let candidate = js_sys::JSON::parse(&candidate);
                                    if let Ok(candidate) = candidate {
                                        let connectionclone = conn.clone();
                                        let candidate = candidate.clone();
                                        let future = async move {
                                            use wasm_bindgen_futures::*;
                                            let fut = JsFuture::from(
                                                connectionclone
                                                    .add_ice_candidate_with_opt_rtc_ice_candidate(
                                                        Some(candidate.unchecked_ref()),
                                                    ),
                                            );
                                            match fut.await {
                                                Ok(_) => Msg::Ignore,
                                                Err(err) => {
                                                    log::error!("Cant set remote desc {:#?}", err);
                                                    Msg::Ignore
                                                }
                                            }
                                        };
                                        crate::peer_handler::send_future(self.link.clone(), future);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else if msg.command.starts_with("S: ") {
            self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::Sealed));
            if let Some(lobby)=&mut self.lobby{
                lobby.sealed=true;

                let lc=lobby.clone();
                self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::LobbyRefresh(lc)));
            }
        }
    }
    fn handle_peer_msg(&mut self, id: u32, msg: TransferData) {
        if msg.command.starts_with("N") {
            if let Some(lobby) = &mut self.lobby {
                if let Some(name) = &msg.id {
                    if let Some(peer) = lobby.peers.get_mut(&id) {
                        peer.name = name.clone();
                        peer.is_ready = true;
                    }
                }
            }

            if let Some(lobby) = &self.lobby {
                self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::LobbyRefresh(
                    lobby.clone(),
                )));
            }
        }
        else if msg.command.starts_with("S") {
            self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::Sealed));
            if let Some(lobby)=&mut self.lobby{
                lobby.sealed=true;

                let lc=lobby.clone();
                self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::LobbyRefresh(lc)));
            }
        }
        else if msg.command.starts_with("T") {
            if let Ok(turn)=msg.id.clone().unwrap_or_default().parse(){
                self.change_turn(&turn,false);
            }
        }
        else if msg.command.starts_with("W") {
            if let Some(word)=&msg.data{
                self.change_word(word.clone(), false);
            }
        }
        

        //Let rest handle
        self.check_right_word(id, &msg);
        {

        
            if let Some(lobby) = &mut self.lobby {

                
                let lc=lobby.clone();
                self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::PeerMessage(
                    id,
                    msg,
                    lc,
                )));
                
            }
        }
    }

    fn check_right_word(&mut self,id:u32,msg:&TransferData)->bool{
        if let Some(lobby)=&mut self.lobby{
            if msg.command.starts_with("C") && msg.data.clone().unwrap_or_default().eq_ignore_ascii_case(&lobby.word){
                if let Some(p)=lobby.peers.get_mut(&id){
                    p.guessed=true;
    
                    log::info!("{} guessed",p.name);
                    let lc=lobby.clone();
                    self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::LobbyRefresh(lc)));
                }
                true
            }else{
                false
            }
        }else{
            false
        }
    }
    fn send_peer_msg(&self, id: &u32, data: &TransferData) {
        if let Some(lobby) = &self.lobby {
            if let Some(peer) = lobby.peers.get(&id) {
                if let Some(channel) = &peer.datachannel {
                    if let Err(err) = channel.send_with_str(&format!(
                        "{}: {}\n{}",
                        data.command,
                        data.id.clone().unwrap_or_default(),
                        data.data.clone().unwrap_or_default()
                    )) {
                        log::error!("{:?}", err);
                    }
                } else {
                    log::error!("Trying to message peer without channel")
                }
            } else {
                log::error!("Trying to message peer doesnt exist");
            }
        } else {
            log::error!("Trying to message peer, without lobby!");
        }
    }
    fn send_peer_binary(&self,id:u32,data:&[u8]){
        if let Some(lobby) = &self.lobby {
            if let Some(peer) = lobby.peers.get(&id) {
                if let Some(channel) = &peer.datachannel {
                    if let Err(err) = channel.send_with_u8_array(
                        data
                    )
                     {
                        log::error!("{:?}", err);
                    }
                } else {
                    log::error!("Trying to message peer without channel")
                }
            } else {
                log::error!("Trying to message peer doesnt exist");
            }
        } else {
            log::error!("Trying to message peer, without lobby!");
        }
    }

    fn peer_broadcast(&mut self, msg: TransferData) {
        {
            let id = self.lobby.clone().and_then(|f|Some(f.selfid)).unwrap_or_default();
             self.check_right_word(id, &msg);

                if let Some(lobby) = &self.lobby {
                    
                        for peer in lobby.peers.iter() {
                            if peer.0 == &lobby.selfid {
                                continue;
                            }
                            self.send_peer_msg(peer.0, &msg);
                        }
                    
                } else {
                    log::error!("Cant peer broadcast,No lobby!");
                }
            
        }
    }
    fn peer_broadcast_binary(&self,msg:Vec<u8>){
        if let Some(lobby) = &self.lobby {
            for peer in lobby.peers.iter() {
                if peer.0 == &lobby.selfid {
                    continue;
                }
                self.send_peer_binary(*peer.0, &msg[..]);
            }
        } else {
            log::error!("Cant peer broadcast,No lobby!");
        }
    }

    fn seal_lobby(&mut self){
        if let Some(lobby)=&mut self.lobby{

            lobby.sealed=true;
            Self::remove_non_ready(&mut lobby.peers);
            let lc = lobby.clone();
            self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::Sealed));
            self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::LobbyRefresh(lc)));
            self.peer_broadcast(TransferData{
                command:"S".to_string(),
                id:None,
                data:None
            });
            self.send_socket_message(
                TransferData{
                    command:"S".to_string(),
                    id:None,
                    data:None
                }
            );
        }
    }

    fn remove_non_ready(peers:&mut std::collections::HashMap<u32,Peer>){
        let mut torm = vec![];
        for p in peers.iter(){
            if !p.1.is_ready{
                torm.push(*p.0);
            }
        }
        for rm in torm.iter(){
            peers.remove(&rm);
        }
    }
    
    fn change_turn(&mut self,turn:&u32,send:bool){
        if let Some(lobby)=&mut self.lobby{

            lobby.turn=*turn;
            lobby.word="".to_string();
            for p in lobby.peers.iter_mut(){
                p.1.guessed=false;
            }
            let oldutrn = lobby.turn;
            let lc = lobby.clone();
            self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::TurnChaned(oldutrn,*turn)));
            self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::LobbyRefresh(lc)));
            if send{
                self.peer_broadcast(TransferData{
                    command:"T".to_string(),
                    id:Some(turn.to_string()),
                    data:None
                });
            }
        }
    }

    fn change_word(&mut self,word:String,send:bool){
        if let Some(lobby)=&mut self.lobby{

            lobby.word=word.clone();
            for p in lobby.peers.iter_mut(){
                p.1.guessed=false;
            }
            let lc = lobby.clone();
            self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::WordChanged(word.clone())));
            self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::LobbyRefresh(lc)));
            if send{
                self.peer_broadcast(TransferData{
                    command:"W".to_string(),
                    id:None,
                    data:Some(word.clone())
                });
            }
        }
    }

    fn peer_connect(&mut self, peerid: u32) {
        if let Some(lobby) = &mut self.lobby {
            let peer = lobby.peers.get_mut(&peerid).unwrap();
            crate::peer_handler::create_peer(peer, &lobby.selfid, self.link.clone());
            log::info!("Peer Connection created {:#?}", peer);
        }
    }
    fn peer_disconnect(&mut self, peerid: u32) {
        if let Some(lobby) = &mut self.lobby {
            let _peer = lobby.peers.remove_entry(&peerid);
            let lobbycl = lobby.clone();
            self.broadcast(AgentOutput::LobbyOutput(LobbyOutputs::LobbyRefresh(
                lobbycl,
            )));
        }
    }
    fn send_socket_message(&self, data: TransferData) {
        match &self.socket {
            Some(socket) => {
                log::debug!("Sending to socket {:#?}", data);
                if let Err(err) = socket.send_with_str(&format!(
                    "{}: {}\n{}",
                    data.command,
                    data.id.unwrap_or_default(),
                    data.data.unwrap_or_default()
                )) {
                    log::error!("{:?}", err);
                }
            }
            None => log::error!("Trying to send data without connection {:#?}", data),
        }
    }
    fn set_data_channel(&mut self, peerid: u32, channel: RtcDataChannel) {
        if let Some(lobby) = &mut self.lobby {
            if let Some(peer) = lobby.peers.get_mut(&peerid) {
                peer.datachannel = Some(channel.clone());
                log::debug!(
                    "Listening for data channel events id {} current state {:#?}",
                    peer.id,
                    channel.ready_state()
                );
                {
                    let peerid = peer.id.clone();

                    let linkclone = self.link.clone();
                    let onopenlistener = Closure::wrap(Box::new(move |_| {
                        linkclone.send_message(Msg::PeerDataChannelOpened(peerid));
                    })
                        as Box<dyn FnMut(JsValue)>);

                    channel.set_onopen(Some(onopenlistener.as_ref().unchecked_ref()));
                    onopenlistener.forget();
                }
                {
                    let peerid = peer.id.clone();

                    let linkclone = self.link.clone();
                    let oncloselistener = Closure::wrap(Box::new(move |_| {
                        linkclone.send_message(Msg::PeerDisconnect(peerid));
                    })
                        as Box<dyn FnMut(JsValue)>);

                    channel.set_onclose(Some(oncloselistener.as_ref().unchecked_ref()));
                    oncloselistener.forget();
                }
                {
                    if let Some(connection) = &peer.connection {
                        let peerid = peer.id.clone();

                        let linkclone = self.link.clone();
                        let connectionclone = connection.clone();
                        let onstatechange = Closure::wrap(Box::new(move |_| {
                            match connectionclone.ice_connection_state() {
                                RtcIceConnectionState::Disconnected
                                | RtcIceConnectionState::Failed
                                | RtcIceConnectionState::Closed => {
                                    linkclone.send_message(Msg::PeerDisconnect(peerid));
                                }
                                _ => {}
                            }
                        })
                            as Box<dyn FnMut(JsValue)>);

                        connection.set_oniceconnectionstatechange(Some(
                            onstatechange.as_ref().unchecked_ref(),
                        ));
                        onstatechange.forget();
                    }
                }
                {
                    let peerid = peer.id.clone();

                    let linkclone = self.link.clone();
                    let onmessage = Closure::wrap(Box::new(move |ev| {
                        let ev: MessageEvent = MessageEvent::from(ev);
                        log::debug!("Data channel Message id {} {:#?}", peerid, ev);
                        if let Some(msg) = ev.data().as_string() {
                            linkclone.send_message(Msg::PeerDataChannelMessage(
                                peerid,
                                parse_string_todata(&msg),
                            ));
                        }else{
                            let uint8array = js_sys::Uint8Array::new(&ev.data());
                            let vec =uint8array.to_vec();
                            linkclone.send_message(
                                Msg::PeerDataChannelBinaryMessage(
                                    peerid,
                                    vec
                                )
                            );
                        }
                    })
                        as Box<dyn FnMut(JsValue)>);

                    channel.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

                    onmessage.forget();
                }
            } else {
                log::error!("Data channel cant be set, peer not found");
            }
        } else {
            log::error!("Data channel cant be set, no lobby!");
        }
    }
}
