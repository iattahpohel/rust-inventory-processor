# Hướng dẫn build Rust → WASM và import vào TypeScript

## Tổng quan

Rust `process-inventory-list.rs` compile sang **WebAssembly (WASM)** và gọi từ TypeScript qua `InventoryHistoryProcessorWasmFn` trong registry.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  TypeScript                                                                  │
│  getInventoryHistoryProcessor() → processInventoryHistoryData (TS hoặc WASM)│
│       ↓ setInventoryHistoryProcessorWasm(wasmFn)                             │
│  toSerializableInput(input) → WASM process_inventory_history_wasm() → result │
│       ↓ fromSerializableResult(serialized)                                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Types khớp với** `inventory-history-processor.types.ts`:
- `HandleInventoryListInputSerializable` (input)
- `ProcessedInventoryHistoryResultSerializable` (output)

---

## Bước 1: Cấu hình Rust cho WASM

### 1.1 Cập nhật `Cargo.toml`

```toml
[package]
name = "rust-inventory-processor"   # tên package → pkg/rust_inventory_processor.js
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]
# Bỏ path nếu dùng src/lib.rs (mặc định)

[features]
default = []
wasm = ["wasm-bindgen", "serde", "serde_json"]

[dependencies]
chrono = { version = "0.4", default-features = false, features = ["serde"] }
wasm-bindgen = { version = "0.2", optional = true }
serde = { version = "1.0", features = ["derive"], optional = true }
serde_json = { version = "1.0", optional = true }

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
```

### 1.2 Cấu trúc thư mục

Chuyển sang cấu trúc `src/` cho wasm-pack:

```
rust-implement/
├── Cargo.toml
├── src/
│   ├── lib.rs              # WASM entry + bindings
│   └── process_inventory_list.rs   # copy/rename từ process-inventory-list.rs
└── pkg/                    # output sau wasm-pack build
```

### 1.3 Tạo WASM wrapper `src/lib.rs`

Tách logic: `process_inventory_list.rs` giữ logic thuần, `lib.rs` làm WASM entry:

```rust
// src/lib.rs
mod process_inventory_list;  // hoặc include!

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandleInventoryListInputSerializable {
    pub inventory: serde_json::Value,
    pub inventory_list: Vec<serde_json::Value>,
    pub asin_outbound_list: Vec<String>,
    pub goods: serde_json::Value,
    pub goods_receipt: serde_json::Value,
    pub supplier: serde_json::Value,
    pub customer: serde_json::Value,
    pub inventory_history_list: Vec<InventoryHistoryItem>,
    pub receipt_orders: Vec<serde_json::Value>,
    pub inventory_id: String,
    pub inventory_ids_map: std::collections::HashMap<String, i32>,
    pub from_date: Option<i32>,
    pub to_date: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryHistoryItem {
    pub id: String,
    pub created_at: i32,
    pub stock_qty: i32,
    pub old_status: i32,
    pub new_status: i32,
    pub quantity: i32,
    pub goods_issue_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedInventoryHistoryResultSerializable {
    pub entries: std::collections::HashMap<String, ProcessedEntry>,
    pub total_duration: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedEntry {
    pub data: serde_json::Value,
    pub goods_issue_ids: Vec<String>,
}

#[wasm_bindgen]
pub fn process_inventory_history_wasm(input_json: &str) -> String {
    let input: HandleInventoryListInputSerializable = serde_json::from_str(input_json).unwrap();
    // Convert to ProcessInventoryHistoryInput, call process_inventory_history_data
    // Convert output to ProcessedInventoryHistoryResultSerializable
    let result = process_inventory_list::process_inventory_history_data(...);
    serde_json::to_string(&result).unwrap()
}
```

---

## Bước 2: Build WASM

```bash
# Cài wasm-pack
cargo install wasm-pack

# Thêm target wasm32
rustup target add wasm32-unknown-unknown

# Build (Node.js backend dùng nodejs, browser dùng web)
cd rust-implement
wasm-pack build --target nodejs --features wasm
# hoặc: wasm-pack build --target web --features wasm
```

Output: `pkg/` với `rust_inventory_processor.js`, `rust_inventory_processor_bg.wasm`, `rust_inventory_processor.d.ts`

**Sau khi build**, copy pkg vào proship-backend-v2-typescript và cài dependency:
```bash
cd proship-backend-v2-typescript
npm install
# (package.json đã có "rust-inventory-processor": "file:../rust-implement/pkg")
```

### 2.1 Publish lên npm

```bash
# 1. Đăng nhập npm (nếu chưa)
npm login

# 2. Dùng scope nếu publish package scoped (@username/package)
wasm-pack build --target nodejs --features wasm --scope YOUR_NPM_USERNAME

# 3. Publish
wasm-pack publish --access public
# hoặc không scope: wasm-pack publish
```

**Lưu ý:**
- Thêm `repository = "https://github.com/..."` vào Cargo.toml trước khi publish (tùy chọn)
- Package scoped (@scope/name) cần `--access public` để public
- Có thể dùng `--tag next` cho pre-release

---

## Bước 3: Import vào TypeScript

### 3.1 Thêm package vào proship-backend-v2-typescript

```bash
# Copy pkg
cp -r rust-implement/pkg proship-backend-v2-typescript/libs/rust-inventory-processor

# Hoặc trong package.json:
"rust-inventory-processor": "file:libs/rust-inventory-processor"
```

### 3.2 Tạo `inventory-history-processor.wasm-loader.ts`

```typescript
import {
  setInventoryHistoryProcessorWasm,
  type InventoryHistoryProcessorWasmFn,
  type ProcessedInventoryHistoryResultSerializable,
} from './inventory-history-processor.registry'

export async function loadRustInventoryProcessor(): Promise<boolean> {
  try {
    const wasm = await import('rust-inventory-processor')
    await wasm.default() // init WASM memory

    const wasmProcessor: InventoryHistoryProcessorWasmFn = (input) => {
      const inputJson = JSON.stringify(input)
      const resultJson = wasm.process_inventory_history_wasm(inputJson)
      return JSON.parse(resultJson) as ProcessedInventoryHistoryResultSerializable
    }
    setInventoryHistoryProcessorWasm(wasmProcessor)
    return true
  } catch (e) {
    console.warn('Rust WASM processor load failed, using TS fallback', e)
    return false
  }
}
```

### 3.3 Gọi khi bootstrap

```typescript
// inventory.module.ts hoặc main.ts
import { loadRustInventoryProcessor } from './processor/inventory-history-processor.wasm-loader'

@Module({...})
export class InventoryModule implements OnModuleInit {
  async onModuleInit() {
    await loadRustInventoryProcessor()
  }
}
```

---

## Bước 4: Định dạng Serializable (khớp TypeScript)

**Input** `HandleInventoryListInputSerializable` (xem `inventory-history-processor.types.ts`):
```typescript
{
  inventory: Record<string, unknown>,
  inventoryList: Array<{ asinOutbound?: string }>,
  asinOutboundList: string[],
  goods: Record<string, unknown>,
  goodsReceipt: Record<string, unknown>,
  supplier: Record<string, unknown>,
  customer: Record<string, unknown>,
  inventoryHistoryList: Array<{
    id: string, createdAt: number, stockQty: number,
    oldStatus: number, newStatus: number, quantity: number,
    goodsIssueId?: string
  }>,
  receiptOrders: Record<string, unknown>[],
  inventoryId: string,
  inventoryIdsMap: Record<string, number>,
  fromDate: number | null,
  toDate: number | null
}
```

**Output** `ProcessedInventoryHistoryResultSerializable`:
```typescript
{
  entries: Record<string, {
    data: Record<string, unknown>,  // InventoryData - dùng InventoryData.fromJSON()
    goodsIssueIds: string[]
  }>,
  totalDuration: number
}
```

---

## Lưu ý

1. **Date format**: `epoch_to_human_gmt7` trả về `%Y%m%d` (vd: `20240115`) – phải khớp với `util.epochToHumanGmt7` trong TS.
2. **Enum status**: `oldStatus`/`newStatus` là number (0–7), map với `EnumProshipType_InventoryStatus`.
3. **Fallback**: Nếu WASM load fail, `getInventoryHistoryProcessor()` trả về `processInventoryHistoryData` (TS).
4. **Chrono trên WASM**: Chrono 0.4 chạy được trên WASM; tránh `std::time::SystemTime::now()` trong WASM, dùng `js_sys::Date` hoặc truyền từ JS.
