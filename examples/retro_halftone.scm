;; Retro Newspaper Halftone
(define logo "input")
(define duotoned (image-duotone logo "#1a1a2e" "#e94560"))
(define half (image-halftone duotoned 6 45.0 "color"))
(define result (image-vignette half 0.4 1.2 #f))
