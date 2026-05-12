"""SeaMagic Agent Benchmark — CLI Runner

Usage:
    python run_benchmark.py --agent-dir outputs/my_agent --agent-name "My Agent"
    python run_benchmark.py --agent-dir outputs/ --compare agents.txt
    python run_benchmark.py --list-tasks
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

import cv2
import numpy as np

sys.path.insert(0, str(Path(__file__).parent))
from evaluator import SMABEvaluator, EvaluationResult


def load_agent_outputs(agent_dir: Path, tasks: dict) -> dict[str, np.ndarray]:
    """Load agent output images from a directory structure: {agent_dir}/{task_id}.png"""
    outputs = {}
    for task_id in tasks:
        img_path = agent_dir / f"{task_id}.png"
        if img_path.exists():
            img = cv2.imread(str(img_path), cv2.IMREAD_UNCHANGED)
            if img is not None:
                outputs[task_id] = img_path
    return outputs


def load_agent_trace(agent_dir: Path) -> dict | None:
    """Load tool call trace from trace.json."""
    trace_path = agent_dir / "trace.json"
    if trace_path.exists():
        with open(trace_path) as f:
            return json.load(f)
    return None


def run_evaluation(
    agent_name: str,
    agent_dir: Path,
    evaluator: SMABEvaluator,
    ground_truth_dir: Path | None = None,
) -> list[EvaluationResult]:
    """Run full evaluation of an agent's outputs."""
    outputs = load_agent_outputs(agent_dir, evaluator.tasks)
    trace = load_agent_trace(agent_dir)

    if not outputs:
        print(f"  No output images found in {agent_dir}")
        return []

    results = []
    for task_id, output_path in outputs.items():
        gt_path = None
        if ground_truth_dir:
            gt = ground_truth_dir / f"{task_id}.png"
            if gt.exists():
                gt_path = str(gt)

        task_trace = None
        if trace and "tool_calls" in trace:
            task_trace = trace

        try:
            result = evaluator.evaluate_task(task_id, str(output_path), ground_truth=gt_path, agent_trace=task_trace)
        except Exception as e:
            result = EvaluationResult(
                task_id=task_id, passed=False,
                pass_reason=f"Evaluation error: {e}",
            )
        results.append(result)

    return results


def print_results(agent_name: str, results: list[EvaluationResult], evaluator: SMABEvaluator):
    """Print formatted evaluation results."""
    if not results:
        print(f"No results for {agent_name}")
        return

    total = len(results)
    passed = sum(1 for r in results if r.passed)

    print(f"\n{'='*60}")
    print(f"Agent: {agent_name}")
    print(f"{'='*60}")
    print(f"Overall: {passed}/{total} passed ({100*passed/total:.0f}%)")
    print()

    header = f"{'Task':<8} {'Pass':<6} {'MSSIM':<8} {'PSNR':<8} {'MAE':<8} {'Reason'}"
    print(header)
    print("-" * len(header))

    for r in results:
        mssim_str = f"{r.mssim:.4f}" if r.mssim is not None else "—"
        psnr_str = f"{r.psnr:.1f}" if r.psnr is not None else "—"
        mae_str = f"{r.mae:.2f}" if r.mae is not None else "—"
        pass_str = "✓" if r.passed else "✗"
        reason = r.pass_reason[:40] if not r.passed else ""
        print(f"{r.task_id:<8} {pass_str:<6} {mssim_str:<8} {psnr_str:<8} {mae_str:<8} {reason}")

    agg = evaluator.compute_aggregate(results)
    print(f"\nAggregate Statistics:")
    print(f"  Mean MSSIM: {agg.get('mean_mssim', '—')}")
    print(f"  Mean PSNR: {agg.get('mean_psnr', '—')}")
    print(f"  Tool Efficiency: {agg.get('mean_tool_efficiency', 0):.2f}")

    if agg.get("by_category"):
        print(f"\n  Per-Category:")
        for cat, stats in agg["by_category"].items():
            print(f"    {cat}: {stats['passed']}/{stats['total']} ({100*stats['pass_rate']:.0f}%)")


def export_results_json(
    agent_name: str, results: list[EvaluationResult],
    evaluator: SMABEvaluator, output_path: Path,
):
    """Export results as JSON."""
    records = []
    for r in results:
        task = evaluator.tasks.get(r.task_id, {})
        records.append({
            "agent": agent_name,
            **r.to_dict(),
            "category": task.get("category", ""),
            "difficulty": task.get("difficulty", 0),
        })

    with open(output_path, "w") as f:
        json.dump(records, f, indent=2)
    print(f"Results exported to {output_path}")


def export_training_jsonl(
    agent_name: str, results: list[EvaluationResult],
    evaluator: SMABEvaluator, output_path: Path,
):
    """Export results as JSONL for training."""
    with open(output_path, "w") as f:
        for r in results:
            task = evaluator.tasks.get(r.task_id, {})
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
            }
            f.write(json.dumps(record) + "\n")
    print(f"Training data exported to {output_path} ({len(results)} records)")


def main():
    parser = argparse.ArgumentParser(description="SeaMagic Agent Benchmark Runner")
    parser.add_argument("--agent-dir", type=Path, help="Directory with agent output images ({task_id}.png)")
    parser.add_argument("--agent-name", type=str, default="", help="Name of the agent being evaluated")
    parser.add_argument("--ground-truth-dir", type=Path, default=None, help="Ground truth image directory")
    parser.add_argument("--benchmark-json", type=Path, default=None, help="Path to benchmark_v1.json")
    parser.add_argument("--export-json", type=Path, default=None, help="Export results as JSON")
    parser.add_argument("--export-jsonl", type=Path, default=None, help="Export training data as JSONL")
    parser.add_argument("--compare", type=str, default=None, help="File with agent names (one per line) to compare")
    parser.add_argument("--list-tasks", action="store_true", help="List all benchmark tasks")
    parser.add_argument("--quiet", action="store_true", help="Only print aggregate results")

    args = parser.parse_args()

    base = Path(__file__).parent
    benchmark_json = args.benchmark_json or base / "tasks" / "benchmark_v1.json"
    gt_dir = args.ground_truth_dir or base / "ground_truth"

    evaluator = SMABEvaluator(str(benchmark_json))

    if args.list_tasks:
        print(f"{'ID':<8} {'Category':<22} {'Name':<30} {'Diff':<6} {'Eval Mode'}")
        print("-" * 80)
        for tid, task in sorted(evaluator.tasks.items()):
            print(f"{tid:<8} {task['category']:<22} {task['name']:<30} {'★'*task['difficulty']:<6} {task['eval_mode']}")
        return

    all_agent_results: dict[str, list[EvaluationResult]] = {}

    if args.agent_dir:
        agent_name = args.agent_name or args.agent_dir.name
        results = run_evaluation(agent_name, args.agent_dir, evaluator, gt_dir)
        all_agent_results[agent_name] = results

        if not args.quiet:
            print_results(agent_name, results, evaluator)

        if args.export_json:
            export_results_json(agent_name, results, evaluator, args.export_json)
        if args.export_jsonl:
            export_training_jsonl(agent_name, results, evaluator, args.export_jsonl)

    if args.compare:
        with open(args.compare) as f:
            agent_names = [line.strip() for line in f if line.strip()]

        for name in agent_names:
            agent_dir = Path(name) if os.path.isabs(name) else base / "outputs" / name
            results = run_evaluation(name, agent_dir, evaluator, gt_dir)
            all_agent_results[name] = results

            if not args.quiet:
                print_results(name, results, evaluator)

    if len(all_agent_results) > 1:
        print(f"\n{'='*60}")
        print("COMPARISON SUMMARY")
        print(f"{'='*60}")
        print(f"{'Agent':<20} {'Pass Rate':<15} {'Mean MSSIM':<12} {'Mean PSNR'}")
        print("-" * 60)
        for name, results in all_agent_results.items():
            total = len(results)
            passed = sum(1 for r in results if r.passed)
            mssims = [r.mssim for r in results if r.mssim is not None]
            psnrs = [r.psnr for r in results if r.psnr is not None]
            pr = f"{passed}/{total} ({100*passed/total:.0f}%)" if total else "—"
            ms = f"{sum(mssims)/len(mssims):.4f}" if mssims else "—"
            ps = f"{sum(psnrs)/len(psnrs):.1f}" if psnrs else "—"
            print(f"{name:<20} {pr:<15} {ms:<12} {ps}")


if __name__ == "__main__":
    main()
