;; Edge-Detect X-Ray with Noise
(define base "input")
(define edges (image-edge-detect base 20 #t))
(define grainy (image-noise edges 0.12 "monochrome" 99))
(define scanned (image-scanlines grainy 4 0.35 2))
(define result (image-tint scanned "#00ff88" 0.15))
