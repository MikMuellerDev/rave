use std::time::Duration;

use actix::{Actor, StreamHandler};
use actix_web::{
    rt,
    web::{self, Data},
    HttpRequest, HttpResponse, Result,
};
use actix_web_actors::ws;
use actix_ws::AggregatedMessage;
use cpal::traits::DeviceTrait;
use crossbeam_channel::TryRecvError;
use serde::{Deserialize, Serialize};

use crate::{app::FromFrontend, audio::SystemMessage, utils::device_from_name};

use super::AppState;

struct MyWebSocket {
    app_state: web::Data<AppState>,
}

// impl Actor for MyWebSocket {
//     type Context = ws::WebsocketContext<Self>;
// }

//
// From frontend message.
//

#[derive(Deserialize, Clone, Debug)]
pub enum WSFromFrontendKind {
    SelectAudioDevice,
    SelectSerialDevice,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WSFromFrontend {
    kind: WSFromFrontendKind,
    value: serde_json::Value,
}

// TODO: json deserial.
impl From<WSFromFrontend> for FromFrontend {
    fn from(value: WSFromFrontend) -> Self {
        match value.kind {
            WSFromFrontendKind::SelectAudioDevice => {
                if value.value == serde_json::Value::Null {
                    Self::SelectInputDevice(None)
                } else {
                    let serde_json::Value::String(device_name) = value.value else {
                        panic!("not a string");
                    };

                    println!("select device convert: {}", &device_name);
                    let device = device_from_name(device_name);
                    println!("device is some: {}", device.is_some());
                    Self::SelectInputDevice(device)
                }
            }
            WSFromFrontendKind::SelectSerialDevice => {
                if value.value == serde_json::Value::Null {
                    Self::SelectSerialDevice(None)
                } else {
                    let device = value.value.to_string();
                    Self::SelectSerialDevice(Some(device))
                }
            }
        }
    }
}

//
// To frontent message,
//

#[derive(Serialize)]
pub enum WSSystemMessageKind {
    Heartbeat,
    Log,
    LoopSpeed,
    AudioSelected,
    AudioDevicesView,
    SerialSelected,
    SerialDevicesView,
    Dmx,
}

#[derive(Serialize)]
pub struct WSSystemMessage {
    kind: WSSystemMessageKind,
    value: serde_json::Value,
}

impl From<SystemMessage> for WSSystemMessage {
    fn from(value: SystemMessage) -> Self {
        match value {
            SystemMessage::Heartbeat(seq) => Self {
                kind: WSSystemMessageKind::Heartbeat,
                value: serde_json::to_value(seq).unwrap(),
            },
            SystemMessage::Log(msg) => Self {
                kind: WSSystemMessageKind::LoopSpeed,
                value: serde_json::to_value(msg).unwrap(),
            },
            SystemMessage::LoopSpeed(duration) => Self {
                kind: WSSystemMessageKind::LoopSpeed,
                value: serde_json::to_value(duration.as_micros()).unwrap(),
            },
            SystemMessage::AudioSelected(device) => Self {
                kind: WSSystemMessageKind::AudioDevicesView,
                value: match device {
                    Some(d) => serde_json::to_value(d.name().unwrap()).unwrap(),
                    None => serde_json::Value::Null,
                },
            },
            SystemMessage::AudioDevicesView(devs) => Self {
                kind: WSSystemMessageKind::AudioDevicesView,
                value: serde_json::to_value(
                    devs.iter()
                        .map(|d| d.1.name().unwrap())
                        .collect::<Vec<String>>(),
                )
                .unwrap(),
            },
            SystemMessage::SerialSelected(serial_port_info) => Self {
                kind: WSSystemMessageKind::SerialSelected,
                value: match serial_port_info {
                    Some(d) => serde_json::to_value(d.port_name).unwrap(),
                    None => serde_json::Value::Null,
                },
            },

            SystemMessage::SerialDevicesView(devs) => Self {
                kind: WSSystemMessageKind::AudioDevicesView,
                value: serde_json::to_value(
                    devs.iter()
                        .map(|d| d.port_name.clone())
                        .collect::<Vec<String>>(),
                )
                .unwrap(),
            },
        }
    }
}

// impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
//     fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
//         match msg {
//             Ok(ws::Message::Text(text)) => {
//                 let msg: WSFromFrontend = serde_json::from_str(text.to_string().as_str()).unwrap();
//                 self.app_state
//                     .from_frontend_sender
//                     .send(msg.clone().into())
//                     .unwrap();
//                 println!("recv ws: {msg:?}");
//                 // ctx.text(format!("Echo: {}", text)); // Echo the received text
//             }
//             Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
//             Ok(ws::Message::Close(reason)) => {
//                 ctx.close(reason);
//                 // ctx.stop();
//             }
//             _ => {
//                 match self.app_state.app_signal_receiver.try_recv() {
//                     Ok(signal) => {
//                         println!("app signal: ${signal:?}");
//                         ctx.text(serde_json::to_string(&signal).unwrap());
//                     }
//                     Err(TryRecvError::Empty) => {}
//                     Err(TryRecvError::Disconnected) => unreachable!(),
//                 }
//
//                 match self.app_state.app_system_receiver.try_recv() {
//                     Ok(sys) => {
//                         ctx.text(serde_json::to_string(&WSSystemMessage::from(sys)).unwrap());
//                     }
//                     Err(TryRecvError::Empty) => {}
//                     Err(TryRecvError::Disconnected) => unreachable!(),
//                 }
//             }
//         }
//     }
// }

// impl Actor for MyWebSocket {
//     type Context = ws::WebsocketContext<Self>;
//
//     fn started(&mut self, ctx: &mut Self::Context) {
//         let app_state = self.app_state.clone();
//         let actor_addr = ctx.address(); // Clone the actor's address
//
//         // Spawn a periodic task to poll the channel
//         actix::spawn(async move {
//             let mut interval = actix::clock::interval(Duration::from_millis(50));
//             loop {
//                 interval.tick().await;
//
//                 match app_state.app_system_receiver.try_recv() {
//                     Ok(sys) => {
//                         let message = serde_json::to_string(&WSSystemMessage::from(sys)).unwrap();
//                         // actor_addr.do_send(WSSendMessage(message)); // Send the message back to the actor
//                     }
//                     Err(TryRecvError::Empty) => {}
//                     Err(TryRecvError::Disconnected) => {
//                         // actor_addr.do_send(); // Notify the actor to close the WebSocket
//                         break; // Exit the loop
//                     }
//                 }
//             }
//         });
//     }
// }

pub async fn ws_handler(
    req: HttpRequest,
    data: Data<AppState>,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    let data2 = data.clone();
    let mut session2 = session.clone();

    rt::spawn(async move {
        match data2.app_signal_receiver.try_recv() {
            Ok(signal) => {
                println!("app signal: ${signal:?}");
                session2.text(serde_json::to_string(&signal).unwrap()).await.unwrap();
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => unreachable!(),
        }

        match data2.app_system_receiver.try_recv() {
            Ok(sys) => {
                session2.text(serde_json::to_string(&WSSystemMessage::from(sys)).unwrap()).await.unwrap();
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => unreachable!(),
        }
    });

    // start task but don't wait for it
    rt::spawn(async move {
        // receive messages from websocket
        while let Some(msg) = stream.recv().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    // echo text message
                    session.text(text.clone()).await.unwrap();

                    let msg: WSFromFrontend =
                        serde_json::from_str(text.to_string().as_str()).unwrap();

                    data.from_frontend_sender.send(msg.clone().into()).unwrap();
                    println!("recv ws: {msg:?}");
                }

                Ok(AggregatedMessage::Binary(bin)) => {
                    // echo binary message
                    session.binary(bin).await.unwrap();
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    // respond to PING frame with PONG frame
                    session.pong(&msg).await.unwrap();
                }

                _ => {}
            }
        }
    });

    // respond immediately with response connected to WS session
    Ok(res)
}

// pub async fn websocket_handler(
//     req: HttpRequest,
//     app_state: web::Data<AppState>,
//     stream: web::Payload,
// ) -> Result<HttpResponse> {
//     ws::start(MyWebSocket { app_state }, &req, stream) }
