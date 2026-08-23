# Reviewer Progress & Verification Log

## Task: Proposals 3, 5, and 6 Implementation Review

### 1. Independent Review & Defect Identification
- **Defect 1: Missing VM Compilation & Execution for Dictionary Literals (`Expression::HashLiteral`)**
  - *Input*: `let d = {"a": 1}` executed via Bytecode VM.
  - *Expected*: Compiles to `OpHash` opcode and instantiates `Object::Hash` on VM stack.
  - *Actual*: Bytecode compiler halted with `Unsupported expression in VM compilation: HashLiteral(...)`.
  - *Root Cause*: `src/compiler.rs` had no compilation arm for `Expression::HashLiteral`, `src/code.rs` lacked `OpHash` (opcode 30), and `src/vm.rs` lacked hash instantiation logic.
  - *Fix*: Added `Opcode::OpHash` (30) to `code.rs`, compilation handler in `compiler.rs`, and VM execution in `vm.rs`.

- **Defect 2: Missing Cycle-Safe Recursion Guard in `std:json:stringify`**
  - *Input*: `let json = import("std:json"); var arr = [1]; arr[0] = arr; json.stringify(arr)`.
  - *Expected*: Serializes safely without stack overflow (emitting `"null"` or cycle marker).
  - *Actual*: Unbounded recursion causing host process stack overflow crash.
  - *Root Cause*: `stringify_json` in `src/stdlib/json.rs` lacked pointer cycle detection.
  - *Fix*: Added thread-local `VISITED_JSON_POINTERS` tracking to `stringify_json` across Arrays, Hashes, and Struct instances.

- **Defect 3: Incomplete UTF-16 Surrogate Pair Decoding in `std:json:parse`**
  - *Input*: `json.parse(r#"{"emoji": "\uD83D\uDE00"}"#)` (Unicode smiley `😀`).
  - *Expected*: Decodes UTF-16 surrogate pairs (`\uD83D\uDE00`) into valid UTF-8 scalar values (`😀`).
  - *Actual*: `char::from_u32(0xD83D)` returned `None`, discarding surrogate pairs and producing empty string.
  - *Root Cause*: `parse_string` in `json.rs` only decoded basic BMP scalar values.
  - *Fix*: Implemented RFC 8259 compliant UTF-16 surrogate pair detection and computation (`0x10000 + (((high - 0xD800) << 10) | (low - 0xDC00))`).

- **Defect 4: Inactive Sandboxing & Capability Controls in `std:fs` and `std:os`**
  - *Input*: Importing `std:fs` or `std:os` with `allow_fs = false` / `allow_os = false` or accessing files outside `fs_root`.
  - *Expected*: Returns unauthorized error when disabled, or permission error on directory traversal outside `fs_root`.
  - *Actual*: `FxConfig` was declared but not wired to module loading or filesystem path validation.
  - *Fix*: Added thread-local `CONFIG` in `src/stdlib/mod.rs` with `set_config` / `get_config`, checked `allow_fs`/`allow_os` in `load_std_module`, and added `validate_path` sandboxing and `max_file_size` limits in `fs.rs`.

### 2. Verification Summary
- `cargo check --all-targets`: Clean compilation with 0 warnings.
- `cargo test`: 47 tests passed with 0 failures (expanded test suite with 12 new adversarial tests).
- `cargo run -- test_advanced_features.fx`: All feature demonstrations run cleanly with expected output.
- `cargo run -- examples/test_features.fx` and `examples/showcase.fx`: Verified no regressions.
