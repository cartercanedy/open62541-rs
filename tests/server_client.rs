use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use open62541::{ClientBuilder, ServerBuilder, VariableNode, ua};

#[derive(Debug)]
enum ClientStrategy {
    Disconnect,
    Drop,
}

// This mirrors test `create_and_destroy_client()` in `open62541-sys`.
#[test]
fn create_and_destroy_client() {
    // This does not actually connect to anything.
    let _client = ua::Client::default();
}

// This mirrors test `open_server_and_connect()` in `open62541-sys`.
async fn open_server_and_connect(client_strategy: ClientStrategy) {
    // Initialize new server listening on random port.
    let (server, runner) = ServerBuilder::default()
        .server_urls(&["opc.tcp://127.0.0.1:0"])
        .build();

    let value_node_id = server
        .add_variable_node(VariableNode {
            requested_new_node_id: None,
            parent_node_id: ua::NodeId::ns0(open62541_sys::UA_NS0ID_OBJECTSFOLDER),
            reference_type_id: ua::NodeId::ns0(open62541_sys::UA_NS0ID_ORGANIZES),
            browse_name: ua::QualifiedName::new(1, "fixture"),
            type_definition: ua::NodeId::ns0(open62541_sys::UA_NS0ID_BASEDATAVARIABLETYPE),
            attributes: ua::VariableAttributes::default()
                .with_data_type(ua::NodeId::ns0(open62541_sys::UA_NS0ID_STRING)),
        })
        .expect("add fixture variable");
    let expected = "fixture value";
    server
        .write_value(
            &value_node_id,
            &ua::Variant::scalar(ua::String::new(expected).unwrap()),
        )
        .expect("configure fixture value");

    // Run server in background thread, iterating event loop.
    let running = Arc::new(AtomicBool::new(true));
    let background_thread = thread::spawn({
        let running = Arc::clone(&running);
        move || {
            runner
                .run_until_cancelled(|| !running.load(Ordering::Relaxed))
                .expect("run server");
        }
    });

    // In main thread, wait for discovery URL (after start-up).
    let discovery_urls = server.discovery_urls().expect("get discovery URLs");
    let discovery_url = discovery_urls
        .as_slice()
        .first()
        .expect("get discovery URL")
        .as_str()
        .expect("valid discovery URL");

    // Connect to given URL.
    let client = ClientBuilder::default()
        .connect(discovery_url)
        .expect("connect client")
        .into_async();

    let node_class = client
        .read_attribute(&value_node_id, ua::AttributeId::NODECLASS_T)
        .await
        .expect("read node class");
    assert_eq!(node_class.scalar_value(), Some(&ua::NodeClass::VARIABLE));
    assert_eq!(
        node_class.into_scalar_value(),
        Some(ua::NodeClass::VARIABLE)
    );

    let value = client.read_value(&value_node_id).await.expect("read value");

    let value: ua::String = value
        .into_value()
        .expect("has value")
        .into_scalar()
        .expect("has string value");

    assert_eq!(value.as_str(), Some(expected));

    // Disconnect or drop the client.
    match client_strategy {
        ClientStrategy::Disconnect => {
            client.disconnect().await;
        }
        ClientStrategy::Drop => {
            drop(client);
        }
    }

    // Shut down server.
    running.store(false, Ordering::Relaxed);
    background_thread.join().unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn open_server_and_connect_disconnect_client_current_thread() {
    open_server_and_connect(ClientStrategy::Disconnect).await;
}

#[tokio::test(flavor = "current_thread")]
async fn open_server_and_connect_drop_client_current_thread() {
    open_server_and_connect(ClientStrategy::Drop).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn open_server_and_connect_disconnect_client_multi_thread() {
    open_server_and_connect(ClientStrategy::Disconnect).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn open_server_and_connect_drop_client_multi_thread() {
    open_server_and_connect(ClientStrategy::Drop).await;
}
