;; Trippy Kaleidoscope + Noise + Brightness
(define base "input")
(define kaleido (image-kaleidoscope base 6 0.0))
(define bulged (image-liquify kaleido "bulge" 0.5 0.5 0.4 0.8))
(define noisy (image-noise bulged 0.08 "grain" 123))
(define result (image-brightness noisy 10.0))
