use yew::prelude::*;

use crate::avatar::getavatarcolor;
use crate::socket_agent::*;
use crate::structures::*;

use gloo::events::EventListener;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{
    Blob, BlobEvent, MediaDevices, MediaRecorder, MediaStream, MediaStreamConstraints, Navigator,
};
use yew::services::interval::{IntervalService, IntervalTask};
use yewtil::future::LinkFuture;

pub struct ChatHistory {
    _socket_agent: Box<dyn yew::Bridge<SocketAgent>>,
    history: Html,
    link: ComponentLink<Self>,
    inputref: NodeRef,
    audioref: NodeRef,
    audiocache: Vec<AudioChunk>,
    audlistener: Option<EventListener>,
    recorder: Option<(MediaRecorder, EventListener, IntervalTask)>,
    chataudio: NodeRef,
    successaudio: NodeRef,
}

pub enum Msg {
    Ignore,
    InputStreamCreated(MediaStream),
    SendChat,
    AddChat(String, String),
    RecordCheck,
    AudioBlob(AudioChunk),
    AudEnded,
    ReceivedAudio(String, AudioChunk),
}

#[derive(Properties, Clone, Debug)]
pub struct Props {
    pub lobby: Lobby,
}

impl Component for ChatHistory {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let agent = SocketAgent::bridge(_link.callback(|data| match data {
            AgentOutput::SocketMessage(msg) => match msg {
                SocketMessage::Chat(name, chat) => Msg::AddChat(name, chat),
                SocketMessage::AudioChat(id, chnk) => Msg::ReceivedAudio(id, chnk),
                _ => Msg::Ignore,
            },
            _ => Msg::Ignore,
        }));
        _link.send_future(async {
            let stream = get_audio_stream().await;
            if let Ok(val) = stream {
                let mediastream: MediaStream = val.into();
                Msg::InputStreamCreated(mediastream)
            } else {
                Msg::Ignore
            }
        });
        Self {
            _socket_agent: agent,
            history: html! {},
            audiocache: vec![],
            audlistener: None,
            link: _link,
            audioref: NodeRef::default(),
            inputref: NodeRef::default(),
            recorder: None,
            chataudio: NodeRef::default(),
            successaudio: NodeRef::default(),
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        match _msg {
            Msg::Ignore => false,
            Msg::AudEnded => {
                if let Some(ad) = self.audiocache.first() {
                    let ad = ad.clone();
                    self.audiocache.remove(0);
                    let ublob = ad.to_blob();
                    match ublob {
                        Ok(blob) => {
                            log::info!(
                                "Reassembled blobtype {:#?} size {:#?}",
                                blob.type_(),
                                blob.size()
                            );
                            let url = web_sys::Url::create_object_url_with_blob(&blob);
                            match url {
                                Ok(url) => {
                                    let audel: web_sys::HtmlAudioElement =
                                        self.audioref.cast().expect("Not audioelement");
                                    audel.set_src(&url);
                                    audel.play();
                                }
                                Err(err) => log::warn!("Cant create blob url {:#?}", err),
                            }
                        }
                        Err(err) => log::warn!("Cant create to blob {:#?}", err),
                    }
                }
                false
            }
            Msg::ReceivedAudio(id, chnk) => {
                self.audiocache.push(chnk);
                if self.audiocache.len()>6{
                    self.audiocache.remove(0);
                }

                let audel: web_sys::HtmlAudioElement =
                self.audioref.cast().expect("Not audioelement");
                if let None = self.audlistener {
                    let link_clone = self.link.clone();
                    let listener = EventListener::new(&audel, "ended", move |ev| {
                        link_clone.send_message(Msg::AudEnded);
                    });
                    self.audlistener = Some(listener);
                    self.link.send_message(Msg::AudEnded);
                }else if audel.paused(){
                    self.link.send_message(Msg::AudEnded);
                }
                false
            }
            Msg::InputStreamCreated(stream) => {
                // self._socket_agent.send(AgentInput::Send(PlayerMessage::))
                log::info!("Media stream created {:#?}", stream);
                let mut options = web_sys::MediaRecorderOptions::new();
                options.mime_type("audio/webm");
                // let recorder = MediaRecorder::new_with_media_stream_and_media_recorder_options(
                //     &stream, &options,
                // );
                let recorder = MediaRecorder::new_with_media_stream(
                    &stream, 
                );
                match recorder {
                    Ok(recorder) => {
                        let recorder: MediaRecorder = recorder;
                        let interval_task = IntervalService::spawn(
                            std::time::Duration::from_millis(1000),
                            self.link.callback(|_| Msg::RecordCheck),
                        );
                        let link_clone = self.link.clone();
                        let listener =
                            EventListener::new(&recorder, "dataavailable", move |event| {
                                let ev = event.clone().dyn_into();
                                match ev {
                                    Ok(ev) => {
                                        let ev: BlobEvent = ev;
                                        let data = ev.data();
                                        if let Some(data) = data {
                                            log::info!(
                                                "blobtype {:#?} size {:#?}",
                                                data.type_(),
                                                data.size()
                                            );
                                            let ptype = data.type_();
                                            link_clone.send_future(async move {
                                                let arr = wasm_bindgen_futures::JsFuture::from(
                                                    data.array_buffer(),
                                                )
                                                .await;
                                                // let arr:js_sys::ArrayBuffer = arr.unwrap().dyn_into().unwrap();
                                                let uintt = js_sys::Uint8Array::new(&arr.unwrap());

                                                log::info!(
                                                    "arrbuf size {:#?} ",
                                                    uintt.byte_length()
                                                );
                                                Msg::AudioBlob(AudioChunk {
                                                    data: uintt.to_vec(),
                                                    type_: ptype,
                                                })
                                            })
                                        }
                                    }
                                    Err(er) => {
                                        log::warn!("Not a blob event {:#?}", er);
                                    }
                                }
                            });
                        recorder.start();
                        log::info!("Recorder created {:#?}", recorder);

                        self.recorder = Some((recorder, listener, interval_task));
                    }
                    Err(err) => log::warn!("Cant create recorder {:#?}", err),
                }
                false
            }
            Msg::RecordCheck => {
                if let Some((recorder, _, _)) = &self.recorder {
                    if let Err(err) = recorder.stop() {
                        log::warn!("Cant request data {:#?}", err);
                    }
                }

                false
            }
            Msg::AddChat(name, chat) => {
                let chatel: web_sys::HtmlAudioElement =
                    self.chataudio.cast().expect("Not audioelement");
                chatel.play();
                self.history = html! {
                    <>
                        <span style=format!("color:{}",getavatarcolor(&name))>
                            {
                                name+" "
                            }
                        </span>
                        <span>
                            {
                                chat
                            }
                        </span>
                        <br/>
                        {
                            self.history.clone()
                        }
                    </>
                };
                true
            }
            Msg::AudioBlob(blob) => {
                if let Some((recorder, _, _)) = &self.recorder {
                    // recorder.stop();
                    recorder.start();
                }
                self._socket_agent
                    .send(AgentInput::Send(PlayerMessage::AudioChat(blob)));
                false
            }
            Msg::SendChat => {
                use web_sys::HtmlInputElement;
                let inputel: HtmlInputElement = self.inputref.cast().expect("Not htmlinputelement");
                let message = inputel.value();
                self._socket_agent
                    .send(AgentInput::Send(PlayerMessage::Chat(message)));
                inputel.set_value("");
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <audio  ref=self.chataudio.clone() hidden=true src="/sounds/Sharp.ogg" />
                <div class="has-text-centered">
                <div class="box" style="display:inline-block;height:50vh;overflow:auto;">
                    <audio id="auid" ref=self.audioref.clone() />
                    <canvas id="osc" height=100 width=300/>

                    <form onsubmit=self.link.callback(|f:FocusEvent|{f.prevent_default();Msg::SendChat})>
                    <div class="field has-addons">
                        <div class="control">
                            <input onsubmit=self.link.callback(|_|Msg::SendChat) ref=self.inputref.clone() class="input" type="text" placeholder="Type to Chat"/>
                        </div>
                        <div class="control">
                            <a onclick=self.link.callback(|_|Msg::SendChat) class="button is-primary">
                            {
                                "Send"
                            }
                            </a>
                        </div>
                    </div>
                    </form>
                    <div class="has-text-justified">
                        {
                            self.history.clone()
                        }
                    </div>
                </div>
                </div>
            </>
        }
    }
}

async fn get_audio_stream() -> Result<JsValue, JsValue> {
    let navigator: Navigator = yew::utils::window().navigator();
    let mediadevices: MediaDevices = navigator.media_devices()?;
    let mut constraints = MediaStreamConstraints::new();
    let mut trackcostraint = web_sys::MediaTrackConstraints::new();
    trackcostraint.auto_gain_control(&JsValue::from_bool(false));
    trackcostraint.noise_suppression(&JsValue::from_bool(true));
    trackcostraint.echo_cancellation(&JsValue::from_bool(true));
    constraints.audio(&trackcostraint);
    let stream = mediadevices.get_user_media_with_constraints(&constraints)?;
    let futu = wasm_bindgen_futures::JsFuture::from(stream).await?;
    Ok(futu)
}

impl AudioChunk {
    fn to_u8_array(&self) -> JsValue {
        let uint = js_sys::Uint8Array::from(self.data.as_slice());
        JsValue::from(uint)
    }
    fn to_blob(&self) -> Result<Blob, JsValue> {
        let arr = JsValue::from(js_sys::Array::of1(&self.to_u8_array()));
        let mut bag = web_sys::BlobPropertyBag::new();
        bag.type_(&self.type_);
        let blob = Blob::new_with_u8_array_sequence_and_options(&arr, &bag);

        blob
    }
}
