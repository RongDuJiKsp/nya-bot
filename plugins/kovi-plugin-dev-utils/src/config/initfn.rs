use anyhow::anyhow;
use kovi::tokio::sync::RwLock;
use kovi::utils::{load_json_data, save_json_data};
use kovi::{PluginBuilder, RuntimeBot};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::any::type_name;
use std::ops::Deref;
use std::path::Path;
use std::sync::{Arc, OnceLock};
fn load_json<T: Default + Serialize + DeserializeOwned>(
    runtime_bot: &RuntimeBot,
    config_name: &str,
) -> Result<T, anyhow::Error> {
    load_json_data(T::default(), runtime_bot.get_data_path().join(config_name))
        .map_err(|e| anyhow!("Error loading JSON data: {}", e))
}
async fn defer_data<T: Default + Serialize + DeserializeOwned>(
    save_path: Arc<Box<Path>>,
    data: Arc<RwLock<T>>,
) {
    let d = data.deref().read().await;
    let _ = save_json_data(&*d, &*save_path);
}
pub fn init_config<T: Default + Serialize + DeserializeOwned>(
    runtime_bot: &RuntimeBot,
    config_name: &str,
    target: &'static OnceLock<T>,
) -> Result<(), anyhow::Error> {
    let config = load_json(runtime_bot, config_name)?;
    target
        .set(config)
        .map_err(|_e| anyhow!("初始化{}时出现重复设置", type_name::<T>()))?;
    Ok(())
}
pub fn init_data<T: Default + Serialize + DeserializeOwned + Send + Sync>(
    runtime_bot: &RuntimeBot,
    data_save_name: &str,
    target: &'static OnceLock<Arc<RwLock<T>>>,
) -> Result<(), anyhow::Error> {
    let json_path = Arc::new(
        runtime_bot
            .get_data_path()
            .join(data_save_name)
            .into_boxed_path(),
    );
    let data_ref = Arc::new(RwLock::new(
        load_json_data(T::default(), &*(json_path.clone()))
            .map_err(|e| anyhow!("Error loading JSON data: {}", e))?,
    ));
    let data = data_ref.clone();
    PluginBuilder::drop(move || defer_data(json_path.clone(), data_ref.clone()));
    target
        .set(data)
        .map_err(|_e| anyhow!("初始化{}的数据时发生异常", type_name::<T>()))?;

    Ok(())
}
