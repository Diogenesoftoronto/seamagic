;;; ─────────────────────────────────────────────────────────────────────
;;;  lib.scm — Standard Library for mcp-image-design Steel Engine
;;;  
;;;  This file provides reusable functions for common image editing patterns,
;;;  effect pipelines, comparisons, and creative compositions. Load it before
;;;  your scripts by prefixing with (include "lib.scm") or copy-paste.
;;; ─────────────────────────────────────────────────────────────────────

;; ─── Helpers ─────────────────────────────────────────────────────────

(define (clamp x lo hi)
  (max lo (min x hi)))

(define (hex? s)
  (and (string? s) (= 7 (string-length s)) (char=? #\# (string-ref s 0))))

;; ─── Aspect Ratio Utilities ──────────────────────────────────────────

;; Crop to 16:9 (widescreen video)
(define (crop-16-9 img)
  (define dims (image-dimensions img))
  (define w (list-ref dims 0))
  (define h (list-ref dims 1))
  (define target-h (/ (* w 9) 16.0))
  (if (<= target-h h)
      (image-crop img 0 (/ (- h target-h) 2.0) w target-h)
      (let ((target-w (/ (* h 16) 9.0)))
        (image-crop img (/ (- w target-w) 2.0) 0 target-w h))))

;; Crop to 1:1 (social post)
(define (crop-1-1 img)
  (image-crop-center img
    (min (car (image-dimensions img)) (cadr (image-dimensions img)))
    (min (car (image-dimensions img)) (cadr (image-dimensions img)))))

;; Crop to 9:16 (stories/reels)
(define (crop-9-16 img)
  (define dims (image-dimensions img))
  (define w (list-ref dims 0))
  (define h (list-ref dims 1))
  (define target-w (/ (* h 9) 16.0))
  (if (<= target-w w)
      (image-crop img (/ (- w target-w) 2.0) 0 target-w h)
      (let ((target-h (/ (* w 16) 9.0)))
        (image-crop img 0 (/ (- h target-h) 2.0) w target-h))))

;; Crop to 4:5 (portrait social)
(define (crop-4-5 img)
  (define dims (image-dimensions img))
  (define w (list-ref dims 0))
  (define h (list-ref dims 1))
  (define target-h (/ (* w 5) 4.0))
  (if (<= target-h h)
      (image-crop img 0 (/ (- h target-h) 2.0) w target-h)
      (let ((target-w (/ (* h 4) 5.0)))
        (image-crop img (/ (- w target-w) 2.0) 0 target-w h))))

;; ─── Smart Resize ────────────────────────────────────────────────────

;; Resize to fit within a box, preserving aspect ratio
(define (fit-in img max-w max-h)
  (define dims (image-dimensions img))
  (define w (list-ref dims 0))
  (define h (list-ref dims 1))
  (define ratio (min (/ max-w (* w 1.0)) (/ max-h (* h 1.0))))
  (image-resize img (* w ratio) (* h ratio) "fit"))

;; ─── Effect Presets ──────────────────────────────────────────────────

;; Vintage film look
(define (effect-vintage img)
  (define s1 (image-grayscale img))
  (define s2 (image-tint s1 "#d4a574" 0.25))
  (define s3 (image-vignette s2 0.5 1.0 #f))
  (define s4 (image-noise s3 0.02 "grain" 42))
  (image-contrast s4 10))

;; Cyberpunk neon
(define (effect-cyberpunk img)
  (define s1 (image-tint img "#ff00ff" 0.15))
  (define s2 (image-tint s1 "#00ffff" 0.05))
  (define s3 (image-chromatic-aberration s2 8.0 #f))
  (define s4 (image-scanlines s3 4 0.2 1))
  (image-edge-detect s4 30 #t))

;; Film noir (high contrast B&W)
(define (effect-noir img)
  (define s1 (image-grayscale img))
  (define s2 (image-contrast s1 40))
  (define s3 (image-vignette s2 0.7 1.0 #f))
  (image-noise s3 0.03 "grain" 99))

;; Dreamy soft-focus
(define (effect-dreamy img)
  (define s1 (image-blur img 3.0))
  (define s2 (image-tint s1 "#ffb6c1" 0.2))
  (define s3 (image-brightness s2 15))
  (image-vignette s3 0.3 0.8 #f))

;; Glitch data-mosh
(define (effect-glitch img)
  (define s1 (image-glitch img "slice_shift" 0.5 42))
  (define s2 (image-chromatic-aberration s1 10.0 #f))
  (image-pixel-sort s2 "horizontal" 0.3 0.8))

;; X-ray / inverted edge
(define (effect-xray img)
  (define s1 (image-edge-detect img 25 #t))
  (image-brightness s1 20))

;; Halftone comic print
(define (effect-halftone-comic img)
  (image-halftone img 8 45.0 "color"))

;; 1970s duotone
(define (effect-duotone img shadow highlight)
  (image-duotone img shadow highlight))

;; Heatmap false-color
(define (effect-heatmap img)
  (image-duotone img "#330000" "#ff6600"))

;; Ocean deep
(define (effect-ocean img)
  (image-duotone img "#001133" "#00ccff"))

;; Forest
(define (effect-forest img)
  (image-duotone img "#0a2f0a" "#7cfc00"))

;; ─── Typography Helpers ──────────────────────────────────────────────

;; Draw centered text on an image
(define (text-centered img str y color size)
  (define dims (image-dimensions img))
  (define w (list-ref dims 0))
  (define text-w (* (string-length str) (* size 0.55)))
  (image-draw-text img str (max 0 (/ (- w text-w) 2)) y color size))

;; Draw title + subtitle block
(define (text-block img title title-y title-size subtitle subtitle-y subtitle-size color)
  (define with-title (image-draw-text img title 50 title-y color title-size))
  (image-draw-text with-title subtitle 50 subtitle-y color subtitle-size))

;; ─── Overlay / Blend Pipelines ───────────────────────────────────────

;; Note: steel engine currently only exposes image-overlay with normal blend.
;; If blend modes are needed, use the edit_image tool with operations=[{op:"overlay", blendMode:"multiply"}]

;; Watermark an image (bottom-right corner)
(define (watermark img mark opacity)
  (define dims (image-dimensions img))
  (define mdims (image-dimensions mark))
  (define w (list-ref dims 0))
  (define h (list-ref dims 1))
  (define mw (list-ref mdims 0))
  (define mh (list-ref mdims 1))
  (image-overlay img mark (- w mw 20) (- h mh 20) opacity))

;; ─── Comparison Helpers ──────────────────────────────────────────────

;; Compare original vs edited side-by-side with labels
(define (compare-labeled original edited label1 label2)
  (image-comparison original edited "horizontal" label1 label2))

;; Compare before/after vertically
(define (compare-vertical original edited)
  (image-comparison original edited "vertical" "Before" "After"))

;; Create a 4-step progress grid: original + 3 stages
(define (progress-grid original s1 s2 s3 labels w h)
  (image-steps-grid (list original s1 s2 s3) labels w h))

;; Show the diff (highlighted changes) between two images
(define (show-diff original edited)
  (image-diff original edited))

;; ─── Frame / Border Presets ──────────────────────────────────────────

;; Polaroid-style frame
(define (polaroid img border pad)
  (define dims (image-dimensions img))
  (define w (list-ref dims 0))
  (define h (list-ref dims 1))
  (define fw (+ w (* 2 pad) (* 2 border)))
  (define fh (+ h (* 2 pad) (* 3 border)))
  (define frame (image-frame fw fh border "#f5f5f5"))
  (image-overlay frame img (+ border pad) (+ border pad) 1.0))

;; Rounded-rect mask (using shape_mask via canvas trick)
(define (rounded-mask img)
  (define dims (image-dimensions img))
  (define w (list-ref dims 0))
  (define h (list-ref dims 1))
  ;; shape_mask not directly exposed in steel, use overlay with pre-masked version
  img)

;; ─── Thumbnail Presets ───────────────────────────────────────────────

;; YouTube-style thumbnail (16:9 with bold title)
(define (youtube-thumb img title)
  (define s1 (crop-16-9 img))
  (define dims (image-dimensions s1))
  (define w (list-ref dims 0))
  (define h (list-ref dims 1))
  (define bg (image-gradient w h "#ff006e" "#8338ec" 45))
  (define blended (image-overlay bg s1 0 0 0.7))
  (define shadowed (image-drop-shadow blended 0 6 16 "#000000cc"))
  (image-draw-text shadowed title 80 (* h 0.65) "#ffffff" 84))

;; Instagram square post
(define (instagram-post img text)
  (define s1 (crop-1-1 img))
  (define dims (image-dimensions s1))
  (define w (list-ref dims 0))
  (define v (image-vignette s1 0.2 1.0 #f))
  (text-centered v text (* w 0.85) "#ffffff" 56))

;; TikTok/Reel cover (9:16)
(define (reel-cover img text)
  (define s1 (crop-9-16 img))
  (define dims (image-dimensions s1))
  (define w (list-ref dims 0))
  (define h (list-ref dims 1))
  (define blurred (image-blur s1 8.0))
  (define with-vig (image-vignette blurred 0.4 1.0 #f))
  (text-centered with-vig text (* h 0.75) "#ffffff" 72))

;; ─── Batch / Multi-step Pipelines ────────────────────────────────────

;; Apply a list of single-argument ops in sequence
(define (pipeline img . ops)
  (if (null? ops)
      img
      (apply pipeline (cons ((car ops) img) (cdr ops)))))

;; Quick preset: make any image look "polished"
(define (polish img)
  (define s1 (image-contrast img 10))
  (define s2 (image-brightness s1 5))
  (define s3 (image-vignette s2 0.2 1.0 #f))
  s3)

;; Quick preset: dramatic cinematic
(define (cinematic img)
  (define s1 (image-tint img "#1a1a2e" 0.15))
  (define s2 (image-vignette s1 0.5 1.0 #f))
  (define s3 (image-contrast s2 15))
  (image-noise s3 0.015 "grain" 123))

;; Quick preset: product photo clean-up
(define (product-clean img)
  (define s1 (image-brightness img 10))
  (define s2 (image-contrast s1 20))
  (define s3 (image-blur s2 0.5))
  s3)

;; ─── Mask / Selection Utilities ──────────────────────────────────────

;; Create a solid color canvas that matches image dimensions
(define (same-size-canvas img color)
  (define dims (image-dimensions img))
  (image-canvas (list-ref dims 0) (list-ref dims 1) color))

;; Pad an image with a border color
(define (pad img px color)
  (define dims (image-dimensions img))
  (define w (list-ref dims 0))
  (define h (list-ref dims 1))
  (define cw (+ w (* 2 px)))
  (define ch (+ h (* 2 px)))
  (define canvas (image-canvas cw ch color))
  (image-overlay canvas img px px 1.0))

;; ─── Grid / Collage Layouts ──────────────────────────────────────────

;; 2x2 grid from 4 images
(define (grid-2x2 img1 img2 img3 img4 w h)
  (image-steps-grid (list img1 img2 img3 img4)
                    (list "TL" "TR" "BL" "BR")
                    w h))

;; Before / After comparison (horizontal)
(define (before-after before after)
  (compare-labeled before after "Before" "After"))

;; ─── Segmentation / Background Removal ───────────────────────────────

;; Segment image into all detected objects. Returns a list of image IDs:
;; [combined_mask, mask1, mask2, ...]
(define (segment-everything img)
  (image-segment-everything img))

;; Remove background, returning just the foreground subject
(define (remove-bg img)
  (image-remove-background img))

;; Apply an individual mask to isolate a specific object from segmentation.
;; Uses the mask as alpha (white = keep, black = transparent).
(define (apply-segmented-mask original mask-img)
  (image-overlay original mask-img 0 0 1.0))

;; Extract object at given segmentation index using mask
(define (extract-segment segments idx)
  (list-ref segments idx))

;; GPT Image 2 edit via fal.ai (natural language or mask-based)
;; mask is optional; if provided, white areas get edited
(define (fal-edit img prompt . mask-opt)
  (image-fal-edit img prompt (if (null? mask-opt) #f (car mask-opt))))

;; ─── Export / Finalize ───────────────────────────────────────────────

;; Convenience: set result and also create a comparison with original
(define (finalize-with-compare original edited)
  (define comparison (before-after original edited))
  (list edited comparison))

;; Convenience: create edited + side-by-side + diff
(define (analyze-changes original edited)
  (define side-by-side (before-after original edited))
  (define diff (show-diff original edited))
  (list edited side-by-side diff))
