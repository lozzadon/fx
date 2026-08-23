# Project: Topia — Declarative Desktop UI Framework for f(x)

## Architecture
- **Standalone Library (`/home/luq/topia`)**: High-performance, declarative GUI library written in Rust using `egui` and `eframe` (v0.36.1). Defines `Node` (Text, Button, VStack, HStack, Empty), `App` configuration, UI rendering traversal, and `eframe::run_native` event loop with `view_builder: FnMut() -> Node` for immediate-mode reactive state re-rendering. Verified with 52 comprehensive unit, adversarial, and stress tests.
- **Language Integration (`/home/luq/fx`)**: `fx` standard library module `topia` (`src/stdlib/topia.rs`) that exports `App`, `VStack`, `HStack`, `Text`, `Button`, and `run` to `f(x)` scripts via `import("topia")`.
- **Dual-Engine Bridge**: Translates `f(x)` `Object` structures into `topia::Node` and wraps `f(x)` closure callbacks (`Object::Function`) to execute against the active `Rc<RefCell<Environment>>` via `apply_function`, operating with full parity in both the AST Evaluator and Bytecode VM. Verified with 130 unit, integration, differential, adversarial, and stress tests.
- **Demo & E2E Validation**: `/home/luq/fx/examples/topia_demo.fx` implements a full declarative counter demo with native interactive desktop window launching across both engines.

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | Standalone `topia` Crate | Cargo.toml configuration with `egui` and `eframe` dependencies | M1 | R1 |
| 2 | Declarative UI Tree (`Node`) | `Text`, `Button`, `VStack`, `HStack`, and `Empty` nodes | M1 | R1, R2 |
| 3 | Egui UI Rendering Traversal | `Node::render(&mut self, ui: &mut egui::Ui)` layout rendering | M1 | R1 |
| 4 | Window Lifecycle & Runner | `App` configuration and `eframe::run_native` runner with `view_builder` | M1 | R1, R2 |
| 5 | Topia Headless Test Harness | Headless verification of widget tree structure and callbacks | M1 | Survey |
| 6 | `fx` Cargo Dependency | Add `topia = { path = "../topia" }` to `/home/luq/fx/Cargo.toml` | M2 | R2 |
| 7 | Standard Library Registration | `import("topia")` and `import("std:topia")` in `stdlib::load_std_module` | M2 | R2 |
| 8 | Declarative MVP Constructors | `App(title, w, h)`, `VStack(children)`, `HStack(children)`, `Text(str)`, `Button(label, cb)` | M2 | R2 |
| 9 | Object to Node Converter | Bridge translating `f(x)` `Object` hierarchy into `topia::Node` | M2 | R2 |
| 10 | AST Evaluator Callback Execution | Invoking `f(x)` closures (`Object::Function`) on button click via `apply_function` | M2 | R2, R3 |
| 11 | Bytecode VM Function Literal Support | Support compiling/evaluating function literals & closures in VM mode | M3 | R3 |
| 12 | Bytecode VM Topia Parity | Seamless execution of `import("topia")` and callback dispatch in VM engine | M3 | R3 |
| 13 | Topia Counter Demo Script | Create `/home/luq/fx/examples/topia_demo.fx` with increment/decrement counter | M4 | Acceptance |
| 14 | Dual-Engine Interactive Verification | Verify `cargo run -- examples/topia_demo.fx` (AST & VM) | M4 | Acceptance |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | M1: Standalone Topia Crate | Implement `/home/luq/topia` with `egui`/`eframe`, `Node`, `App`, and headless tests | none | DONE |
| 2 | M2: f(x) Integration & AST Engine | Add dependency, `src/stdlib/topia.rs`, `import("topia")`, AST callback bridge | M1 | DONE |
| 3 | M3: Bytecode VM Parity | Compiler & VM parity for function closures and Topia UI execution | M2 | DONE |
| 4 | M4: Topia Demo & E2E Validation | Create `examples/topia_demo.fx`, end-to-end multi-tier testing and verification | M3 | DONE |

## Interface Contracts
### `topia` Crate (`/home/luq/topia`)
```rust
pub enum Node {
    Text { text: String },
    Button { label: String, on_click: Box<dyn FnMut() + 'static> },
    VStack { children: Vec<Node>, spacing: Option<f32> },
    HStack { children: Vec<Node>, spacing: Option<f32> },
    Empty,
}

pub struct App {
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub resizable: bool,
}

impl App {
    pub fn new(title: impl Into<String>, width: f32, height: f32) -> Self;
    pub fn run<F>(self, view_builder: F) -> Result<(), String>
    where
        F: FnMut() -> Node + 'static;
}
```

### `f(x)` Topia Stdlib Module (`import("topia")`)
- `topia.App(title: string, width: int|float, height: int|float) -> Hash`
- `topia.VStack(children: array) -> Hash`
- `topia.HStack(children: array) -> Hash`
- `topia.Text(content: string) -> Hash`
- `topia.Button(label: string, callback: function) -> Hash`
- `app.run(view_builder: function) -> Null` or `topia.run(app, view_builder) -> Null`

## Code Layout
- `/home/luq/topia/Cargo.toml`
- `/home/luq/topia/src/lib.rs`
- `/home/luq/topia/src/node.rs`
- `/home/luq/topia/src/app.rs`
- `/home/luq/topia/tests/`
- `/home/luq/fx/Cargo.toml`
- `/home/luq/fx/src/stdlib/mod.rs`
- `/home/luq/fx/src/stdlib/topia.rs`
- `/home/luq/fx/src/evaluator.rs`
- `/home/luq/fx/src/compiler.rs`
- `/home/luq/fx/src/vm.rs`
- `/home/luq/fx/examples/topia_demo.fx`
