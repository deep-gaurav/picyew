use std::collections::HashMap;

use serde::{Serialize,Deserialize};


#[derive(Default,Debug)]
pub struct Lobbies {
    pub private_lobbies: HashMap<String, Lobby>,
}

#[derive(Deserialize,Debug,Clone)]
pub struct Lobby {
    pub id: String,
    pub players: HashMap<String, Player>,
    pub state:State
}


#[derive(Debug,Deserialize,Clone)]
pub enum State{
    Lobby(String),
    Game(String,GameData)
}

use std::collections::HashSet;
#[derive(Debug,Deserialize,Clone,Default)]
pub struct GameData{
    pub drawing:Vec<Point>,
    pub guessed:HashSet<String>,
    pub word:String
}


use web_sys::*;
impl Point {
    pub fn get_x(&self, canvas: &HtmlCanvasElement) -> f64 {
        let rect = canvas.get_bounding_client_rect();
        self.x * {
            if rect.width() < rect.height() {
                (rect.width() as f64) / self.width
            } else {
                (rect.height() as f64) / self.height
            }
        }
    }
    pub fn get_y(&self, canvas: &HtmlCanvasElement) -> f64 {
        let rect = canvas.get_bounding_client_rect();
        self.y * {
            if rect.width() < rect.height() {
                (rect.width() as f64) / self.width
            } else {
                (rect.height() as f64) / self.height
            }
        }
    }

    pub fn get_scale_factor(&self,canvas: &HtmlCanvasElement)->f64{
        if self.width > self.height {
            (canvas.width() as f64) / self.width
        } else {
            (canvas.height() as f64) / self.height
        }   
    }
}


#[derive(Serialize, Deserialize, Clone,Debug)]
pub struct Point {
    pub id: u32,
    pub line_width: u32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub draw: bool,
    pub color: String,
    pub eraser: bool
}

impl State {
    pub fn leader(&self)->&str{
        match &self{
            State::Lobby(id)=>id,
            State::Game(id,_)=>id
        }
    }
}


#[derive(Debug,Clone,Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub status:PlayerStatus
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum PlayerStatus{
    Initiated,
    JoinedLobby(String),
}

#[derive(Debug,Deserialize,Copy, Clone)]
pub enum CloseCodes {
    WrongInit,
    CantCreateLobby,
    CantLoinLobbyDoestExist,
    NewSessionOpened
}
impl std::fmt::Display for CloseCodes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize,Deserialize,Debug)]
pub enum PlayerMessage{
    Initialize(String,String),
    JoinLobby(String),
    CreateLobby,
    Ping,

    Chat(String),
    StartGame,

    AddPoints(Vec<Point>),

}

#[derive(Debug,Deserialize,Clone)]
pub enum SocketMessage {
    LobbyJoined(Lobby),
    PlayerJoined(Player),
    PlayerDisconnected(Player),
    Close(CloseCodes),

    Chat(String,String),
    LeaderChange(State),
    GameStart(State),

    AddPoints(Vec<Point>),

    Pong
}