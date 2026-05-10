;;; effects_gallery.scm — Show every effect preset in lib.scm
;;; Run: cargo run --example run_example scheme <input>.png examples/effects_gallery.scm output.png
;;; This creates a comparison grid of all available effects.

;; lib.scm is auto-loaded by the Steel engine

(define base input)
(define dims (image-dimensions base))
(define w (list-ref dims 0))
(define h (list-ref dims 1))

;; Generate one version per effect
(define v1  (effect-vintage base))
(define v2  (effect-cyberpunk base))
(define v3  (effect-noir base))
(define v4  (effect-dreamy base))
(define v5  (effect-glitch base))
(define v6  (effect-xray base))
(define v7  (effect-halftone-comic base))
(define v8  (effect-heatmap base))
(define v9  (effect-ocean base))
(define v10 (effect-forest base))
(define v11 (polish base))
(define v12 (cinematic base))

;; Build a 4x3 comparison grid
(define result (image-steps-grid
  (list v1 v2 v3 v4 v5 v6 v7 v8 v9 v10 v11 v12)
  (list "Vintage" "Cyberpunk" "Noir" "Dreamy" "Glitch"
        "X-Ray" "Halftone" "Heatmap" "Ocean" "Forest"
        "Polish" "Cinematic")
  2400 1800))
