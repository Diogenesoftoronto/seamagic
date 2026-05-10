;; Fisheye Meme Warp
(define meme "input")
(define fisheyed (image-warp meme "fisheye" 25.0 0.05))
(define glitched (image-glitch fisheyed "slice_shift" 0.4 7))
(define result (image-chromatic-aberration glitched 5.0 #f))
