//! WASM bindings for inventory history processor.

use crate::process_inventory_list::{
    EnumProshipType_InventoryStatus, InventoryData, ProcessInventoryHistoryInput,
    ProcessInventoryHistoryOutput, ProshipDimension, ProshipGoodsReceipt, ProshipInventory,
    ProshipInventoryHistory,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HandleInventoryListInputSerializable {
    inventory: serde_json::Value,
    inventory_list: Vec<serde_json::Value>,
    asin_outbound_list: Vec<String>,
    goods: serde_json::Value,
    goods_receipt: serde_json::Value,
    _supplier: serde_json::Value,
    _customer: serde_json::Value,
    inventory_history_list: Vec<InventoryHistoryItem>,
    _receipt_orders: Vec<serde_json::Value>,
    inventory_id: String,
    inventory_ids_map: HashMap<String, i32>,
    from_date: Option<i32>,
    to_date: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InventoryHistoryItem {
    id: String,
    created_at: i32,
    stock_qty: i32,
    old_status: i32,
    new_status: i32,
    quantity: i32,
    goods_issue_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessedInventoryHistoryResultSerializable {
    entries: HashMap<String, ProcessedEntry>,
    total_duration: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessedEntry {
    data: serde_json::Value,
    goods_issue_ids: Vec<String>,
}

fn json_f64(v: &serde_json::Value, key: &str) -> f64 {
    v.get(key).and_then(|x| x.as_f64()).unwrap_or(0.0)
}

fn json_f32(v: &serde_json::Value, key: &str) -> f32 {
    json_f64(v, key) as f32
}

fn json_i32(v: &serde_json::Value, key: &str) -> i32 {
    v.get(key).and_then(|x| x.as_i64()).unwrap_or(0) as i32
}

fn json_i64(v: &serde_json::Value, key: &str) -> i64 {
    v.get(key).and_then(|x| x.as_i64()).unwrap_or(0)
}

fn json_str(v: &serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string()
}

fn parse_dimension(v: &serde_json::Value) -> Option<ProshipDimension> {
    let obj = v.as_object()?;
    Some(ProshipDimension {
        length: obj.get("length").and_then(|x| x.as_f64()).unwrap_or(0.0),
        width: obj.get("width").and_then(|x| x.as_f64()).unwrap_or(0.0),
        height: obj.get("height").and_then(|x| x.as_f64()).unwrap_or(0.0),
    })
}

fn inventory_from_json(v: &serde_json::Value) -> ProshipInventory {
    let status_num = json_i32(v, "status");
    let status = EnumProshipType_InventoryStatus::from_i32(status_num);
    let master_dim = v.get("masterDimension").and_then(parse_dimension);
    let dim = v.get("dimension").and_then(parse_dimension);
    ProshipInventory {
        id: json_str(v, "id"),
        creator_id: json_i64(v, "creatorId"),
        created_at: json_i32(v, "createdAt"),
        updated_at: json_i32(v, "updatedAt"),
        status,
        shelf_code: json_str(v, "shelfCode"),
        customer_id: json_i64(v, "customerId"),
        stock_qty: json_i32(v, "stockQty"),
        stock_cbm: json_f32(v, "stockCbm"),
        goods_receipt_id: json_str(v, "goodsReceiptId"),
        goods_issue_id: json_str(v, "goodsIssueId"),
        goods_id: json_str(v, "goodsId"),
        duration: json_i32(v, "duration"),
        export_at: json_i32(v, "exportAt"),
        asin: json_str(v, "asin"),
        supplier_id: json_str(v, "supplierId"),
        asin_outbound: json_str(v, "asinOutbound"),
        index_customs_declaration: json_str(v, "indexCustomsDeclaration"),
        unit_price: json_f32(v, "unitPrice"),
        inner_qty_on_mas: json_i32(v, "innerQtyOnMas").max(1),
        po_no: json_str(v, "poNo"),
        master_dimension: master_dim,
        dimension: dim,
        volume: json_f32(v, "volume"),
        master_volume: json_f32(v, "masterVolume"),
        master_qty: json_i32(v, "masterQty"),
        do_no: json_str(v, "doNo"),
    }
}

fn goods_receipt_from_json(v: &serde_json::Value) -> ProshipGoodsReceipt {
    ProshipGoodsReceipt {
        imported_at: json_i32(v, "importedAt"),
    }
}

fn inventory_data_to_json(d: &InventoryData) -> serde_json::Value {
    serde_json::json!({
        "openingStock": d.opening_stock,
        "openingCbm": d.opening_cbm,
        "openingMasterQty": d.opening_master_qty,
        "asin": d.asin,
        "asinOutbound": d.asin_outbound,
        "unitPrice": d.unit_price,
        "receivedDate": d.received_date,
        "innerQtyOnMas": d.inner_qty_on_mas,
        "date": d.date,
        "lineInCd": d.line_in_cd,
        "poNo": d.po_no,
        "masterDimension": {"length": d.master_dimension.length, "width": d.master_dimension.width, "height": d.master_dimension.height},
        "dimension": {"length": d.dimension.length, "width": d.dimension.width, "height": d.dimension.height},
        "inboundQty": d.inbound_qty,
        "inboundCbm": d.inbound_cbm,
        "inboundMasterQty": d.inbound_master_qty,
        "closingStock": d.closing_stock,
        "closingCbm": d.closing_cbm,
        "closingMasterQty": d.closing_master_qty,
        "allocatedQty": d.allocated_qty,
        "allocatedCbm": d.allocated_cbm,
        "allocatedMasterQty": d.allocated_master_qty,
        "disposalStock": d.disposal_stock,
        "disposalCbm": d.disposal_cbm,
        "disposalMasterQty": d.disposal_master_qty,
        "restoreStockQty": d.restore_stock_qty,
        "restoreStockCbm": d.restore_stock_cbm,
        "restoreMasterQty": d.restore_master_qty,
        "outboundQty": d.outbound_qty,
        "outboundCbm": d.outbound_cbm,
        "outboundMasterQty": d.outbound_master_qty,
        "storageTimeDays": d.storage_time_days,
    })
}

#[wasm_bindgen]
pub fn process_inventory_history_wasm(input_json: &str) -> String {
    let input: HandleInventoryListInputSerializable =
        serde_json::from_str(input_json).expect("Invalid input JSON");

    let inventory = inventory_from_json(&input.inventory);
    let goods_receipt = goods_receipt_from_json(&input.goods_receipt);

    let inventory_history_list: Vec<ProshipInventoryHistory> = input
        .inventory_history_list
        .into_iter()
        .map(|h| ProshipInventoryHistory {
            created_at: h.created_at,
            stock_qty: h.stock_qty,
            old_status: EnumProshipType_InventoryStatus::from_i32(h.old_status),
            new_status: EnumProshipType_InventoryStatus::from_i32(h.new_status),
            quantity: h.quantity,
            goods_issue_id: h.goods_issue_id.unwrap_or_default(),
        })
        .collect();

    let process_input = ProcessInventoryHistoryInput {
        inventory: &inventory,
        inventory_id: &input.inventory_id,
        asin_outbound_list: &input.asin_outbound_list,
        goods_receipt: &goods_receipt,
        inventory_history_list,
        inventory_ids: input.inventory_ids_map,
        from_date: input.from_date,
        to_date: input.to_date,
    };

    let output: ProcessInventoryHistoryOutput =
        crate::process_inventory_list::process_inventory_history_data(process_input);

    let mut entries: HashMap<String, ProcessedEntry> = HashMap::new();
    for (k, (data, goods_issue_ids)) in output.merged_inventory_history {
        entries.insert(
            k,
            ProcessedEntry {
                data: inventory_data_to_json(&data),
                goods_issue_ids: goods_issue_ids.into_iter().collect::<Vec<String>>(),
            },
        );
    }

    let result = ProcessedInventoryHistoryResultSerializable {
        entries,
        total_duration: output.total_duration,
    };

    serde_json::to_string(&result).expect("Failed to serialize result")
}
