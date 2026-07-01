pub mod client;
pub mod repl;
pub mod ui;

pub fn start(host: &str, port: u16) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        repl::start_repl(host, port).await;
    });
}
