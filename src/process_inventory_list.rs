//! Split: process_inventory_history_data (pure) + handle_inventory_list (fetch + process + return)
//! Self-contained - no external imports. For TypeScript implementation reference.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
#[cfg(not(feature = "wasm"))]
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Datelike, DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};

// =============================================================================
// Types - all definitions inline for standalone use
// =============================================================================

#[allow(nonstandard_style)]
#[derive(Clone, PartialEq, Copy, Default)]
#[repr(u8)]
pub enum EnumProshipType_InventoryStatus {
    #[default]
    OTHER_INVENTORY_STATUS = 0,
    AVALABLE_INVENTORY_STATUS = 1,
    ON_HAND_INVENTORY_STATUS = 2,
    DAMAGED_INVENTORY_STATUS = 3,
    RETURN_INVENTORY_STATUS = 4,
    LIQUIDATION_INVENTORY_STATUS = 5,
    EXPORTED_INVENTORY_STATUS = 6,
    PENDING_FOR_IMPORT_INVENTORY_STATUS = 7,
}

impl EnumProshipType_InventoryStatus {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::AVALABLE_INVENTORY_STATUS,
            2 => Self::ON_HAND_INVENTORY_STATUS,
            3 => Self::DAMAGED_INVENTORY_STATUS,
            4 => Self::RETURN_INVENTORY_STATUS,
            5 => Self::LIQUIDATION_INVENTORY_STATUS,
            6 => Self::EXPORTED_INVENTORY_STATUS,
            7 => Self::PENDING_FOR_IMPORT_INVENTORY_STATUS,
            _ => Self::OTHER_INVENTORY_STATUS,
        }
    }
}

#[derive(PartialEq, Clone, Default)]
pub struct ProshipDimension {
    pub length: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(PartialEq, Clone, Default)]
pub struct ProshipInventory {
    pub id: String,
    pub creator_id: i64,
    pub created_at: i32,
    pub updated_at: i32,
    pub status: EnumProshipType_InventoryStatus,
    pub shelf_code: String,
    pub customer_id: i64,
    pub stock_qty: i32,
    pub stock_cbm: f32,
    pub goods_receipt_id: String,
    pub goods_issue_id: String,
    pub goods_id: String,
    pub duration: i32,
    pub export_at: i32,
    pub asin: String,
    pub supplier_id: String,
    pub asin_outbound: String,
    pub index_customs_declaration: String,
    pub unit_price: f32,
    pub inner_qty_on_mas: i32,
    pub po_no: String,
    pub master_dimension: Option<ProshipDimension>,
    pub dimension: Option<ProshipDimension>,
    pub volume: f32,
    pub master_volume: f32,
    pub master_qty: i32,
    pub do_no: String,
}

impl ProshipInventory {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn get_volume(&self) -> f32 {
        self.volume
    }
    pub fn get_master_volume(&self) -> f32 {
        self.master_volume
    }
    pub fn get_inner_qty_on_mas(&self) -> i32 {
        self.inner_qty_on_mas
    }
    pub fn get_asin(&self) -> &str {
        &self.asin
    }
    pub fn get_unit_price(&self) -> f32 {
        self.unit_price
    }
    pub fn get_index_customs_declaration(&self) -> &str {
        &self.index_customs_declaration
    }
    pub fn get_po_no(&self) -> &str {
        &self.po_no
    }
    pub fn get_master_dimension(&self) -> ProshipDimension {
        self.master_dimension.clone().unwrap_or_default()
    }
    pub fn get_dimension(&self) -> ProshipDimension {
        self.dimension.clone().unwrap_or_default()
    }
}

#[derive(Default)]
pub struct ProshipGoodsReceipt {
    pub imported_at: i32,
}

impl ProshipGoodsReceipt {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn get_imported_at(&self) -> i32 {
        self.imported_at
    }
}

#[derive(Default)]
pub struct ProshipInventoryHistory {
    pub created_at: i32,
    pub stock_qty: i32,
    pub old_status: EnumProshipType_InventoryStatus,
    pub new_status: EnumProshipType_InventoryStatus,
    pub quantity: i32,
    pub goods_issue_id: String,
}

impl ProshipInventoryHistory {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn get_created_at(&self) -> i32 {
        self.created_at
    }
    pub fn get_stock_qty(&self) -> i32 {
        self.stock_qty
    }
    pub fn get_old_status(&self) -> EnumProshipType_InventoryStatus {
        self.old_status
    }
    pub fn get_new_status(&self) -> EnumProshipType_InventoryStatus {
        self.new_status
    }
    pub fn get_quantity(&self) -> i32 {
        self.quantity
    }
    pub fn get_goods_issue_id(&self) -> &str {
        &self.goods_issue_id
    }
}

#[derive(Default)]
pub struct InventoryData {
    pub opening_stock: i32,
    pub opening_cbm: f32,
    pub opening_master_qty: i32,
    pub asin: String,
    pub asin_outbound: Vec<String>,
    pub unit_price: f32,
    pub received_date: i32,
    pub inner_qty_on_mas: i32,
    pub date: i32,
    pub line_in_cd: String,
    pub po_no: String,
    pub master_dimension: ProshipDimension,
    pub dimension: ProshipDimension,
    pub inbound_qty: i32,
    pub inbound_cbm: f32,
    pub inbound_master_qty: i32,
    pub closing_stock: i32,
    pub closing_cbm: f32,
    pub closing_master_qty: i32,
    pub allocated_qty: i32,
    pub allocated_cbm: f32,
    pub allocated_master_qty: i32,
    pub disposal_stock: i32,
    pub disposal_cbm: f32,
    pub disposal_master_qty: i32,
    pub restore_stock_qty: i32,
    pub restore_stock_cbm: f32,
    pub restore_master_qty: i32,
    pub outbound_qty: i32,
    pub outbound_cbm: f32,
    pub outbound_master_qty: i32,
    pub storage_time_days: i32,
}

impl InventoryData {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn set_opening_stock(&mut self, v: i32) {
        self.opening_stock = v;
    }
    pub fn set_opening_cbm(&mut self, v: f32) {
        self.opening_cbm = v;
    }
    pub fn set_opening_master_qty(&mut self, v: i32) {
        self.opening_master_qty = v;
    }
    pub fn set_asin(&mut self, v: String) {
        self.asin = v;
    }
    pub fn set_asin_outbound(&mut self, v: Vec<String>) {
        self.asin_outbound = v;
    }
    pub fn set_unit_price(&mut self, v: f32) {
        self.unit_price = v;
    }
    pub fn set_received_date(&mut self, v: i32) {
        self.received_date = v;
    }
    pub fn set_inner_qty_on_mas(&mut self, v: i32) {
        self.inner_qty_on_mas = v;
    }
    pub fn set_date(&mut self, v: i32) {
        self.date = v;
    }
    pub fn set_line_in_cd(&mut self, v: String) {
        self.line_in_cd = v;
    }
    pub fn set_po_no(&mut self, v: String) {
        self.po_no = v;
    }
    pub fn set_master_dimension(&mut self, v: ProshipDimension) {
        self.master_dimension = v;
    }
    pub fn set_dimension(&mut self, v: ProshipDimension) {
        self.dimension = v;
    }
    pub fn set_inbound_qty(&mut self, v: i32) {
        self.inbound_qty = v;
    }
    pub fn set_inbound_cbm(&mut self, v: f32) {
        self.inbound_cbm = v;
    }
    pub fn set_inbound_master_qty(&mut self, v: i32) {
        self.inbound_master_qty = v;
    }
    pub fn set_closing_stock(&mut self, v: i32) {
        self.closing_stock = v;
    }
    pub fn set_closing_cbm(&mut self, v: f32) {
        self.closing_cbm = v;
    }
    pub fn set_closing_master_qty(&mut self, v: i32) {
        self.closing_master_qty = v;
    }
    pub fn get_allocated_qty(&self) -> i32 {
        self.allocated_qty
    }
    pub fn set_allocated_qty(&mut self, v: i32) {
        self.allocated_qty = v;
    }
    pub fn set_allocated_cbm(&mut self, v: f32) {
        self.allocated_cbm = v;
    }
    pub fn set_allocated_master_qty(&mut self, v: i32) {
        self.allocated_master_qty = v;
    }
    pub fn get_disposal_stock(&self) -> i32 {
        self.disposal_stock
    }
    pub fn set_disposal_stock(&mut self, v: i32) {
        self.disposal_stock = v;
    }
    pub fn set_disposal_cbm(&mut self, v: f32) {
        self.disposal_cbm = v;
    }
    pub fn set_disposal_master_qty(&mut self, v: i32) {
        self.disposal_master_qty = v;
    }
    pub fn get_restore_stock_qty(&self) -> i32 {
        self.restore_stock_qty
    }
    pub fn set_restore_stock_qty(&mut self, v: i32) {
        self.restore_stock_qty = v;
    }
    pub fn set_restore_stock_cbm(&mut self, v: f32) {
        self.restore_stock_cbm = v;
    }
    pub fn set_restore_master_qty(&mut self, v: i32) {
        self.restore_master_qty = v;
    }
    pub fn get_outbound_qty(&self) -> i32 {
        self.outbound_qty
    }
    pub fn set_outbound_qty(&mut self, v: i32) {
        self.outbound_qty = v;
    }
    pub fn set_outbound_cbm(&mut self, v: f32) {
        self.outbound_cbm = v;
    }
    pub fn set_outbound_master_qty(&mut self, v: i32) {
        self.outbound_master_qty = v;
    }
    pub fn set_storage_time_days(&mut self, v: i32) {
        self.storage_time_days = v;
    }
    pub fn get_date(&self) -> i32 {
        self.date
    }
    pub fn get_storage_time_days(&self) -> i32 {
        self.storage_time_days
    }
}

// =============================================================================
// Helper functions - all inline for standalone use
// =============================================================================

fn get_disposal_key(date: String, status: EnumProshipType_InventoryStatus) -> String {
    format!("{}-{}", date, status as i32)
}

fn get_on_hand_key(date: String, goods_issue_id: String) -> String {
    format!("{}-{}", date, goods_issue_id)
}
/// Converts epoch time to human readable string in GMT+7 with format yyyymmdd
fn epoch_to_human_gmt7(ts: u64) -> String {
    let vietnam_offset = FixedOffset::east_opt(7 * 3600).unwrap();
    let naive_datetime = NaiveDateTime::from_timestamp_opt(ts as i64, 0).unwrap_or_default();
    let datetime_again: DateTime<FixedOffset> =
        DateTime::from_naive_utc_and_offset(naive_datetime, vietnam_offset);
    format!(
        "{:04}{:02}{:02}",
        datetime_again.year(),
        datetime_again.month(),
        datetime_again.day()
    )
}

/// Checks if two Unix timestamps fall on the same day in GMT+7.
fn is_same_day_gmt7(timestamp1: u64, timestamp2: u64) -> bool {
    let vietnam_offset = FixedOffset::east_opt(7 * 3600).unwrap();
    let date1 = NaiveDateTime::from_timestamp_opt(timestamp1 as i64, 0)
        .unwrap_or_default()
        .date()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(vietnam_offset)
        .unwrap();
    let date2 = NaiveDateTime::from_timestamp_opt(timestamp2 as i64, 0)
        .unwrap_or_default()
        .date()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(vietnam_offset)
        .unwrap();
    date1 == date2
}

/// Calculates the number of days between two Unix timestamps.
/// Returns 0 if to is before from.
fn days_between(from_unix_timestamp: u64, to_unix_timestamp: Option<u64>) -> i32 {
    let vietnam_offset = FixedOffset::east_opt(7 * 3600).unwrap();

    let from_dt = vietnam_offset
        .timestamp_opt(from_unix_timestamp as i64, 0)
        .unwrap();
    let from_midnight = from_dt.date_naive().and_hms_opt(0, 0, 0).unwrap();

    let to_timestamp = to_unix_timestamp.unwrap_or_else(now_to_epoch);
    let to_dt = vietnam_offset
        .timestamp_opt(to_timestamp as i64, 0)
        .unwrap();
    let to_midnight = to_dt.date_naive().and_hms_opt(0, 0, 0).unwrap();

    let diff_in_days = (to_midnight - from_midnight).num_days();
    if diff_in_days > 0 {
        diff_in_days as i32
    } else {
        0
    }
}

#[cfg(not(feature = "wasm"))]
fn now_to_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(feature = "wasm")]
fn now_to_epoch() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

fn round_float<T>(value: T, precision: Option<i32>) -> T
where
    T: FloatRound,
{
    let precision = precision.unwrap_or(2);
    T::round_with_precision(value, precision)
}

trait FloatRound: Copy {
    fn round_with_precision(self, precision: i32) -> Self;
}

impl FloatRound for f32 {
    fn round_with_precision(self, precision: i32) -> Self {
        let multiplier = 10_f32.powi(precision);
        (self * multiplier).round() / multiplier
    }
}

impl FloatRound for f64 {
    fn round_with_precision(self, precision: i32) -> Self {
        let multiplier = 10_f64.powi(precision);
        (self * multiplier).round() / multiplier
    }
}

pub fn calculate_cbm(quantity: i32, inner_qty_on_mas: i32, volume: f32, master_volume: f32) -> f32 {
    let master_boxes = quantity / inner_qty_on_mas; // Total master boxes
    let inner_boxes = quantity % inner_qty_on_mas; // Remaining inner boxes
    let cbm: f32;

    // Calculate total cbm
    if master_volume > 0.0 {
        cbm = (master_boxes as f32 * master_volume) + (inner_boxes as f32 * volume);
    } else {
        cbm = quantity as f32 * volume;
    }

    round_float(cbm, Some(3))
}

pub fn calculate_master_qty(quantity: i32, inner_qty_on_mas: i32) -> i32 {
    (quantity + inner_qty_on_mas - 1) / inner_qty_on_mas
}

// =============================================================================
// Main logic
// =============================================================================

/// Input for process_inventory_history_data - all data fetched upfront.
pub struct ProcessInventoryHistoryInput<'a> {
    pub inventory: &'a ProshipInventory,
    pub inventory_id: &'a str,
    pub asin_outbound_list: &'a [String],
    pub goods_receipt: &'a ProshipGoodsReceipt,
    pub inventory_history_list: Vec<ProshipInventoryHistory>,
    pub inventory_ids: HashMap<String, i32>,
    pub from_date: Option<i32>,
    pub to_date: Option<i32>,
}

/// Output of process_inventory_history_data - processed data only.
pub struct ProcessInventoryHistoryOutput {
    pub merged_inventory_history: HashMap<String, (InventoryData, HashSet<String>)>,
    pub total_duration: i32,
}

/// Pure function: process inventory history data. No async, no I/O.
pub fn process_inventory_history_data(
    input: ProcessInventoryHistoryInput<'_>,
) -> ProcessInventoryHistoryOutput {
    let ProcessInventoryHistoryInput {
        inventory,
        inventory_id,
        asin_outbound_list,
        goods_receipt,
        inventory_history_list,
        inventory_ids,
        from_date,
        to_date,
    } = input;

    let volume = inventory.get_volume();
    let master_volume = inventory.get_master_volume();

    let mut merged_inventory_history: HashMap<String, (InventoryData, HashSet<String>)> =
        HashMap::new();
    let mut last_stock_qty = 0;
    let mut on_hand_map: HashMap<String, i32> = HashMap::new();
    let mut disposal_map: HashMap<String, i32> = HashMap::new();
    let mut total_disposal_qty: HashMap<i32, i32> = HashMap::new();
    total_disposal_qty.insert(3, 0);
    total_disposal_qty.insert(4, 0);
    total_disposal_qty.insert(5, 0);
    let mut total_allocated_qty: HashMap<String, i32> = HashMap::new();

    for history in inventory_history_list {
        if history.get_new_status()
            == EnumProshipType_InventoryStatus::PENDING_FOR_IMPORT_INVENTORY_STATUS
        {
            continue;
        }

        let date = epoch_to_human_gmt7(history.get_created_at() as u64);
        let entry = merged_inventory_history
            .entry(date.clone())
            .or_insert_with(|| {
                let mut data = InventoryData::new();
                data.set_opening_stock(last_stock_qty);
                data.set_opening_cbm(calculate_cbm(
                    last_stock_qty,
                    inventory.get_inner_qty_on_mas(),
                    volume,
                    master_volume,
                ));
                data.set_opening_master_qty(calculate_master_qty(
                    last_stock_qty,
                    inventory.get_inner_qty_on_mas(),
                ));
                data.set_asin(inventory.get_asin().to_string());
                data.set_asin_outbound(asin_outbound_list.to_vec());
                data.set_unit_price(inventory.get_unit_price());
                data.set_received_date(goods_receipt.get_imported_at());
                data.set_inner_qty_on_mas(inventory.get_inner_qty_on_mas());
                data.set_date(history.get_created_at());
                data.set_line_in_cd(inventory.get_index_customs_declaration().to_string());
                data.set_po_no(inventory.get_po_no().to_string());
                data.set_master_dimension(inventory.get_master_dimension().clone());
                data.set_dimension(inventory.get_dimension().clone());
                if is_same_day_gmt7(
                    history.get_created_at() as u64,
                    goods_receipt.get_imported_at() as u64,
                ) {
                    let inbound_qty = inventory_ids.get(inventory_id).unwrap_or(&0);
                    data.set_inbound_qty(*inbound_qty);
                    data.set_inbound_cbm(calculate_cbm(
                        *inbound_qty,
                        inventory.get_inner_qty_on_mas(),
                        volume,
                        master_volume,
                    ));
                    data.set_inbound_master_qty(calculate_master_qty(
                        *inbound_qty,
                        inventory.get_inner_qty_on_mas(),
                    ));
                }
                (data, HashSet::new())
            });

        let (data, goods_issue_ids) = entry;
        data.set_closing_stock(history.get_stock_qty());
        data.set_closing_cbm(calculate_cbm(
            history.get_stock_qty(),
            inventory.get_inner_qty_on_mas(),
            volume,
            master_volume,
        ));
        data.set_closing_master_qty(calculate_master_qty(
            history.get_stock_qty(),
            inventory.get_inner_qty_on_mas(),
        ));
        last_stock_qty = history.get_stock_qty();

        match history.get_old_status() {
            EnumProshipType_InventoryStatus::AVALABLE_INVENTORY_STATUS => {
                match history.get_new_status() {
                    EnumProshipType_InventoryStatus::ON_HAND_INVENTORY_STATUS => {
                        let onhand_qty = on_hand_map
                            .entry(get_on_hand_key(
                                date.clone(),
                                history.get_goods_issue_id().to_string(),
                            ))
                            .or_insert(0);
                        *onhand_qty += history.get_quantity();

                        let total_allocated_qty_data = total_allocated_qty
                            .entry(history.get_goods_issue_id().to_string())
                            .or_insert(0);
                        *total_allocated_qty_data += history.get_quantity();

                        let allocated_qty = data.get_allocated_qty() + history.get_quantity();
                        data.set_allocated_qty(allocated_qty);
                        data.set_allocated_cbm(calculate_cbm(
                            allocated_qty,
                            inventory.get_inner_qty_on_mas(),
                            volume,
                            master_volume,
                        ));
                        data.set_allocated_master_qty(calculate_master_qty(
                            allocated_qty,
                            inventory.get_inner_qty_on_mas(),
                        ));
                    }
                    EnumProshipType_InventoryStatus::DAMAGED_INVENTORY_STATUS
                    | EnumProshipType_InventoryStatus::RETURN_INVENTORY_STATUS
                    | EnumProshipType_InventoryStatus::LIQUIDATION_INVENTORY_STATUS => {
                        let disposal_qty = disposal_map
                            .entry(get_disposal_key(date.clone(), history.get_new_status()))
                            .or_insert(0);
                        *disposal_qty += history.get_quantity();

                        if let Some(total_disposal_qty_data) =
                            total_disposal_qty.get_mut(&(history.get_new_status() as i32))
                        {
                            *total_disposal_qty_data += history.get_quantity();
                        }

                        let disposal_qty = data.get_disposal_stock() + history.get_quantity();
                        data.set_disposal_stock(disposal_qty);
                        data.set_disposal_cbm(calculate_cbm(
                            disposal_qty,
                            inventory.get_inner_qty_on_mas(),
                            volume,
                            master_volume,
                        ));
                        data.set_disposal_master_qty(calculate_master_qty(
                            disposal_qty,
                            inventory.get_inner_qty_on_mas(),
                        ));
                    }
                    _ => {}
                }
            }
            EnumProshipType_InventoryStatus::ON_HAND_INVENTORY_STATUS => {
                match history.get_new_status() {
                    EnumProshipType_InventoryStatus::AVALABLE_INVENTORY_STATUS => {
                        let total_allocated_qty_data = total_allocated_qty
                            .entry(history.get_goods_issue_id().to_string())
                            .or_insert(0);
                        if let Some(onhand_qty) = on_hand_map.get_mut(&get_on_hand_key(
                            date.clone(),
                            history.get_goods_issue_id().to_string(),
                        )) {
                            match (*total_allocated_qty_data - history.get_quantity())
                                .cmp(onhand_qty)
                            {
                                Ordering::Greater | Ordering::Equal => {
                                    let restore_stock_qty =
                                        data.get_restore_stock_qty() + history.get_quantity();
                                    data.set_restore_stock_qty(restore_stock_qty);
                                    data.set_restore_stock_cbm(calculate_cbm(
                                        restore_stock_qty,
                                        inventory.get_inner_qty_on_mas(),
                                        volume,
                                        master_volume,
                                    ));
                                    data.set_restore_master_qty(calculate_master_qty(
                                        restore_stock_qty,
                                        inventory.get_inner_qty_on_mas(),
                                    ));
                                }
                                Ordering::Less => {
                                    let previous_onhand_qty =
                                        *total_allocated_qty_data - *onhand_qty;
                                    let restore_stock_qty =
                                        data.get_restore_stock_qty() + previous_onhand_qty;
                                    data.set_restore_stock_qty(restore_stock_qty);
                                    data.set_restore_stock_cbm(calculate_cbm(
                                        restore_stock_qty,
                                        inventory.get_inner_qty_on_mas(),
                                        volume,
                                        master_volume,
                                    ));
                                    data.set_restore_master_qty(calculate_master_qty(
                                        restore_stock_qty,
                                        inventory.get_inner_qty_on_mas(),
                                    ));
                                    let allocated_qty = data.get_allocated_qty()
                                        - (history.get_quantity() - previous_onhand_qty);
                                    data.set_allocated_qty(allocated_qty);
                                    data.set_allocated_cbm(calculate_cbm(
                                        allocated_qty,
                                        inventory.get_inner_qty_on_mas(),
                                        volume,
                                        master_volume,
                                    ));
                                    data.set_allocated_master_qty(calculate_master_qty(
                                        allocated_qty,
                                        inventory.get_inner_qty_on_mas(),
                                    ));
                                    *onhand_qty =
                                        *total_allocated_qty_data - history.get_quantity();
                                }
                            }
                        } else {
                            let restore_stock_qty =
                                data.get_restore_stock_qty() + history.get_quantity();
                            data.set_restore_stock_qty(restore_stock_qty);
                            data.set_restore_stock_cbm(calculate_cbm(
                                restore_stock_qty,
                                inventory.get_inner_qty_on_mas(),
                                volume,
                                master_volume,
                            ));
                            data.set_restore_master_qty(calculate_master_qty(
                                restore_stock_qty,
                                inventory.get_inner_qty_on_mas(),
                            ));
                        }
                        *total_allocated_qty_data -= history.get_quantity();
                    }
                    EnumProshipType_InventoryStatus::DAMAGED_INVENTORY_STATUS
                    | EnumProshipType_InventoryStatus::RETURN_INVENTORY_STATUS
                    | EnumProshipType_InventoryStatus::LIQUIDATION_INVENTORY_STATUS => {
                        let total_allocated_qty_data = total_allocated_qty
                            .entry(history.get_goods_issue_id().to_string())
                            .or_insert(0);
                        if let Some(onhand_qty) = on_hand_map.get_mut(&get_on_hand_key(
                            date.clone(),
                            history.get_goods_issue_id().to_string(),
                        )) {
                            if (*total_allocated_qty_data - history.get_quantity()) < *onhand_qty {
                                let previous_onhand_qty =
                                    *total_allocated_qty_data - *onhand_qty;
                                let onhand_disposal_qty =
                                    history.get_quantity() - previous_onhand_qty;

                                let allocated_qty = data.get_allocated_qty() - onhand_disposal_qty;
                                data.set_allocated_qty(allocated_qty);
                                data.set_allocated_cbm(calculate_cbm(
                                    allocated_qty,
                                    inventory.get_inner_qty_on_mas(),
                                    volume,
                                    master_volume,
                                ));
                                data.set_allocated_master_qty(calculate_master_qty(
                                    allocated_qty,
                                    inventory.get_inner_qty_on_mas(),
                                ));
                                *onhand_qty -= onhand_disposal_qty;

                                let disposal_qty = data.get_disposal_stock() + onhand_disposal_qty;
                                data.set_disposal_stock(disposal_qty);
                                data.set_disposal_cbm(calculate_cbm(
                                    disposal_qty,
                                    inventory.get_inner_qty_on_mas(),
                                    volume,
                                    master_volume,
                                ));
                                data.set_disposal_master_qty(calculate_master_qty(
                                    disposal_qty,
                                    inventory.get_inner_qty_on_mas(),
                                ));
                            }
                        }

                        *total_allocated_qty_data -= history.get_quantity();
                        if let Some(total_disposal_qty_data) =
                            total_disposal_qty.get_mut(&(history.get_new_status() as i32))
                        {
                            *total_disposal_qty_data += history.get_quantity();
                        }

                        let key = get_disposal_key(date.clone(), history.get_new_status());
                        *disposal_map.entry(key).or_insert(0) += history.get_quantity();
                    }
                    EnumProshipType_InventoryStatus::EXPORTED_INVENTORY_STATUS => {
                        let total_allocated_qty_data = total_allocated_qty
                            .entry(history.get_goods_issue_id().to_string())
                            .or_insert(0);
                        if let Some(onhand_qty) = on_hand_map.get_mut(&get_on_hand_key(
                            date.clone(),
                            history.get_goods_issue_id().to_string(),
                        )) {
                            let allocated_qty = data.get_allocated_qty() - history.get_quantity();
                            data.set_allocated_qty(allocated_qty);
                            data.set_allocated_cbm(calculate_cbm(
                                allocated_qty,
                                inventory.get_inner_qty_on_mas(),
                                volume,
                                master_volume,
                            ));
                            data.set_allocated_master_qty(calculate_master_qty(
                                allocated_qty,
                                inventory.get_inner_qty_on_mas(),
                            ));
                            *onhand_qty -= history.get_quantity();
                        }
                        *total_allocated_qty_data -= history.get_quantity();

                        let outbound_qty = data.get_outbound_qty() + history.get_quantity();
                        data.set_outbound_qty(outbound_qty);
                        data.set_outbound_cbm(calculate_cbm(
                            outbound_qty,
                            inventory.get_inner_qty_on_mas(),
                            volume,
                            master_volume,
                        ));
                        data.set_outbound_master_qty(calculate_master_qty(
                            outbound_qty,
                            inventory.get_inner_qty_on_mas(),
                        ));
                        goods_issue_ids.insert(history.get_goods_issue_id().to_string());
                        data.set_storage_time_days(
                            days_between(
                                from_date.map_or_else(
                                    || goods_receipt.get_imported_at() as u64,
                                    |fd| {
                                        if fd < goods_receipt.get_imported_at() {
                                            goods_receipt.get_imported_at() as u64
                                        } else {
                                            fd as u64
                                        }
                                    },
                                ),
                                to_date.map_or_else(
                                    || Some(history.get_created_at() as u64),
                                    |td| {
                                        if td <= history.get_created_at() {
                                            Some(td as u64)
                                        } else {
                                            Some(history.get_created_at() as u64)
                                        }
                                    },
                                ),
                            ) as i32
                                + 1,
                        );
                    }
                    _ => {}
                }
            }
            EnumProshipType_InventoryStatus::DAMAGED_INVENTORY_STATUS
            | EnumProshipType_InventoryStatus::RETURN_INVENTORY_STATUS
            | EnumProshipType_InventoryStatus::LIQUIDATION_INVENTORY_STATUS => {
                match history.get_new_status() {
                    EnumProshipType_InventoryStatus::AVALABLE_INVENTORY_STATUS => {
                        if let Some(total_disposal_qty_data) =
                            total_disposal_qty.get_mut(&(history.get_old_status() as i32))
                        {
                            if let Some(disposal_qty) = disposal_map
                                .get_mut(&get_disposal_key(date.clone(), history.get_old_status()))
                            {
                                match (*total_disposal_qty_data - history.get_quantity())
                                    .cmp(disposal_qty)
                                {
                                    Ordering::Greater | Ordering::Equal => {
                                        let restore_stock_qty =
                                            data.get_restore_stock_qty() + history.get_quantity();
                                        data.set_restore_stock_qty(restore_stock_qty);
                                        data.set_restore_stock_cbm(calculate_cbm(
                                            restore_stock_qty,
                                            inventory.get_inner_qty_on_mas(),
                                            volume,
                                            master_volume,
                                        ));
                                        data.set_restore_master_qty(calculate_master_qty(
                                            restore_stock_qty,
                                            inventory.get_inner_qty_on_mas(),
                                        ));
                                    }
                                    Ordering::Less => {
                                        let previous_disposal_qty =
                                            *total_disposal_qty_data - *disposal_qty;
                                        let disposal_available_qty =
                                            history.get_quantity() - previous_disposal_qty;
                                        let disposal_qty_data =
                                            data.get_disposal_stock() - disposal_available_qty;
                                        data.set_disposal_stock(disposal_qty_data);
                                        data.set_disposal_cbm(calculate_cbm(
                                            disposal_qty_data,
                                            inventory.get_inner_qty_on_mas(),
                                            volume,
                                            master_volume,
                                        ));
                                        data.set_disposal_master_qty(calculate_master_qty(
                                            disposal_qty_data,
                                            inventory.get_inner_qty_on_mas(),
                                        ));
                                        *disposal_qty =
                                            *total_disposal_qty_data - history.get_quantity();
                                    }
                                }
                            } else {
                                let restore_stock_qty =
                                    data.get_restore_stock_qty() + history.get_quantity();
                                data.set_restore_stock_qty(restore_stock_qty);
                                data.set_restore_stock_cbm(calculate_cbm(
                                    restore_stock_qty,
                                    inventory.get_inner_qty_on_mas(),
                                    volume,
                                    master_volume,
                                ));
                                data.set_restore_master_qty(calculate_master_qty(
                                    restore_stock_qty,
                                    inventory.get_inner_qty_on_mas(),
                                ));
                            }
                            *total_disposal_qty_data -= history.get_quantity();
                        }
                    }
                    EnumProshipType_InventoryStatus::ON_HAND_INVENTORY_STATUS => {
                        if let Some(total_disposal_qty_data) =
                            total_disposal_qty.get_mut(&(history.get_old_status() as i32))
                        {
                            if let Some(disposal_qty) = disposal_map
                                .get_mut(&get_disposal_key(date.clone(), history.get_old_status()))
                            {
                                match (*total_disposal_qty_data - history.get_quantity())
                                    .cmp(disposal_qty)
                                {
                                    Ordering::Greater | Ordering::Equal => {
                                        let onhand_qty =
                                            data.get_allocated_qty() + history.get_quantity();
                                        data.set_allocated_qty(onhand_qty);
                                        data.set_allocated_cbm(calculate_cbm(
                                            onhand_qty,
                                            inventory.get_inner_qty_on_mas(),
                                            volume,
                                            master_volume,
                                        ));
                                        data.set_allocated_master_qty(calculate_master_qty(
                                            onhand_qty,
                                            inventory.get_inner_qty_on_mas(),
                                        ));
                                    }
                                    Ordering::Less => {
                                        let previous_disposal_qty =
                                            *total_disposal_qty_data - *disposal_qty;
                                        let disposal_onhand_qty =
                                            history.get_quantity() - previous_disposal_qty;
                                        let disposal_qty_value =
                                            data.get_disposal_stock() - disposal_onhand_qty;
                                        data.set_disposal_stock(disposal_qty_value);
                                        data.set_disposal_cbm(calculate_cbm(
                                            disposal_qty_value,
                                            inventory.get_inner_qty_on_mas(),
                                            volume,
                                            master_volume,
                                        ));
                                        data.set_disposal_master_qty(calculate_master_qty(
                                            disposal_qty_value,
                                            inventory.get_inner_qty_on_mas(),
                                        ));
                                        *disposal_qty -= disposal_onhand_qty;

                                        let onhand_qty =
                                            data.get_allocated_qty() + disposal_onhand_qty;
                                        data.set_allocated_qty(onhand_qty);
                                        data.set_allocated_cbm(calculate_cbm(
                                            onhand_qty,
                                            inventory.get_inner_qty_on_mas(),
                                            volume,
                                            master_volume,
                                        ));
                                        data.set_allocated_master_qty(calculate_master_qty(
                                            onhand_qty,
                                            inventory.get_inner_qty_on_mas(),
                                        ));
                                    }
                                }
                            } else {
                                let onhand_qty = data.get_allocated_qty() + history.get_quantity();
                                data.set_allocated_qty(onhand_qty);
                                data.set_allocated_cbm(calculate_cbm(
                                    onhand_qty,
                                    inventory.get_inner_qty_on_mas(),
                                    volume,
                                    master_volume,
                                ));
                                data.set_allocated_master_qty(calculate_master_qty(
                                    onhand_qty,
                                    inventory.get_inner_qty_on_mas(),
                                ));
                            }

                            let key = get_on_hand_key(
                                date.clone(),
                                history.get_goods_issue_id().to_string(),
                            );
                            *on_hand_map.entry(key).or_insert(0) += history.get_quantity();
                            *total_disposal_qty_data -= history.get_quantity();
                            *total_allocated_qty
                                .entry(history.get_goods_issue_id().to_string())
                                .or_insert(0) += history.get_quantity();
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    let timestamp = to_date.map_or_else(now_to_epoch, |td| {
        let now = now_to_epoch();
        if td as u64 > now {
            now
        } else {
            td as u64
        }
    });
    let now = epoch_to_human_gmt7(timestamp);

    if merged_inventory_history.contains_key(&now) {
        let (data, _) = merged_inventory_history.get_mut(&now).unwrap();
        data.set_storage_time_days(
            days_between(
                from_date.map_or_else(
                    || goods_receipt.get_imported_at() as u64,
                    |fd| {
                        if fd < goods_receipt.get_imported_at() {
                            goods_receipt.get_imported_at() as u64
                        } else {
                            fd as u64
                        }
                    },
                ),
                Some(timestamp),
            ) as i32
                + 1,
        );
    } else if last_stock_qty > 0 {
        let mut data = InventoryData::new();
        data.set_opening_stock(last_stock_qty);
        data.set_opening_cbm(calculate_cbm(
            last_stock_qty,
            inventory.get_inner_qty_on_mas(),
            volume,
            master_volume,
        ));
        data.set_opening_master_qty(calculate_master_qty(
            last_stock_qty,
            inventory.get_inner_qty_on_mas(),
        ));
        data.set_asin(inventory.get_asin().to_string());
        data.set_asin_outbound(asin_outbound_list.to_vec());
        data.set_unit_price(inventory.get_unit_price());
        data.set_received_date(goods_receipt.get_imported_at());
        data.set_po_no(inventory.get_po_no().to_string());
        data.set_inner_qty_on_mas(inventory.get_inner_qty_on_mas());
        data.set_date(timestamp as i32);
        data.set_line_in_cd(inventory.get_index_customs_declaration().to_string());
        data.set_master_dimension(inventory.get_master_dimension().clone());
        data.set_dimension(inventory.get_dimension().clone());
        data.set_storage_time_days(
            days_between(
                from_date.map_or_else(
                    || goods_receipt.get_imported_at() as u64,
                    |fd| {
                        if fd < goods_receipt.get_imported_at() {
                            goods_receipt.get_imported_at() as u64
                        } else {
                            fd as u64
                        }
                    },
                ),
                Some(timestamp),
            ) as i32
                + 1,
        );
        merged_inventory_history.insert(now, (data, HashSet::new()));
    }

    let mut total_duration = 0;
    for (_, (inventory_data, _)) in &merged_inventory_history {
        if let (Some(from), Some(to)) = (from_date, to_date) {
            if inventory_data.get_date() < from || inventory_data.get_date() > to {
                continue;
            }
        }
        total_duration += inventory_data.get_storage_time_days();
    }

    ProcessInventoryHistoryOutput {
        merged_inventory_history,
        total_duration,
    }
}
