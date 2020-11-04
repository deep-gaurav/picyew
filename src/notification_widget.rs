use yew::prelude::*;

use crate::notification_agent::*;
use yew::services::timeout::{TimeoutService, TimeoutTask};

pub struct NotificationWidget {
    link: ComponentLink<Self>,
    notif_agent: Box<dyn yew::Bridge<NotificationAgent>>,
    notifs: Vec<(Notification, TimeoutTask)>,
}

pub enum Msg {
    AddNotif(Notification),
    RemoveNotif(Notification),
}

#[derive(Properties, Clone)]
pub struct Props {}

impl Component for NotificationWidget {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut agent = NotificationAgent::bridge(link.callback(|data| match data {
            NotificationAgentOutput::Notify(notif) => Msg::AddNotif(notif),
        }));

        Self {
            link,
            notif_agent: agent,
            notifs: vec![],
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddNotif(notif) => {
                let notifc = notif.clone();
                let endt = TimeoutService::spawn(
                    std::time::Duration::from_secs(3),
                    self.link
                        .callback(move |_| Msg::RemoveNotif(notifc.clone())),
                );
                self.notifs.push((notif, endt));
                true
            }
            Msg::RemoveNotif(notif) => {
                if let Some(pos) = self.notifs.iter().position(|f| f.0 == notif) {
                    self.notifs.remove(pos);
                }
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let notifs = self.notifs.iter().map(
            |(notif,_)|{
                let notifc = notif.clone();
                html!{
                    <div >
                        <div class={
                            let mut class= String::from("notification ");
                            class+=match notif.notification_type{
                                NotificationType::Info=>"is-info",
                                NotificationType::Warning=>"is-warning",
                                NotificationType::Error=>"is-danger",
                                NotificationType::Success=>"is-primary",  
                            };
                            class
                        }>
                            <button class="delete" onclick=self.link.callback(move |_|Msg::RemoveNotif(notifc.clone()))/>
                            {
                                &notif.content
                            }
                        </div>
                    </div>
                }
            }
        );

        html! {
            <div style="position:fixed;bottom:10px;z-index:2000;">
            {
                for notifs
            }
            </div>
        }
    }
}
