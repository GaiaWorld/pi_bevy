use std::sync::{Arc, Mutex, OnceLock};
use std::io::Result;

use ahash::HashMap;
use crossbeam::queue::SegQueue;
use futures::future::{BoxFuture, FutureExt, LocalBoxFuture};
use json::JsonValue;
use pi_world::schedule::First;
use pi_ws::{connect::WsSocket, server::WebsocketListener, utils::{ChildProtocol, WsFrameType, WsSession}};
use pi_tcp::{
    SocketConfig, SocketEvent,
    connect::TcpSocket,
    server::{PortsAdapterFactory, SocketListener}
};
use pi_world::{app::App, prelude::{Plugin, World}, schedule::{End, Last}};
use pi_async_rt::rt::serial::AsyncRuntimeBuilder;
use serde::{Deserialize, Serialize};

pub static CMDS: OnceLock<Arc<SegQueue<(String, WsSocket<TcpSocket>)>>> = OnceLock::new();
pub static SOCKETS: OnceLock<Mutex<HashMap<usize, WsSocket<TcpSocket>>>> = OnceLock::new();

pub use crate::SpectorNode;

#[derive(Serialize, Debug, Clone, Default)]
#[allow(non_snake_case)]
pub struct Cmd<T: serde::Serialize> {
    pub cmd: String,
    pub payload: T,
}

pub fn send_cmd<T: serde::Serialize>(cmd: Cmd<T>, sockets: &HashMap<usize, WsSocket<TcpSocket>>) {
    let cmd_string = serde_json::to_string(&cmd).unwrap();
    for socket in sockets.values() {
        if let Err(e) = socket.send(WsFrameType::Text, cmd_string.as_bytes().to_vec()) {
            log::error!("Error sending message: {:?}", e);
        }
    }
}

/// 指令的播放方法
pub type FnCmdParse = fn(& mut World, WsSocket<TcpSocket>, json::object::Object);


#[derive(Default)]
pub struct CMDCalls {
    pub cmdcalls: HashMap<String, FnCmdParse>,
}

struct MyChildProtocol;

impl ChildProtocol<TcpSocket> for MyChildProtocol {
    fn protocol_name(&self) -> &str {
        "echo"
    }

    fn is_strict(&self) -> bool {
        false
    }

    fn decode_protocol(&self,
                       connect: WsSocket<TcpSocket>,
                       context: &mut WsSession
    ) -> LocalBoxFuture<'static, Result<()>> {
        let uid = connect.get_uid();
        let sockes = SOCKETS.get().unwrap();
        let mut sockes = sockes.lock().unwrap();
        if sockes.get(&uid).is_none() {
            sockes.insert(uid, connect.clone());
        }

        let msg = context.pop_msg();
        // println!("!!!!!!receive ok, msg: {:?}", (String::from_utf8(msg.clone()), uid));
        CMDS.get().unwrap().push(( String::from_utf8(msg).unwrap(), connect));

        async move {
            Ok(())
        }.boxed_local()
    }

    fn close_protocol(&self,
                      connect: WsSocket<TcpSocket>,
                      _context: WsSession,
                      reason: Result<()>
    ) -> LocalBoxFuture<'static, ()> {
        let uid = connect.get_uid();
        let sockes = SOCKETS.get().unwrap();
        sockes.lock().unwrap().remove(&uid);
        println!("websocket closed");
        async move {
            if let Err(e) = reason {
                return println!("websocket closed, reason: {:?}", e);
            }

            println!("websocket closed");
        }.boxed_local()
    }
    fn protocol_timeout(&self,
                        _connect: WsSocket<TcpSocket>,
                        _context: &mut WsSession,
                        _event: SocketEvent
    ) -> LocalBoxFuture<'static, Result<()>> {
        async move {
            println!("websocket timeout");

            Ok(())
        }.boxed_local()
    }
}


fn start_websocket_server() {

    let rt0 = AsyncRuntimeBuilder::default_local_thread(None, None);
    let rt1 = AsyncRuntimeBuilder::default_local_thread(None, None);

    let mut factory = PortsAdapterFactory::<TcpSocket>::new();
    factory.bind(3001,
                 Box::new(WebsocketListener::with_protocol(Arc::new(MyChildProtocol))));
    let mut config = SocketConfig::new("0.0.0.0", factory.ports().as_slice());
    config.set_option(16384, 16384, 16384, 16);

    match SocketListener::bind(vec![rt0, rt1],
                               factory,
                               config,
                               1024,
                               1024 * 1024,
                               1024,
                               16,
                               4096,
                               4096,
                               Some(1000)) {
        Err(e) => {
            println!("!!!> Websocket Listener Bind Error, reason: {:?}", e);
        },
        Ok(_driver) => {
            println!("===> Websocket Listener in: {:?}", "0.0.0.0:3001");
        }
    }

}


pub fn sys_parse_cmd(world: &mut World) {
    let cmds = CMDS.get().unwrap();
    let mut cur_cmd = cmds.pop();
    let cmdcalls = world.get_single_res::<CMDCalls>().unwrap().cmdcalls.clone();
    while let Some(data) = cur_cmd {
        let parsed = json::parse(&data.0);
        match parsed {
            Ok(JsonValue::Object(obj)) => {
                match obj.get("cmd") {
                    Some(JsonValue::Short(cmd)) => {
                        if let Some(call) = cmdcalls.get(cmd.as_str()) {
                            call(world, data.1, obj);
                        } else {
                            log::error!("Spector CMD Not Match: {:?}", cmd.as_str());
                        }
                    },
                    r => {
                        log::error!("cmd invalid: {:?}", r);
                    }
                };

            },
            r => {
                log::error!("message invalid: {:?}", r);
            }
        };
        cur_cmd = cmds.pop();
    }
}

pub struct PluginSpector;
impl Plugin for PluginSpector {
    fn build(&self, app: &mut App) {
        // 初始化全局变量
        CMDS.get_or_init(|| {
            Arc::new(SegQueue::new())
        });
        SOCKETS.get_or_init(|| {
            Mutex::new(HashMap::default())
        });

        // 启动一个http服务
        // let _out = std::process::Command::new("node")
        // .args([
        //     "src/devtools/http_server.js",
        // ])
        // .spawn();

        //启动一个websocket服务
        start_websocket_server();
        app.world.insert_single_res(CMDCalls::default());
        app.add_system(First, sys_parse_cmd);
    }
}
