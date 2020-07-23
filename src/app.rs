use yew::prelude::*;
use yew_router::prelude::*;

use crate::room::Room;
use crate::gameroom::Game;
use crate::home::Home;
use crate::notification_widget::NotificationWidget;
use crate::notification_agent::*;

use crate::socket_agent::*;
use crate::structures::*;

pub struct App {
    _agent: Box<dyn yew::Bridge<SocketAgent>>,
    notif_agent: Box<dyn yew::Bridge::<NotificationAgent>>,
    lobby:Option<Lobby>,
    selfid: String,
    link: ComponentLink<Self>,
    ping_interval: yew::services::interval::IntervalTask,
}

pub enum Msg {
    Ignore,
    Ping,
    LobbyJoined(String,Lobby),
    GameStart(Lobby),

    Disconnected,
    PlayerDisconnected(Player),
    PlayerJoined(Player),
}

#[derive(Switch, Debug, Clone)]
pub enum AppRoute {
    #[to = "/{roomid}"]
    Room(String),
    #[to = "/"]
    Home,
}
pub fn go_to_route(route: Route) {
    use yew_router::agent::RouteRequest;
    let mut dispatcher = RouteAgentDispatcher::<()>::new();
    dispatcher.send(RouteRequest::ChangeRoute(route));
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let mut notif_agent = NotificationAgent::bridge(
            _link.callback(|_|Msg::Ignore)
        );
        let agent = SocketAgent::bridge(_link.callback(|data| match data {
            AgentOutput::SocketMessage(msg)=>{
                match msg{
                    SocketMessage::PlayerJoined(p)=>{
                        Msg::PlayerJoined(p)
                    }
                    SocketMessage::PlayerDisconnected(p)=>{
                        Msg::PlayerDisconnected(p)
                    }
                    _=>Msg::Ignore
                }
            }
            AgentOutput::SocketDisconnected=>{
                Msg::Disconnected
            }
            _ => Msg::Ignore,
        }));
        let pinginterval = yew::services::IntervalService::spawn(std::time::Duration::from_secs(1),
            _link.callback(
                |_|{
                    Msg::Ping
                }
            )
        );
        App { _agent: agent ,
            notif_agent,
            lobby:None,
            link:_link,
            selfid:unsafe{crate::home::get_uid()},
            ping_interval:pinginterval
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        match _msg {
            Msg::Ping=>{
                if self.lobby.is_some(){
                    self._agent.send(
                        AgentInput::Send(
                            PlayerMessage::Ping
                        )
                    )
                }
                false
            }
            Msg::Ignore => false,
            Msg::LobbyJoined(selfid,lob)=>{
                self.selfid=selfid;
                self.lobby=Some(lob);
                true
            }
            Msg::GameStart(lob)=>{
                self.lobby=Some(lob);
                true
            }

            Msg::Disconnected=>{
                self.notif_agent.send(
                    NotificationAgentInput::Notify(
                        Notification{
                            notification_type:NotificationType::Error,
                            content:"Disconnected from server".to_string()
                        }
                    )
                );
                false
            }
            Msg::PlayerJoined(p)=>{
                self.notif_agent.send(
                    NotificationAgentInput::Notify(
                        Notification{
                            notification_type:NotificationType::Info,
                            content:format!("{} joined",p.name)
                        }
                    )
                );
                false
            }
            Msg::PlayerDisconnected(p)=>{
                self.notif_agent.send(
                    NotificationAgentInput::Notify(
                        Notification{
                            notification_type:NotificationType::Warning,
                            content:format!("{} left",p.name)
                        }
                    )
                );
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {

        let home = html!{
            <Home prefillroomid="".to_string() lobbyjoinedcb=self.link.callback(move |f:(String,Lobby)|Msg::LobbyJoined(f.0,f.1))/>
        };
        let lobby = self.lobby.clone();
        let selfid = self.selfid.clone();
        let linkclone = self.link.clone();
        html! {
            <div>
                <Router<AppRoute, ()>
                    render = Router::render(move |switch: AppRoute| {
                        let home = home.clone();
                        let lobby = lobby.clone();
                        let selfid = selfid.clone();
                        let link = linkclone.clone();
                        match switch {
                            AppRoute::Home=>home.clone(),
                            AppRoute::Room(_roomid)=>{
                                if let Some(lobby)=lobby.clone(){
                                    match &lobby.state{
                                        State::Lobby(leader)=>{
                                            html!{
                                                <Room gamestartcb=link.callback(|lob|Msg::GameStart(lob)) selfid=selfid lobby=lobby />
                                            }
                                        }
                                        State::Game(id,_,_)=>{
                                            html!{
                                                <Game selfid=selfid lobby=lobby />
                                            }
                                        }
                                    }
                                }else{
                                    html!{
                                        <Home prefillroomid=_roomid lobbyjoinedcb=linkclone.callback(move |f:(String,Lobby)|Msg::LobbyJoined(f.0,f.1))/>
                                    }
                                }
                            },
                        }
                    })
                />
                <NotificationWidget/>
            </div>
        }
    }
}
