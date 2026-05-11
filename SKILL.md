# mcp-image-design Skill

## Building

```bash
cd /home/diogenes/Projects/mcp-image-design
cargo build --release
```

Binary: `target/release/seamagic`

## MCP Server Tools

The binary exposes five MCP tools when run as a stdio server:

1. **`view_diff`** — Compare images. Modes: `side_by_side`, `grid`, `pixel_diff`. Provides result + compressed preview.
2. **`run_steel`** — Execute a Steel Scheme script for complex multi-step editing. Pass `inputImage` as base64, access as `"input"` in Scheme, assign final to `result`.
3. **`edit_image`** — Apply a single operation directly (blur, brightness, contrast, grayscale, tint, vignette, duotone, crop, resize, rotate, flip, draw text/shapes, noise, drop shadow). Faster than Scheme for one-shot edits.
4. **`create_image`** — Generate images from scratch: solid canvas, linear gradient, or framed canvas. No input image needed.
5. **`get_info`** — Get dimensions and format metadata from a base64 image.

Example Steel script:
```scheme
(define base "input")
(define blurred (image-blur base 2.0))
(define result (image-vignette blurred 0.5 1.0 #f))
```

## Running Example Scripts on Images

An example runner is available at `examples/run_example.rs`. It supports two modes: `legacy` (line-based scripts) and `scheme` (Steel Scheme DSL).

```bash
# Legacy script mode (uses .script files)
cargo run --example run_example -- legacy <input_image> <script_file> <output_image>

# Steel Scheme mode (uses .scm files)
cargo run --example run_example -- scheme <input_image> <script_file> <output_image>
```

**Example:**
```bash
cargo run --example run_example -- scheme \
  /home/diogenes/Pictures/Personal/dio.jpg \
  examples/cyberpunk_glitch.scm \
  output.png
```

Original images remain untouched. Output is written as a new PNG.

## Script Formats

### Legacy Scripts (.script)

Line-based pipeline, one operation per line. Lines starting with `#` are comments.

```
crop_center width=500 height=500
blur sigma=0.5
shape_mask shape=circle x=250 y=250 width=500 height=500 feather=true
```

**Known pitfall:** The parser previously stripped `#` anywhere in a line, breaking hex colors like `color=#667eea`. This was fixed — now it only strips comments preceded by a space.

### Steel Scheme Scripts (.scm)

Functional DSL using Steel Scheme. The input image is registered under the string `"input"`.

```scheme
(define base "input")
(define wavy (image-warp base "wave" 15.0 0.08))
(define result (image-posterize wavy 6))
```

**Critical rules:**

- The input image ID must be the **string** `"input"`, not a bare symbol `input`. Steel does not auto-bind a variable named `input`.
- `image-gradient` requires the angle parameter as a **float** (e.g., `135.0` not `135`). Integer angles cause a `ConversionError`.
- The final image must be assigned to a variable named `result` or `output`.
- Generator functions (`image-canvas`, `image-gradient`, `image-frame`) do not require an input image.

## Available Operations

### Generators (no input needed)
- `image-canvas width height color`
- `image-gradient width height start_color end_color angle` — angle must be float
- `image-frame width height frame_width color`

### Transform
- `image-crop`, `image-crop-center`, `image-smart-crop`, `image-resize`, `image-thumbnail`, `image-rotate`, `image-flip`

### Filters
- `image-blur`, `image-brightness`, `image-contrast`, `image-grayscale`, `image-tint`, `image-vignette`, `image-duotone`

### Distortions
- `image-liquify`, `image-warp`, `image-glitch`, `image-chromatic-aberration`, `image-pixel-sort`, `image-scanlines`, `image-halftone`, `image-posterize`, `image-noise`, `image-kaleidoscope`, `image-emboss`, `image-edge-detect`, `image-solarize`

### Draw
- `image-draw-text`, `image-draw-rectangle`, `image-draw-circle`, `image-draw-line`

### Compose
- `image-overlay`, `image-apply-mask`, `image-drop-shadow`

### Utility
- `image-dimensions` → returns `[width height]`
- `image-comparison`, `image-steps-grid`, `image-diff`

## Example Pipelines

**Cyberpunk Glitch Portrait:**
```scheme
(define base "input")
(define wavy (image-warp base "wave" 15.0 0.08))
(define glitched (image-glitch wavy "rgb_split" 0.7 42))
(define chromatic (image-chromatic-aberration glitched 8.0 #t))
(define scan (image-scanlines chromatic 3 0.25 1))
(define result (image-posterize scan 6))
```

**Retro Halftone Logo:**
```scheme
(define logo "input")
(define duotoned (image-duotone logo "#1a1a2e" "#e94560"))
(define half (image-halftone duotoned 6 45.0 "color"))
(define result (image-vignette half 0.4 1.2 #f))
```

**Poster from Scratch:**
```scheme
(define bg (image-gradient 1080 1350 "#0f0c29" "#302b63" 135.0))
(define v (image-vignette bg 0.6 1.0 #f))
(define half (image-halftone v 10 30.0 "monochrome"))
(define noise-bg (image-noise half 0.05 "grain" 1))
(define text-bg (image-draw-rectangle noise-bg 80 500 920 140 "#16213e" #t))
(define result (image-draw-text text-bg "POSTER DESIGN" 200 560 "#e94560" 72))
```
