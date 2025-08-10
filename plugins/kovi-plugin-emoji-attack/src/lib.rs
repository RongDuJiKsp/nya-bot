mod app;
mod config;
mod data;
mod handle;

#[kovi::plugin]
async fn main() {
    app::init().await;
}
