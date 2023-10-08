#[cfg(not(target_arch = "wasm32"))]
pub mod pc;

#[cfg(target_arch = "wasm32")] 
pub mod web;

pub mod utils;