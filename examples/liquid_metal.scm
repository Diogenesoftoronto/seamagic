;; Liquid Metal / Melted Photo
(define photo "input")
(define pinched (image-liquify photo "pinch" 0.3 0.3 0.5 0.6))
(define swirled (image-liquify pinched "swirl" 0.7 0.7 0.35 1.2))
(define embossed (image-emboss swirled 1.0))
(define solar (image-solarize embossed 120))
(define result (image-pixel-sort solar "vertical" 0.1 0.7))
