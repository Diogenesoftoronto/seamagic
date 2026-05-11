# seamagic

A Rust image design toolkit with both a CLI and an MCP (Model Context Protocol) server. Think of it as giving your AI — or yourself — a Canva-like toolkit for image manipulation.

> "The computer programmer is a creator of universes for which he alone is the lawgiver."
> — Joseph Weizenbaum

## Installation

```bash
cargo install --git https://github.com/diogenesoftoronto/seamagic
```

Or build from source:

```bash
git clone https://github.com/diogenesoftoronto/seamagic
cd seamagic
cargo build --release
```

Binary: `target/release/seamagic`

## Getting Started

### As a CLI Tool

Edit a single photo with one command:

```bash
seamagic blur photo.png -o blurred.png --sigma 3.0
seamagic vignette photo.png -o dark-edges.png --strength 0.6
seamagic draw-text photo.png -o captioned.png --text "Hello" --color "#ff0000" --size 48
```

Generate images from nothing:

```bash
seamagic canvas -o black.png --width 1080 --height 1080 --color "#000000"
seamagic gradient -o sunset.png --width 1920 --height 1080 --start "#ff7e5f" --end "#feb47b" --angle 135
```

Chain operations with a Scheme script (Steel DSL):

```bash
cargo run --example run_example -- scheme \
  photo.jpg examples/cyberpunk_glitch.scm output.png
```

### As an MCP Server

```bash
# Start the server
seamagic

# Or explicitly
seamagic mcp
```

Then use any MCP client (Claude, Cursor, etc.):

```json
{
  "mcpServers": {
    "seamagic": {
      "command": "/path/to/seamagic"
    }
  }
}
```

### With tcli (Terminal CLI for AI MCP)

```bash
# Get image dimensions
tcli seamagic get_info --image "$(base64 -w0 photo.png)"

# Apply a quick blur
tcli seamagic edit_image \
  --image "$(base64 -w0 photo.png)" \
  --operation blur \
  --sigma 2.5 \
  --output_file blurred.png

# Create a canvas
tcli seamagic create_image \
  --type canvas \
  --width 1080 --height 1080 \
  --color "#1a1a2e" \
  --output_file bg.png

# Run a full Scheme pipeline
tcli seamagic run_steel \
  --inputImage "$(base64 -w0 portrait.jpg)" \
  --script '((define b "input")
    (define w (image-warp b "wave" 15.0 0.08))
    (define g (image-glitch w "rgb_split" 0.7 42))
    (define c (image-chromatic-aberration g 8.0 #t))
    (define s (image-scanlines c 3 0.25 1))
    (define result (image-posterize s 6)))' \
  --output_file cyberpunk.png

# Compare two images side-by-side
tcli seamagic view_diff \
  --mode side_by_side \
  --originalImage "$(base64 -w0 original.png)" \
  --editedImage "$(base64 -w0 edited.png)" \
  --output_file comparison.png
```

## CLI Usage

### Quick Examples

```bash
# Get image info
seamagic info photo.png

# Resize
seamagic resize photo.png -o thumb.png --width 300 --height 300 --mode fit

# Apply filters
seamagic grayscale photo.png -o bw.png
seamagic blur photo.png -o blurred.png --sigma 5.0
seamagic brightness photo.png -o bright.png --value 30

# Creative effects
seamagic glitch photo.png -o glitched.png
seamagic kaleidoscope photo.png -o trippy.png --segments 8
seamagic liquify photo.png -o melted.png --mode bulge
seamagic scanlines photo.png -o retro.png

# Draw shapes and text
seamagic canvas -o bg.png --width 1080 --height 1080 --color "#1a1a2e"
seamagic gradient -o grad.png --width 1080 --height 1080 --start "#667eea" --end "#764ba2"
seamagic draw-text bg.png -o labeled.png --text "HELLO" --x 200 --y 200 --color "#e94560" --size 64

# Overlay images
seamagic overlay base.png top.png -o composite.png --x 100 --y 100 --blend-mode multiply

# Pipeline from JSON config
seamagic pipeline photo.png -o out.png --config effects.json
```

### Full Command List

| Command | Description |
|---|---|
| `info` | Get image dimensions |
| `crop` | Crop to region |
| `crop-center` | Center crop |
| `smart-crop` | Aspect-ratio aware crop |
| `resize` | Resize (exact/fit/fill/scale) |
| `thumbnail` | Create thumbnail |
| `rotate` | Rotate by degrees |
| `flip` | Horizontal/vertical flip |
| `blur` | Gaussian blur |
| `brightness` | Adjust brightness |
| `contrast` | Adjust contrast |
| `grayscale` | Convert to grayscale |
| `tint` | Color tint overlay |
| `vignette` | Darken edges |
| `duotone` | Two-color effect |
| `noise` | Add grain/color/monochrome noise |
| `liquify` | Bulge/pinch/swirl distortion |
| `warp` | Wave/fisheye/spherical warp |
| `glitch` | RGB split/slice shift/datamosh |
| `scanlines` | CRT-style scanlines |
| `halftone` | Dot-based newspaper effect |
| `posterize` | Reduce color levels |
| `edge-detect` | Sobel edge detection |
| `emboss` | Emboss filter |
| `solarize` | Invert above threshold |
| `chromatic` | Chromatic aberration |
| `pixel-sort` | Sort pixels by brightness |
| `kaleidoscope` | Mirror into sectors |
| `overlay` | Composite with blend modes |
| `draw-text` | Overlay pixel text |
| `draw-rect` | Draw rectangle |
| `draw-circle` | Draw circle |
| `canvas` | Create solid color image |
| `gradient` | Create gradient image |
| `mask` | Apply shape mask |
| `pipeline` | Chain ops from JSON config |
| `mcp` | Start MCP stdio server |

### Pipeline Config Format

Create a JSON file with an array of operations:

```json
[
  { "op": "grayscale" },
  { "op": "blur", "sigma": 2.0 },
  { "op": "vignette", "strength": 0.6, "radius": 1.2 },
  { "op": "text", "text": "SEAMAGIC", "x": 100, "y": 100, "color": "#ff0000", "fontSize": 48 }
]
```

```bash
seamagic pipeline input.png -o output.png --config pipeline.json
```

## As an MCP Server

When invoked without arguments, `seamagic` runs as an MCP stdio server with five tools:

### `view_diff`
Compare images visually. Modes: `side_by_side`, `grid`, `pixel_diff`.

### `run_steel`
Execute a Steel Scheme script for complex multi-step image editing. Input via base64, all operations exposed.

### `edit_image`
Apply a single operation directly: blur, brightness, contrast, grayscale, tint, vignette, duotone, crop, resize, rotate, flip, draw text/shapes, noise, drop shadow. Faster than Scheme for one-shot edits.

### `create_image`
Generate images from scratch: solid canvas, linear gradient, or framed canvas. No input needed.

### `get_info`
Get dimensions and basic metadata from a base64 image.

### Config

```json
{
  "mcpServers": {
    "seamagic": {
      "command": "/path/to/seamagic"
    }
  }
}
```

## Steel Scheme Scripting

Chain operations using embedded [Steel Scheme](https://github.com/mattwparas/steel). The input image is bound as `"input"`; assign the final result to `result`:

```scheme
(define bg (image-canvas 1080 1080 "#1a1a2e"))
(define grad (image-gradient 1080 1080 "#667eea" "#764ba2" 135.0))
(define v (image-vignette grad 0.3 1.0 #f))
(define result (image-draw-text v "HELLO WORLD" 200 445 "#e94560" 72))
```

## Capabilities

- **Transforms**: crop, resize, rotate, flip
- **Filters**: blur, brightness, contrast, grayscale, tint, vignette, duotone
- **Distortions**: liquify, warp, glitch, chromatic aberration, pixel sort, scanlines, halftone, posterize, noise, kaleidoscope, emboss, edge detect, solarize
- **Drawing**: text (ab_glyph with system fonts), rectangles, circles, lines, gradients, canvas
- **Compositing**: overlay with blend modes, drop shadows, shape masks
- **AI**: SAM-2 segmentation, background removal, fal.ai image editing
- **Parallel**: Pixel-level filters (tint, duotone, vignette) use rayon for multi-threading

## License

MIT

## Author

Dio the Debugger — me@dio.computer
