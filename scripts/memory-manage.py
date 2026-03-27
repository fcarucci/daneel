#!/usr/bin/env python3
"""Deterministic memory management operations for MEMORY.md.

Supports two memory tiers:
  - User memory:    ~/.config/daneel/MEMORY.md  (personal, default for writes)
  - Project memory: <repo>/MEMORY.md            (shared, explicit promotion only)

Usage:
    python scripts/memory-manage.py validate
    python scripts/memory-manage.py validate --scope user
    python scripts/memory-manage.py check-duplicate --section experiences --candidate "text"
    python scripts/memory-manage.py check-duplicate --section experiences --candidate "text" --scope both
    python scripts/memory-manage.py update-confidence --section beliefs --index 0 --delta 0.1
    python scripts/memory-manage.py update-confidence --section beliefs --index 0 --delta 0.1 --scope user
    python scripts/memory-manage.py extract-entities --text "some text"
    python scripts/memory-manage.py prune-beliefs --threshold 0.2
    python scripts/memory-manage.py suggest-summaries
    python scripts/memory-manage.py init-user
    python scripts/memory-manage.py promote --section experiences --index 0
"""

import argparse
import json
import re
import sys
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
        if not wf.entities:
            warnings.append(f"World fact {i}: no entity tags")

    for i, b in enumerate(bank.beliefs):
        if b.confidence is None:
            warnings.append(f"Belief {i}: missing confidence score")
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
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")

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

    return {
        "candidates": sorted(candidates),
        "count": len(candidates),
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


def promote(user_path: Path, project_path: Path, section: str, index: int) -> dict:
    """Copy a memory entry from user scope to project scope.

    The entry remains in user memory (not deleted). The caller can
    remove it from user memory separately if desired.
    """
    user_bank = recall_mod.parse_memory_file(user_path)
    project_content = project_path.read_text(encoding="utf-8") if project_path.exists() else ""

    raw_line: Optional[str] = None
    if section == "experiences":
        if index >= len(user_bank.experiences):
            return {"success": False, "error": f"Index {index} out of range in user experiences"}
        entry = user_bank.experiences[index]
        raw_line = entry.raw
        dup = check_duplicate(project_path, section, entry.text)
    elif section == "world_knowledge":
        if index >= len(user_bank.world_knowledge):
            return {"success": False, "error": f"Index {index} out of range in user world_knowledge"}
        entry = user_bank.world_knowledge[index]
        raw_line = entry.raw
        dup = check_duplicate(project_path, section, entry.text)
    elif section == "beliefs":
        if index >= len(user_bank.beliefs):
            return {"success": False, "error": f"Index {index} out of range in user beliefs"}
        entry = user_bank.beliefs[index]
        raw_line = entry.raw
        dup = check_duplicate(project_path, section, entry.text)
    else:
        return {"success": False, "error": f"Cannot promote from section '{section}'"}

    if dup["is_duplicate"]:
        return {
            "success": False,
            "error": "Duplicate already exists in project memory",
            "matches": dup["matches"],
        }

    section_headers = {
        "experiences": "## Experiences",
        "world_knowledge": "## World Knowledge",
        "beliefs": "## Beliefs",
    }
    header = section_headers[section]

    lines = project_content.splitlines()
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
            else:
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
        return {"success": False, "error": f"Could not find section '{header}' in project memory"}

    lines.insert(insert_idx, raw_line)
    project_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    return {
        "success": True,
        "section": section,
        "index": index,
        "promoted_text": raw_line,
        "target": str(project_path),
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

    prune_parser = sub.add_parser("prune-beliefs", help="Find beliefs below confidence threshold")
    prune_parser.add_argument("--threshold", type=float, default=0.2)

    sub.add_parser("suggest-summaries", help="Suggest entities needing summaries")
    sub.add_parser("init-user", help="Create user memory directory and template")

    promote_parser = sub.add_parser("promote", help="Copy a memory from user to project scope")
    promote_parser.add_argument("--section", required=True,
                                choices=["experiences", "world_knowledge", "beliefs"])
    promote_parser.add_argument("--index", required=True, type=int,
                                help="Index of the entry in user memory to promote")

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
    elif args.command == "prune-beliefs":
        result = prune_beliefs(get_path("user"), args.threshold)
    elif args.command == "suggest-summaries":
        result = suggest_summaries(get_path("user"))
    elif args.command == "init-user":
        result = init_user()
    elif args.command == "promote":
        result = promote(USER_MEMORY_PATH, PROJECT_MEMORY_PATH,
                         args.section, args.index)
    else:
        parser.error(f"Unknown command: {args.command}")
        return

    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
