#!/usr/bin/env python3
"""Deterministic memory management operations for MEMORY.md.

Supports two memory tiers:
  - User memory:    ~/.agents/memory/MEMORY.md  (personal, default for writes)
  - Project memory: <repo>/MEMORY.md            (shared, explicit promotion only)

Usage:
    python skills/memory/scripts/memory-manage.py validate
    python skills/memory/scripts/memory-manage.py validate --scope user
    python skills/memory/scripts/memory-manage.py check-duplicate --section experiences --candidate "text"
    python skills/memory/scripts/memory-manage.py check-duplicate --section experiences --candidate "text" --cross-scope
    python skills/memory/scripts/memory-manage.py screen-text --text "some text"
    python skills/memory/scripts/memory-manage.py append-entry --section experiences --text "text" --date 2026-03-27
    python skills/memory/scripts/memory-manage.py update-confidence --section beliefs --index 0 --delta 0.1
    python skills/memory/scripts/memory-manage.py update-confidence --section beliefs --index 0 --delta 0.1 --scope user
    python skills/memory/scripts/memory-manage.py extract-entities --text "some text"
    python skills/memory/scripts/memory-manage.py prune-beliefs --threshold 0.2
    python skills/memory/scripts/memory-manage.py suggest-summaries
    python skills/memory/scripts/memory-manage.py init-user
    python skills/memory/scripts/memory-manage.py promote --section experiences --index 0 --allow-project-promotion
"""

import argparse
import hashlib
import json
import os
import re
import sys
import tempfile
from dataclasses import dataclass
from difflib import SequenceMatcher
from pathlib import Path
from typing import Optional

sys.path.insert(0, str(Path(__file__).resolve().parent))
from importlib import import_module
recall_mod = import_module("memory-recall")

MEMORY_PATH = recall_mod.MEMORY_PATH
PROJECT_MEMORY_PATH = recall_mod.PROJECT_MEMORY_PATH
USER_MEMORY_PATH = recall_mod.USER_MEMORY_PATH


def resolve_path(scope: str) -> Path:
    """Return the memory file path for a single-file scope."""
    if scope == "user":
        return USER_MEMORY_PATH
    return PROJECT_MEMORY_PATH

DUPLICATE_THRESHOLD = 0.65

STOPWORDS = frozenset({
    "a", "an", "the", "is", "was", "are", "were", "be", "been", "being",
    "have", "has", "had", "do", "does", "did", "will", "would", "could",
    "should", "may", "might", "shall", "can", "need", "must",
    "in", "on", "at", "to", "for", "of", "with", "by", "from", "as",
    "into", "through", "during", "before", "after", "above", "below",
    "between", "under", "over",
    "and", "but", "or", "nor", "not", "so", "yet",
    "it", "its", "this", "that", "these", "those",
    "i", "we", "you", "he", "she", "they", "me", "us", "him", "her", "them",
    "my", "our", "your", "his", "their",
    "today", "yesterday", "session", "currently",
})

CANONICAL_CONTEXT_TAGS = frozenset({
    "debug", "testing", "tooling", "workflow", "decision", "preference",
    "infra", "docs", "ui", "backend", "security",
})

CONTEXT_ALIASES = {
    "debugging": "debug",
    "test": "testing",
    "tests": "testing",
    "preferences": "preference",
    "infrastructure": "infra",
    "documentation": "docs",
}

PASCAL_COMMON = frozenset({
    "The", "This", "That", "These", "Those", "What", "When", "Where",
    "Which", "Who", "How", "And", "But", "For", "Not", "All", "Any",
    "Each", "Every", "Some", "Its", "Our", "His", "Her", "Can",
    "May", "Use", "Run", "Set", "Get", "Add", "See", "Try",
})

ENTITY_PATTERN = re.compile(
    r"(?:"
    r"\b[A-Z][a-zA-Z]{2,}\b"  # single PascalCase word (OpenClaw, Dioxus, Tailwind)
    r"|\b[a-zA-Z0-9]+[-_][a-zA-Z0-9]+(?:[-_][a-zA-Z0-9]+)*\b"  # hyphenated/underscored (e2e-mock-gateway)
    r"|`[^`]+`"  # backtick-quoted identifiers
    r")"
)

LOWERCASE_ENTITY_ALIASES = {
    "api": "api",
    "cargo": "cargo",
    "docker": "docker",
    "dioxus": "dioxus",
    "dx": "dx",
    "fastapi": "fastapi",
    "gateway": "gateway",
    "npm": "npm",
    "openclaw": "openclaw",
    "playwright": "playwright",
    "postgres": "postgresql",
    "postgresql": "postgresql",
    "redis": "redis",
    "redis-cli": "redis-cli",
    "rust": "rust",
    "sqlalchemy": "sqlalchemy",
    "sqlite": "sqlite",
    "tailwind": "tailwind",
    "websocket": "websocket",
}

SENSITIVE_VALUE_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    (
        "credential-assignment",
        re.compile(
            r"(?i)\b(password|passwd|secret|api[_-]?key|token|auth[_-]?token|cookie|session[_-]?id)\b"
            r"(\s*(?:=|:|is)\s*)([^\s,;]+)"
        ),
    ),
    ("private-key-material", re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----")),
    (
        "known-secret-prefix",
        re.compile(r"\b(?:ghp_[A-Za-z0-9]+|github_pat_[A-Za-z0-9_]+|sk_(?:live|test)_[A-Za-z0-9]+|AKIA[0-9A-Z]{16})\b"),
    ),
    ("credential-url", re.compile(r"(?i)\b[a-z][a-z0-9+.-]*://[^/\s:@]+:[^@\s]+@")),
)


def normalize_for_comparison(text: str) -> str:
    """Normalize text for duplicate comparison.

    Strips metadata, lowercases, splits compound identifiers,
    removes stopwords and extra whitespace.
    """
    text = recall_mod.strip_metadata(text)
    text = text.lower()
    text = re.sub(r"[_\-]", " ", text)
    text = re.sub(r"[^\w\s]", " ", text)
    words = [w for w in text.split() if w not in STOPWORDS]
    return " ".join(words)


def content_hash(text: str) -> str:
    """Return a stable content hash for optimistic concurrency checks."""
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def write_text_if_unchanged(path: Path, new_text: str, expected_hash: str) -> dict:
    """Atomically replace a file only if it still matches expected_hash."""
    current_text = path.read_text(encoding="utf-8") if path.exists() else ""
    current_hash = content_hash(current_text)
    if current_hash != expected_hash:
        return {
            "success": False,
            "error": "stale write blocked: file changed since it was read",
            "expected_hash": expected_hash,
            "current_hash": current_hash,
        }

    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=str(path.parent))
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            handle.write(new_text)
        os.replace(tmp_name, path)
    finally:
        if os.path.exists(tmp_name):
            os.unlink(tmp_name)

    return {"success": True, "path": str(path), "hash": content_hash(new_text)}


def normalize_context_tag(context: Optional[str]) -> Optional[str]:
    """Normalize free-form context tags to the canonical vocabulary."""
    if context is None:
        return None
    normalized = context.strip().lower().replace("_", "-")
    normalized = CONTEXT_ALIASES.get(normalized, normalized)
    if normalized in CANONICAL_CONTEXT_TAGS:
        return normalized
    return None


def canonicalize_entity(entity: str) -> str:
    """Normalize entity tags to lowercase hyphenated identifiers."""
    normalized = entity.strip().strip("`").lower()
    normalized = re.sub(r"[\s_]+", "-", normalized)
    normalized = re.sub(r"[^a-z0-9-]", "", normalized)
    normalized = re.sub(r"-{2,}", "-", normalized).strip("-")
    return LOWERCASE_ENTITY_ALIASES.get(normalized, normalized)


def canonicalize_entities(entities: list[str]) -> list[str]:
    """Deduplicate and sort canonical entity tags."""
    canonical = {
        canonicalize_entity(entity)
        for entity in entities
        if canonicalize_entity(entity)
    }
    return sorted(canonical)


def screen_text(text: str) -> dict:
    """Detect sensitive content and produce a sanitized preview."""
    sanitized = text
    issues = []

    for issue_type, pattern in SENSITIVE_VALUE_PATTERNS:
        if issue_type == "credential-assignment":
            def repl(match: re.Match[str]) -> str:
                issues.append({
                    "type": issue_type,
                    "match": match.group(0),
                    "field": match.group(1).lower(),
                })
                return f"{match.group(1)}{match.group(2)}[REDACTED]"

            sanitized = pattern.sub(repl, sanitized)
            continue

        match = pattern.search(sanitized)
        if match:
            issues.append({"type": issue_type, "match": match.group(0)})
            sanitized = pattern.sub("[REDACTED]", sanitized)

    return {
        "safe": len(issues) == 0,
        "issues": issues,
        "sanitized_text": sanitized,
    }


def similarity(a: str, b: str) -> float:
    """Compute similarity using three complementary metrics.

    SequenceMatcher catches rewordings that preserve order.
    Jaccard catches shared-term overlap.
    Overlap coefficient handles asymmetric lengths (short query
    contained within a longer entry).
    """
    na = normalize_for_comparison(a)
    nb = normalize_for_comparison(b)
    if not na or not nb:
        return 0.0
    seq_ratio = SequenceMatcher(None, na, nb).ratio()
    tokens_a = set(na.split())
    tokens_b = set(nb.split())
    intersection = len(tokens_a & tokens_b)
    union = tokens_a | tokens_b
    jaccard = intersection / len(union) if union else 0.0
    min_size = min(len(tokens_a), len(tokens_b))
    overlap = intersection / min_size if min_size else 0.0
    return max(seq_ratio, jaccard, overlap)


def validate(path: Path) -> dict:
    """Validate MEMORY.md structure and return diagnostics."""
    if not path.exists():
        return {"valid": False, "errors": ["MEMORY.md does not exist"], "warnings": []}

    content = path.read_text(encoding="utf-8")
    errors = []
    warnings = []

    valid_headings = ("# Agent Memory", "# User Memory",
                       "# Daneel Agent Memory", "# Daneel User Memory")
    if not any(h in content for h in valid_headings):
        errors.append(
            "Missing top-level heading (expected '# Agent Memory' or '# User Memory')"
        )
    if "## Experiences" not in content:
        errors.append("Missing section '## Experiences'")
    if "## World Knowledge" not in content:
        errors.append("Missing section '## World Knowledge'")
    if "## Beliefs" not in content:
        errors.append("Missing section '## Beliefs'")
    if "## Entity Summaries" not in content:
        errors.append("Missing section '## Entity Summaries'")

    bank = recall_mod.parse_memory_file(path)

    for i, exp in enumerate(bank.experiences):
        if not exp.date:
            warnings.append(f"Experience {i}: missing date")
        if not exp.entities:
            warnings.append(f"Experience {i}: no entity tags")
        if len(exp.text) < 20:
            warnings.append(f"Experience {i}: very short text ({len(exp.text)} chars) — may lack narrative quality")

    for i, wf in enumerate(bank.world_knowledge):
        if wf.confidence is None:
            warnings.append(f"World fact {i}: missing confidence score")
        elif not 0.0 <= wf.confidence <= 1.0:
            errors.append(f"World fact {i}: confidence must be between 0.0 and 1.0")
        if not wf.entities:
            warnings.append(f"World fact {i}: no entity tags")

    for i, b in enumerate(bank.beliefs):
        if b.confidence is None:
            warnings.append(f"Belief {i}: missing confidence score")
        elif not 0.0 <= b.confidence <= 1.0:
            errors.append(f"Belief {i}: confidence must be between 0.0 and 1.0")
        if b.formed is None:
            warnings.append(f"Belief {i}: missing formed date")
        if not b.entities:
            warnings.append(f"Belief {i}: no entity tags")

    return {
        "valid": len(errors) == 0,
        "errors": errors,
        "warnings": warnings,
        "counts": {
            "experiences": len(bank.experiences),
            "world_knowledge": len(bank.world_knowledge),
            "beliefs": len(bank.beliefs),
            "entity_summaries": len(bank.entity_summaries),
        },
    }


def check_duplicate(path: Path, section: str, candidate: str,
                     extra_paths: Optional[list[tuple[str, Path]]] = None) -> dict:
    """Check if a candidate text is a near-duplicate of existing entries.

    When extra_paths is provided, also checks those files for duplicates
    (used for cross-scope checking: e.g. check user memory candidate
    against project memory too).
    """
    sources: list[tuple[str, Path]] = [("target", path)]
    if extra_paths:
        sources.extend(extra_paths)

    candidate_norm = normalize_for_comparison(candidate)
    matches = []

    for source_label, source_path in sources:
        bank = recall_mod.parse_memory_file(source_path)

        items: list[str] = []
        if section == "experiences":
            items = [e.text for e in bank.experiences]
        elif section == "world_knowledge":
            items = [w.text for w in bank.world_knowledge]
        elif section == "beliefs":
            items = [b.text for b in bank.beliefs]

        for i, item in enumerate(items):
            sim = similarity(candidate, item)
            if sim >= DUPLICATE_THRESHOLD:
                entry = {
                    "index": i,
                    "similarity": round(sim, 3),
                    "existing_text": item,
                }
                if source_label != "target":
                    entry["source"] = source_label
                matches.append(entry)

    matches.sort(key=lambda m: m["similarity"], reverse=True)

    return {
        "is_duplicate": len(matches) > 0,
        "candidate_normalized": candidate_norm,
        "matches": matches,
    }


def update_confidence(path: Path, section: str, index: int, delta: float) -> dict:
    """Deterministically update a confidence score and rewrite the file.

    delta > 0 reinforces, delta < 0 weakens. Clamped to [0.0, 1.0].
    """
    if section not in ("beliefs", "world_knowledge"):
        return {"success": False, "error": f"Section '{section}' does not have confidence scores"}

    content = path.read_text(encoding="utf-8")
    original_hash = content_hash(content)
    lines = content.splitlines()

    section_header = "## Beliefs" if section == "beliefs" else "## World Knowledge"
    in_section = False
    in_comment = False
    item_count = 0
    target_line_idx = None

    for i, line in enumerate(lines):
        stripped = line.strip()
        if in_comment:
            if "-->" in stripped:
                in_comment = False
            continue
        if stripped.startswith("<!--"):
            if "-->" not in stripped:
                in_comment = True
            continue
        if stripped == section_header:
            in_section = True
            continue
        if stripped.startswith("## ") and in_section:
            break
        if in_section and stripped.startswith("- "):
            if item_count == index:
                target_line_idx = i
                break
            item_count += 1

    if target_line_idx is None:
        return {"success": False, "error": f"Index {index} not found in {section}"}

    line = lines[target_line_idx]
    conf_match = recall_mod.CONFIDENCE_RE.search(line)
    if not conf_match:
        return {"success": False, "error": f"No confidence score found at index {index}"}

    old_conf = float(conf_match.group(1))
    new_conf = round(max(0.0, min(1.0, old_conf + delta)), 2)

    new_line = line[:conf_match.start(1)] + f"{new_conf}" + line[conf_match.end(1):]

    if section == "beliefs":
        today = __import__("datetime").date.today().isoformat()
        updated_match = recall_mod.UPDATED_RE.search(new_line)
        if updated_match:
            new_line = new_line[:updated_match.start(1)] + today + new_line[updated_match.end(1):]

    lines[target_line_idx] = new_line
    write_result = write_text_if_unchanged(path, "\n".join(lines) + "\n", original_hash)
    if not write_result["success"]:
        return write_result

    return {
        "success": True,
        "section": section,
        "index": index,
        "old_confidence": old_conf,
        "new_confidence": new_conf,
        "delta": delta,
    }


def extract_entities(text: str) -> dict:
    """Extract candidate entity names from free text.

    Uses heuristic patterns: PascalCase words, hyphenated identifiers,
    and backtick-quoted terms. The subagent uses this as a suggestion
    list and applies judgment to finalize the entity set.
    """
    candidates = set()
    for match in ENTITY_PATTERN.finditer(text):
        term = match.group().strip("`")
        if term.lower() not in STOPWORDS and len(term) > 1 and term not in PASCAL_COMMON:
            candidates.add(term)

    backtick_re = re.compile(r"`([^`]+)`")
    for m in backtick_re.finditer(text):
        term = m.group(1).strip()
        if term and len(term) > 1:
            candidates.add(term)

    for token in re.findall(r"\b[a-z][a-z0-9_-]*\b", text.lower()):
        if token in LOWERCASE_ENTITY_ALIASES:
            candidates.add(token)

    canonical_candidates = canonicalize_entities(sorted(candidates))

    return {
        "candidates": canonical_candidates,
        "count": len(canonical_candidates),
    }


def prune_beliefs(path: Path, threshold: float) -> dict:
    """Identify beliefs below the confidence threshold for removal."""
    bank = recall_mod.parse_memory_file(path)
    prunable = []
    for i, b in enumerate(bank.beliefs):
        if b.confidence is not None and b.confidence < threshold:
            prunable.append({
                "index": i,
                "confidence": b.confidence,
                "text": b.text,
            })
    return {
        "threshold": threshold,
        "prunable_count": len(prunable),
        "prunable": prunable,
        "total_beliefs": len(bank.beliefs),
    }


CONFLICT_SIGNALS = {
    "positive": frozenset({
        "reliable", "better", "preferred", "recommended", "effective",
        "faster", "easier", "useful", "valuable", "important", "strong",
        "always", "best", "safe", "stable", "consistent",
    }),
    "negative": frozenset({
        "unreliable", "worse", "avoid", "fragile", "slow", "harder",
        "useless", "risky", "dangerous", "weak", "never", "worst",
        "unstable", "inconsistent", "broken", "fails", "flawed",
    }),
}


def _sentiment_words(text: str) -> tuple[set[str], set[str]]:
    """Extract positive and negative sentiment words from text."""
    words = set(normalize_for_comparison(text).split())
    pos = words & CONFLICT_SIGNALS["positive"]
    neg = words & CONFLICT_SIGNALS["negative"]
    return pos, neg


def check_conflicts(path: Path) -> dict:
    """Detect potential contradictions between belief pairs.

    Two beliefs conflict when they share entities but express opposing
    sentiment. Returns pairs with a conflict score and recommendation.
    """
    bank = recall_mod.parse_memory_file(path)
    conflicts = []

    for i, a in enumerate(bank.beliefs):
        for j, b in enumerate(bank.beliefs):
            if j <= i:
                continue

            shared_entities = set(a.entities) & set(b.entities)
            if not shared_entities:
                continue

            text_sim = similarity(a.text, b.text)
            if text_sim < 0.3:
                continue

            pos_a, neg_a = _sentiment_words(a.text)
            pos_b, neg_b = _sentiment_words(b.text)

            opposing = (pos_a & neg_b) or (neg_a & pos_b)
            a_positive = len(pos_a) > len(neg_a)
            b_positive = len(pos_b) > len(neg_b)
            sentiment_conflict = (a_positive != b_positive) and (pos_a or neg_a) and (pos_b or neg_b)

            if not opposing and not sentiment_conflict:
                continue

            conf_a = a.confidence if a.confidence is not None else 0.5
            conf_b = b.confidence if b.confidence is not None else 0.5

            conflicts.append({
                "belief_a": {"index": i, "text": a.text, "confidence": conf_a},
                "belief_b": {"index": j, "text": b.text, "confidence": conf_b},
                "shared_entities": sorted(shared_entities),
                "text_similarity": round(text_sim, 3),
                "recommendation": (
                    f"keep index {i} (higher confidence)"
                    if conf_a > conf_b
                    else f"keep index {j} (higher confidence)"
                    if conf_b > conf_a
                    else "merge into a nuanced belief"
                ),
            })

    conflicts.sort(key=lambda c: c["text_similarity"], reverse=True)

    return {
        "conflict_count": len(conflicts),
        "conflicts": conflicts,
        "total_beliefs": len(bank.beliefs),
    }


def suggest_summaries(path: Path) -> dict:
    """Identify entities with 3+ mentions that lack a summary."""
    bank = recall_mod.parse_memory_file(path)
    entity_index = recall_mod.collect_all_entities(bank)

    entity_counts: dict[str, int] = {}
    for exp in bank.experiences:
        for e in exp.entities:
            entity_counts[e] = entity_counts.get(e, 0) + 1
    for wf in bank.world_knowledge:
        for e in wf.entities:
            entity_counts[e] = entity_counts.get(e, 0) + 1
    for b in bank.beliefs:
        for e in b.entities:
            entity_counts[e] = entity_counts.get(e, 0) + 1

    existing_summaries = {es.name.lower() for es in bank.entity_summaries}
    suggestions = []
    for entity, count in sorted(entity_counts.items(), key=lambda x: -x[1]):
        if count >= 3 and entity.lower() not in existing_summaries:
            suggestions.append({
                "entity": entity,
                "mention_count": count,
                "sections": entity_index.get(entity, []),
            })

    return {
        "suggestions": suggestions,
        "existing_summary_count": len(bank.entity_summaries),
    }


def init_user() -> dict:
    """Create the user memory directory and template file."""
    path = recall_mod.ensure_user_memory()
    return {
        "success": True,
        "path": str(path),
        "created": path.exists(),
    }


def _ensure_memory_file(path: Path, scope_label: str) -> Path:
    """Create a template memory file when needed."""
    if path.exists():
        return path
    if scope_label == "user":
        return recall_mod.ensure_user_memory()

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        recall_mod.USER_MEMORY_TEMPLATE.replace("# User Memory", "# Agent Memory"),
        encoding="utf-8",
    )
    return path


def _insert_entry(content: str, section: str, raw_line: str) -> tuple[bool, Optional[str]]:
    """Insert a raw line at the top of a target section."""
    section_headers = {
        "experiences": "## Experiences",
        "world_knowledge": "## World Knowledge",
        "beliefs": "## Beliefs",
    }
    header = section_headers[section]

    lines = content.splitlines()
    insert_idx = None
    in_section = False
    in_comment = False

    for i, line in enumerate(lines):
        stripped = line.strip()
        if in_comment:
            if "-->" in stripped:
                in_comment = False
                insert_idx = i + 1
            continue
        if stripped.startswith("<!--"):
            if "-->" not in stripped:
                in_comment = True
            elif in_section:
                insert_idx = i + 1
            continue
        if stripped == header:
            in_section = True
            insert_idx = i + 1
            continue
        if in_section:
            if stripped.startswith("## "):
                break
            if stripped.startswith("- "):
                insert_idx = i
                break
            if not stripped:
                insert_idx = i
                continue

    if insert_idx is None:
        return False, None

    lines.insert(insert_idx, raw_line)
    return True, "\n".join(lines) + "\n"


def append_entry(
    path: Path,
    *,
    section: str,
    text: str,
    scope_label: str,
    date: Optional[str] = None,
    context: Optional[str] = None,
    entities: Optional[list[str]] = None,
    confidence: Optional[float] = None,
    sources: Optional[int] = None,
    formed: Optional[str] = None,
    updated: Optional[str] = None,
    cross_scope_path: Optional[Path] = None,
) -> dict:
    """Append a new entry after safety, duplicate, and format checks."""
    path = _ensure_memory_file(path, scope_label)
    screening = screen_text(text)
    if not screening["safe"]:
        return {
            "success": False,
            "error": "Sensitive content detected; refusing to persist memory text",
            "issues": screening["issues"],
            "sanitized_text": screening["sanitized_text"],
        }

    normalized_entities = canonicalize_entities(
        entities or extract_entities(screening["sanitized_text"])["candidates"]
    )
    if not normalized_entities:
        return {"success": False, "error": "At least one entity tag is required"}

    normalized_context = normalize_context_tag(context)
    if context and normalized_context is None:
        return {"success": False, "error": f"Unknown context tag '{context}'"}

    duplicate = check_duplicate(
        path,
        section,
        screening["sanitized_text"],
        extra_paths=[("other-scope", cross_scope_path)] if cross_scope_path and cross_scope_path.exists() else None,
    )
    if duplicate["is_duplicate"]:
        return {
            "success": False,
            "error": "Duplicate or near-duplicate memory already exists",
            "matches": duplicate["matches"],
        }

    build_result = _build_entry_line(
        section=section,
        text=screening["sanitized_text"],
        date=date,
        context=normalized_context,
        entities=normalized_entities,
        confidence=confidence,
        sources=sources,
        formed=formed,
        updated=updated,
    )
    if "error" in build_result:
        return {"success": False, "error": build_result["error"]}

    raw_line = build_result["raw_line"]
    original_content = path.read_text(encoding="utf-8")
    original_hash = content_hash(original_content)
    inserted, new_content = _insert_entry(original_content, section, raw_line)
    if not inserted or new_content is None:
        return {"success": False, "error": f"Could not find section '{section}' in memory file"}

    write_result = write_text_if_unchanged(path, new_content, original_hash)
    if not write_result["success"]:
        return write_result

    return {
        "success": True,
        "section": section,
        "path": str(path),
        "entry": raw_line,
        "entities": normalized_entities,
        "context": normalized_context,
    }


def _build_entry_line(
    *,
    section: str,
    text: str,
    date: Optional[str],
    context: Optional[str],
    entities: list[str],
    confidence: Optional[float],
    sources: Optional[int],
    formed: Optional[str],
    updated: Optional[str],
) -> dict:
    """Build one formatted memory line for the target section."""
    if section == "experiences":
        if not date:
            return {"error": "Experiences require a date"}
        raw_line = f"- **{date}**"
        if context:
            raw_line += f" [{context}]"
        raw_line += f" {{entities: {', '.join(entities)}}} {text}"
    elif section == "world_knowledge":
        if confidence is None or sources is None:
            return {"error": "World knowledge requires confidence and sources"}
        raw_line = (
            f"- {{entities: {', '.join(entities)}}} {text} "
            f"(confidence: {confidence:.2f}, sources: {sources})"
        )
    elif section == "beliefs":
        if confidence is None:
            return {"error": "Beliefs require confidence"}
        formed_value = formed or __import__("datetime").date.today().isoformat()
        updated_value = updated or formed_value
        raw_line = (
            f"- {{entities: {', '.join(entities)}}} {text} "
            f"(confidence: {confidence:.2f}, formed: {formed_value}, updated: {updated_value})"
        )
    else:
        return {"error": f"Cannot append to section '{section}'"}

    return {"raw_line": raw_line}


def promote(
    user_path: Path,
    project_path: Path,
    section: str,
    index: int,
    *,
    allow_project_promotion: bool = False,
) -> dict:
    """Copy a memory entry from user scope to project scope.

    The entry remains in user memory (not deleted). The caller can
    remove it from user memory separately if desired.
    """
    if not allow_project_promotion:
        return {
            "success": False,
            "error": "Promotion requires explicit --allow-project-promotion approval",
        }

    user_bank = recall_mod.parse_memory_file(user_path)
    _ensure_memory_file(project_path, "project")
    project_content = project_path.read_text(encoding="utf-8")
    original_hash = content_hash(project_content)

    raw_line: Optional[str] = None
    entry_context: Optional[str] = None
    entry_text: Optional[str] = None
    if section == "experiences":
        if index >= len(user_bank.experiences):
            return {"success": False, "error": f"Index {index} out of range in user experiences"}
        entry = user_bank.experiences[index]
        raw_line = entry.raw
        entry_context = entry.context
        entry_text = entry.text
        dup = check_duplicate(project_path, section, entry.text)
    elif section == "world_knowledge":
        if index >= len(user_bank.world_knowledge):
            return {"success": False, "error": f"Index {index} out of range in user world_knowledge"}
        entry = user_bank.world_knowledge[index]
        raw_line = entry.raw
        entry_text = entry.text
        dup = check_duplicate(project_path, section, entry.text)
    elif section == "beliefs":
        if index >= len(user_bank.beliefs):
            return {"success": False, "error": f"Index {index} out of range in user beliefs"}
        entry = user_bank.beliefs[index]
        raw_line = entry.raw
        entry_text = entry.text
        dup = check_duplicate(project_path, section, entry.text)
    else:
        return {"success": False, "error": f"Cannot promote from section '{section}'"}

    if dup["is_duplicate"]:
        return {
            "success": False,
            "error": "Duplicate already exists in project memory",
            "matches": dup["matches"],
        }

    if section == "experiences" and entry_context == "preference":
        return {
            "success": False,
            "error": "preference experiences cannot be promoted to project memory",
        }

    screening = screen_text(entry_text or "")
    if not screening["safe"]:
        return {
            "success": False,
            "error": "Sensitive content detected; refusing to promote memory",
            "issues": screening["issues"],
        }

    inserted, new_content = _insert_entry(project_content, section, raw_line)
    if not inserted or new_content is None:
        return {"success": False, "error": f"Could not find section '{section}' in project memory"}

    write_result = write_text_if_unchanged(project_path, new_content, original_hash)
    if not write_result["success"]:
        return write_result

    return {
        "success": True,
        "section": section,
        "index": index,
        "promoted_text": raw_line,
        "target": str(project_path),
    }


def find_matches(path: Path, query: str, threshold: float = 0.4) -> dict:
    """Fuzzy-search all sections for memories matching a query.

    Returns candidates sorted by similarity, grouped by section,
    with indices for use with delete-entry.
    Uses a lower threshold than duplicate detection since the user
    is describing a memory from (possibly faulty) recollection.
    """
    bank = recall_mod.parse_memory_file(path)
    matches = []

    sections: list[tuple[str, list]] = [
        ("experiences", bank.experiences),
        ("world_knowledge", bank.world_knowledge),
        ("beliefs", bank.beliefs),
        ("reflections", bank.reflections),
    ]

    for section_name, items in sections:
        for i, item in enumerate(items):
            text = item.text
            sim = similarity(query, text)
            if sim >= threshold:
                entry: dict = {
                    "section": section_name,
                    "index": i,
                    "similarity": round(sim, 3),
                    "text": text,
                    "raw": item.raw,
                }
                if hasattr(item, "date") and item.date:
                    entry["date"] = item.date
                if hasattr(item, "confidence") and item.confidence is not None:
                    entry["confidence"] = item.confidence
                matches.append(entry)

    matches.sort(key=lambda m: m["similarity"], reverse=True)

    return {
        "query": query,
        "threshold": threshold,
        "match_count": len(matches),
        "matches": matches,
    }


def delete_entry(path: Path, section: str, index: int) -> dict:
    """Remove a specific memory entry by section and index.

    Uses the guarded-write path to prevent stale overwrites.
    """
    valid_sections = ("experiences", "world_knowledge", "beliefs", "reflections")
    if section not in valid_sections:
        return {"success": False, "error": f"Cannot delete from section '{section}'"}

    if not path.exists():
        return {"success": False, "error": "Memory file does not exist"}

    content = path.read_text(encoding="utf-8")
    original_hash = content_hash(content)
    lines = content.splitlines()

    section_headers = {
        "experiences": "## Experiences",
        "world_knowledge": "## World Knowledge",
        "beliefs": "## Beliefs",
        "reflections": "## Reflections",
    }
    header = section_headers[section]
    in_section = False
    in_comment = False
    item_count = 0
    target_line_idx = None

    for i, line in enumerate(lines):
        stripped = line.strip()
        if in_comment:
            if "-->" in stripped:
                in_comment = False
            continue
        if stripped.startswith("<!--"):
            if "-->" not in stripped:
                in_comment = True
            continue
        if stripped == header:
            in_section = True
            continue
        if stripped.startswith("## ") and in_section:
            break
        if in_section and stripped.startswith("- "):
            if item_count == index:
                target_line_idx = i
                break
            item_count += 1

    if target_line_idx is None:
        return {"success": False, "error": f"Index {index} not found in {section}"}

    deleted_line = lines[target_line_idx]
    del lines[target_line_idx]

    write_result = write_text_if_unchanged(path, "\n".join(lines) + "\n", original_hash)
    if not write_result["success"]:
        return write_result

    return {
        "success": True,
        "section": section,
        "index": index,
        "deleted": deleted_line,
    }


def main():
    parser = argparse.ArgumentParser(description="Memory management operations")
    parser.add_argument("--file", type=Path, default=None,
                        help="Explicit path to MEMORY.md (overrides --scope)")
    parser.add_argument("--scope", choices=["user", "project"], default=None,
                        help="Memory scope (default: project for validate/suggest, user for writes)")
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("validate", help="Validate MEMORY.md structure")

    dup_parser = sub.add_parser("check-duplicate", help="Check for near-duplicates")
    dup_parser.add_argument("--section", required=True,
                            choices=["experiences", "world_knowledge", "beliefs"])
    dup_parser.add_argument("--candidate", required=True, help="Candidate text")
    dup_parser.add_argument("--cross-scope", action="store_true",
                            help="Also check the other scope for duplicates")

    conf_parser = sub.add_parser("update-confidence", help="Update a confidence score")
    conf_parser.add_argument("--section", required=True,
                             choices=["beliefs", "world_knowledge"])
    conf_parser.add_argument("--index", required=True, type=int)
    conf_parser.add_argument("--delta", required=True, type=float,
                             help="Amount to add (positive=reinforce, negative=weaken)")

    ent_parser = sub.add_parser("extract-entities", help="Extract entity candidates from text")
    ent_parser.add_argument("--text", required=True)

    screen_parser = sub.add_parser("screen-text", help="Screen text for secrets or sensitive content")
    screen_parser.add_argument("--text", required=True)

    prune_parser = sub.add_parser("prune-beliefs", help="Find beliefs below confidence threshold")
    prune_parser.add_argument("--threshold", type=float, default=0.2)

    sub.add_parser("suggest-summaries", help="Suggest entities needing summaries")
    sub.add_parser("check-conflicts", help="Detect contradictions between belief pairs")
    sub.add_parser("init-user", help="Create user memory directory and template")

    find_parser = sub.add_parser("find-matches", help="Fuzzy-search memories for forget operation")
    find_parser.add_argument("--query", required=True, help="Fuzzy description of the memory to find")
    find_parser.add_argument("--threshold", type=float, default=0.4,
                             help="Minimum similarity (default: 0.4, lower than dedup)")

    del_parser = sub.add_parser("delete-entry", help="Delete a specific memory entry by section and index")
    del_parser.add_argument("--section", required=True,
                            choices=["experiences", "world_knowledge", "beliefs", "reflections"])
    del_parser.add_argument("--index", required=True, type=int)

    append_parser = sub.add_parser("append-entry", help="Append a new memory entry safely")
    append_parser.add_argument("--section", required=True,
                               choices=["experiences", "world_knowledge", "beliefs"])
    append_parser.add_argument("--text", required=True)
    append_parser.add_argument("--scope", choices=["user", "project"], default="user")
    append_parser.add_argument("--date", help="Required for experiences")
    append_parser.add_argument("--context", help="Optional context tag for experiences")
    append_parser.add_argument("--entities", default="",
                               help="Comma-separated entity tags; omitted = auto-extract")
    append_parser.add_argument("--confidence", type=float)
    append_parser.add_argument("--sources", type=int)
    append_parser.add_argument("--formed")
    append_parser.add_argument("--updated")

    promote_parser = sub.add_parser("promote", help="Copy a memory from user to project scope")
    promote_parser.add_argument("--section", required=True,
                                choices=["experiences", "world_knowledge", "beliefs"])
    promote_parser.add_argument("--index", required=True, type=int,
                                help="Index of the entry in user memory to promote")
    promote_parser.add_argument("--allow-project-promotion", action="store_true",
                                help="Required explicit approval before writing to project memory")

    args = parser.parse_args()

    def get_path(default_scope: str = "project") -> Path:
        if args.file:
            return args.file
        scope = args.scope or default_scope
        return resolve_path(scope)

    if args.command == "validate":
        result = validate(get_path("project"))
    elif args.command == "check-duplicate":
        target_path = get_path("user")
        extra: Optional[list[tuple[str, Path]]] = None
        if getattr(args, "cross_scope", False):
            other_scope = "project" if (args.scope or "user") == "user" else "user"
            other_path = resolve_path(other_scope)
            if other_path.exists():
                extra = [(other_scope, other_path)]
        result = check_duplicate(target_path, args.section, args.candidate,
                                  extra_paths=extra)
    elif args.command == "update-confidence":
        result = update_confidence(get_path("user"), args.section, args.index, args.delta)
    elif args.command == "extract-entities":
        result = extract_entities(args.text)
    elif args.command == "screen-text":
        result = screen_text(args.text)
    elif args.command == "prune-beliefs":
        result = prune_beliefs(get_path("user"), args.threshold)
    elif args.command == "suggest-summaries":
        result = suggest_summaries(get_path("user"))
    elif args.command == "check-conflicts":
        result = check_conflicts(get_path("user"))
    elif args.command == "init-user":
        result = init_user()
    elif args.command == "find-matches":
        result = find_matches(get_path("user"), args.query, args.threshold)
    elif args.command == "delete-entry":
        result = delete_entry(get_path("user"), args.section, args.index)
    elif args.command == "append-entry":
        target_path = get_path(args.scope)
        other_scope = "project" if args.scope == "user" else "user"
        entity_list = [e.strip() for e in args.entities.split(",") if e.strip()]
        result = append_entry(
            target_path,
            section=args.section,
            text=args.text,
            scope_label=args.scope,
            date=args.date,
            context=args.context,
            entities=entity_list,
            confidence=args.confidence,
            sources=args.sources,
            formed=args.formed,
            updated=args.updated,
            cross_scope_path=resolve_path(other_scope),
        )
    elif args.command == "promote":
        result = promote(USER_MEMORY_PATH, PROJECT_MEMORY_PATH,
                         args.section, args.index,
                         allow_project_promotion=args.allow_project_promotion)
    else:
        parser.error(f"Unknown command: {args.command}")
        return

    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
