// use bevy::prelude::*;
// use bevy_egui::{egui, EguiContext, EguiPlugin};

// fn main() {
//   App::new()
//     .add_plugins(DefaultPlugins)
//     .add_plugin(EguiPlugin)
//     // Systems that create Egui widgets should be run during the `CoreStage::Update` stage,
//     // or after the `EguiSystem::BeginFrame` system (which belongs to the `CoreStage::PreUpdate` stage).
//     .add_system(ui_example)
//     .run();
// }

// fn ui_example(mut egui_context: ResMut<EguiContext>) {
//   egui::Window::new("Hello").show(egui_context.ctx_mut(), |ui| {
//     let response = ui.label("world");
//     // let response = response.interact(egui::Sense::click());
//     if response.clicked() {
//       println!("Click!");
//     }
//   });
// }

use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use std::thread;
#[cfg(target_arch = "wasm32")]
use wasm_thread as thread;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use crate::main;
    use wasm_bindgen::prelude::*;

    // Prevent `wasm_bindgen` from autostarting main on all spawned threads
    #[wasm_bindgen(start)]
    pub fn dummy_main() {}

    // Export explicit run function to start main
    #[wasm_bindgen]
    pub fn run() {
        console_log::init().unwrap();
        console_error_panic_hook::set_once();
        main();
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    for _ in 0..2 {
        thread::spawn(|| {
            for i in 1..3 {
                log::info!(
                    "hi number {} from the spawned thread {:?}!",
                    i,
                    thread::current().id()
                );
                thread::sleep(Duration::from_millis(1));
            }
        });
    }

    for i in 1..3 {
        log::info!(
            "hi number {} from the main thread {:?}!",
            i,
            thread::current().id()
        );
    }
}