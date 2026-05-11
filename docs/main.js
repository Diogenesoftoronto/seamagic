// seamagic showcase — interactive demo
// Note: The WASM module is a future enhancement. The image processing below
// uses the Canvas 2D API to demonstrate the filter concepts client-side.

(function() {
    'use strict';

    // ================================================================
    // HERO CANVAS — generative particles + noise field
    // ================================================================
    (function initHeroCanvas() {
        const canvas = document.getElementById('heroCanvas');
        if (!canvas) return;

        const ctx = canvas.getContext('2d');
        const particles = [];
        const PARTICLE_COUNT = 60;
        let time = 0;
        let animId = null;

        function resize() {
            const rect = canvas.getBoundingClientRect();
            const dpr = Math.min(window.devicePixelRatio || 1, 2);
            canvas.width = rect.width * dpr;
            canvas.height = rect.height * dpr;
            ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
        }

        class Particle {
            constructor() {
                this.reset();
            }
            reset() {
                const rect = canvas.getBoundingClientRect();
                this.x = Math.random() * rect.width;
                this.y = Math.random() * rect.height;
                this.vx = (Math.random() - 0.5) * 0.5;
                this.vy = (Math.random() - 0.5) * 0.5;
                this.radius = Math.random() * 2.5 + 1;
                this.phase = Math.random() * Math.PI * 2;
                this.color = ['#5c7cfa', '#9f7aea', '#00cfc8', '#f6ad55'][Math.floor(Math.random() * 4)];
            }
            update() {
                const rect = canvas.getBoundingClientRect();
                this.x += this.vx;
                this.y += this.vy;
                if (this.x < 0 || this.x > rect.width) this.vx *= -1;
                if (this.y < 0 || this.y > rect.height) this.vy *= -1;
            }
            draw() {
                const pulse = 0.7 + 0.3 * Math.sin(time * 0.02 + this.phase);
                ctx.globalAlpha = pulse * 0.6;
                ctx.fillStyle = this.color;
                ctx.beginPath();
                ctx.arc(this.x, this.y, this.radius * pulse, 0, Math.PI * 2);
                ctx.fill();
            }
        }

        function initParticles() {
            particles.length = 0;
            for (let i = 0; i < PARTICLE_COUNT; i++) {
                particles.push(new Particle());
            }
        }

        function drawConnections() {
            const maxDist = 100;
            for (let i = 0; i < particles.length; i++) {
                for (let j = i + 1; j < particles.length; j++) {
                    const dx = particles[i].x - particles[j].x;
                    const dy = particles[i].y - particles[j].y;
                    const dist = Math.sqrt(dx * dx + dy * dy);
                    if (dist < maxDist) {
                        ctx.globalAlpha = (1 - dist / maxDist) * 0.15;
                        ctx.strokeStyle = '#5c7cfa';
                        ctx.lineWidth = 0.8;
                        ctx.beginPath();
                        ctx.moveTo(particles[i].x, particles[i].y);
                        ctx.lineTo(particles[j].x, particles[j].y);
                        ctx.stroke();
                    }
                }
            }
        }

        function drawGridOverlay() {
            const rect = canvas.getBoundingClientRect();
            const gridSize = 30;
            ctx.globalAlpha = 0.04;
            ctx.strokeStyle = '#5c7cfa';
            ctx.lineWidth = 0.5;
            for (let x = 0; x < rect.width; x += gridSize) {
                ctx.beginPath();
                ctx.moveTo(x, 0);
                ctx.lineTo(x, rect.height);
                ctx.stroke();
            }
            for (let y = 0; y < rect.height; y += gridSize) {
                ctx.beginPath();
                ctx.moveTo(0, y);
                ctx.lineTo(rect.width, y);
                ctx.stroke();
            }
        }

        function render() {
            const rect = canvas.getBoundingClientRect();
            ctx.clearRect(0, 0, rect.width, rect.height);

            drawGridOverlay();

            particles.forEach(p => { p.update(); p.draw(); });
            drawConnections();

            ctx.globalAlpha = 1;

            // Perlin-ish noise effect using overlapping sine waves
            const imageData = ctx.getImageData(0, 0, rect.width, rect.height);
            const data = imageData.data;
            for (let y = 0; y < rect.height; y += 3) {
                for (let x = 0; x < rect.width; x += 3) {
                    const n = Math.sin(x * 0.02 + time * 0.01) * Math.cos(y * 0.02 - time * 0.008);
                    const alpha = (n + 1) * 0.5 * 8;
                    const idx = (y * rect.width + x) * 4;
                    data[idx] = 92;
                    data[idx + 1] = 124;
                    data[idx + 2] = 250;
                    data[idx + 3] = alpha;
                }
            }
            ctx.putImageData(imageData, 0, 0);

            time++;
            animId = requestAnimationFrame(render);
        }

        resize();
        initParticles();
        render();

        window.addEventListener('resize', () => {
            resize();
            initParticles();
        });
    })();

    // ================================================================
    // NAVIGATION
    // ================================================================
    (function initNav() {
        const nav = document.getElementById('nav');
        const toggle = document.getElementById('navToggle');
        const links = document.querySelector('.nav-links');

        window.addEventListener('scroll', () => {
            nav.classList.toggle('nav-scrolled', window.scrollY > 20);
        });

        if (toggle) {
            toggle.addEventListener('click', () => {
                links.classList.toggle('active');
                toggle.classList.toggle('active');
            });
        }
    })();

    // ================================================================
    // IMAGE DEMO — Canvas-based filter pipeline (WASM placeholder)
    // ================================================================
    (function initDemo() {
        const uploadZone = document.getElementById('uploadZone');
        const imageInput = document.getElementById('imageInput');
        const uploadPanel = document.getElementById('uploadPanel');
        const editorPanel = document.getElementById('editorPanel');
        const previewPlaceholder = document.getElementById('previewPlaceholder');
        const previewContainer = document.getElementById('previewContainer');
        const previewImage = document.getElementById('previewImage');
        const previewProcessing = document.getElementById('previewProcessing');
        const imageDimensions = document.getElementById('imageDimensions');
        const imageFormat = document.getElementById('imageFormat');
        const wasmStatus = document.getElementById('wasmStatus');
        const wasmStatusText = document.getElementById('wasmStatusText');
        const wasmIndicator = document.getElementById('wasmIndicator');

        let originalImage = null;
        let currentImage = null;
        let isProcessing = false;
        let pendingUpdate = false;

        // Mark WASM as ready (pure JS mode for now)
        setTimeout(() => {
            wasmStatus.classList.add('ready');
            wasmStatusText.textContent = 'Ready (Canvas 2D mode)';
            wasmIndicator.style.animation = 'none';
            wasmIndicator.style.background = 'var(--accent-green)';
        }, 1500);

        // Upload handling
        uploadZone.addEventListener('click', () => imageInput.click());
        uploadZone.addEventListener('dragover', (e) => {
            e.preventDefault();
            uploadZone.classList.add('dragover');
        });
        uploadZone.addEventListener('dragleave', () => {
            uploadZone.classList.remove('dragover');
        });
        uploadZone.addEventListener('drop', (e) => {
            e.preventDefault();
            uploadZone.classList.remove('dragover');
            const file = e.dataTransfer.files[0];
            if (file && file.type.startsWith('image/')) {
                loadFile(file);
            }
        });
        imageInput.addEventListener('change', () => {
            if (imageInput.files[0]) loadFile(imageInput.files[0]);
        });

        // Preset buttons — generate procedural canvases
        document.querySelectorAll('.preset-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const canvas = document.createElement('canvas');
                canvas.width = 600;
                canvas.height = 400;
                const ctx = canvas.getContext('2d');

                if (btn.dataset.preset === 'sample1') {
                    const grad = ctx.createLinearGradient(0, 0, 600, 400);
                    grad.addColorStop(0, '#ff6b6b');
                    grad.addColorStop(0.5, '#4ecdc4');
                    grad.addColorStop(1, '#45b7d1');
                    ctx.fillStyle = grad;
                    ctx.fillRect(0, 0, 600, 400);
                } else if (btn.dataset.preset === 'sample2') {
                    const grad = ctx.createLinearGradient(0, 0, 0, 400);
                    grad.addColorStop(0, '#1a1a2e');
                    grad.addColorStop(1, '#16213e');
                    ctx.fillStyle = grad;
                    ctx.fillRect(0, 0, 600, 400);
                    // Stars
                    for (let i = 0; i < 80; i++) {
                        ctx.fillStyle = `rgba(255,255,255,${Math.random() * 0.5})`;
                        ctx.beginPath();
                        ctx.arc(Math.random() * 600, Math.random() * 200, Math.random() * 1.5, 0, Math.PI * 2);
                        ctx.fill();
                    }
                } else if (btn.dataset.preset === 'sample3') {
                    const grad = ctx.createLinearGradient(0, 200, 600, 200);
                    grad.addColorStop(0, '#f8b500');
                    grad.addColorStop(0.5, '#fceabb');
                    grad.addColorStop(1, '#ff8c00');
                    ctx.fillStyle = grad;
                    ctx.fillRect(0, 0, 600, 400);
                }

                canvas.toBlob(blob => {
                    const file = new File([blob], 'preset.png', { type: 'image/png' });
                    loadFile(file);
                });
            });
        });

        function loadFile(file) {
            const url = URL.createObjectURL(file);
            const img = new Image();
            img.onload = () => {
                originalImage = createOffscreenCanvas(img);
                currentImage = createOffscreenCanvas(img);
                previewImage.src = url;
                previewPlaceholder.style.display = 'none';
                previewContainer.style.display = 'flex';
                editorPanel.style.display = 'block';
                imageDimensions.textContent = `${img.naturalWidth}x${img.naturalHeight}`;
                imageFormat.textContent = file.type.split('/')[1].toUpperCase();
                updatePreview();
            };
            img.src = url;
        }

        function createOffscreenCanvas(img) {
            const c = document.createElement('canvas');
            c.width = img.naturalWidth;
            c.height = img.naturalHeight;
            const x = c.getContext('2d');
            x.drawImage(img, 0, 0);
            return c;
        }

        function createPreviewCanvas(sourceCanvas) {
            // Create a scaled preview (max 800px width)
            const maxW = 800;
            let w = sourceCanvas.width;
            let h = sourceCanvas.height;
            if (w > maxW) {
                h = (h * maxW) / w;
                w = maxW;
            }
            const c = document.createElement('canvas');
            c.width = w;
            c.height = h;
            const ctx = c.getContext('2d');
            ctx.drawImage(sourceCanvas, 0, 0, w, h);
            return c;
        }

        // Filter state
        const filters = {
            blur: 0,
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            grayscale: false,
            sepia: false,
            invert: false,
            pixelate: false,
            edge: false,
            emboss: false,
            duotone: false,
            duotoneA: '#ff0066',
            duotoneB: '#00ccff',
        };

        // Slider bindings
        const sliders = {
            blurSlider: { key: 'blur', display: 'blurValue' },
            brightnessSlider: { key: 'brightness', display: 'brightnessValue' },
            contrastSlider: { key: 'contrast', display: 'contrastValue' },
            saturationSlider: { key: 'saturation', display: 'saturationValue' },
        };

        Object.entries(sliders).forEach(([id, info]) => {
            const el = document.getElementById(id);
            const disp = document.getElementById(info.display);
            if (!el) return;
            el.addEventListener('input', () => {
                filters[info.key] = parseFloat(el.value);
                disp.textContent = el.value;
                scheduleUpdate();
            });
        });

        // Toggle bindings
        const toggles = {
            grayscaleToggle: 'grayscale',
            sepiaToggle: 'sepia',
            invertToggle: 'invert',
            pixelateToggle: 'pixelate',
            edgeToggle: 'edge',
            embossToggle: 'emboss',
            duotoneToggle: 'duotone',
        };

        Object.entries(toggles).forEach(([id, key]) => {
            const el = document.getElementById(id);
            if (!el) return;
            el.addEventListener('click', () => {
                filters[key] = !filters[key];
                el.dataset.active = filters[key];
                scheduleUpdate();
            });
        });

        // Color pickers
        const colorA = document.getElementById('duotoneColorA');
        const colorB = document.getElementById('duotoneColorB');
        const hexA = document.getElementById('duotoneHexA');
        const hexB = document.getElementById('duotoneHexB');

        if (colorA) {
            colorA.addEventListener('input', () => {
                filters.duotoneA = colorA.value;
                hexA.textContent = colorA.value;
                if (filters.duotone) scheduleUpdate();
            });
        }
        if (colorB) {
            colorB.addEventListener('input', () => {
                filters.duotoneB = colorB.value;
                hexB.textContent = colorB.value;
                if (filters.duotone) scheduleUpdate();
            });
        }

        document.getElementById('resetFilters')?.addEventListener('click', () => {
            filters.blur = 0;
            filters.brightness = 1.0;
            filters.contrast = 1.0;
            filters.saturation = 1.0;
            filters.grayscale = false;
            filters.sepia = false;
            filters.invert = false;
            filters.pixelate = false;
            filters.edge = false;
            filters.emboss = false;
            filters.duotone = false;

            document.getElementById('blurSlider').value = 0;
            document.getElementById('blurValue').textContent = '0';
            document.getElementById('brightnessSlider').value = 1;
            document.getElementById('brightnessValue').textContent = '1.0';
            document.getElementById('contrastSlider').value = 1;
            document.getElementById('contrastValue').textContent = '1.0';
            document.getElementById('saturationSlider').value = 1;
            document.getElementById('saturationValue').textContent = '1.0';

            Object.keys(toggles).forEach(id => {
                document.getElementById(id).dataset.active = 'false';
            });

            scheduleUpdate();
        });

        document.getElementById('downloadImage')?.addEventListener('click', () => {
            if (!currentImage) return;
            const link = document.createElement('a');
            link.download = 'seamagic-edited.png';
            link.href = currentImage.toDataURL('image/png');
            link.click();
        });

        function scheduleUpdate() {
            if (isProcessing) {
                pendingUpdate = true;
                return;
            }
            isProcessing = true;
            previewProcessing.classList.remove('hidden');

            // Small delay to show processing state and batch rapid changes
            setTimeout(() => {
                applyFilters();
                previewProcessing.classList.add('hidden');
                isProcessing = false;
                if (pendingUpdate) {
                    pendingUpdate = false;
                    scheduleUpdate();
                }
            }, 100);
        }

        function updatePreview() {
            if (!originalImage) return;
            scheduleUpdate();
        }

        function applyFilters() {
            if (!originalImage) return;

            const src = originalImage;
            const w = src.width;
            const h = src.height;
            const canvas = document.createElement('canvas');
            canvas.width = w;
            canvas.height = h;
            const ctx = canvas.getContext('2d');

            // Build filter string for CSS filter property
            const cssFilters = [];
            if (filters.grayscale) cssFilters.push('grayscale(1)');
            if (filters.sepia) cssFilters.push('sepia(1)');
            if (filters.invert) cssFilters.push('invert(1)');
            if (filters.brightness !== 1.0) cssFilters.push(`brightness(${filters.brightness})`);
            if (filters.contrast !== 1.0) cssFilters.push(`contrast(${filters.contrast})`);
            if (filters.saturation !== 1.0) cssFilters.push(`saturate(${filters.saturation})`);
            if (filters.blur > 0) cssFilters.push(`blur(${filters.blur}px)`);

            ctx.filter = cssFilters.join(' ') || 'none';
            ctx.drawImage(src, 0, 0);
            ctx.filter = 'none';

            // Pixelate requires manual pixel manipulation
            if (filters.pixelate) {
                const pixelSize = Math.max(4, Math.floor(Math.min(w, h) / 60));
                const temp = document.createElement('canvas');
                temp.width = Math.ceil(w / pixelSize);
                temp.height = Math.ceil(h / pixelSize);
                const tctx = temp.getContext('2d');
                tctx.drawImage(canvas, 0, 0, temp.width, temp.height);
                ctx.imageSmoothingEnabled = false;
                ctx.clearRect(0, 0, w, h);
                ctx.drawImage(temp, 0, 0, w, h);
                ctx.imageSmoothingEnabled = true;
            }

            // Duotone (applied using canvas blend modes)
            if (filters.duotone) {
                const gray = document.createElement('canvas');
                gray.width = w;
                gray.height = h;
                const gctx = gray.getContext('2d');
                gctx.drawImage(canvas, 0, 0);
                gctx.globalCompositeOperation = 'saturation';
                gctx.fillStyle = 'black';
                gctx.fillRect(0, 0, w, h);

                ctx.globalCompositeOperation = 'source-atop';
                const grad = ctx.createLinearGradient(0, 0, w, h);
                grad.addColorStop(0, filters.duotoneA);
                grad.addColorStop(1, filters.duotoneB);
                ctx.fillStyle = grad;
                ctx.fillRect(0, 0, w, h);
                ctx.globalCompositeOperation = 'source-over';
            }

            // Edge detection via convolutions
            if (filters.edge) {
                applyConvolution(canvas, w, h, [
                    -1, -1, -1,
                    -1,  8, -1,
                    -1, -1, -1
                ], 1, 0);
            }

            // Emboss
            if (filters.emboss) {
                applyConvolution(canvas, w, h, [
                    -2, -1,  0,
                    -1,  1,  1,
                     0,  1,  2
                ], 1, 128);
            }

            currentImage = canvas;
            const preview = createPreviewCanvas(canvas);
            previewImage.src = preview.toDataURL('image/png');
        }

        function applyConvolution(canvas, w, h, kernel, divisor, offset) {
            const ctx = canvas.getContext('2d');
            const imageData = ctx.getImageData(0, 0, w, h);
            const data = imageData.data;
            const newData = new Uint8ClampedArray(data);
            const side = Math.sqrt(kernel.length);
            const half = Math.floor(side / 2);

            for (let y = 0; y < h; y++) {
                for (let x = 0; x < w; x++) {
                    let r = 0, g = 0, b = 0;
                    for (let cy = 0; cy < side; cy++) {
                        for (let cx = 0; cx < side; cx++) {
                            const sy = Math.min(h - 1, Math.max(0, y + cy - half));
                            const sx = Math.min(w - 1, Math.max(0, x + cx - half));
                            const idx = (sy * w + sx) * 4;
                            const k = kernel[cy * side + cx];
                            r += data[idx] * k;
                            g += data[idx + 1] * k;
                            b += data[idx + 2] * k;
                        }
                    }
                    const idx = (y * w + x) * 4;
                    newData[idx] = Math.min(255, Math.max(0, r / divisor + offset));
                    newData[idx + 1] = Math.min(255, Math.max(0, g / divisor + offset));
                    newData[idx + 2] = Math.min(255, Math.max(0, b / divisor + offset));
                }
            }

            for (let i = 0; i < data.length; i++) {
                data[i] = newData[i];
            }
            ctx.putImageData(imageData, 0, 0);
        }
    })();

    // ================================================================
    // EXAMPLE TABS
    // ================================================================
    (function initExamples() {
        const tabs = document.querySelectorAll('.examples-tab');
        const panels = document.querySelectorAll('.example-panel');

        tabs.forEach(tab => {
            tab.addEventListener('click', () => {
                tabs.forEach(t => t.classList.remove('examples-tab-active'));
                tab.classList.add('examples-tab-active');

                panels.forEach(p => p.classList.remove('example-panel-active'));
                const panel = document.getElementById(tab.dataset.tab);
                if (panel) panel.classList.add('example-panel-active');
            });
        });

        // Code language toggles within each example
        document.querySelectorAll('.example-main').forEach(main => {
            const btns = main.querySelectorAll('.example-code-btn');
            const blocks = main.querySelectorAll('.example-code-block');

            btns.forEach(btn => {
                btn.addEventListener('click', () => {
                    const lang = btn.dataset.lang;
                    btns.forEach(b => b.classList.remove('example-code-btn-active'));
                    btn.classList.add('example-code-btn-active');
                    blocks.forEach(b => {
                        b.classList.toggle('example-code-block-active', b.dataset.lang === lang);
                    });
                });
            });
        });
    })();

    // ================================================================
    // COPY BUTTONS
    // ================================================================
    (function initCopyButtons() {
        document.querySelectorAll('[data-copy]').forEach(btn => {
            btn.addEventListener('click', async () => {
                const targetId = btn.dataset.copy;
                let text;

                // Check if target is an element ID
                const el = document.getElementById(targetId);
                if (el) {
                    text = el.textContent;
                } else {
                    text = targetId;
                }

                try {
                    await navigator.clipboard.writeText(text);
                    const original = btn.innerHTML;
                    btn.classList.add('copied');
                    btn.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg> Copied!';
                    setTimeout(() => {
                        btn.classList.remove('copied');
                        btn.innerHTML = original;
                    }, 2000);
                } catch (e) {
                    console.error('Copy failed:', e);
                }
            });
        });
    })();

    // ================================================================
    // HLJS CODE HIGHLIGHTING
    // ================================================================
    if (typeof hljs !== 'undefined') {
        document.querySelectorAll('pre code').forEach(block => {
            hljs.highlightElement(block);
        });
    }
})();
