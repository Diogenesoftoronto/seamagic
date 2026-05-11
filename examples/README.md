# Seamagic Examples

This directory contains example scripts and pipelines demonstrating what seamagic can do. Files are organized by format: `.scm` for Steel Scheme (functional DSL), `.script` for legacy line-based pipelines.

> Run any `.scm` example:
> ```bash
> cargo run --example run_example -- scheme input.jpg examples/cyberpunk_glitch.scm output.png
> ```

---

## Creative Effects

### `cyberpunk_glitch.scm` — Cyberpunk Portrait
Wave warp → RGB glitch → chromatic aberration → scanlines → posterize

```scheme
(define base "input")
(define wavy (image-warp base "wave" 15.0 0.08))
(define glitched (image-glitch wavy "rgb_split" 0.7 42))
(define chromatic (image-chromatic-aberration glitched 8.0 #t))
(define scan (image-scanlines chromatic 3 0.25 1))
(define result (image-posterize scan 6))
```

With tcli:
```bash
tcli seamagic run_steel \
  --inputImage "$(base64 -w0 portrait.jpg)" \
  --script '((define b "input")
    (define w (image-warp b "wave" 15.0 0.08))
    (define g (image-glitch w "rgb_split" 0.7 42))
    (define c (image-chromatic-aberration g 8.0 #t))
    (define s (image-scanlines c 3 0.25 1))
    (define result (image-posterize s 6)))' \
  --output_file glitch.png
```

### `retro_halftone.scm` — Retro Newspaper Logo
Duotone → halftone dots → vignette

```scheme
(define logo "input")
(define duotoned (image-duotone logo "#1a1a2e" "#e94560"))
(define half (image-halftone duotoned 6 45.0 "color"))
(define result (image-vignette half 0.4 1.2 #f))
```

### `liquid_metal.scm` — Liquid Metal Texture
Metallic chrome effect using emboss + tint combinations.

### `edge_detect_xray.scm` — X-Ray Effect
Sobel edge detection with inverted output. Great for architectural photography.

### `kaleidoscope.scm` — Kaleidoscope Mandala
Mirror effect producing repeating geometric patterns.

### `fisheye_meme.scm` — Fisheye Meme
Spherical warp for classic meme distortion.

---

## Design from Scratch

### `poster_from_scratch.scm` — Full Poster Design
No input image needed. Builds a complete poster with gradient, halftone, noise, shapes, and text.

```scheme
(define bg (image-gradient 1080 1350 "#0f0c29" "#302b63" 135.0))
(define v (image-vignette bg 0.6 1.0 #f))
(define half (image-halftone v 10 30.0 "monochrome"))
(define noise-bg (image-noise half 0.05 "grain" 1))
(define text-bg (image-draw-rectangle noise-bg 80 500 920 140 "#16213e" #t))
(define result (image-draw-text text-bg "POSTER DESIGN" 200 560 "#e94560" 72))
```

With tcli (using create_image for canvas):
```bash
tcli seamagic create_image \
  --type gradient --width 1080 --height 1350 \
  --startColor "#0f0c29" --endColor "#302b63" --angle 135.0 \
  --output_file gradient.png

tcli seamagic run_steel \
  --inputImage "$(base64 -w0 gradient.png)" \
  --script '((define bg "input")
    (define v (image-vignette bg 0.6 1.0 #f))
    (define half (image-halftone v 10 30.0 "monochrome"))
    (define noise-bg (image-noise half 0.05 "grain" 1))
    (define text-bg (image-draw-rectangle noise-bg 80 500 920 140 "#16213e" #t))
    (define result (image-draw-text text-bg "POSTER DESIGN" 200 560 "#e94560" 72)))' \
  --output_file poster.png
```

### `poster.script` — Legacy Movie Poster
Line-based pipeline. Blur, vignette, tint, text overlay.

```
blur sigma=3
vignette strength=0.6 color=#000000
tint color=#1a1a2e amount=0.3
drop_shadow offset_x=0 offset_y=4 blur=8 color=#000000ff
text text="EPIC TITLE" x=100 y=700 color=#e94560 font_size=96
line x1=100 y1=810 x2=900 y2=810 color=#ffffff thickness=2
text text="A FILM BY YOU" x=100 y=840 color=#ffffff font_size=32
```

### `social_post.script` — Social Media Post from Scratch
Creates a 1080x1080 post with gradient background, text, and decorative circles.

---

## Platform-Specific

### `social_media_kit.scm` — One Image → Every Platform
Takes one photo and produces YouTube, Instagram, TikTok, Twitter, and feed formats in a single grid.

```scheme
(define base input)
(define youtube (youtube-thumb base "MUST WATCH"))
(define instagram (instagram-post base "NEW DROP"))
(define tiktok (reel-cover base "Swipe up!"))
(define twitter (crop-16-9 base))
(define feed (crop-4-5 base))
(define result (image-steps-grid
  (list youtube instagram tiktok twitter feed)
  (list "YouTube" "Insta Sq" "TikTok" "Twitter" "Feed 4:5")
  3000 1200))
```

### `youtube_thumbnail.script` — YouTube Thumbnail
Legacy script for 1280x720 with bold text.

### `profile_picture.script` — Circular Avatar
Crop center → circle mask → border → drop shadow.

---

## Comparison & Workflow

### `comparison_pipeline.scm` — Before/After + Progress Grid
Creates comparison images and step-by-step grids to show work progression.

```scheme
;; Side-by-side comparison
(define comparison (compare-labeled original edited "Before" "After"))

;; Pixel diff highlighting changes
(define diff (show-diff original edited))

;; Multi-step progress grid
(define progress (progress-grid step1 step2 step3 step4
  (list "Original" "Blur" "Tint" "Vignette" "Final")
  2000 500))
```

With tcli:
```bash
# Side-by-side comparison
tcli seamagic view_diff \
  --mode side_by_side \
  --originalImage "$(base64 -w0 before.png)" \
  --editedImage "$(base64 -w0 after.png)" \
  --output_file comparison.png

# Pixel diff (shows exactly what changed)
tcli seamagic view_diff \
  --mode pixel_diff \
  --originalImage "$(base64 -w0 before.png)" \
  --editedImage "$(base64 -w0 after.png)" \
  --output_file diff.png
```

---

## Batch Processing

### `effects_gallery.scm` — Apply Every Filter
Generates output with blur, brightness, contrast, grayscale, tint, vignette, duotone, noise, posterize, and more — all in one run.

### `scheme_examples.scm` — Reference Cookbook
A collection of common patterns: color temperature, high-key portrait, low-key noir, tilt-shift simulation, double exposure, glitch data_mosh.

---

## Quick tcli Reference

| Task | tcli Command |
|------|-------------|
| Single filter | `tcli seamagic edit_image --image "$(base64 -w0 in.png)" --operation blur --sigma 2.0 --output_file out.png` |
| Create canvas | `tcli seamagic create_image --type canvas --width 1080 --height 1080 --color "#000" --output_file bg.png` |
| Get info | `tcli seamagic get_info --image "$(base64 -w0 photo.png)"` |
| Scheme pipeline | `tcli seamagic run_steel --inputImage "$(base64 -w0 in.png)" --script '((define result (image-blur "input" 2.0)))' --output_file out.png` |
| Compare | `tcli seamagic view_diff --mode side_by_side --originalImage "$(base64 -w0 a.png)" --editedImage "$(base64 -w0 b.png)" --output_file cmp.png` |
