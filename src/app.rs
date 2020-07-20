use yew::prelude::*;
use yew_router::prelude::*;

use crate::draw_widget::DrawWidget;
use crate::home::Home;
use crate::room_mediator::{RoomMediator,Page};

use crate::socket_agent::*;

pub struct App {
    _agent: Box<dyn yew::Bridge<SocketAgent>>,
}

pub enum Msg {
    Ignore,
}

#[derive(Switch, Debug, Clone)]
pub enum AppRoute {
    #[to = "/game/{roomid}"]
    Game(String),
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
        let agent = SocketAgent::bridge(_link.callback(|data| match data {
            _ => Msg::Ignore,
        }));
        App { _agent: agent }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        match _msg {
            Msg::Ignore => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <Router<AppRoute, ()>
                    render = Router::render(|switch: AppRoute| {
                        match switch {
                            AppRoute::Home=>html!{<Home/>},

                            AppRoute::Game(_roomid)=>html!{
                                <RoomMediator key=format!("game") roomid=_roomid page=Page::Game/>
                            },
                            AppRoute::Room(_roomid)=>html!{
                                <RoomMediator key=format!("room") roomid=_roomid page=Page::Lobby/>
                            },
                        }
                    })
                />
            </div>
        }
    }
}
