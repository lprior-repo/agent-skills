# GEPA Adapter Guide Source

Source: `https://raw.githubusercontent.com/gepa-ai/gepa/main/docs/docs/guides/adapters.md`

This file preserves the GEPA adapter guide as pulled on 2026-06-03 so local skill workflows can use it without refetching the website.

# Creating Adapters

Most users don't need a custom adapter. The `optimize_anything` API handles most use cases by letting the user write an evaluator function. Custom adapters are for advanced scenarios where the workflow needs full control over batch evaluation, trace capture, or reflective dataset formatting.

GEPA can optimize any system consisting of text components by implementing the `GEPAAdapter` protocol.

## The GEPAAdapter Protocol

Every adapter must implement two methods:

```python
from gepa.core.adapter import GEPAAdapter, EvaluationBatch

class MyAdapter(GEPAAdapter[DataInst, Trajectory, RolloutOutput]):
    def evaluate(
        self,
        batch: list[DataInst],
        candidate: dict[str, str],
        capture_traces: bool = False,
    ) -> EvaluationBatch[Trajectory, RolloutOutput]:
        """Execute the system and return scores."""
        ...

    def make_reflective_dataset(
        self,
        candidate: dict[str, str],
        eval_batch: EvaluationBatch[Trajectory, RolloutOutput],
        components_to_update: list[str],
    ) -> dict[str, list[dict]]:
        """Build dataset for reflection."""
        ...
```

## Step 1: Define Your Types

```python
from dataclasses import dataclass
from typing import Any

@dataclass
class TaskInput:
    question: str
    context: str
    expected_answer: str

@dataclass
class ExecutionTrace:
    prompt_used: str
    model_response: str
    intermediate_steps: list[str]

@dataclass
class TaskOutput:
    answer: str
    confidence: float
```

## Step 2: Implement `evaluate`

The `evaluate` method runs the system on a batch of inputs:

```python
from gepa.core.adapter import EvaluationBatch

class MyAdapter:
    def __init__(self, model_name: str):
        self.model_name = model_name

    def evaluate(
        self,
        batch: list[TaskInput],
        candidate: dict[str, str],
        capture_traces: bool = False,
    ) -> EvaluationBatch[ExecutionTrace, TaskOutput]:
        outputs = []
        scores = []
        trajectories = [] if capture_traces else None

        for task in batch:
            prompt = candidate["system_prompt"] + "\n" + task.question
            response = self._call_model(prompt)
            output = TaskOutput(answer=response, confidence=0.9)
            outputs.append(output)
            score = 1.0 if output.answer == task.expected_answer else 0.0
            scores.append(score)

            if capture_traces:
                trace = ExecutionTrace(
                    prompt_used=prompt,
                    model_response=response,
                    intermediate_steps=[],
                )
                trajectories.append(trace)

        return EvaluationBatch(
            outputs=outputs,
            scores=scores,
            trajectories=trajectories,
        )
```

## Step 3: Implement `make_reflective_dataset`

This method creates data for the reflection LLM to propose improvements:

```python
def make_reflective_dataset(
    self,
    candidate: dict[str, str],
    eval_batch: EvaluationBatch[ExecutionTrace, TaskOutput],
    components_to_update: list[str],
) -> dict[str, list[dict]]:
    """Build a reflective dataset for each component."""

    dataset = {}

    for component_name in components_to_update:
        component_data = []

        for i, trace in enumerate(eval_batch.trajectories):
            record = {
                "Inputs": {
                    "prompt": trace.prompt_used,
                },
                "Generated Outputs": {
                    "response": trace.model_response,
                },
                "Feedback": self._generate_feedback(
                    trace,
                    eval_batch.outputs[i],
                    eval_batch.scores[i],
                ),
            }
            component_data.append(record)

        dataset[component_name] = component_data

    return dataset

def _generate_feedback(self, trace, output, score):
    """Generate helpful feedback for the reflection LLM."""
    if score == 1.0:
        return "Correct! The answer matched the expected output."
    else:
        return f"Incorrect. The model answered '{output.answer}' but this was wrong."
```

## Best Practices

### Rich Feedback

The more informative the feedback, the better GEPA can optimize:

```python
def _generate_feedback(self, trace, output, expected, score):
    feedback_parts = []
    feedback_parts.append(f"Score: {score}")

    if score < 1.0:
        feedback_parts.append(f"Expected: {expected}")
        feedback_parts.append(f"Got: {output.answer}")

        if len(output.answer) > 100:
            feedback_parts.append("Issue: Response too verbose")
        if expected.lower() not in output.answer.lower():
            feedback_parts.append("Issue: Key information missing")

    return "\n".join(feedback_parts)
```

### Error Handling

Handle failures gracefully:

```python
def evaluate(self, batch, candidate, capture_traces=False):
    outputs, scores, trajectories = [], [], []

    for task in batch:
        try:
            output = self._run_task(task, candidate)
            score = self._compute_score(output, task)
        except Exception as e:
            output = TaskOutput(answer="ERROR", confidence=0.0)
            score = 0.0
            if capture_traces:
                trajectories.append(ExecutionTrace(
                    error=str(e),
                ))

        outputs.append(output)
        scores.append(score)

    return EvaluationBatch(outputs=outputs, scores=scores, trajectories=trajectories)
```

### Multi-Objective Optimization

Support multiple objectives:

```python
def evaluate(self, batch, candidate, capture_traces=False):
    # ... evaluation logic ...

    objective_scores = []
    for output in outputs:
        objective_scores.append({
            "accuracy": 1.0 if output.correct else 0.0,
            "latency": 1.0 / (1.0 + output.latency),
            "cost": 1.0 / (1.0 + output.token_count),
        })

    return EvaluationBatch(
        outputs=outputs,
        scores=scores,
        trajectories=trajectories,
        objective_scores=objective_scores,
    )
```

## Complete Adapter Example

```python
from dataclasses import dataclass
from typing import Any
import litellm
from gepa.core.adapter import GEPAAdapter, EvaluationBatch

@dataclass
class QAInput:
    question: str
    answer: str

@dataclass
class QATrace:
    prompt: str
    response: str

@dataclass
class QAOutput:
    answer: str

class SimpleQAAdapter(GEPAAdapter[QAInput, QATrace, QAOutput]):
    def __init__(self, model: str = "openai/gpt-4o-mini"):
        self.model = model

    def evaluate(
        self,
        batch: list[QAInput],
        candidate: dict[str, str],
        capture_traces: bool = False,
    ) -> EvaluationBatch[QATrace, QAOutput]:
        outputs, scores = [], []
        trajectories = [] if capture_traces else None

        for item in batch:
            prompt = f"{candidate['system_prompt']}\n\nQuestion: {item.question}"
            response = litellm.completion(
                model=self.model,
                messages=[{"role": "user", "content": prompt}],
            )
            answer = response.choices[0].message.content
            output = QAOutput(answer=answer)
            score = 1.0 if item.answer.lower() in answer.lower() else 0.0
            outputs.append(output)
            scores.append(score)

            if capture_traces:
                trajectories.append(QATrace(prompt=prompt, response=answer))

        return EvaluationBatch(
            outputs=outputs,
            scores=scores,
            trajectories=trajectories,
        )

    def make_reflective_dataset(
        self,
        candidate: dict[str, str],
        eval_batch: EvaluationBatch[QATrace, QAOutput],
        components_to_update: list[str],
    ) -> dict[str, list[dict]]:
        dataset = {"system_prompt": []}

        for i, trace in enumerate(eval_batch.trajectories or []):
            dataset["system_prompt"].append({
                "Inputs": {"question": trace.prompt.split("Question: ")[-1]},
                "Generated Outputs": {"answer": trace.response},
                "Feedback": f"Score: {eval_batch.scores[i]}",
            })

        return dataset

adapter = SimpleQAAdapter(model="openai/gpt-4o-mini")
result = gepa.optimize(
    seed_candidate={"system_prompt": "Answer questions accurately."},
    trainset=trainset,
    adapter=adapter,
    reflection_lm="openai/gpt-4o",
    max_metric_calls=50,
)
```

## Built-In Adapters

| Adapter | Description | Use Case |
|---|---|---|
| DefaultAdapter | Simple adapter for prompt optimization with any LLM | General prompt tuning, Q&A systems |
| ConfidenceAdapter | Logprob-aware adapter for structured-output classification | Category classification, enum label prediction |
| DSPy Adapter | Optimizes DSPy program instructions and prompts | DSPy module optimization |
| DSPy Full Program Adapter | Evolves entire DSPy programs including structure | Full program evolution, architecture search |
| RAG Adapter | Optimizes RAG pipeline components | Retrieval-augmented generation systems |
| MCP Adapter | Optimizes MCP tool descriptions and system prompts | Tool-using agents, MCP servers |
| TerminalBench Adapter | Optimizes agents for terminal/shell environments | CLI agents, shell automation |

## Adapter Selection

- Use `DefaultAdapter` for simple prompt optimization tasks.
- Use `ConfidenceAdapter` for classification tasks where the LLM returns structured JSON with enum-constrained fields and logprob-based confidence is useful.
- Use `DSPy Adapter` when optimizing instructions for individual DSPy predictors while keeping the program structure fixed.
- Use `DSPy Full Program Adapter` when evolving an entire DSPy program, including structure and module composition.
- Use `RAG Adapter` for retrieval-augmented generation systems.
- Use `MCP Adapter` for Model Context Protocol tool usage.
- Use `TerminalBench Adapter` for agents that interact with terminal or shell environments.

## Next Steps

- See the API reference for complete `GEPAAdapter` protocol documentation.
- Explore built-in adapters for the specific use case.
- Read `DefaultAdapter` source for a reference implementation.
