use std::ffi::OsString;
use std::os::windows::prelude::OsStringExt;

use ctrlc;
use fabric_ext::{IFabricWaitableCallback, WaitableCallback};
use log::info;
use service_fabric_rs::FabricCommon::FabricRuntime::{
    FabricBeginGetNodeContext, FabricCreateRuntime, FabricEndGetNodeContext,
    FabricGetActivationContext, IFabricCodePackageActivationContext, IFabricNodeContextResult,
    IFabricRuntime,
};
use service_fabric_rs::FabricCommon::IFabricAsyncOperationCallback;
use std::sync::mpsc::channel;
use windows::core::{Interface, Vtable, PCWSTR};
use windows::w;
pub mod app;

fn main() -> windows::core::Result<()> {
    env_logger::init();
    // set ctrc event
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    info!("echomain start");
    // hack to wait for debugger
    // std::thread::sleep(std::time::Duration::from_secs(90));
    // info!("sleep ended");

    let rawruntime =
        unsafe { FabricCreateRuntime(&IFabricRuntime::IID).expect("cannot create runtime") };
    let runtime = unsafe { IFabricRuntime::from_raw(rawruntime) };

    let raw_activation_ctx = unsafe {
        FabricGetActivationContext(&IFabricCodePackageActivationContext::IID)
            .expect("Cannot get activation ctx")
    };

    let activation_ctx =
        unsafe { IFabricCodePackageActivationContext::from_raw(raw_activation_ctx) };

    run_app(&runtime, &activation_ctx);

    info!("Waiting for Ctrl-C...");
    rx.recv().expect("Could not receive from channel.");
    info!("Got it! Exiting...");
    Ok(())
}

fn run_app(runtime: &IFabricRuntime, activation_ctx: &IFabricCodePackageActivationContext) {
    let port = get_port(activation_ctx);
    let hostname = get_hostname();
    app::run(runtime, port, hostname);
}

fn get_port(activation_ctx: &IFabricCodePackageActivationContext) -> u32 {
    info!("trying to get port");
    let endpoint_name = w!("ServiceEndpoint1");
    let endpoint = unsafe {
        activation_ctx
            .GetServiceEndpointResource(endpoint_name)
            .expect("cannot get endpoint")
    };
    return unsafe { (*endpoint).Port };
}

fn get_hostname() -> OsString {
    // let result = String::from_utf16_lossy(std::slice::from_raw_parts(
    let callback: IFabricWaitableCallback = WaitableCallback::new().into();

    let callback_arg: IFabricAsyncOperationCallback = callback.cast().expect("castfailed");
    let ctx = unsafe { FabricBeginGetNodeContext(1000, &callback_arg).expect("getctx failed") };

    unsafe { callback.wait() };

    let result_raw = unsafe { FabricEndGetNodeContext(&ctx).expect("end failed") };

    let result = unsafe { IFabricNodeContextResult::from_raw(result_raw) };

    let node_ctx = unsafe { result.get_NodeContext() };

    let hostname_raw: PCWSTR = unsafe { (*node_ctx).IPAddressOrFQDN };

    let ret = OsString::from_wide(unsafe { hostname_raw.as_wide() });
    info!("got hostname: {:?}", ret);
    return ret;
}
