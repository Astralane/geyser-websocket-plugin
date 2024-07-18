use crate::plugin::GeyserPluginPostgres;
use log::info;
use solana_geyser_plugin_interface::geyser_plugin_interface::GeyserPlugin;

#[no_mangle]
#[allow(improper_ctypes_definitions)]
/// # Safety
///
/// This function simply allocates a GeyserPluginHook,
/// and returns a pointer to it as trait GeyserPlugin.
pub unsafe extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin {
    info!("Creating plugin");
    let plugin = GeyserPluginPostgres::new();
    info!("Plugin created");
    let plugin: Box<dyn GeyserPlugin> = Box::new(plugin);
    Box::into_raw(plugin)
}
