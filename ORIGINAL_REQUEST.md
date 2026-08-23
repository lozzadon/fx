# Original User Request

## 2026-08-23T17:47:58Z

# Teamwork Project Prompt — Draft

> Status: Ready for launch — awaiting user approval
> Goal: Craft prompt → get user approval → delegate to teamwork_preview
> Requested team: A small, focused team (best for implementing 1-2 contained features).

This is a single self-contained feature implementation; keep it small and focused.
Implement Proposal 3, Proposal 5, and Proposal 6 from the `docs/FEATURE_PROPOSALS.md` specification into the `f(x)` codebase.

Working directory: /home/luq/fx
Integrity mode: development

## Verification Resources
- The detailed architectural specifications for these features are documented in `/home/luq/fx/docs/FEATURE_PROPOSALS.md`.
- The Rust unit testing framework is set up; `cargo test` can be used to verify parsing and evaluation.

## Requirements

### R1. Container Element Mutation & Shared References (Proposal 3)
Refactor the Object model to use `Rc<RefCell<...>>` for Arrays and Hashes. Implement L-value parsing for index assignments (e.g. `arr[i] = val`) and implement the `OpSetIndex` execution logic in the VM and Evaluator.

### R2. Struct Records & Field Access (Proposal 5)
Implement nominal struct declarations (`struct Point { x: Int }`), instantiation, and dot-notation field access and mutation (`p.x = 10`). This must be supported by both engines.

### R3. Standard Library Architecture (Proposal 6)
Implement a modular standard library architecture that allows users to import built-in namespaces (e.g., `let fs = import("std:fs")`).

## Acceptance Criteria

### Automated Verification
- [ ] `cargo test` runs and passes successfully with 0 failures, including new unit tests for the implemented features.
- [ ] `cargo check` executes cleanly with 0 compiler warnings.
- [ ] A file named `test_advanced_features.fx` is created in `/home/luq/fx/` demonstrating all 3 features (in-place array mutation, struct field assignment, and a std module call). Running `cargo run -- test_advanced_features.fx` must execute successfully.
