# E2E Test Infra: Topia Framework for f(x)

## Test Philosophy
- Opaque-box, requirement-driven. No dependency on implementation design.
- Methodology: Category-Partition + BVA + Pairwise + Workload Testing.
- Dual-Engine verification: every test must execute in both AST Evaluator (`fx script.fx`) and Bytecode VM (`fx --vm script.fx`).

## Feature Inventory
| # | Feature | Source (requirement) | Tier 1 | Tier 2 | Tier 3 |
|---|---------|---------------------|:------:|:------:|:------:|
| 1 | `import("topia")` module resolution | ORIGINAL_REQUEST §R2 | 5 | 5 | ✓ |
| 2 | `topia.App(title, width, height)` | ORIGINAL_REQUEST §R2 | 5 | 5 | ✓ |
| 3 | `topia.Text(string)` | ORIGINAL_REQUEST §R2 | 5 | 5 | ✓ |
| 4 | `topia.VStack(children)` | ORIGINAL_REQUEST §R2 | 5 | 5 | ✓ |
| 5 | `topia.HStack(children)` | ORIGINAL_REQUEST §R2 | 5 | 5 | ✓ |
| 6 | `topia.Button(label, callback)` | ORIGINAL_REQUEST §R2 | 5 | 5 | ✓ |
| 7 | Closure state mutation in callbacks | ORIGINAL_REQUEST §R2, §R3 | 5 | 5 | ✓ |
| 8 | Dual-Engine AST & VM Parity | ORIGINAL_REQUEST §R3 | 5 | 5 | ✓ |

## Test Architecture
- Test runner: `cargo test` in `/home/luq/topia` and `cargo test` in `/home/luq/fx`, plus headless E2E verification test suite.
- Format: Native Rust unit/integration tests and `.fx` test scripts executed through the interpreter & VM harnesses.
- Pass/fail semantics: Exit code 0, 0 test failures, 0 compilation warnings/errors.

## Real-World Application Scenarios (Tier 4)
| # | Scenario | Features Exercised | Complexity |
|---|----------|--------------------|------------|
| 1 | Counter with Increment/Decrement/Reset | App, VStack, HStack, Text, Button, Closures | Medium |
| 2 | Dynamic String Concatenation & Multi-Text Layout | App, VStack, Text, String expressions | Medium |
| 3 | Multi-Button Form with State Updates | App, VStack, HStack, Button, Variables | Medium |
| 4 | Nested Layout Hierarchies (VStack of HStacks) | App, Nested VStack/HStack, Text, Button | High |
| 5 | Dual-Engine Parity Benchmark on Counter Demo | All features in AST vs VM mode | High |

## Coverage Thresholds
- Tier 1: ≥5 per feature (≥40 tests)
- Tier 2: ≥5 per feature (≥40 tests)
- Tier 3: Pairwise coverage of major feature interactions
- Tier 4: ≥5 realistic application scenarios
