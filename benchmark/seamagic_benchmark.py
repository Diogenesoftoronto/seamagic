"""SeaMagic Agent Benchmark (SMAB) v1.0 — Interactive Evaluation Notebook

Run with: marimo run benchmark/seamagic_benchmark.py
Edit with: marimo edit benchmark/seamagic_benchmark.py
"""

import marimo

__generated_with = "0.13.11"
app = marimo.App(width="full")


@app.cell
def _():
    import json
    import os
    import sys
    from pathlib import Path

    import cv2
    import marimo as mo
    import matplotlib
    import matplotlib.pyplot as plt
    import numpy as np
    import pandas as pd
    from PIL import Image

    matplotlib.use("Agg")

    BASE = Path(__file__).parent if "__file__" in dir() else Path(".")
    BENCHMARK_DIR = BASE
    TASKS_PATH = BENCHMARK_DIR / "tasks" / "benchmark_v1.json"
    GT_DIR = BENCHMARK_DIR / "ground_truth"
    FIXTURES_DIR = BENCHMARK_DIR / "fixtures"
    OUTPUTS_DIR = BENCHMARK_DIR / "outputs"

    with open(TASKS_PATH) as f:
        benchmark_data = json.load(f)

    tasks = benchmark_data["tasks"]
    categories = benchmark_data["categories"]
    metrics_info = benchmark_data["metrics"]

    sys.path.insert(0, str(BENCHMARK_DIR))
    from evaluator import SMABEvaluator, EvaluationResult

    evaluator = SMABEvaluator(str(TASKS_PATH))
    return (
        BASE, BENCHMARK_DIR, FIXTURES_DIR, GT_DIR, OUTPUTS_DIR,
        TASKS_PATH, benchmark_data, categories, evaluator, metrics_info,
        tasks, mo, np, pd, plt, cv2, Image, json, os, Path,
    )


@app.cell
def _(mo, categories, metrics_info):
    _intro = mo.md(
        """# SeaMagic Agent Benchmark (SMAB) v1.0

        A comprehensive benchmark for evaluating AI agents' ability to use the
        **SeaMagic MCP server** for image manipulation.

        ## Dimensions Tested

        | Dimension | Category | Skills |
        |-----------|----------|--------|
        | **Coding** | Scheme Pipeline | DSL programming, functional composition |
        | **Instruction Following** | Single Edit, Generation | Parameter understanding, spatial reasoning |
        | **Agentic Workflows** | Agentic Workflow, Image Understanding | Iterative improvement, error recovery |
        | **Image Understanding** | Image Understanding | Visual analysis, dimension extraction |

        ## Evaluation Modes

        - **Exact Match**: Pixel-level comparison (MSSIM, PSNR, MAE)
        - **Constraint Satisfaction**: Structural/visual constraint checking
        - **LLM Judge**: Subjective quality via vision-language model
        """
    )
    mo.vstack([_intro])
    return


@app.cell
def _(mo, pd, tasks):
    _rows = []
    for t in tasks:
        gt_exists = ""
        if t.get("eval_mode") == "exact_match":
            gt_path = f"ground_truth/{t['id']}.png"
            gt_exists = "✓" if True else "✗"
        _rows.append({
            "ID": t["id"],
            "Category": t["category"],
            "Name": t["name"],
            "Difficulty": "★" * t["difficulty"] + "☆" * (5 - t["difficulty"]),
            "Eval Mode": t["eval_mode"],
            "GT": gt_exists,
        })

    _df = pd.DataFrame(_rows)
    mo.ui.table(_df, selection=None, page_size=25, label="Benchmark Tasks")
    return


@app.cell
def _(mo, tasks, GT_DIR, FIXTURES_DIR, Image, plt, np, Path):
    _task_selector = mo.ui.dropdown(
        options=[f"{t['id']}: {t['name']}" for t in tasks],
        value=f"{tasks[0]['id']}: {tasks[0]['name']}",
        label="Select task to visualize",
    )

    _selected_id = _task_selector.value.split(":")[0] if _task_selector.value else ""
    _task = next((t for t in tasks if t["id"] == _selected_id), None)

    def _display_task(task):
        if task is None:
            return mo.md("No task selected")

        parts = [mo.md(f"## {task['id']}: {task['name']}")]
        parts.append(mo.md(f"**Category**: {task['category']} | **Difficulty**: {'★' * task['difficulty']} | **Eval Mode**: {task['eval_mode']}"))
        parts.append(mo.md(f"**Description**: {task['description']}"))

        fig, axes = plt.subplots(1, 3, figsize=(15, 5))

        input_path = task.get("input")
        if input_path:
            inp = FIXTURES_DIR / Path(input_path).name
            if inp.exists():
                img = Image.open(inp)
                axes[0].imshow(np.array(img))
                axes[0].set_title(f"Input\n{img.size[0]}x{img.size[1]}")
            else:
                axes[0].text(0.5, 0.5, "Input not found", ha="center", va="center")
                axes[0].set_title("Input")
        else:
            axes[0].text(0.5, 0.5, "No input\n(generation task)", ha="center", va="center")
            axes[0].set_title("Input")

        gt_path = GT_DIR / f"{task['id']}.png"
        if gt_path.exists():
            gt = Image.open(gt_path)
            axes[1].imshow(np.array(gt))
            axes[1].set_title(f"Ground Truth\n{gt.size[0]}x{gt.size[1]}")
        else:
            axes[1].text(0.5, 0.5, "No GT generated", ha="center", va="center")
            axes[1].set_title("Ground Truth")

        thresholds = task.get("thresholds", {})
        thresh_text = "\n".join(f"{k}: {v}" for k, v in thresholds.items())
        axes[2].text(0.5, 0.5, thresh_text, ha="center", va="center", fontsize=10, family="monospace")
        axes[2].set_title("Thresholds")

        for ax in axes:
            ax.axis("off")

        plt.tight_layout()
        parts.append(mo.as_html(fig))
        plt.close(fig)

        return mo.vstack(parts)

    _display = _display_task(_task) if _task else mo.md("Select a task")
    mo.vstack([_task_selector, _display])
    return


@app.cell
def _(mo, np, cv2, GT_DIR, FIXTURES_DIR, Image, Path, tasks):
    mo.md("## Ground Truth Gallery")
    return


@app.cell
def _(mo, GT_DIR, Image, np, plt, tasks):
    _exact_tasks = [t for t in tasks if t.get("eval_mode") == "exact_match" and (GT_DIR / f"{t['id']}.png").exists()]

    if not _exact_tasks:
        mo.md("No ground truth images generated yet.")
    else:
        n = len(_exact_tasks)
        cols = min(n, 6)
        rows_count = (n + cols - 1) // cols
        fig, axes = plt.subplots(rows_count, cols, figsize=(3 * cols, 3 * rows_count))
        if rows_count == 1:
            axes = [axes] if cols == 1 else axes
        for idx, task in enumerate(_exact_tasks):
            r, c = divmod(idx, cols)
            ax = axes[r][c] if rows_count > 1 else axes[c]
            gt_path = GT_DIR / f"{task['id']}.png"
            gt = np.array(Image.open(gt_path))
            ax.imshow(gt)
            ax.set_title(f"{task['id']}", fontsize=9)
            ax.axis("off")
        for idx in range(n, rows_count * cols):
            r, c = divmod(idx, cols)
            ax = axes[r][c] if rows_count > 1 else axes[c]
            ax.axis("off")
        plt.suptitle("Ground Truth Images", fontsize=14)
        plt.tight_layout()
        mo.as_html(fig)
        plt.close(fig)
    return


@app.cell
def _(mo, np, cv2, evaluator, GT_DIR, Image, Path, tasks):
    mo.md("## Mock Agent Simulation")
    return


@app.cell
def _(mo):
    mo.md(
        """Simulate three synthetic agents with different capability levels:

        | Agent | Strategy | Expected Behavior |
        |-------|----------|-------------------|
        | **Gold** | Returns ground truth directly | Perfect scores (baseline sanity check) |
        | **Silver** | Applies approximate filters | Near-threshold scores, some failures |
        | **Bronze** | Applies wrong/random operations | Below-threshold scores, many failures |
        """
    )
    return


@app.cell
def _(np, cv2, GT_DIR, Image, Path, tasks):
    class MockAgent:
        """Base mock agent that produces synthetic outputs."""

        def __init__(self, name: str, noise_level: float = 0.0, wrong_prob: float = 0.0):
            self.name = name
            self.noise_level = noise_level
            self.wrong_prob = wrong_prob
            self.rng = np.random.RandomState(42)

        def generate_output(self, task: dict) -> np.ndarray | None:
            task_id = task["id"]
            gt_path = GT_DIR / f"{task_id}.png"

            if self.rng.random() < self.wrong_prob:
                return self._wrong_output(task)

            if gt_path.exists():
                gt = cv2.imread(str(gt_path), cv2.IMREAD_UNCHANGED)
                if gt is not None:
                    if self.noise_level > 0:
                        noise = self.rng.normal(0, self.noise_level, gt.shape).astype(np.float32)
                        gt = np.clip(gt.astype(np.float32) + noise, 0, 255).astype(np.uint8)
                    return gt

            return self._fallback_output(task)

        def _wrong_output(self, task: dict) -> np.ndarray:
            h, w = 512, 512
            thresholds = task.get("thresholds", {})
            if "dimension_match" in thresholds:
                w, h = thresholds["dimension_match"]
            return self.rng.randint(0, 255, (h, w, 3), dtype=np.uint8)

        def _fallback_output(self, task: dict) -> np.ndarray:
            thresholds = task.get("thresholds", {})
            h, w = 512, 512
            if "dimension_match" in thresholds:
                w, h = thresholds["dimension_match"]
            canvas = np.zeros((h, w, 3), dtype=np.uint8)
            gradient = np.linspace(0, 255, w, dtype=np.uint8)
            canvas[:, :, 0] = gradient[np.newaxis, :]
            return canvas

    gold_agent = MockAgent("Gold", noise_level=0.0, wrong_prob=0.0)
    silver_agent = MockAgent("Silver", noise_level=15.0, wrong_prob=0.3)
    bronze_agent = MockAgent("Bronze", noise_level=40.0, wrong_prob=0.7)

    agents = {"Gold": gold_agent, "Silver": silver_agent, "Bronze": bronze_agent}
    return agents, MockAgent, gold_agent, silver_agent, bronze_agent


@app.cell
def _(agents, evaluator, GT_DIR, np, tasks, mo):
    mo.md("## Benchmark Evaluation")
    return


@app.cell
def _(agents, evaluator, GT_DIR, np, pd, tasks):
    all_results = {}

    for agent_name, agent in agents.items():
        results = []
        for task in tasks:
            tid = task["id"]
            output = agent.generate_output(task)
            if output is None:
                results.append(EvaluationResult(
                    task_id=tid, passed=False,
                    pass_reason="Agent produced no output"
                ))
                continue

            gt_path = GT_DIR / f"{tid}.png"
            gt = str(gt_path) if gt_path.exists() else None

            try:
                result = evaluator.evaluate_task(tid, output, ground_truth=gt)
            except Exception as e:
                result = EvaluationResult(
                    task_id=tid, passed=False,
                    pass_reason=f"Evaluation error: {e}"
                )
            results.append(result)

        all_results[agent_name] = results

    results_summary = []
    for agent_name, results in all_results.items():
        for r in results:
            task = next((t for t in tasks if t["id"] == r.task_id), {})
            results_summary.append({
                "Agent": agent_name,
                "Task": r.task_id,
                "Category": task.get("category", ""),
                "Difficulty": task.get("difficulty", 0),
                "Passed": "✓" if r.passed else "✗",
                "MSSIM": f"{r.mssim:.4f}" if r.mssim is not None else "—",
                "PSNR": f"{r.psnr:.1f}" if r.psnr is not None else "—",
                "MAE": f"{r.mae:.2f}" if r.mae is not None else "—",
                "Reason": r.pass_reason[:60],
            })

    results_df = pd.DataFrame(results_summary)
    return all_results, results_df


@app.cell
def _(mo, results_df):
    mo.ui.table(results_df, selection=None, page_size=25, label="Evaluation Results")
    return


@app.cell
def _(mo, all_results, tasks, pd):
    mo.md("## Agent Comparison Dashboard")
    return


@app.cell
def _(all_results, pd, mo):
    _summary_rows = []
    for agent_name, results in all_results.items():
        total = len(results)
        passed = sum(1 for r in results if r.passed)
        mssims = [r.mssim for r in results if r.mssim is not None]
        psnrs = [r.psnr for r in results if r.psnr is not None]
        _summary_rows.append({
            "Agent": agent_name,
            "Pass Rate": f"{passed}/{total} ({100*passed/total:.0f}%)",
            "Mean MSSIM": f"{sum(mssims)/len(mssims):.4f}" if mssims else "—",
            "Mean PSNR": f"{sum(psnrs)/len(psnrs):.1f}" if psnrs else "—",
            "Total Errors": sum(r.error_count for r in results),
            "Avg Tool Efficiency": f"{sum(r.tool_efficiency for r in results)/total:.2f}",
        })

    _summary_df = pd.DataFrame(_summary_rows)
    mo.ui.table(_summary_df, selection=None, label="Agent Summary")
    return


@app.cell
def _(all_results, tasks, plt, mo):
    _fig, _axes = plt.subplots(1, 3, figsize=(18, 5))

    _categories = ["single_edit", "scheme_pipeline", "generation", "agentic_workflow", "image_understanding"]
    _cat_labels = ["Single\nEdit", "Scheme\nPipeline", "Generation", "Agentic\nWorkflow", "Image\nUnderstanding"]

    _agent_names = list(all_results.keys())
    _x = np.arange(len(_categories))
    _width = 0.25

    for i, (agent_name, results) in enumerate(all_results.items()):
        pass_rates = []
        for cat in _categories:
            cat_tasks = [t for t in tasks if t["category"] == cat]
            cat_ids = {t["id"] for t in cat_tasks}
            cat_results = [r for r in results if r.task_id in cat_ids]
            if cat_results:
                pass_rates.append(sum(1 for r in cat_results if r.passed) / len(cat_results) * 100)
            else:
                pass_rates.append(0)
        _axes[0].bar(_x + i * _width, pass_rates, _width, label=agent_name)

    _axes[0].set_xlabel("Category")
    _axes[0].set_ylabel("Pass Rate (%)")
    _axes[0].set_title("Pass Rate by Category")
    _axes[0].set_xticks(_x + _width)
    _axes[0].set_xticklabels(_cat_labels, fontsize=8)
    _axes[0].legend()
    _axes[0].set_ylim(0, 105)

    for i, (agent_name, results) in enumerate(all_results.items()):
        mssims_by_cat = []
        for cat in _categories:
            cat_tasks = [t for t in tasks if t["category"] == cat]
            cat_ids = {t["id"] for t in cat_tasks}
            cat_mssims = [r.mssim for r in results if r.task_id in cat_ids and r.mssim is not None]
            mssims_by_cat.append(sum(cat_mssims) / len(cat_mssims) if cat_mssims else 0)
        _axes[1].bar(_x + i * _width, mssims_by_cat, _width, label=agent_name)

    _axes[1].set_xlabel("Category")
    _axes[1].set_ylabel("Mean MSSIM")
    _axes[1].set_title("MSSIM by Category")
    _axes[1].set_xticks(_x + _width)
    _axes[1].set_xticklabels(_cat_labels, fontsize=8)
    _axes[1].legend()

    for i, (agent_name, results) in enumerate(all_results.items()):
        diff_bins = {"★ (1-2)": 0, "★★★ (3)": 0, "★★★★ (4)": 0, "★★★★★ (5)": 0}
        diff_pass = {"★ (1-2)": 0, "★★★ (3)": 0, "★★★★ (4)": 0, "★★★★★ (5)": 0}
        for r in results:
            task = next((t for t in tasks if t["id"] == r.task_id), {})
            d = task.get("difficulty", 3)
            if d <= 2:
                key = "★ (1-2)"
            elif d == 3:
                key = "★★★ (3)"
            elif d == 4:
                key = "★★★★ (4)"
            else:
                key = "★★★★★ (5)"
            diff_bins[key] += 1
            if r.passed:
                diff_pass[key] += 1
        rates = [diff_pass[k] / diff_bins[k] * 100 if diff_bins[k] else 0 for k in diff_bins]
        _axes[2].bar(np.arange(4) + i * _width, rates, _width, label=agent_name)

    _axes[2].set_xlabel("Difficulty")
    _axes[2].set_ylabel("Pass Rate (%)")
    _axes[2].set_title("Pass Rate by Difficulty")
    _axes[2].set_xticks(np.arange(4) + _width)
    _axes[2].set_xticklabels(list(diff_bins.keys()), fontsize=8)
    _axes[2].legend()

    plt.suptitle("SeaMagic Agent Benchmark — Comparison Dashboard", fontsize=14, fontweight="bold")
    plt.tight_layout()
    mo.as_html(_fig)
    plt.close(_fig)
    return


@app.cell
def _(all_results, tasks, plt, mo):
    mo.md("### Radar Chart — Agent Capability Profile")
    return


@app.cell
def _(all_results, tasks, plt, mo, np):
    _categories = ["single_edit", "scheme_pipeline", "generation", "agentic_workflow", "image_understanding"]
    _cat_short = ["Single Edit", "Scheme/Pipeline", "Generation", "Agentic Workflow", "Image Understanding"]

    fig_radar, ax_radar = plt.subplots(figsize=(8, 8), subplot_kw=dict(polar=True))

    angles = np.linspace(0, 2 * np.pi, len(_categories), endpoint=False).tolist()
    angles += angles[:1]

    colors = {"Gold": "#FFD700", "Silver": "#C0C0C0", "Bronze": "#CD7F32"}

    for agent_name, results in all_results.items():
        pass_rates = []
        for cat in _categories:
            cat_tasks = [t for t in tasks if t["category"] == cat]
            cat_ids = {t["id"] for t in cat_tasks}
            cat_results = [r for r in results if r.task_id in cat_ids]
            if cat_results:
                pass_rates.append(sum(1 for r in cat_results if r.passed) / len(cat_results) * 100)
            else:
                pass_rates.append(0)
        pass_rates += pass_rates[:1]
        ax_radar.plot(angles, pass_rates, "o-", linewidth=2, label=agent_name, color=colors.get(agent_name, "gray"))
        ax_radar.fill(angles, pass_rates, alpha=0.15, color=colors.get(agent_name, "gray"))

    ax_radar.set_thetagrids(np.degrees(angles[:-1]), _cat_short)
    ax_radar.set_ylim(0, 105)
    ax_radar.set_title("Agent Capability Radar", pad=20, fontsize=14)
    ax_radar.legend(loc="upper right", bbox_to_anchor=(1.3, 1.1))

    mo.as_html(fig_radar)
    plt.close(fig_radar)
    return


@app.cell
def _(mo, evaluator, all_results, tasks):
    mo.md("## Aggregate Statistics")
    return


@app.cell
def _(evaluator, all_results, mo, pd):
    _agg_rows = []
    for agent_name, results in all_results.items():
        agg = evaluator.compute_aggregate(results)
        _agg_rows.append({
            "Agent": agent_name,
            "Total Tasks": agg.get("total_tasks", 0),
            "Passed": agg.get("passed", 0),
            "Overall Pass Rate": f"{agg.get('overall_pass_rate', 0):.1%}",
            "Mean MSSIM": f"{agg['mean_mssim']:.4f}" if agg.get("mean_mssim") is not None else "—",
            "Mean PSNR": f"{agg['mean_psnr']:.1f}" if agg.get("mean_psnr") is not None else "—",
            "Mean Tool Efficiency": f"{agg.get('mean_tool_efficiency', 0):.2f}",
            "Error Recovery Rate": f"{agg.get('error_recovery_rate', 0):.1%}",
        })

    _agg_df = pd.DataFrame(_agg_rows)
    mo.ui.table(_agg_df, selection=None, label="Aggregate Statistics")
    return


@app.cell
def _(evaluator, all_results, mo, pd, tasks):
    _cat_detail_rows = []
    for agent_name, results in all_results.items():
        agg = evaluator.compute_aggregate(results)
        for cat, stats in agg.get("by_category", {}).items():
            _cat_detail_rows.append({
                "Agent": agent_name,
                "Category": cat,
                "Tasks": stats["total"],
                "Passed": stats["passed"],
                "Pass Rate": f"{stats['pass_rate']:.1%}",
                "Mean MSSIM": f"{stats['mean_mssim']:.4f}" if stats.get("mean_mssim") is not None else "—",
            })

    _cat_df = pd.DataFrame(_cat_detail_rows)
    mo.ui.table(_cat_df, selection=None, label="Per-Category Breakdown")
    return


@app.cell
def _(mo, evaluator, all_results, tasks):
    mo.md("## Detailed Per-Task Analysis")
    return


@app.cell
def _(mo, results_df):
    _task_filter = mo.ui.dropdown(
        options=["All"] + sorted(results_df["Task"].unique().tolist()),
        value="All",
        label="Filter by task",
    )

    _agent_filter = mo.ui.dropdown(
        options=["All"] + sorted(results_df["Agent"].unique().tolist()),
        value="All",
        label="Filter by agent",
    )

    _filtered = results_df.copy()
    if _task_filter.value != "All":
        _filtered = _filtered[_filtered["Task"] == _task_filter.value]
    if _agent_filter.value != "All":
        _filtered = _filtered[_filtered["Agent"] == _agent_filter.value]

    mo.vstack([mo.hstack([_task_filter, _agent_filter]), mo.ui.table(_filtered, selection=None, page_size=25)])
    return


@app.cell
def _(mo):
    mo.md("## Training Data Export")
    return


@app.cell
def _(mo, all_results, agents, evaluator, GT_DIR, json, os, OUTPUTS_DIR, Path, tasks, np):
    _export_button = mo.ui.run_button(label="Export Training Data as JSONL")

    _export_status = mo.md("Click the button above to export.")

    if _export_button.value:
        OUTPUTS_DIR.mkdir(parents=True, exist_ok=True)
        output_file = OUTPUTS_DIR / "training_data.jsonl"

        records = []
        for agent_name, results in all_results.items():
            agent = agents[agent_name]
            for r in results:
                task = next((t for t in tasks if t["id"] == r.task_id), {})
                prompt = task.get("description", "")
                category = task.get("category", "")
                difficulty = task.get("difficulty", 0)
                eval_mode = task.get("eval_mode", "")
                thresholds = task.get("thresholds", {})

                tool_calls = []
                if r.tool_calls > 0:
                    tool_calls = [{"tool": "edit_image", "operation": "mock"}] * r.tool_calls

                record = {
                    "task_id": r.task_id,
                    "agent": agent_name,
                    "category": category,
                    "difficulty": difficulty,
                    "eval_mode": eval_mode,
                    "prompt": prompt,
                    "thresholds": thresholds,
                    "tool_calls": tool_calls,
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
                records.append(record)

        with open(output_file, "w") as f:
            for rec in records:
                f.write(json.dumps(rec) + "\n")

        _export_status = mo.md(f"Exported **{len(records)}** records to `{output_file}`")

    mo.vstack([
        mo.md("### Export agent trajectories for RLHF/SFT training"),
        _export_button,
        _export_status,
    ])
    return


@app.cell
def _(mo):
    mo.md("## Paper Section Draft")
    return


@app.cell
def _(all_results, evaluator, mo, tasks):
    _agg = evaluator.compute_aggregate(all_results.get("Gold", []))

    _paper_methods = mo.md(
        f"""### Benchmark Methodology

        The SeaMagic Agent Benchmark (SMAB) evaluates AI agents across **5 task categories**
        and **3 evaluation dimensions**:

        **Task Categories** (n={len(tasks)} tasks):
        - Single Edit ({len([t for t in tasks if t['category']=='single_edit'])} tasks): One-shot tool calls testing API parameter understanding
        - Scheme Pipeline ({len([t for t in tasks if t['category']=='scheme_pipeline'])} tasks): Multi-step Steel Scheme scripts testing DSL programming
        - Generation ({len([t for t in tasks if t['category']=='generation'])} tasks): Creating images from scratch testing visual design
        - Agentic Workflow ({len([t for t in tasks if t['category']=='agentic_workflow'])} tasks): Long-horizon iterative tasks testing self-correction
        - Image Understanding ({len([t for t in tasks if t['category']=='image_understanding'])} tasks): Tasks requiring visual analysis before editing

        **Evaluation Modes**:
        - *Exact Match* ({len([t for t in tasks if t['eval_mode']=='exact_match'])} tasks): Pixel-level comparison using MSSIM, PSNR, MAE, color histogram chi-squared
        - *Constraint Satisfaction* ({len([t for t in tasks if t['eval_mode']=='constraint_satisfaction'])} tasks): Structural/visual constraint checking (dimensions, text presence, color bars)
        - *LLM Judge* ({len([t for t in tasks if t['eval_mode']=='llm_judge'])} tasks): Subjective quality evaluation via vision-language model

        **Primary Metrics**: MSSIM (structural similarity), PSNR (signal fidelity), MAE (error magnitude).
        **Secondary Metrics**: Tool efficiency, error recovery rate, tool call count.
        """
    )

    mo.vstack([mo.md("### Auto-Generated Methodology Section"), _paper_methods])
    return


@app.cell
def _(all_results, evaluator, mo, pd, tasks):
    _results_table_rows = []
    for agent_name, results in all_results.items():
        total = len(results)
        passed = sum(1 for r in results if r.passed)
        mssims = [r.mssim for r in results if r.mssim is not None]
        psnrs = [r.psnr for r in results if r.psnr is not None]
        _results_table_rows.append({
            "Agent": agent_name,
            "Pass Rate": f"{passed}/{total} ({100*passed/total:.0f}\\%)",
            "MSSIM": f"{sum(mssims)/len(mssims):.3f}" if mssims else "—",
            "PSNR (dB)": f"{sum(psnrs)/len(psnrs):.1f}" if psnrs else "—",
        })

    _paper_results = mo.md(
        """### Results

        | Agent | Pass Rate | MSSIM | PSNR (dB) |
        |-------|-----------|-------|-----------|
        """
        + "\n".join(
            f"| {r['Agent']} | {r['Pass Rate']} | {r['MSSIM']} | {r['PSNR (dB)']} |"
            for r in _results_table_rows
        )
    )

    mo.vstack([mo.md("### Auto-Generated Results Section"), _paper_results])
    return


@app.cell
def _(mo):
    mo.md(
        """## LLM Judge Integration

        Tasks with `eval_mode: "llm_judge"` require a vision-language model for evaluation.
        Integration points:

        - **OpenAI GPT-4V**: Send image + judge prompt, parse score from response
        - **Claude with Vision**: Use Claude's image understanding for scoring
        - **Local LLaVA**: Self-hosted vision-language model for offline evaluation

        To integrate, implement the `_eval_llm_judge` method in `evaluator.py` with your
        preferred VLM API. The judge prompt is provided in each task's `judge_prompt` field.
        """
    )
    return


@app.cell
def _(mo):
    mo.md(
        """## Real Agent Integration

        To evaluate real AI agents (not mock agents):

        1. **Agent outputs directory**: Place agent-produced images in `benchmark/outputs/{agent_name}/{task_id}.png`
        2. **Tool traces**: Provide a JSON file `benchmark/outputs/{agent_name}/trace.json` with tool call records
        3. **Run evaluation**: Use the CLI runner (`run_benchmark.py`) or this notebook

        ### Expected trace format

        ```json
        {
          "agent_name": "my-agent",
          "tool_calls": [
            {"tool": "edit_image", "operation": "blur", "params": {"sigma": 3.0}, "error": null},
            {"tool": "run_steel", "script": "(define result ...)", "error": null}
          ],
          "ideal_tool_calls": 2
        }
        ```
        """
    )
    return


if __name__ == "__main__":
    app.run()
