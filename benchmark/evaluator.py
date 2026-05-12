"""
SeaMagic Agent Benchmark (SMAB) Evaluator

Computes image similarity metrics between agent outputs and ground truth targets.
Supports: MSSIM, PSNR, LPIPS, pixel diff, color histogram, text extraction,
          dimension matching, and constraint checking.

Usage:
    from evaluator import SMABEvaluator
    ev = SMABEvaluator()
    scores = ev.evaluate_task(task_id, agent_output_path, ground_truth_path)
"""

from __future__ import annotations

import json
import math
import os
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import cv2
import numpy as np
from PIL import Image
from skimage.metrics import peak_signal_noise_ratio, structural_similarity


@dataclass
class EvaluationResult:
    task_id: str
    passed: bool
    mssim: float | None = None
    psnr: float | None = None
    mae: float | None = None
    pixel_diff_ratio: float | None = None
    color_hist_chi2: float | None = None
    dimensions_match: bool | None = None
    text_match_score: float | None = None
    tool_calls: int = 0
    tool_efficiency: float = 0.0
    error_count: int = 0
    error_recovery: bool = False
    llm_score: float | None = None
    pass_reason: str = ""
    extra_metrics: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        d = {
            "task_id": self.task_id,
            "passed": self.passed,
            "mssim": self.mssim,
            "psnr": self.psnr,
            "mae": self.mae,
            "pixel_diff_ratio": self.pixel_diff_ratio,
            "color_hist_chi2": self.color_hist_chi2,
            "dimensions_match": self.dimensions_match,
            "text_match_score": self.text_match_score,
            "tool_calls": self.tool_calls,
            "tool_efficiency": self.tool_efficiency,
            "error_count": self.error_count,
            "error_recovery": self.error_recovery,
            "llm_score": self.llm_score,
            "pass_reason": self.pass_reason,
            "extra_metrics": self.extra_metrics,
        }
        return {k: v for k, v in d.items() if v is not None}


class SMABEvaluator:
    """Evaluates agent outputs against benchmark task definitions."""

    def __init__(self, benchmark_path: str | None = None):
        base = Path(__file__).parent
        self.benchmark_path = (
            Path(benchmark_path) if benchmark_path else base / "tasks" / "benchmark_v1.json"
        )
        self.tasks = self._load_tasks()
        # Try to import lpips — optional, may not have torch installed
        self._lpips = None
        self._lpips_alex = None
        try:
            import lpips
            self._lpips = lpips.LPIPS(net="vgg")
            self._lpips_alex = lpips.LPIPS(net="alex")
        except ImportError:
            pass

    def _load_tasks(self) -> dict[str, dict]:
        with open(self.benchmark_path) as f:
            data = json.load(f)
        return {t["id"]: t for t in data["tasks"]}

    def evaluate_task(
        self,
        task_id: str,
        agent_output: str | np.ndarray,
        ground_truth: str | np.ndarray | None = None,
        agent_trace: dict | None = None,
    ) -> EvaluationResult:
        """Evaluate a single task output."""
        task = self.tasks.get(task_id)
        if task is None:
            raise ValueError(f"Unknown task: {task_id}")

        # Load images
        agent_img = self._load_image(agent_output)
        if ground_truth is not None:
            gt_img = self._load_image(ground_truth)
        else:
            gt_img = self._generate_ground_truth(task)

        thresholds = task.get("thresholds", {})
        eval_mode = task.get("eval_mode", "exact_match")

        # Dimension check
        dims_match = None
        if "dimension_match" in thresholds and gt_img is not None:
            expected = thresholds["dimension_match"]
            dims_match = (
                agent_img.shape[1] == expected[0] and agent_img.shape[0] == expected[1]
            )

        if eval_mode == "exact_match" and gt_img is not None:
            return self._eval_exact_match(task_id, agent_img, gt_img, thresholds, dims_match, agent_trace)
        elif eval_mode == "constraint_satisfaction":
            return self._eval_constraints(
                task_id, agent_img, gt_img, thresholds, dims_match, agent_trace
            )
        elif eval_mode == "llm_judge":
            return self._eval_llm_judge(task_id, agent_img, thresholds, agent_trace)
        else:
            return self._eval_constraints(task_id, agent_img, gt_img, thresholds, dims_match, agent_trace)

    def _eval_exact_match(
        self,
        task_id: str,
        agent_img: np.ndarray,
        gt_img: np.ndarray,
        thresholds: dict,
        dims_match: bool | None,
        agent_trace: dict | None,
    ) -> EvaluationResult:
        scores = self._compute_all_scores(agent_img, gt_img)
        passed = True
        reasons: list[str] = []

        if "mssim_min" in thresholds:
            mssim_val = scores["mssim"] if scores["mssim"] is not None else 0.0
            if mssim_val < thresholds["mssim_min"]:
                passed = False
                reasons.append(
                    f"MSSIM {mssim_val:.3f} < {thresholds['mssim_min']}"
                )

        if "psnr_min" in thresholds:
            psnr_val = scores["psnr"] or 0
            if psnr_val == float("inf"):
                psnr_val = 999.0
            if psnr_val < thresholds["psnr_min"]:
                passed = False
                reasons.append(
                    f"PSNR {scores['psnr']:.1f} < {thresholds['psnr_min']}"
                )

        if "mae_max" in thresholds:
            mae_val = scores["mae"] if scores["mae"] is not None else float("inf")
            if mae_val > thresholds["mae_max"]:
                passed = False
                reasons.append(f"MAE {mae_val:.2f} > {thresholds['mae_max']}")

        if "pixel_diff_max" in thresholds:
            pdr = scores["pixel_diff_ratio"] if scores["pixel_diff_ratio"] is not None else float("inf")
            if pdr > thresholds["pixel_diff_max"]:
                passed = False
                reasons.append(
                    f"Pixel diff {pdr:.3f} > {thresholds['pixel_diff_max']}"
                )

        if "color_hist_max" in thresholds:
            chi2 = scores["color_hist_chi2"] if scores["color_hist_chi2"] is not None else float("inf")
            if chi2 > thresholds["color_hist_max"]:
                passed = False
                reasons.append(
                    f"Color hist {chi2:.3f} > {thresholds['color_hist_max']}"
                )

        if dims_match is not None and not dims_match:
            passed = False
            reasons.append(
                f"Dimension mismatch: got {agent_img.shape[1]}x{agent_img.shape[0]}"
            )

        # Text presence check using OCR
        text_score = None
        if "text_presence" in thresholds:
            expected_text = thresholds["text_presence"]
            if isinstance(expected_text, str):
                expected_text = [expected_text]
            extracted = self._extract_text(agent_img)
            matched = [t for t in expected_text if any(t.lower() in e.lower() for e in extracted)]
            text_score = len(matched) / len(expected_text)
            if text_score < 1.0:
                passed = False
                reasons.append(f"Text mismatch: found {extracted}, expected {expected_text}")

        trace_metrics = self._parse_trace(agent_trace)

        return EvaluationResult(
            task_id=task_id,
            passed=passed,
            mssim=scores["mssim"],
            psnr=scores["psnr"],
            mae=scores["mae"],
            pixel_diff_ratio=scores["pixel_diff_ratio"],
            color_hist_chi2=scores["color_hist_chi2"],
            dimensions_match=dims_match,
            text_match_score=text_score,
            tool_calls=trace_metrics["tool_calls"],
            tool_efficiency=trace_metrics["tool_efficiency"],
            error_count=trace_metrics["error_count"],
            error_recovery=trace_metrics["error_recovery"],
            pass_reason="; ".join(reasons) if reasons else "All metrics within thresholds",
        )

    def _eval_constraints(
        self,
        task_id: str,
        agent_img: np.ndarray,
        gt_img: np.ndarray | None,
        thresholds: dict,
        dims_match: bool | None,
        agent_trace: dict | None,
    ) -> EvaluationResult:
        """Evaluate constraint-satisfaction tasks (no exact ground truth needed)."""
        passed = True
        reasons: list[str] = []
        extra: dict[str, Any] = {}

        scores = {}
        if gt_img is not None:
            scores = self._compute_all_scores(agent_img, gt_img)

        if dims_match is not None and not dims_match:
            passed = False
            expected = thresholds.get("dimension_match", [])
            reasons.append(
                f"Dimension mismatch: expected {expected}, got {agent_img.shape[1]}x{agent_img.shape[0]}"
            )

        if "text_presence" in thresholds:
            expected_text = thresholds["text_presence"]
            if isinstance(expected_text, str):
                expected_text = [expected_text]
            extracted = self._extract_text(agent_img)
            matched = [t for t in expected_text if any(t.lower() in e.lower() for e in extracted)]
            extra["text_match_ratio"] = len(matched) / len(expected_text)
            if extra["text_match_ratio"] < (
                1.0 if thresholds.get("text_presence_strict", False) else 0.5
            ):
                passed = False
                reasons.append(f"Text not found: expected {expected_text}, got {extracted}")

        if "color_bars" in thresholds:
            expected_colors = thresholds["color_bars"]
            detected_colors = self._detect_color_bars(agent_img)
            extra["detected_colors"] = detected_colors
            extra["color_match_ratio"] = len(
                set(expected_colors) & set(detected_colors)
            ) / len(expected_colors)

        if "border_presence" in thresholds:
            has_border = self._detect_border(agent_img)
            extra["has_border"] = has_border
            if thresholds["border_presence"] and not has_border:
                passed = False
                reasons.append("Border not detected")

        if "tool_sequence" in thresholds:
            seq = self._get_tool_sequence(agent_trace)
            expected_seq = thresholds["tool_sequence"]
            extra["tool_sequence_match"] = seq[: len(expected_seq)] == expected_seq

        if "minimum_effects_count" in thresholds:
            effects = self._count_effects(agent_trace)
            extra["effects_count"] = effects
            if effects < thresholds["minimum_effects_count"]:
                passed = False
                reasons.append(
                    f"Only {effects} effects applied, need {thresholds['minimum_effects_count']}"
                )

        if "tool_calls_min" in thresholds:
            tc = self._count_tool_calls(agent_trace)
            extra["actual_tool_calls"] = tc
            if tc < thresholds["tool_calls_min"]:
                passed = False
                reasons.append(f"Only {tc} tool calls, need >= {thresholds['tool_calls_min']}")

        trace_metrics = self._parse_trace(agent_trace)

        return EvaluationResult(
            task_id=task_id,
            passed=passed,
            mssim=scores.get("mssim"),
            psnr=scores.get("psnr"),
            mae=scores.get("mae"),
            dimensions_match=dims_match,
            tool_calls=trace_metrics["tool_calls"],
            tool_efficiency=trace_metrics["tool_efficiency"],
            error_count=trace_metrics["error_count"],
            error_recovery=trace_metrics["error_recovery"],
            extra_metrics=extra,
            pass_reason="; ".join(reasons) if reasons else "All constraints satisfied",
        )

    def _eval_llm_judge(
        self,
        task_id: str,
        agent_img: np.ndarray,
        thresholds: dict,
        agent_trace: dict | None,
    ) -> EvaluationResult:
        """Placeholder for LLM-as-judge evaluation. Requires manual or external LLM call."""
        # LLM judge requires an external call.
        return EvaluationResult(
            task_id=task_id,
            passed=False,  # Mark as needing manual review
            llm_score=None,
            pass_reason="LLM judge requires external evaluation — mark as pending",
        )

    def _compute_all_scores(self, img1: np.ndarray, img2: np.ndarray) -> dict[str, Any]:
        """Compute all pixel-level comparison metrics."""
        # Resize to common size if different
        h1, w1 = img1.shape[:2]
        h2, w2 = img2.shape[:2]
        if h1 != h2 or w1 != w2:
            target_size = (min(w1, w2), min(h1, h2))
            img1 = cv2.resize(img1, target_size)
            img2 = cv2.resize(img2, target_size)

        # Ensure RGB
        if img1.shape[2] == 4:
            img1 = cv2.cvtColor(img1, cv2.COLOR_BGRA2BGR)
        if img2.shape[2] == 4:
            img2 = cv2.cvtColor(img2, cv2.COLOR_BGRA2BGR)

        # MSSIM
        try:
            mssim = structural_similarity(
                img1, img2, channel_axis=2, data_range=255
            )
        except Exception:
            mssim = None

        # PSNR
        try:
            psnr = peak_signal_noise_ratio(img1, img2, data_range=255)
            if psnr == float("inf"):
                psnr = 100.0
        except Exception:
            psnr = None

        # MAE
        mae = float(np.mean(np.abs(img1.astype(float) - img2.astype(float))))

        # Pixel diff ratio
        diff = np.any(img1 != img2, axis=2)
        pixel_diff_ratio = float(np.mean(diff))

        # Color histogram distance
        hist1 = cv2.calcHist(
            [cv2.cvtColor(img1, cv2.COLOR_BGR2HSV)], [0, 1], None, [16, 16], [0, 180, 0, 256]
        )
        hist2 = cv2.calcHist(
            [cv2.cvtColor(img2, cv2.COLOR_BGR2HSV)], [0, 1], None, [16, 16], [0, 180, 0, 256]
        )
        hist1 = cv2.normalize(hist1, hist1)
        hist2 = cv2.normalize(hist2, hist2)
        color_hist_chi2 = float(cv2.compareHist(hist1, hist2, cv2.HISTCMP_CHISQR))

        return {
            "mssim": mssim,
            "psnr": psnr,
            "mae": mae,
            "pixel_diff_ratio": pixel_diff_ratio,
            "color_hist_chi2": color_hist_chi2,
        }

    def _extract_text(self, img: np.ndarray) -> list[str]:
        """Extract probable text regions from image using simple OCR."""
        try:
            import pytesseract  # type: ignore[import-untyped]
            pil_img = Image.fromarray(cv2.cvtColor(img, cv2.COLOR_BGR2RGB))
            config = "--psm 6"
            detected = pytesseract.image_to_string(pil_img, config=config)
            return [line.strip() for line in detected.split("\n") if line.strip()]
        except ImportError:
            # Fallback: detect high-contrast text regions and return bounding boxes as proxy
            gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
            _, binary = cv2.threshold(gray, 0, 255, cv2.THRESH_BINARY + cv2.THRESH_OTSU)
            contours, _ = cv2.findContours(binary, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
            text_regions = []
            for cnt in contours:
                x, y, w, h = cv2.boundingRect(cnt)
                if w > 30 and h > 10 and w / h > 2:
                    text_regions.append(f"region_{x}_{y}_{w}x{h}")
            return text_regions if text_regions else ["no_text_detected"]

    def _detect_color_bars(self, img: np.ndarray) -> list[str]:
        """Detect dominant colors in equal vertical bars."""
        h, w = img.shape[:2]
        bar_w = w // 4
        colors: list[str] = []
        for i in range(4):
            region = img[:, i * bar_w : (i + 1) * bar_w]
            avg = np.mean(region.reshape(-1, 3), axis=0)
            colors.append(self._rgb_to_hex(int(avg[2]), int(avg[1]), int(avg[0])))
        return colors

    def _detect_border(self, img: np.ndarray) -> bool:
        """Check if image has a non-uniform border."""
        h, w = img.shape[:2]
        if h < 50 or w < 50:
            return False
        # Sample edge pixels
        top = img[0, :, :]
        bottom = img[-1, :, :]
        left = img[:, 0, :]
        right = img[:, -1, :]
        # Check variance within edges
        edges = [top, bottom, left, right]
        for edge in edges:
            if np.std(edge.astype(float)) > 5:
                return True
        return False

    def _get_tool_sequence(self, trace: dict | None) -> list[str]:
        if not trace or "tool_calls" not in trace:
            return []
        return [t.get("tool", "") for t in trace["tool_calls"]]

    def _count_effects(self, trace: dict | None) -> int:
        if not trace or "tool_calls" not in trace:
            return 0
        effects = {"blur", "brightness", "contrast", "grayscale", "tint", "vignette",
                   "duotone", "noise", "liquify", "warp", "glitch", "scanlines",
                   "halftone", "posterize", "edge-detect", "chromatic", "kaleidoscope",
                   "emboss", "solarize", "pixel-sort"}
        count = 0
        for call in trace["tool_calls"]:
            op = call.get("operation", "").lower()
            if any(e in op for e in effects):
                count += 1
            # Also check in run_steel scripts
            script = call.get("script", "").lower()
            for e in effects:
                if f"image-{e}" in script:
                    count += 1
        return count

    def _count_tool_calls(self, trace: dict | None) -> int:
        if not trace:
            return 0
        return len(trace.get("tool_calls", []))

    def _parse_trace(self, trace: dict | None) -> dict[str, Any]:
        if not trace:
            return {"tool_calls": 0, "tool_efficiency": 0.0, "error_count": 0, "error_recovery": False}

        tool_calls = len(trace.get("tool_calls", []))
        error_count = sum(
            1 for t in trace.get("tool_calls", []) if t.get("error")
        )
        # Recovery: error_count before successful final call
        calls = trace.get("tool_calls", [])
        error_recovery = False
        for i, call in enumerate(calls):
            if call.get("error"):
                for j in range(i + 1, len(calls)):
                    if not calls[j].get("error"):
                        error_recovery = True
                        break

        # Tool efficiency: ideal is 1 call per major operation
        ideal = trace.get("ideal_tool_calls", max(1, tool_calls))
        efficiency = ideal / max(tool_calls, 1) if tool_calls > 0 else 0.0

        return {
            "tool_calls": tool_calls,
            "tool_efficiency": min(efficiency, 1.0),
            "error_count": error_count,
            "error_recovery": error_recovery,
        }

    def _generate_ground_truth(self, task: dict) -> np.ndarray | None:
        """Generate ground truth image for a task by replicating the known-correct operation."""
        base = Path(__file__).parent

        gt_params = task.get("ground_truth_params")
        gt_script = task.get("ground_truth_script")
        input_path = task.get("input")

        if gt_script:
            script_path = base.parent / gt_script
            if script_path.exists():
                return self._run_ground_truth_script(script_path, input_path)

        if gt_params:
            return self._run_ground_truth_params(gt_params, input_path)

        return None

    def _run_ground_truth_script(self, script_path: Path, input_path: str | None) -> np.ndarray:
        """Execute a Steel Scheme script using cargo run --example to generate ground truth."""
        import subprocess
        import base64

        project_root = Path(__file__).parent.parent
        out = tempfile.NamedTemporaryFile(suffix=".png", delete=False)
        out.close()

        # Scheme execution is only available via cargo run --example run_example
        # Usage: cargo run --example run_example -- scheme <input> <script.scm> <output.png>
        if input_path:
            inp = str(project_root / input_path) if not os.path.isabs(input_path) else input_path
        else:
            # For generation tasks, provide a minimal valid input image (scheme needs something to bind)
            dummy = tempfile.NamedTemporaryFile(suffix=".png", delete=False)
            dummy.close()
            blank = np.zeros((10, 10, 3), dtype=np.uint8)
            cv2.imwrite(dummy.name, blank)
            inp = dummy.name

        cmd = ["cargo", "run", "--release", "--example", "run_example", "--",
               "scheme", inp, str(script_path), out.name]

        try:
            result = subprocess.run(
                cmd, capture_output=True, text=True, timeout=60,
                cwd=str(project_root),
            )
            if result.returncode != 0:
                pass  # Fall through to blank image
        except Exception:
            pass

        img = cv2.imread(out.name, cv2.IMREAD_UNCHANGED)
        os.unlink(out.name)
        if not input_path and os.path.exists(inp):
            os.unlink(inp)
        if img is None:
            return np.zeros((512, 512, 3), dtype=np.uint8)
        return img

    def _run_ground_truth_params(self, params: dict, input_path: str | None) -> np.ndarray:
        """Execute a single operation using seamagic CLI."""
        import subprocess

        seamagic = Path(__file__).parent.parent / "target" / "release" / "seamagic"
        out = tempfile.NamedTemporaryFile(suffix=".png", delete=False)
        out.close()
        project_root = Path(__file__).parent.parent

        # Create a small dummy input for generation tasks
        dummy_input = None
        def _make_dummy():
            nonlocal dummy_input
            dummy = tempfile.NamedTemporaryFile(suffix=".png", delete=False)
            dummy.close()
            blank = np.zeros((10, 10, 3), dtype=np.uint8)
            cv2.imwrite(dummy.name, blank)
            dummy_input = dummy.name
            return dummy.name

        if "tool" in params and params["tool"] == "create_image":
            kind = params.get("type", "canvas")
            cmd = [str(seamagic), kind, "-o", out.name]
            for key, val in params.items():
                if key in ("tool", "type"):
                    continue
                flag = f"--{key.replace('_', '-')}"
                cmd += [flag, str(val)]
        elif "operations" in params:
            # Multi-step: generate via scheme
            ops = params["operations"]
            scheme = "(define base \"input\")\n"
            current = "base"
            for i, op in enumerate(ops):
                if op["operation"] == "brightness":
                    scheme += f"(define s{i} (image-brightness {current} {op['value']}.0))\n"
                elif op["operation"] == "contrast":
                    scheme += f"(define s{i} (image-contrast {current} {op['value']}.0))\n"
                elif op["operation"] == "edge-detect":
                    scheme += f"(define s{i} (image-edge-detect {current} 20 #f))\n"
                else:
                    scheme += f"(define s{i} (image-{op['operation']} {current}))\n"
                current = f"s{i}"
            scheme += f"(define result {current})\n"
            script_file = tempfile.NamedTemporaryFile(mode="w", suffix=".scm", delete=False)
            script_file.write(scheme)
            script_file.close()

            # Use cargo run --example for scheme execution
            project_root = Path(__file__).parent.parent
            inp = str(project_root / input_path) if input_path and not os.path.isabs(input_path) else (input_path or _make_dummy())
            cmd = ["cargo", "run", "--release", "--example", "run_example", "--",
                   "scheme", inp, script_file.name, out.name]
            subprocess.run(cmd, capture_output=True, text=True, timeout=60, cwd=str(project_root))
            os.unlink(script_file.name)
        else:
            # Single operation
            op = params.get("operation", "blur")
            if not input_path:
                _make_dummy()
                inp = dummy_input
            else:
                inp = str(project_root / input_path) if not os.path.isabs(input_path) else input_path
            cmd = [str(seamagic), op, inp, "-o", out.name]
            for key, val in params.items():
                if key == "operation":
                    continue
                flag = f"--{key.replace('_', '-')}"
                cmd += [flag, str(val)]

        try:
            subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        except Exception:
            pass

        img = cv2.imread(out.name, cv2.IMREAD_UNCHANGED)
        os.unlink(out.name)
        if img is None:
            return np.zeros((512, 512, 3), dtype=np.uint8)
        return img

    def _load_image(self, path_or_array: str | np.ndarray) -> np.ndarray:
        if isinstance(path_or_array, np.ndarray):
            return path_or_array
        img = cv2.imread(str(path_or_array), cv2.IMREAD_UNCHANGED)
        if img is None:
            raise FileNotFoundError(f"Could not load image: {path_or_array}")
        return img

    @staticmethod
    def _rgb_to_hex(r: int, g: int, b: int) -> str:
        return f"#{r:02x}{g:02x}{b:02x}"

    def compute_aggregate(self, results: list[EvaluationResult]) -> dict[str, Any]:
        """Compute aggregate statistics across all evaluated tasks."""
        if not results:
            return {}

        total = len(results)
        passed = sum(1 for r in results if r.passed)
        by_category: dict[str, list[EvaluationResult]] = {}
        for r in results:
            task = self.tasks.get(r.task_id, {})
            cat = task.get("category", "unknown")
            by_category.setdefault(cat, []).append(r)

        cat_stats = {}
        for cat, rs in by_category.items():
            cat_passed = sum(1 for r in rs if r.passed)
            mssims = [r.mssim for r in rs if r.mssim is not None]
            cat_stats[cat] = {
                "total": len(rs),
                "passed": cat_passed,
                "pass_rate": cat_passed / len(rs) if rs else 0.0,
                "mean_mssim": sum(mssims) / len(mssims) if mssims else None,
            }

        diffs = [r for r in results if r.task_id.startswith(("S", "P", "G"))]
        easy = [r for r in diffs if self.tasks.get(r.task_id, {}).get("difficulty", 5) <= 2]
        medium = [r for r in diffs if 3 <= self.tasks.get(r.task_id, {}).get("difficulty", 5) <= 4]
        hard = [r for r in diffs if self.tasks.get(r.task_id, {}).get("difficulty", 5) >= 5]

        return {
            "total_tasks": total,
            "passed": passed,
            "overall_pass_rate": passed / total if total else 0.0,
            "by_category": cat_stats,
            "by_difficulty": {
                "easy": {"total": len(easy), "passed": sum(1 for r in easy if r.passed)} if easy else None,
                "medium": {"total": len(medium), "passed": sum(1 for r in medium if r.passed)} if medium else None,
                "hard": {"total": len(hard), "passed": sum(1 for r in hard if r.passed)} if hard else None,
            },
            "mean_mssim": sum(r.mssim for r in results if r.mssim is not None) / len([r for r in results if r.mssim is not None]) if any(r.mssim is not None for r in results) else None,
            "mean_psnr": sum(r.psnr for r in results if r.psnr is not None) / len([r for r in results if r.psnr is not None]) if any(r.psnr is not None for r in results) else None,
            "mean_tool_efficiency": sum(r.tool_efficiency for r in results) / len(results) if results else 0.0,
            "total_error_count": sum(r.error_count for r in results),
            "error_recovery_rate": sum(1 for r in results if r.error_recovery) / len(results) if results else 0.0,
        }

    def export_training_jsonl(
        self,
        results: list[EvaluationResult],
        agent_name: str = "",
        output_path: str | None = None,
        include_b64: bool = False,
    ) -> str:
        """Export evaluation results as JSONL for RLHF/SFT training.

        Args:
            results: List of evaluation results.
            agent_name: Agent identifier.
            output_path: Output file path. If None, uses benchmark/outputs/training_data.jsonl.
            include_b64: If True, include base64-encoded output images (large).

        Returns:
            Path to the exported JSONL file.
        """
        if output_path is None:
            output_path = str(Path(__file__).parent / "outputs" / "training_data.jsonl")

        os.makedirs(os.path.dirname(output_path), exist_ok=True)

        records = []
        for r in results:
            task = self.tasks.get(r.task_id, {})
            record = {
                "task_id": r.task_id,
                "agent": agent_name,
                "category": task.get("category", ""),
                "difficulty": task.get("difficulty", 0),
                "eval_mode": task.get("eval_mode", ""),
                "prompt": task.get("description", ""),
                "thresholds": task.get("thresholds", {}),
                "scores": {
                    "mssim": r.mssim,
                    "psnr": r.psnr,
                    "mae": r.mae,
                    "pixel_diff_ratio": r.pixel_diff_ratio,
                    "color_hist_chi2": r.color_hist_chi2,
                    "dimensions_match": r.dimensions_match,
                    "text_match_score": r.text_match_score,
                },
                "passed": r.passed,
                "pass_reason": r.pass_reason,
                "tool_efficiency": r.tool_efficiency,
                "error_count": r.error_count,
                "error_recovery": r.error_recovery,
                "llm_score": r.llm_score,
            }
            records.append(record)

        with open(output_path, "w") as f:
            for rec in records:
                f.write(json.dumps(rec) + "\n")

        return output_path
