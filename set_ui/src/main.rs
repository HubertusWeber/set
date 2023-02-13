#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "set",
        native_options,
        Box::new(|cc| Box::new(set_ui::SetUI::new(cc))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "set",
            web_options,
            Box::new(|cc| Box::new(set_ui::SetUI::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
