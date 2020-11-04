use yew::agent::{Agent, AgentLink, Context, HandlerId};
use yew::prelude::*;

#[derive(Clone)]
pub enum NotificationAgentInput {
    Notify(Notification),
}

#[derive(Clone)]
pub enum NotificationAgentOutput {
    Notify(Notification),
}

#[derive(Clone, PartialEq)]
pub struct Notification {
    pub content: String,
    pub notification_type: NotificationType,
}

#[derive(Clone, PartialEq)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
}

pub struct NotificationAgent {
    link: AgentLink<Self>,
    subscribers: Vec<HandlerId>,
}

pub enum Msg {}

impl Agent for NotificationAgent {
    type Reach = Context<Self>;
    type Message = Msg;
    type Input = NotificationAgentInput;
    type Output = NotificationAgentOutput;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: vec![],
        }
    }

    fn connected(&mut self, _id: HandlerId) {
        self.subscribers.push(_id);
    }

    fn disconnected(&mut self, _id: HandlerId) {
        if let Some(idx) = self.subscribers.iter().position(|f| f == &_id) {
            self.subscribers.remove(idx);
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            NotificationAgentInput::Notify(notif) => {
                self.broadcast(NotificationAgentOutput::Notify(notif));
            }
        }
    }
}

impl NotificationAgent {
    fn broadcast(&self, output: NotificationAgentOutput) {
        for sub in self.subscribers.iter() {
            self.link.respond(sub.clone(), output.clone());
        }
    }
}
