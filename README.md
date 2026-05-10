# seamagic

A Rust image design toolkit with both a CLI and an MCP (Model Context Protocol) server. Think of it as giving your AI — or yourself — a Canva-like toolkit for image manipulation.

> "I made this because I needed tools that brought image."

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

For AI agent integration:

```json
{
  "mcpServers": {
    "seamagic": {
      "command": "/path/to/seamagic"
    }
  }
}
```

When invoked without arguments, `seamagic` runs as an MCP stdio server.

## Steel Scheme Scripting

Chain operations using embedded [Steel Scheme](https://github.com/mattwparas/steel):

```scheme
(define bg (image-canvas 1080 1080 "#1a1a2e"))
(define grad (image-gradient 1080 1080 "#667eea" "#764ba2" 135))
(define v (image-vignette grad 0.3 1.0 #f))
(define result (image-draw-text v "HELLO WORLD" 200 445 "#e94560" 72))
```

## Capabilities

- **Transforms**: crop, resize, rotate, flip
- **Filters**: blur, brightness, contrast, grayscale, tint, vignette, duotone
- **Distortions**: liquify, warp, glitch, chromatic aberration, pixel sort, scanlines, halftone, posterize, noise, kaleidoscope, emboss, edge detect, solarize
- **Drawing**: text, rectangles, circles, lines, polygons, gradients
- **Compositing**: overlay with blend modes (normal, multiply, screen, overlay)
- **Masking**: shape masks with feathering
- **AI Generation**: OpenAI DALL-E, fal.ai GPT Image 2 (via MCP tools)

## License

MIT

## Author

Dio the Debugger — me@dio.computer
