# rust-inventory-processor-native

Native Node.js addon for [rust-inventory-processor](https://github.com/iattahpohel/rust-inventory-processor). Uses napi-rs for near-native performance.

## Install

```bash
npm install rust-inventory-processor-native
```

## Usage

```typescript
import { processInventoryHistory } from 'rust-inventory-processor-native'

const resultJson = processInventoryHistory(inputJson)
const result = JSON.parse(resultJson)
```

## Supported platforms

Prebuilt binaries: darwin-arm64 (Apple Silicon). Other platforms require building from source.

## License

MIT
