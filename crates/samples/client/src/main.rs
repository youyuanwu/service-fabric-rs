use fabric_ext::{IFabricWaitableCallback, WaitableCallback};
use service_fabric_rs::FabricCommon::{FabricClient::*, IFabricAsyncOperationCallback};
use service_fabric_rs::{FABRIC_NODE_QUERY_DESCRIPTION, FABRIC_NODE_QUERY_RESULT_ITEM};
use windows::core::*;

fn main() -> windows::core::Result<()> {
    println!("GetNodeCli");

    let rawclient = unsafe {
        FabricCreateLocalClient(&IFabricQueryClient::IID).expect("cannot get localclient")
    };
    // todo: figure out owner ship
    let c: IFabricQueryClient = unsafe { IFabricQueryClient::from_raw(rawclient) };

    let callback: IFabricWaitableCallback = WaitableCallback::new().into();

    let callback_arg: IFabricAsyncOperationCallback = callback.cast().expect("castfailed");

    let querydescription = FABRIC_NODE_QUERY_DESCRIPTION::default();

    let ctx = unsafe {
        c.BeginGetNodeList(&querydescription, 1000, &callback_arg)
            .expect("cannot get ctx")
    };

    // wait callback to be triggered
    unsafe { callback.wait() };

    // note: there must be a variable to hold COM object, ortherwise it is released.
    // result.expect().get_NodeList() will give a released/garbage node description pointer.
    let result = unsafe { c.EndGetNodeList(&ctx) };
    let result_node_list = result.expect("endcall_failed");

    let nodes = unsafe { result_node_list.get_NodeList() };
    let node_count = unsafe { (*nodes).Count };
    let node_list = unsafe { (*nodes).Items };

    println!("node_count {}", node_count);

    if !node_list.is_null() {
        let node: FABRIC_NODE_QUERY_RESULT_ITEM = unsafe { *node_list };
        // this is ugly
        // println!("node info: name: {:#?}", node);
        println!("node info: name: {}", unsafe { node.NodeName.display() });
    }

    Ok(())
}
