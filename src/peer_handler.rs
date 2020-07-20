use yew::agent::AgentLink;
use yew::prelude::*;

use std::future::Future;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;

use gloo::events::EventListener;
use web_sys::*;

use serde::{Deserialize, Serialize};

use crate::lobby::*;
use crate::socket_agent::*;

pub fn send_future<COMP: yew::agent::Agent, F>(link: AgentLink<COMP>, future: F)
where
    F: Future<Output = COMP::Message> + 'static,
{
    spawn_local(async move {
        link.send_message(future.await);
    });
}

#[derive(Serialize, Deserialize)]
struct IceServers {
    urls: Vec<String>,
}

pub fn create_peer(peer: &mut Peer, selfid: &u32, link: AgentLink<SocketAgent>) {
    let mut config = RtcConfiguration::new();
    config.ice_servers(
        &JsValue::from_serde(
            &(vec![IceServers {
                urls: vec!["stun:stun.l.google.com:19302".to_string(),

"stun:stun1.l.google.com:19302".to_string(),
"stun:stun2.l.google.com:19302".to_string(),
"stun:stun3.l.google.com:19302".to_string(),
"stun:stun4.l.google.com:19302".to_string(),

],
            }]),
        )
        .unwrap(),
    );
    let connection = RtcPeerConnection::new_with_configuration(&config);
    match connection {
        Ok(connection) => {
            peer.connection = Some(connection.clone());
            if peer.id > *selfid {
                let connectionclone = connection.clone();
                let linkclone = link.clone();
                let peerid = peer.id.clone();
                let negotiationlistender =
                    EventListener::new(&connection, "negotiationneeded", move |_event| {
                        log::debug!("Renegotiation needed ");
                        let connectionclone = connectionclone.clone();
                        let linkclone = linkclone.clone();
                        let future = async move {
                            let offer = connectionclone.create_offer();
                            let offerpromise = JsFuture::from(offer).await;
                            match offerpromise {
                                Ok(offer) => {
                                    use web_sys::RtcSessionDescriptionInit;
                                    let offer = RtcSessionDescriptionInit::from(offer);
                                    if let Err(err) = JsFuture::from(
                                        connectionclone.set_local_description(&offer),
                                    )
                                    .await
                                    {
                                        log::error!("Cannot ser local description {:#?}", err);
                                        Msg::Ignore
                                    } else {
                                        Msg::SendSocketMessage(TransferData {
                                            command: "O".to_string(),
                                            id: Some(peerid.to_string()),
                                            data: Some(
                                                yew::utils::window()
                                                    .btoa(&String::from(
                                                        js_sys::JSON::stringify(&offer).unwrap(),
                                                    ))
                                                    .unwrap(),
                                            ),
                                        })
                                    }
                                }
                                Err(err) => {
                                    log::error!("Cant create offer {:#?}", err);
                                    Msg::Ignore
                                }
                            }
                        };
                        send_future(linkclone, future);
                        // connection.set_local_description();
                    });
                peer.negotiationlistener = Some(negotiationlistender);
            }

            {
                //ICE Candidate
                let linkclone = link.clone();
                let peerid = peer.id.clone();
                let icecandidatelistener =
                    EventListener::new(&connection, "icecandidate", move |event| {
                        let event = event.clone().unchecked_into::<RtcPeerConnectionIceEvent>();
                        log::debug!("Ice Candidate {:#?}", event);
                        let linkclone = linkclone.clone();
                        if let Some(candidate) = event.candidate() {
                            linkclone.send_message(Msg::SendSocketMessage(TransferData {
                                command: "C".to_string(),
                                id: Some(peerid.to_string()),
                                data: Some(
                                    yew::utils::window()
                                        .btoa(&String::from(
                                            js_sys::JSON::stringify(&candidate).unwrap(),
                                        ))
                                        .unwrap(),
                                ),
                            }));
                        }
                        // connection.set_local_description();
                    });

                peer.icecandidatelistener = Some(icecandidatelistener);
            }

            {
                //Data Channel
                let linkclone = link.clone();
                let peerid = peer.id.clone();
                let datachannellistener =
                    EventListener::new(&connection, "datachannel", move |event| {
                        let event = event.clone().unchecked_into::<RtcDataChannelEvent>();
                        log::debug!("Data Channel event {:#?}", event);
                        let linkclone = linkclone.clone();
                        {
                            let channel = event.channel();
                            linkclone.send_message(Msg::PeerDataChannel(peerid, channel));
                        }
                        // connection.set_local_description();
                    });

                peer.datachannellistener = Some(datachannellistener);
            }

            {
                if *selfid < peer.id {
                    let data_channel = connection.create_data_channel("data");
                    link.send_message(Msg::PeerDataChannel(peer.id, data_channel));
                }
            }
        }
        Err(err) => {
            log::error!("Cant create rtc {:#?}", err);
        }
    }
}
