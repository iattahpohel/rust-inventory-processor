//! Native Node.js addon for rust-inventory-processor.
//! Build: cd native && npm run build

#![deny(clippy::all)]

use std::panic::catch_unwind;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rust_inventory_processor::process_inventory_history_native;

#[napi]
pub fn process_inventory_history(input_json: String) -> Result<String> {
    catch_unwind(|| process_inventory_history_native(&input_json)).map_err(|e| {
        let msg = if let Some(s) = e.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = e.downcast_ref::<String>() {
            s.clone()
        } else {
            "Processing failed".to_string()
        };
        Error::from_reason(msg)
    })
}
