use yew::prelude::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::structures::*;
use crate::socket_agent::*;

use web_sys::*;

pub struct DrawWidget {
    _socket_agent: Box<dyn yew::Bridge<SocketAgent>>,
    canvas_ref: NodeRef,
    cursor_ref: NodeRef,
    link: ComponentLink<Self>,
    context: Option<CanvasRenderingContext2d>,
    points: Vec<Point>,
    todraw: Vec<Point>,
    tosend: Vec<Point>,
    pressed: bool,
    toolboxopen: ToolBoxOpen,
    current_color: String,
    current_width: u32,
    is_eraser: bool,
    send_interval: yew::services::interval::IntervalTask,
    refresh_interval: yew::services::interval::IntervalTask,
    props:Props
}

pub enum ToolBoxOpen {
    Brush,
    Eraser,
    Color,
    None,
}


pub enum Msg {
    Ignore,
    Setup,

    CanvasResize,
    Refresh,

    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    MouseMove(MouseEvent),
    MouseExit(MouseEvent),
    MouseWheel(WheelEvent),

    TouchStart(TouchEvent),
    TouchMove(TouchEvent),
    TouchEnd(TouchEvent),
    TouchCancel(TouchEvent),

    SetToolBox(ToolBoxOpen),

    SetColor(String),
    SetSize(u32),
    ToggleEraser,

    ClearDoc,

    SendData,
    SetData(Vec<Point>)
}

#[derive(Properties, Clone, Debug)]
pub struct Props {
    pub draw:bool,
    pub initialpoints:Vec<Point>
}

impl Component for DrawWidget {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        log::info!("new Drawboard created");
        let draw = _props.draw;
        let agent = SocketAgent::bridge(_link.callback(move |data| match data {
            AgentOutput::SocketMessage(msg)=>match msg{
                SocketMessage::AddPoints(pts)=>{
                    if draw{
                       Msg::Ignore
                    }
                    else{
                        Msg::SetData(pts)
                    }
                },
                _=>Msg::Ignore
            }
            _=>Msg::Ignore
        }));
        let interval = yew::services::IntervalService::spawn(std::time::Duration::from_millis(100),
            _link.callback(
                |_|{
                    Msg::SendData
                }
            )
        );
        let refreshinterval = yew::services::IntervalService::spawn(std::time::Duration::from_secs(1),
            _link.callback(
                |_|{
                    Msg::Refresh
                }
            )
        );
        Self {
            _socket_agent: agent,
            canvas_ref: NodeRef::default(),
            cursor_ref: NodeRef::default(),
            link: _link,
            context: None,
            points: _props.initialpoints.clone(),
            todraw: vec![],
            tosend: vec![],
            pressed: false,
            toolboxopen: ToolBoxOpen::None,
            current_color: "black".to_string(),
            current_width: 2,
            is_eraser:false,
            send_interval:interval,
            props:_props,
            refresh_interval:refreshinterval
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        match _msg {
            Msg::Ignore => false,
            Msg::Refresh => true,
            Msg::CanvasResize => {
                self.resetcanvas();
                true
            }
            Msg::Setup => {
                self.initcanvas();
                false
            }
            Msg::MouseDown(ev) => {
                self.mousedown(ev);
                false
            }
            Msg::MouseMove(ev) => {
                self.mousemove(ev);
                false
            }
            Msg::MouseUp(ev) => {
                self.mouseup(ev);
                false
            }
            Msg::MouseExit(ev)=>{
                self.pressed=false;
                self.hide_cursor();
                false
            }
            Msg::MouseWheel(ev)=>{
                log::debug!("Change size with wheel size: {} {} {}",ev.delta_x(),ev.delta_y(),ev.delta_z());
                self.current_width = (self.current_width as f64 - ev.delta_y())as u32;
                let wid:HtmlElement = self.cursor_ref.cast().expect("Not htmlelement");
                let style = wid.style();
                style.set_property("display",&format!("unset")).expect("Cant set top");
                style.set_property("width",&format!("{}px",self.current_width)).expect("Cant set width");
                style.set_property("height",&format!("{}px",self.current_width)).expect("Cant set height");
                ev.prevent_default();
                false
            }
            Msg::TouchStart(ev) => {
                self.touchstart(ev);
                false
            }
            Msg::TouchMove(ev) => {
                self.touchmove(ev);
                false
            }
            Msg::TouchEnd(ev) => {
                self.touchend(ev);
                false
            }
            Msg::TouchCancel(ev) => {
                self.touchend(ev);
                false
            }
            Msg::SetToolBox(tool) => {
                self.toolboxopen = tool;
                true
            }
            Msg::SetColor(color) => {
                self.current_color = color;
                self.toolboxopen = ToolBoxOpen::None;
                true
            }
            Msg::SetSize(size) => {
                self.current_width = size;
                self.toolboxopen = ToolBoxOpen::None;
                true
            }
            Msg::ToggleEraser=>{
                self.is_eraser=!self.is_eraser;
                true
            }
            Msg::ClearDoc =>{
                self.points.clear();
                self.resetcanvas();
                false
            }
            Msg::SendData=>{
                if !self.todraw.is_empty(){
                    self.draw();
                }
                if self.tosend.is_empty(){
                   return true; 
                }
                // TODO: FIX DRAW WIDGET
                // self._socket_agent.send(
                //     AgentInput::LobbyInput(
                //         LobbyInputs::PeerBroadcastBinaryMessage(
                //             bincode::serialize(&self.tosend).unwrap()
                //         )
                //     )
                // );
                self._socket_agent.send(
                    AgentInput::Send(
                        PlayerMessage::AddPoints(self.tosend.clone())
                    )
                );
                self.tosend.clear();
                false
            }
            Msg::SetData(mut data)=>{
                self.points.append(&mut data.clone());
                self.todraw.append(&mut data);
                self.draw();
                false
            }
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        // self.initcanvas();
        if _first_render{
            self.initcanvas();
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        if self.props.draw{

            html! {
                <>
                    <div style="">
                    
                    <canvas style="border-color:black;border-style:solid;touch-action: none;width:100%;min-height:50vh;position:relative;cursor:none;" key="drawboard" onload=self.link.callback(|_|Msg::Setup)
                        onmousedown=self.link.callback(|ev|Msg::MouseDown(ev))
                        onmouseup=self.link.callback(|ev|Msg::MouseUp(ev))
                        onmousemove=self.link.callback(|ev|Msg::MouseMove(ev))
                        onmouseleave=self.link.callback(|ev|Msg::MouseExit(ev))
                        onwheel=self.link.callback(|ev|Msg::MouseWheel(ev))
    
                        ontouchstart=self.link.callback(|ev|Msg::TouchStart(ev))
                        ontouchmove=self.link.callback(|ev|Msg::TouchMove(ev))
                        ontouchend=self.link.callback(|ev|Msg::TouchEnd(ev))
                        ontouchcancel=self.link.callback(|ev|Msg::TouchCancel(ev))
    
                        onresize=self.link.callback(|_|Msg::CanvasResize)
    
                    ref=self.canvas_ref.clone()>
                    </canvas>
                    <div ref=self.cursor_ref.clone() style="display:none;width:5px;height:5px;background-color:grey;z-index:20;position:fixed;border-radius:50%;pointer-events: none;transform:translate(-50%,-50%);">
                        
                    </div>
    
                    </div>
    
    
                    <div class="box ">
                        {
                            match self.toolboxopen{
                                ToolBoxOpen::None=>html!{},
                                ToolBoxOpen::Brush=>html!{
                                    <div class="container">
                                        <div class="columns is-mobile">
                                            {
                                                for [2,5,10,15,20].iter().map(
                                                    |size|{
                                                        html!{
                                                            <div class="column"
                                                                onclick=self.link.callback(
                                                                    move |_|Msg::SetSize(size.clone())
                                                                )
                                                            >
                                                                {
                                                                    brushsize(size.clone(),&self.current_color)
                                                                }
                                                            </div>
                                                        }
                                                    }
                                                )
                                            }
                                        </div>
                                    </div>
                                },
                                ToolBoxOpen::Color=>html!{
                                    <div class="container">
                                        <div class="columns is-mobile">
                                            {
                                                for vec!["black","red","green","blue","yellow"].iter().map(
                                                    |color|{
                                                        let color = color.clone();
                                                        html!{
                                                            <div class="column"
                                                                onclick=self.link.callback(
                                                                    move |_|Msg::SetColor(format!("{}",color))
                                                                )
                                                            >
                                                                {
                                                                    colorpallet(color)
                                                                }
                                                            </div>
                                                        }
                                                    }
                                                )
                                            }
                                        </div>
                                    </div>
                                },
                                _=>html!{}
                            }
                        }
                        <div class="level is-mobile">
    
                            <div class="level-left">
                                <div class="level-item"
                                    onclick=self.link.callback(|_|Msg::SetToolBox(ToolBoxOpen::Brush))
                                >
                                    <span class="icon">
                                        {
                                            brushsize(self.current_width,&self.current_color)
                                        }
                                    </span>
                                </div>
                                <div class="level-item"
                                    onclick=self.link.callback(|_|Msg::ToggleEraser)
                                >
                                    <span class="icon">
                                    {
                                        if self.is_eraser{
                                            html!{
                                                <svg style="width:24px;height:24px" viewBox="0 0 24 24">
                                                    <path fill="currentColor" d="M15.14,3C14.63,3 14.12,3.2 13.73,3.59L2.59,14.73C1.81,15.5 1.81,16.77 2.59,17.56L5.03,20H12.69L21.41,11.27C22.2,10.5 22.2,9.23 21.41,8.44L16.56,3.59C16.17,3.2 15.65,3 15.14,3M17,18L15,20H22V18" />
                                                </svg>
                                            }
                                        }else{
                                            html!{
    
                                                <svg style="width:24px;height:24px" viewBox="0 0 24 24">
                                                    <path fill="currentColor" d="M16.24,3.56L21.19,8.5C21.97,9.29 21.97,10.55 21.19,11.34L12,20.53C10.44,22.09 7.91,22.09 6.34,20.53L2.81,17C2.03,16.21 2.03,14.95 2.81,14.16L13.41,3.56C14.2,2.78 15.46,2.78 16.24,3.56M4.22,15.58L7.76,19.11C8.54,19.9 9.8,19.9 10.59,19.11L14.12,15.58L9.17,10.63L4.22,15.58Z" />
                                                </svg>
                                            }
                                        }
                                    }
                                    </span>
                                </div>
                            </div>
    
                            <div class="level-right">
    
                                <div class="level-item"
                                    onclick=self.link.callback(|_|Msg::ClearDoc)
                                >
                                    <span class="icon">
                                        <svg style="width:24px;height:24px" viewBox="0 0 24 24">
                                            <path fill="currentColor" d="M13,9V3.5L18.5,9M6,2C4.89,2 4,2.89 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2H6Z" />
                                        </svg>
                                    </span>
                                </div>
                                <div class="level-item"
                                onclick=self.link.callback(|_|Msg::SetToolBox(ToolBoxOpen::Color))
                                >
                                    {
                                        colorpallet(&self.current_color)
                                    }
                                </div>
                            </div>
                        </div>
                    </div>
                </>
            }
        }
        else{
            html!{
                <div>
                <canvas style="border-color:black;border-style:solid;touch-action: none;width:100%;height:100%;min-height:50vh;position:relative;" key="drawboard" onload=self.link.callback(|_|Msg::Setup)

                    onresize=self.link.callback(|_|Msg::CanvasResize)

                ref=self.canvas_ref.clone()/>
                
                </div>
            }
        }
    }
}

fn colorpallet(color: &str) -> Html {
    html! {
        <span class="icon" style=format!("color:{}",color)>
            <svg style="width:24px;height:24px" viewBox="0 0 24 24">
                <path fill="currentColor" d="M17.5,12A1.5,1.5 0 0,1 16,10.5A1.5,1.5 0 0,1 17.5,9A1.5,1.5 0 0,1 19,10.5A1.5,1.5 0 0,1 17.5,12M14.5,8A1.5,1.5 0 0,1 13,6.5A1.5,1.5 0 0,1 14.5,5A1.5,1.5 0 0,1 16,6.5A1.5,1.5 0 0,1 14.5,8M9.5,8A1.5,1.5 0 0,1 8,6.5A1.5,1.5 0 0,1 9.5,5A1.5,1.5 0 0,1 11,6.5A1.5,1.5 0 0,1 9.5,8M6.5,12A1.5,1.5 0 0,1 5,10.5A1.5,1.5 0 0,1 6.5,9A1.5,1.5 0 0,1 8,10.5A1.5,1.5 0 0,1 6.5,12M12,3A9,9 0 0,0 3,12A9,9 0 0,0 12,21A1.5,1.5 0 0,0 13.5,19.5C13.5,19.11 13.35,18.76 13.11,18.5C12.88,18.23 12.73,17.88 12.73,17.5A1.5,1.5 0 0,1 14.23,16H16A5,5 0 0,0 21,11C21,6.58 16.97,3 12,3Z" />
            </svg>
        </span>
    }
}

fn brushsize(size: u32, color: &str) -> Html {
    let size = size;
    html! {
        <>
        <span class="icon">
            <svg style=format!("width:24px;height:24px") viewBox=format!("0 0 24 24")>
                <path fill="currentColor" d="M20.71,4.63L19.37,3.29C19,2.9 18.35,2.9 17.96,3.29L9,12.25L11.75,15L20.71,6.04C21.1,5.65 21.1,5 20.71,4.63M7,14A3,3 0 0,0 4,17C4,18.31 2.84,19 2,19C2.92,20.22 4.5,21 6,21A4,4 0 0,0 10,17A3,3 0 0,0 7,14Z" />
            </svg>
        </span>
        <div style=format!("width:{}px;height:{}px;background-color:{};",size,size,color)>
        </div>
        </>

    }
}

impl DrawWidget {
    fn initcanvas(&mut self) {
        log::debug!("Load context");
        let canvas: HtmlCanvasElement = self.canvas_ref.cast().expect("Not html canvas element");
        let context = canvas
            .get_context("2d")
            .expect("No context 2d")
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .expect("Not canvas context");
        self.resetcanvas();
        // canvas.set_height(canvas.width() as u32);
        self.context = Some(context);
    }

    fn set_point(&mut self,point:&Point){
     
        self.tosend.push(point.clone());
        self.todraw.push(point.clone());
        self.points.push(point.clone());  
        self.draw(); 
    }

    fn mousedown(&mut self, event: MouseEvent) {
        if let Some(context) = &self.context {
            let canvas: HtmlCanvasElement =
                self.canvas_ref.cast().expect("Not html canvas element");
            let rect = canvas.get_bounding_client_rect();
            // context.move_to(event.offset_x() as f64, event.offset_y() as f64);
            let point = Point {
                id: self.points.len() as u32,
                x: event.offset_x() as f64,
                y: event.offset_y() as f64,
                width: rect.width(),
                height: rect.height(),
                draw: false,
                color: self.current_color.clone(),
                line_width: self.current_width,
                eraser:self.is_eraser
            };

            self.pressed = true;

            self.set_point(&point);

        } else {
            log::warn!("Context not ready, not drawing");
        }
    }

    fn set_cursor_pos(&mut self,ev:&MouseEvent){
        let wid:HtmlElement = self.cursor_ref.cast().expect("Not htmlelement");
        let style = wid.style();
        style.set_property("display",&format!("unset")).expect("Cant set top");
        style.set_property("top",&format!("{}px",ev.client_y()-1)).expect("Cant set top");
        style.set_property("left",&format!("{}px",ev.client_x()-1)).expect("Cant set left");
        style.set_property("width",&format!("{}px",self.current_width)).expect("Cant set width");
        style.set_property("height",&format!("{}px",self.current_width)).expect("Cant set height");
        if self.is_eraser{
            style.set_property("background-color","white").unwrap();
            style.set_property("border-color","black").unwrap();
            style.set_property("border-width","1px").unwrap();
            style.set_property("border-style","solid").unwrap();
        }else{
            style.set_property("background-color",&format!("{}",self.current_color)).expect("Cant set height");   
            style.set_property("border-color","black").unwrap();
            style.set_property("border-width","1px").unwrap();
            style.set_property("border-style","solid").unwrap();
        }
    }
    fn hide_cursor(&mut self){
        let wid:HtmlElement = self.cursor_ref.cast().expect("Not htmlelement");
        let style = wid.style();
        style.set_property("display",&format!("none")).expect("Cant set display");
    }

    fn mouseup(&mut self, event: MouseEvent) {
        if let Some(context) = &self.context {
            self.pressed = false;
            let canvas: HtmlCanvasElement =
                self.canvas_ref.cast().expect("Not html canvas element");
            let rect = canvas.get_bounding_client_rect();
            let point = Point {
                id: self.points.len() as u32,
                x: event.offset_x() as f64,
                y: event.offset_y() as f64,
                width: rect.width(),
                height: rect.height(),
                draw: true,
                color: self.current_color.clone(),
                line_width: self.current_width,
                eraser:self.is_eraser,
            };

            self.set_point(&point);
            // self.draw();


        // self.points.push(
        //     Point{
        //         x:event.offset_x() as f64,
        //         y:event.offset_y() as f64
        //     }
        // );
        // context.line_to(event.offset_x() as f64, event.offset_y() as f64);
        // context.stroke();
        } else {
            log::warn!("Context not ready, not drawing");
        }
    }

    fn mousemove(&mut self, event: MouseEvent) {
        self.set_cursor_pos(&event);
        if let Some(context) = &self.context {
            if self.pressed {
                let canvas: HtmlCanvasElement =
                    self.canvas_ref.cast().expect("Not html canvas element");
                let rect = canvas.get_bounding_client_rect();
                let point = Point {
                    id: self.points.len() as u32,
                    x: event.offset_x() as f64,
                    y: event.offset_y() as f64,
                    width: rect.width(),
                    height: rect.height(),
                    draw: true,
                    color: self.current_color.clone(),
                    line_width: self.current_width,
                    eraser: self.is_eraser
                };

                self.set_point(&point);


                self.draw();
                // context.line_to(event.offset_x() as f64, event.offset_y() as f64);
                // context.stroke();

                // context.begin_path();
                // context.move_to(event.offset_x() as f64, event.offset_y() as f64);
            }
        }
    }

    fn resetcanvas(&mut self) {
        let canvas: HtmlCanvasElement = self.canvas_ref.cast().expect("Not html canvas element");
        let rect = canvas.get_bounding_client_rect();
        canvas.set_width(rect.width() as u32);
        canvas.set_height(rect.height() as u32);

        self.todraw.clear();
        self.todraw.append(&mut self.points.clone());
        // self.draw();
    }

    fn draw(&mut self) {
        if let Some(context) = &self.context {
            let canvas: HtmlCanvasElement =
                self.canvas_ref.cast().expect("Not html canvas element");
            for point in self.todraw.iter() {
                // log::debug!("{:#?}",point);
                if point.id == 0 || !point.draw {
                    if point.id == 0 {
                        context.clear_rect(
                            0_f64,
                            0_f64,
                            canvas.width() as f64,
                            canvas.height() as f64,
                        );
                    }
                    // context.stroke();
                    // context.begin_path();
                    // context.line_to(point.get_x(&canvas), point.get_y(&canvas));
                    // context.stroke();
                    context.move_to(point.get_x(&canvas), point.get_y(&canvas));
                }
                else {
                    context.line_to(point.get_x(&canvas), point.get_y(&canvas));
                    if point.eraser{
                        context.set_stroke_style(&JsValue::from_str("white"));
                    }else{
                        context.set_stroke_style(&JsValue::from_str(&point.color));
                    }
                    context.set_line_width(point.line_width as f64 * point.get_scale_factor(&canvas));
                    context.set_line_cap("round");
                    context.stroke();
                    context.begin_path();
                    context.move_to(point.get_x(&canvas), point.get_y(&canvas));
                }
            }
            self.todraw.clear();
        } else {
            log::warn!("Cant draw, no context");
        }
    }

    fn touchstart(&mut self, event: TouchEvent) {
        if let Some(context) = &self.context {
            let canvas: HtmlCanvasElement =
                self.canvas_ref.cast().expect("Not html canvas element");
            let rect = canvas.get_bounding_client_rect();
            let point = Point {
                id: self.points.len() as u32,
                x: self.get_offset_x(&event),
                y: self.get_offset_y(&event),
                width: rect.width(),
                height: rect.height(),
                draw: false,
                color: self.current_color.clone(),
                line_width: self.current_width,
                eraser:self.is_eraser
            };

            self.set_point(&point);


            self.pressed = true;
        } else {
            log::warn!("Context not ready, not drawing");
        }
    }

    fn touchend(&mut self, event: TouchEvent) {
        if let Some(context) = &self.context {
            self.pressed = false;
            let canvas: HtmlCanvasElement =
                self.canvas_ref.cast().expect("Not html canvas element");
            let rect = canvas.get_bounding_client_rect();
            let point = Point {
                id: self.points.len() as u32,
                x: self.get_offset_x(&event),
                y: self.get_offset_y(&event),
                width: rect.width(),
                height: rect.height(),
                draw: false,
                color: self.current_color.clone(),
                line_width: self.current_width,
                eraser:self.is_eraser
            };

            self.set_point(&point);

        // context.line_to(self.get_offset_x(&event), self.get_offset_y(&event));
        // context.stroke();
        } else {
            log::warn!("Context not ready, not drawing");
        }
    }

    fn touchmove(&mut self, event: TouchEvent) {
        if let Some(context) = &self.context {
            if self.pressed {
                let canvas: HtmlCanvasElement =
                    self.canvas_ref.cast().expect("Not html canvas element");
                let rect = canvas.get_bounding_client_rect();
                let point = Point {
                    id: self.points.len() as u32,
                    x: self.get_offset_x(&event),
                    y: self.get_offset_y(&event),
                    width: rect.width(),
                    height: rect.height(),
                    draw: true,
                    color: self.current_color.clone(),
                    line_width: self.current_width,
                    eraser:self.is_eraser
                };
                self.set_point(&point);

                self.draw();
            }
        }
    }

    fn get_offset_x(&self, event: &TouchEvent) -> f64 {
        let canvas: HtmlCanvasElement = self.canvas_ref.cast().expect("Not html canvas element");
        let rect = canvas.get_bounding_client_rect();
        let ev0 = event.target_touches().get(0);
        if let Some(ev) = ev0 {
            let pgx = ev.client_x();
            (pgx as f64) - rect.left()
        } else {
            log::warn!("No touch 0 in event {:#?}", event);
            0_f64
        }
    }
    fn get_offset_y(&self, event: &TouchEvent) -> f64 {
        let canvas: HtmlCanvasElement = self.canvas_ref.cast().expect("Not html canvas element");
        let rect = canvas.get_bounding_client_rect();
        let ev0 = event.target_touches().get(0);
        if let Some(ev) = ev0 {
            let pgx = ev.client_y();
            (pgx as f64) - rect.top()
        } else {
            log::warn!("No touch 0 in event {:#?}", event);
            0_f64
        }
    }
}
