; Steel Scheme Examples for mcp-image-design
; Each expression returns a new image ID, assigned to `result`

; ─── Example 1: Social Media Post with Gradient ───
(define bg (image-canvas 1080 1080 "#1a1a2e"))
(define grad (image-gradient 1080 1080 "#667eea" "#764ba2" 135))
(define v (image-vignette grad 0.3 1.0 #f))
(define text-area (image-draw-rectangle v 80 380 920 140 "#16213e" #t))
(define result (image-draw-text text-area "HELLO WORLD" 200 445 "#e94560" 72))
; result now holds the final image ID

; ─── Example 2: Profile Picture (circular mask) ───
(define cropped (image-crop-center input 500 500))
(define blurred (image-blur cropped 0.5))
(define mask (image-apply-mask blurred (image-canvas 500 500 "#FFFFFF")))
; For circular mask using shape_mask is more complex in Scheme
; Use the profile_picture.script format with run_script instead

; ─── Example 3: Poster with Duotone ───
(define poster-bg (image-gradient 800 1200 "#0f0f23" "#1a1a2e" 45))
(define with-shadow (image-drop-shadow poster-bg 0 4 8 "#00000080"))
(define with-text (image-draw-text with-shadow "EPIC TITLE" 100 900 "#e94560" 96))
(define result (image-draw-text with-text "A FILM BY YOU" 100 1040 "#ffffff" 32))

; ─── Example 4: Thumbnail with border ───
(define centered (image-crop-center input 400 400))
(define framed (image-frame 420 420 10 "#333333"))
(define result (image-overlay centered framed 0 0 1.0))

; ─── Example 5: Decompose image into regions ───
; Split a large image into 4 quadrants
(define dims (image-dimensions input))
(define w (list-ref dims 0))
(define h (list-ref dims 1))
(define hw (/ w 2))
(define hh (/ h 2))
(define tl (image-crop input 0 0 hw hh))
(define tr (image-crop input hw 0 (- w hw) hh))
(define bl (image-crop input 0 hh hw (- h hh)))
(define br (image-crop input hw hh (- w hw) (- h hh)))
; Each quadrant is now a separate image in the registry

; ─── Example 6: Quick filter pipeline ───
(define bw (image-grayscale input))
(define tinted (image-tint bw "#667eea" 0.3))
(define result (image-vignette tinted 0.4 1.0 #f))

; ─── Example 7: Multi-layer composition ───
(define bg (image-canvas 1200 800 "#ffffff"))
(define banner (image-gradient 1200 200 "#667eea" "#764ba2" 135))
(define with-banner (image-overlay bg banner 0 0 1.0))
(define with-text (image-draw-text with-banner "PHOTO COLLECTION" 50 80 "#ffffff" 64))
(define result with-text)
; Then overlay individual photos using image-overlay

; ─── Example 8: Show editing steps in a grid ───
(define step1 input)
(define step2 (image-grayscale step1))
(define step3 (image-tint step2 "#667eea" 0.3))
(define step4 (image-vignette step3 0.4 1.0 #f))
(define result (image-steps-grid
  (list step1 step2 step3 step4)
  (list "Original" "Grayscale" "Tinted" "Final")
  1600 400))

; ─── Example 9: Side-by-side comparison ───
(define blurred (image-blur input 5.0))
(define result (image-comparison input blurred "horizontal"))

; ─── Example 10: Visual diff ───
(define edited (image-tint (image-blur input 2.0) "#ff0000" 0.2))
(define result (image-diff input edited))
