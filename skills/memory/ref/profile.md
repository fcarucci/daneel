# Behavioral Profile (CARA)

> **Source:** Adapted from the CARA (Coherent Adaptive Reasoning Agents)
> component of the [Hindsight](https://arxiv.org/abs/2512.12818) architecture.
> CARA shapes *how* the agent reasons over memories, not just *what* it stores.

## Profile location

The behavioral profile lives in `~/.agents/memory/profile.json`.
If the file does not exist, the agent uses neutral defaults.

```json
{
  "skepticism": 3,
  "literalism": 3,
  "empathy": 3,
  "bias_strength": 0.2
}
```

## Disposition parameters

Three dimensions define the agent's reasoning disposition. Each is an
integer from 1 to 5.

### Skepticism (S)

How cautiously the agent evaluates claims and evidence.

| Value | Style | Effect on reflect |
|-------|-------|-------------------|
| 1 | Trusting | Accepts evidence at face value; reinforces beliefs readily |
| 3 | Balanced | Standard evidence evaluation |
| 5 | Skeptical | Demands strong evidence; resists confidence increases; applies larger contradiction penalties |

### Literalism (L)

How closely the agent adheres to explicit wording vs. inferred intent.

| Value | Style | Effect on reflect |
|-------|-------|-------------------|
| 1 | Flexible | Reads between the lines; infers implicit connections between memories |
| 3 | Balanced | Standard interpretation |
| 5 | Literal | Requires explicit entity overlap; ignores implicit semantic connections |

### Empathy (E)

How much weight the agent gives to user preferences and interpersonal
context when forming or updating beliefs.

| Value | Style | Effect on reflect |
|-------|-------|-------------------|
| 1 | Detached | Task-first; ignores preference signals in experiences |
| 3 | Balanced | Standard preference awareness |
| 5 | Empathetic | Prioritizes user-stated preferences; resistant to contradicting beliefs the user expressed directly |

## Bias strength (β)

A float in `[0.0, 1.0]` that controls how strongly the disposition
parameters influence reasoning:

| Value | Behavior |
|-------|----------|
| 0.0 | Primarily fact-based; disposition barely affects confidence updates |
| 0.2 | Mild influence (default) |
| 0.5 | Moderate; noticeable skew toward disposition style |
| 1.0 | Strong; disposition dominates evidence evaluation |

## How to apply during reflect

The profile modulates the reflect workflow at three points:

### 1. Evidence assessment (step 2–3 in ref/reflect.md)

Before computing base deltas, the subagent reads the profile and adjusts
its reasoning posture:

- **High skepticism (S ≥ 4):** require 2+ supporting experiences before
  any upward confidence change; double the magnitude of contradiction
  penalties.
- **Low skepticism (S ≤ 2):** single supporting experience is sufficient
  for reinforcement; halve contradiction penalties.
- **High literalism (L ≥ 4):** only count experiences that share at least
  one entity with the belief. Keyword-only matches don't count.
- **Low literalism (L ≤ 2):** accept keyword matches and semantic
  similarity as evidence, even without shared entities.
- **High empathy (E ≥ 4):** experiences tagged `[preference]` or
  `[decision]` receive 1.5× weight in evidence assessment.
- **Low empathy (E ≤ 2):** ignore context tags when weighting evidence.

### 2. Confidence calibration (ref/reflect-techniques.md, technique 2)

The bias strength β scales how much the disposition adjustments affect
the final calibrated delta:

```
adjusted_delta = base_delta + (disposition_modifier × β)
```

At β = 0.0, disposition has no effect. At β = 1.0, it fully applies.

### 3. Belief conflict resolution (ref/reflect-techniques.md, technique 4)

When resolving conflicts between beliefs:

- **High skepticism:** prefer the belief with more supporting evidence,
  regardless of confidence score.
- **High empathy:** prefer the belief that aligns with user-stated
  preferences.
- **High literalism:** prefer the belief with more direct entity overlap
  to supporting experiences.

## Verbalization

When the profile influences a reflect decision, the subagent should
include a brief note in the output explaining which parameter drove
the adjustment. Example:

```
Belief 2: confidence +0.1 → +0.05 (skepticism=4: required stronger evidence)
Belief 5: kept over belief 3 in conflict (empathy=5: aligns with user preference)
```

## Reading the profile

```bash
cat ~/.agents/memory/profile.json 2>/dev/null || echo '{"skepticism":3,"literalism":3,"empathy":3,"bias_strength":0.2}'
```

The subagent reads this at the start of a reflect pass. If the file
is missing or malformed, use neutral defaults (all 3s, β = 0.2).

## Changing the profile

The user can modify the profile by editing the JSON file directly or
by telling the agent: "Be more skeptical", "Trust evidence more easily",
"Pay attention to my preferences", etc. The agent maps these to parameter
changes and writes the updated profile.

| User says | Parameter change |
|-----------|-----------------|
| "Be more skeptical" | skepticism += 1 (max 5) |
| "Trust things more easily" | skepticism -= 1 (min 1) |
| "Read between the lines" | literalism -= 1 (min 1) |
| "Stick to what I said exactly" | literalism += 1 (max 5) |
| "Pay more attention to my preferences" | empathy += 1 (max 5) |
| "Be more objective" | empathy -= 1 (min 1) |
| "Rely more on facts" | bias_strength -= 0.2 (min 0.0) |
| "Use your judgment more" | bias_strength += 0.2 (max 1.0) |
