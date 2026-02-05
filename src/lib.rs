//! Rust inventory history processor.
//! WASM build: wasm-pack build --target web --features wasm

mod process_inventory_list;

pub use process_inventory_list::{
    process_inventory_history_data, ProcessInventoryHistoryInput,
    ProcessInventoryHistoryOutput,
};

#[cfg(feature = "wasm")]
mod wasm;
