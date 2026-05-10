;;; social_media_kit.scm — One image → every platform format
;;; Automatically crops and styles an input image for YouTube, Instagram, TikTok, etc.

;; lib.scm is auto-loaded by the Steel engine

(define base input)

;; YouTube thumbnail (1280x720)
(define youtube (youtube-thumb base "MUST WATCH"))

;; Instagram square (1080x1080)
(define instagram (instagram-post base "NEW DROP"))

;; TikTok/Reels (1080x1920)
(define tiktok (reel-cover base "Swipe up!"))

;; Twitter/X header-ish wide crop
(define twitter (crop-16-9 base))

;; 4:5 portrait for Instagram feed
(define feed (crop-4-5 base))

;; Create a collage showing all formats
(define result (image-steps-grid
  (list youtube instagram tiktok twitter feed)
  (list "YouTube" "Insta Sq" "TikTok" "Twitter" "Feed 4:5")
  3000 1200))
