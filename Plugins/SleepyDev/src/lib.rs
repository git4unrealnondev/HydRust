#[path = "../../../src/scr/sharedtypes.rs"]
mod sharedtypes;

static PLUGIN_NAME: &str = "SleepyDev";
static PLUGIN_DESCRIPTION: &str = "Just makes the plugin manager wait. Dev use only lol";

#[no_mangle]
pub fn return_info() -> sharedtypes::PluginInfo {
    let callbackvec = vec![sharedtypes::PluginCallback::OnStart];
    sharedtypes::PluginInfo {
        name: PLUGIN_NAME.to_string(),
        description: PLUGIN_DESCRIPTION.to_string(),
        version: 1.00,
        api_version: 1.00,
        callbacks: callbackvec,
        communication: Some(sharedtypes::PluginSharedData {
            thread: sharedtypes::PluginThreadType::Spawn,
            com_channel: Some(sharedtypes::PluginCommunicationChannel::Pipe(
                "".to_string(),
            )),
        }),
    }
}

#[no_mangle]
pub fn on_start() {
    use std::{thread, time};
    let wait = time::Duration::from_secs(1);
    loop {
        thread::sleep(wait);
    }
}
