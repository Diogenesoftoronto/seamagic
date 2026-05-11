# WebAssembly Support

The browser demo currently uses the Canvas 2D API for client-side image processing. 
WASM integration is planned but requires significant refactoring to feature-gate 
tokio, reqwest, steel-core, and other WASM-incompatible dependencies.

## Architecture Plan

For a proper WASM build, the image operations (`src/operations/*`) would be 
extracted into a standalone crate with no async/threading dependencies. The 
`image` crate itself works on WASM, but `rayon` (parallel processing) would 
need to be disabled or replaced with sequential loops for the wasm target.

Key modules needed for WASM:
- `operations/filters.rs` — Blur, brightness, contrast, tint, etc.
- `operations/resize.rs` — Resize, thumbnail
- `operations/crop.rs` — Crop operations
- `operations/shapes.rs` — Draw rectangles, circles
- `operations/text.rs` — Text overlay (may need font loading changes)
- `operations/generation.rs` — Gradient, noise, solid canvas
- `operations/composite.rs` — Overlay, blend modes

Modules to exclude:
- `mcp/` — Requires tokio, rmcp (async MCP protocol)
- `steel_engine/` — Requires steel-core (Scheme VM)
- `script/` — Script execution engine
- `main.rs` — CLI depends on clap, tokio runtime

## Build Strategy

Option A: Feature flags in root crate
```toml
[features]
default = ["cli"]
cli = ["tokio", "clap", "reqwest", "rmcp", "steel-core"]
wasm = ["wasm-bindgen"]
```

Option B: Separate `wasm` workspace crate (recommended)
```
seamagic/
  Cargo.toml      # Main CLI/lib crate
  wasm/
    Cargo.toml    # WASM-specific crate
    src/lib.rs
```

The separate crate is cleaner because it avoids conditional compilation 
complexity across the entire codebase.
