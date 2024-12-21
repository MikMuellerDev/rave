#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
                                                                   //
use anyhow::anyhow;
use blaulicht::dmx;
use std::thread;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    use std::sync::mpsc;

    use anyhow::bail;
    use blaulicht::{app, config};
    // Log to stderr (if you run with `RUST_LOG=debug`).
    use egui::TextBuffer;
    env_logger::init();

    // Load config
    let config_path = config::config_path()?;
    let Some(config) = config::read_config(config_path.clone())? else {
        eprintln!(
            "Created default config file at `{}`",
            config_path.to_string_lossy()
        );
        return Ok(());
    };

    let (_from_frontend_sender, from_frontend_receiver) = crossbeam_channel::unbounded();
    // let (signal_out, signal_receiver) = crossbeam_channel::unbounded();

    // let (dmx_signal_out, dmx_signal_receiver) = crossbeam_channel::unbounded();
    let (app_signal_out, app_signal_receiver) = crossbeam_channel::unbounded();

    // {
    // Fanout thread.
    // TODO: remove this in the long run, this will add latency.
    // thread::spawn(move || loop {
    //     let s = signal_receiver.recv().unwrap();
    //     dmx_signal_out.send(s).unwrap();
    //     app_signal_out.send(s).unwrap();
    // });
    // }

    let (system_out, _system_receiver) = crossbeam_channel::unbounded();

    {
        // Audio recording and analysis thread.
        let system_out = system_out.clone();
        thread::spawn(|| {
            dmx::audio_thread(
                from_frontend_receiver,
                app_signal_out,
                system_out,
            )
        });
    }

    // let (dmx_control_sender, dmx_control_receiver) = crossbeam_channel::unbounded();

    // {
        // DMX thread.
        // thread::spawn(move || {
        //     dmx::dmx_thread(dmx_control_receiver, dmx_signal_receiver, system_out)
        // });
    // }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };

    let crate_name = env!("CARGO_CRATE_NAME");
    let crate_version = env!("CARGO_PKG_VERSION");
    eframe::run_native(
        format!("{crate_name} v{crate_version}").as_str(),
        native_options,
        Box::new(|cc| {
            Ok(Box::new(blaulicht::BlaulichtApp::new(
                cc,
                app_signal_receiver,
                config,
            )))
        }),
    )
    .map_err(|err| anyhow!(err.to_string()))?;

    Ok(())
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(eframe_template::TemplateApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
