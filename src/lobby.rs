use crate::peer_handler::*;
use std::collections::HashMap;
use web_sys::*;

use gloo::events::EventListener;

#[derive(Clone, Debug)]
pub struct Lobby {
    pub selfid: u32,
    pub peers: HashMap<u32, Peer>,
    pub roomid: String,
    pub sealed: bool,
    pub turn: u32,
    pub word: String
}

#[derive(Debug, Default)]
pub struct Peer {
    pub name: String,
    pub id: u32,
    pub is_self: bool,
    pub is_ready: bool,
    pub connection: Option<RtcPeerConnection>,
    pub negotiationlistener: Option<EventListener>,
    pub icecandidatelistener: Option<EventListener>,
    pub datachannellistener: Option<EventListener>,
    pub datachannelopenlistener: Option<EventListener>,
    pub datachannelmessagelistener: Option<EventListener>,
    pub datachannel: Option<RtcDataChannel>,
    pub guessed:bool
}

impl Clone for Peer {
    fn clone(&self) -> Self {
        Self {
            connection: self.connection.clone(),
            id: self.id,
            is_self: self.is_self,
            is_ready: self.is_ready,
            name: self.name.clone(),
            negotiationlistener: None,
            icecandidatelistener: None,
            datachannellistener: None,
            datachannelopenlistener: None,
            datachannelmessagelistener: None,
            datachannel: self.datachannel.clone(),
            guessed:self.guessed.clone()
        }
    }
}
