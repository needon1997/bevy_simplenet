//local shortcuts

//third-party shortcuts
use serde::{Serialize, Deserialize};

//standard shortcuts
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// message from server
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemoServerMsg(pub u64);

/// message from client
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemoClientMsg(pub u64);

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

#[test]
fn bevy_simplenet_hello_world()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    tracing::info!("ws hello world test: start");
    */

    // prepare tokio runtimes for server and client
    let server_runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
    let client_runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());

    // prepare connection acceptor
    let plain_acceptor = ezsockets::tungstenite::Acceptor::Plain;

    // launch websocket server
    tracing::info!("ws hello world test: launching server...");
    let websocket_server = server_demo_factory().new_server(
            server_runtime,
            "127.0.0.1:0",
            plain_acceptor,
            bevy_simplenet::Authenticator::None,
            bevy_simplenet::ConnectionConfig{
                max_connections   : 10,
                max_msg_size      : 1_000,
                rate_limit_config : bevy_simplenet::RateLimitConfig{
                        period    : std::time::Duration::from_secs(1),
                        max_count : 20
                    }
            }
        );

    let websocket_url = bevy_simplenet::make_websocket_url(websocket_server.address()).unwrap();



    // make client (block until connected)
    tracing::info!("ws hello world test: launching client...");
    let connect_msg1 = DemoConnectMsg(String::from("hello!"));
    let websocket_client = client_demo_factory().new_client(
            client_runtime.clone(),
            websocket_url.clone(),
            bevy_simplenet::AuthRequest::None{ client_id: 44718u128 },
            connect_msg1.clone()
        ).extract().unwrap().unwrap();
    assert!(!websocket_client.is_dead());

    std::thread::sleep(std::time::Duration::from_millis(25));  //wait for async machinery

    let Some(bevy_simplenet::ConnectionReport::Connected(client_id, connect_msg)) =
        websocket_server.try_get_next_connection_report()
    else { panic!("server should be connected once client is connected"); };
    assert_eq!(connect_msg.0, connect_msg1.0);


    // send message: client -> server
    tracing::info!("ws hello world test: client sending msg...");
    let client_val = 42;
    websocket_client.send_msg(&DemoClientMsg(client_val)).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(25));  //wait for async machinery

    let Some((msg_client_id, DemoClientMsg(msg_client_val))) = websocket_server.try_get_next_msg()
    else { panic!("server did not receive client msg"); };
    assert_eq!(client_id, msg_client_id);
    assert_eq!(client_val, msg_client_val);


    // send message: server -> client
    tracing::info!("ws hello world test: server sending msg...");
    let server_val = 24;
    websocket_server.send_msg(client_id, DemoServerMsg(server_val)).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(25));  //wait for async machinery

    let Some(DemoServerMsg(msg_server_val)) = websocket_client.try_get_next_msg()
    else { panic!("client did not receive server msg"); };
    assert_eq!(server_val, msg_server_val);


    // server closes client
    tracing::info!("ws hello world test: server closing client...");
    let closure_frame =
        ezsockets::CloseFrame{
            code   : ezsockets::CloseCode::Normal,
            reason : String::from("test")
        };
    websocket_server.close_session(client_id, closure_frame).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(25));  //wait for async machinery

    assert!(!websocket_server.is_dead());
    assert!(websocket_client.is_dead());

    let Some(bevy_simplenet::ConnectionReport::Disconnected(dc_client_id)) = websocket_server.try_get_next_connection_report()
    else { panic!("server should be disconnected after client is disconnected (by server)"); };
    assert_eq!(client_id, dc_client_id);



    // new client (block until connected)
    tracing::info!("ws hello world test: launching client 2...");
    let connect_msg2 = DemoConnectMsg(String::from("hello 2!"));
    let websocket_client = client_demo_factory().new_client(
            client_runtime.clone(),
            websocket_url,
            bevy_simplenet::AuthRequest::None{ client_id: 872657u128 },
            connect_msg2.clone()
        ).extract().unwrap().unwrap();
    assert!(!websocket_client.is_dead());

    std::thread::sleep(std::time::Duration::from_millis(25));  //wait for async machinery

    let Some(bevy_simplenet::ConnectionReport::Connected(client_id, connect_msg)) =
        websocket_server.try_get_next_connection_report()
    else { panic!("server should be connected once client is connected"); };
    assert_eq!(connect_msg.0, connect_msg2.0);


    // client closes client
    tracing::info!("ws hello world test: client closing client...");
    websocket_client.close();

    std::thread::sleep(std::time::Duration::from_millis(25));  //wait for async machinery

    assert!(!websocket_server.is_dead());
    assert!(websocket_client.is_dead());

    let Some(bevy_simplenet::ConnectionReport::Disconnected(dc_client_id)) = websocket_server.try_get_next_connection_report()
    else { panic!("server should be disconnected after client is disconnected (by client)"); };
    assert_eq!(client_id, dc_client_id);
}

//-------------------------------------------------------------------------------------------------------------------
