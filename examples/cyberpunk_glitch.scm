;; Cyberpunk Glitch Portrait Pipeline
(define base "input")
(define wavy (image-warp base "wave" 15.0 0.08))
(define glitched (image-glitch wavy "rgb_split" 0.7 42))
(define chromatic (image-chromatic-aberration glitched 8.0 #t))
(define scan (image-scanlines chromatic 3 0.25 1))
(define result (image-posterize scan 6))
