;;; comparison_pipeline.scm — How the comparison + preview pipeline works
;;; 
;;; When you use the `diff` tool (side_by_side or pixel_diff mode), the server:
;;;   1. Creates the comparison image (full resolution)
;;;   2. Generates a compressed JPEG preview (max 512px) 
;;;   3. Returns BOTH: the full PNG for editing + the compressed preview for AI viewing
;;;
;;; The compressed preview uses image::imageops::FilterType::Lanczos3 for quality
;;; and JPEG encoding for ~90% size reduction.
;;; lib.scm is auto-loaded by the Steel engine

;;; ─── Example 1: Manual before/after in Scheme ──────────────────────
(define original input)
(define edited (effect-cyberpunk original))

;; Create a side-by-side comparison image
(define comparison (compare-labeled original edited "Before" "After"))

;; Also create a pixel-level diff highlighting changes
(define diff (show-diff original edited))

;; Return the comparison as the main result
(define result comparison)

;; ─── Example 2: Multi-step work with progress tracking ───────────────
(define step1  input)
(define step2  (image-blur step1 2.0))
(define step3  (image-tint step2 "#667eea" 0.3))
(define step4  (image-vignette step3 0.4 1.0 #f))
(define step5  (image-draw-text step4 "FINAL" 100 100 "#ffffff" 72))

;; Show all steps in a grid
(define progress (progress-grid step1 step2 step3 step4
  (list "Original" "Blur" "Tint" "Vignette" "Final")
  2000 500))

;; Return progress grid
;; (define result progress)
