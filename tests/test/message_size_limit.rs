//local shortcuts

//third-party shortcuts
use serde::{Serialize, Deserialize};

//standard shortcuts
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// message from server
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemoServerMsg(pub u64);

/// message from client
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemoClientMsg(pub String);

/// client connect message
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemoConnectMsg(pub String);

type ServerDemo = bevy_simplenet::Server::<DemoServerMsg, DemoClientMsg, DemoConnectMsg>;
type ClientDemo = bevy_simplenet::Client::<DemoServerMsg, DemoClientMsg, DemoConnectMsg>;

fn server_demo_factory() -> ServerDemo::Factory
{
    ServerDemo::Factory::new("test")
}

fn client_demo_factory() -> ClientDemo::Factory
{
    ClientDemo::Factory::new("test")
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn message_size_limit_test(max_msg_size: u32)
{
    // prepare tokio runtimes for server and client
    let server_runtime = enfync::builtin::Handle::default();
    let client_runtime = enfync::builtin::Handle::default();

    // prepare connection acceptor
    let plain_acceptor = ezsockets::tungstenite::Acceptor::Plain;

    // launch websocket server
    let websocket_server = enfync::blocking::extract(server_demo_factory().new_server(
            server_runtime,
            "127.0.0.1:0",
            plain_acceptor,
            bevy_simplenet::Authenticator::None,
            bevy_simplenet::ServerConfig{
                max_connections   : 10,
                max_msg_size,
                rate_limit_config : bevy_simplenet::RateLimitConfig{
                        period    : std::time::Duration::from_millis(15),
                        max_count : 25
                    }
            }
        )).unwrap();

    let websocket_url = websocket_server.url();


    // 1. prepare message that is too large
    let mut msg_vec = Vec::<u8>::new();
    msg_vec.resize((max_msg_size + 1) as usize, 1u8);
    let large_msg = String::from_utf8(msg_vec).unwrap();


    // 2. connect message size limit

    // make client with invalid connect message size
    let large_connect_msg = DemoConnectMsg(large_msg.clone());
    let websocket_client = enfync::blocking::extract(client_demo_factory().new_client(
            client_runtime.clone(),
            websocket_url.clone(),
            bevy_simplenet::AuthRequest::None{ client_id: 67891u128 },
            bevy_simplenet::ClientConfig::default(),
            large_connect_msg
        )).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(25));  //wait for async machinery

    assert!(websocket_client.is_dead());  //failed to connect
    let Some(bevy_simplenet::ClientReport::Connected) = websocket_client.next_report()
    else { panic!("client should be connected to server"); };
    let Some(bevy_simplenet::ClientReport::ClosedByServer(_)) = websocket_client.next_report()
    else { panic!("client should be closed by server"); };


    // 3. client message size limit

    // make client
    let connect_msg = DemoConnectMsg(String::from(""));
    let websocket_client = enfync::blocking::extract(client_demo_factory().new_client(
            client_runtime.clone(),
            websocket_url,
            bevy_simplenet::AuthRequest::None{ client_id: 4678587u128 },
            bevy_simplenet::ClientConfig::default(),
            connect_msg.clone()
        )).unwrap();
    assert!(!websocket_client.is_dead());

    std::thread::sleep(std::time::Duration::from_millis(25));  //wait for async machinery

    let Some(bevy_simplenet::ServerReport::Connected(client_id, connect_msg)) = websocket_server.next_report()
    else { panic!("server should be connected once client is connected"); };
    let Some(bevy_simplenet::ClientReport::Connected) = websocket_client.next_report()
    else { panic!("client should be connected to server"); };
    assert_eq!(connect_msg.0, connect_msg.0);

    // send message with invalid size: client -> server
    websocket_client.send(&DemoClientMsg(large_msg)).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(25));  //wait for async machinery

    // expect no message acquired by server
    let None = websocket_server.next_msg() else { panic!("server received client msg"); };

    // expect client was disconnected
    assert!(websocket_client.is_dead());

    let Some(bevy_simplenet::ServerReport::Disconnected(dc_client_id)) = websocket_server.next_report()
    else { panic!("client should be disconnected"); };
    let Some(bevy_simplenet::ClientReport::ClosedByServer(_)) = websocket_client.next_report()
    else { panic!("client should be closed by server"); };
    assert_eq!(client_id, dc_client_id);


    // no more connection reports
    let None = websocket_server.next_report()
    else { panic!("server should receive no more connection reports"); };
    let None = websocket_client.next_report()
    else { panic!("client should receive no more connection reports"); };
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[test]
fn bevy_simplenet_message_size_limit()
{
    message_size_limit_test(25);
    message_size_limit_test(40);
    message_size_limit_test(100);
}

//-------------------------------------------------------------------------------------------------------------------