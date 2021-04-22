// use serde::{Deserialize, Serialize};
// use serde_json::Result;
// use winit::event::VirtualKeyCode;

// TODO: custom key mapping
// #[derive(Serialize, Deserialize)]
// pub struct Keyboard {
//     right: VirtualKeyCode,
//     up: VirtualKeyCode,
//     left: VirtualKeyCode,
//     down: VirtualKeyCode,
//     a: VirtualKeyCode,
//     b: VirtualKeyCode,
//     select: VirtualKeyCode,
//     start: VirtualKeyCode,
// }

// #[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct Config {
    file_path: String,
}

impl Config {
    pub fn new(file_path: String) -> Self {
        Self { file_path }
    }

    pub fn get_file_path(&self) -> &str {
        &self.file_path
    }
}
