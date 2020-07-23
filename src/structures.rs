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
    pub state:State,
    pub draw_time: u32,
}


#[derive(Debug,Deserialize,Clone)]
pub enum State{
    Lobby(String),
    Game(String,Scores, GameData),
}

use std::collections::HashSet;
#[derive(Debug,Deserialize,Clone,Default)]
pub struct GameData{
    pub drawing: Vec<Point>,
    pub guessed: HashSet<String>,
    pub time: u32,
    pub word: WordState,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Scores{
    pub scores: HashMap<String,u32>
}

#[derive(Debug, Deserialize,Clone)]
pub enum WordState{
    ChoseWords(Vec<String>),
    Word(String),
}
impl Default for WordState {
    fn default() -> Self {
        WordState::ChoseWords(vec![])
    }
}

use web_sys::*;
impl Point {
    pub fn get_x(&self, canvas: &HtmlCanvasElement) -> f64 {
        self.x * self.get_scale_factor(canvas)
    }
    pub fn get_y(&self, canvas: &HtmlCanvasElement) -> f64 {
        self.y * self.get_scale_factor(canvas)
    }

    pub fn get_scale_factor(&self,canvas: &HtmlCanvasElement)->f64{
        let wf = canvas.width() as f64/self.width;
        let hf = canvas.height() as f64/self.height;
        if wf<hf{
            wf
        } else{
            hf
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
            State::Game(id,_,_)=>id
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
pub enum PlayerMessage {
    Initialize(String, String),
    JoinLobby(String),
    WordChosen(String),
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

    Chat(String, String),
    LeaderChange(State),
    ScoreChange(State),
    TimeUpdate(State),

    GameStart(State),

    AddPoints(Vec<Point>),

    Pong,
}