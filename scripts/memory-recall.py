#!/usr/bin/env python3
"""Structured recall over MEMORY.md (user and project scopes).

Supports two memory tiers:
  - User memory:    ~/.config/daneel/MEMORY.md  (personal, default)
  - Project memory: <repo>/MEMORY.md            (shared, committed)

Recall searches both by default. Results are tagged with their source
scope so the caller can distinguish personal from shared memories.

Usage:
    python scripts/memory-recall.py --keyword "gateway"
    python scripts/memory-recall.py --entity "dx-serve" --scope user
    python scripts/memory-recall.py --scope project --stats
    python scripts/memory-recall.py --since 2026-03-01 --until 2026-03-31
    python scripts/memory-recall.py --section experiences --keyword "debug"
    python scripts/memory-recall.py --entity "tailwind" --cross-section
    python scripts/memory-recall.py --stats
"""

import argparse
import json
import re
import sys
from dataclasses import dataclass, field, asdict
from datetime import date, datetime
from pathlib import Path
from typing import Optional


PROJECT_MEMORY_PATH = Path(__file__).resolve().parent.parent / "MEMORY.md"
USER_MEMORY_DIR = Path.home() / ".agents" / "memory"
USER_MEMORY_PATH = USER_MEMORY_DIR / "MEMORY.md"

MEMORY_PATH = PROJECT_MEMORY_PATH

SCOPES = ("user", "project", "both")

USER_MEMORY_TEMPLATE = """\
# User Memory

## Experiences

<!-- Newest first. Format: - **YYYY-MM-DD** [context] {entities: e1, e2} Narrative memory text. -->

## World Knowledge

<!-- Verified, objective facts about the project and environment. Format:
- {entities: e1} Fact text. (confidence: 0.XX, sources: N) -->

## Beliefs

<!-- Agent's subjective judgments that evolve over time. Format:
- {entities: e1} Belief text. (confidence: 0.XX, formed: YYYY-MM-DD, updated: YYYY-MM-DD) -->

## Entity Summaries

<!-- Synthesized profiles of key entities, regenerated when underlying memories change. Format:
### entity-name
Summary paragraph. -->
"""


def resolve_memory_paths(scope: str) -> list[tuple[str, Path]]:
    """Return (label, path) pairs for the requested scope."""
    if scope == "user":
        return [("user", USER_MEMORY_PATH)]
    elif scope == "project":
        return [("project", PROJECT_MEMORY_PATH)]
    else:
        return [("user", USER_MEMORY_PATH), ("project", PROJECT_MEMORY_PATH)]


def ensure_user_memory() -> Path:
    """Create user memory dir and template if they don't exist."""
    USER_MEMORY_DIR.mkdir(parents=True, exist_ok=True)
    if not USER_MEMORY_PATH.exists():
        USER_MEMORY_PATH.write_text(USER_MEMORY_TEMPLATE, encoding="utf-8")
    return USER_MEMORY_PATH

SECTION_NAMES = ("experiences", "world_knowledge", "beliefs", "entity_summaries")

ENTITY_RE = re.compile(r"\{entities:\s*([^}]+)\}")
DATE_RE = re.compile(r"\*\*(\d{4}-\d{2}-\d{2})\*\*")
CONFIDENCE_RE = re.compile(r"\(confidence:\s*([\d.]+)")
SOURCES_RE = re.compile(r"sources:\s*(\d+)\)")
FORMED_RE = re.compile(r"formed:\s*(\d{4}-\d{2}-\d{2})")
UPDATED_RE = re.compile(r"updated:\s*(\d{4}-\d{2}-\d{2})")
CONTEXT_RE = re.compile(r"\[(\w[\w-]*)\]")
SUMMARY_HEADING_RE = re.compile(r"^###\s+(.+)$")


@dataclass
class Experience:
    date: Optional[str]
    context: Optional[str]
    entities: list[str]
    text: str
    raw: str

@dataclass
class WorldFact:
    entities: list[str]
    text: str
    confidence: Optional[float]
    sources: Optional[int]
    raw: str

@dataclass
class Belief:
    entities: list[str]
    text: str
    confidence: Optional[float]
    formed: Optional[str]
    updated: Optional[str]
    raw: str

@dataclass
class EntitySummary:
    name: str
    text: str
    raw: str

@dataclass
class MemoryBank:
    experiences: list[Experience] = field(default_factory=list)
    world_knowledge: list[WorldFact] = field(default_factory=list)
    beliefs: list[Belief] = field(default_factory=list)
    entity_summaries: list[EntitySummary] = field(default_factory=list)


def parse_entities(line: str) -> list[str]:
    m = ENTITY_RE.search(line)
    if not m:
        return []
    return [e.strip() for e in m.group(1).split(",") if e.strip()]


def strip_metadata(line: str) -> str:
    """Remove inline metadata markers to get the core text."""
    text = line.lstrip("- ").strip()
    text = DATE_RE.sub("", text).strip()
    text = CONTEXT_RE.sub("", text, count=1).strip()
    text = ENTITY_RE.sub("", text).strip()
    text = re.sub(r"\(confidence:.*?\)", "", text).strip()
    text = re.sub(r"\(sources:.*?\)", "", text).strip()
    text = re.sub(r"\(formed:.*?\)", "", text).strip()
    text = re.sub(r"\(updated:.*?\)", "", text).strip()
    return text


def parse_experience(line: str) -> Experience:
    date_match = DATE_RE.search(line)
    ctx_match = CONTEXT_RE.search(line)
    return Experience(
        date=date_match.group(1) if date_match else None,
        context=ctx_match.group(1) if ctx_match else None,
        entities=parse_entities(line),
        text=strip_metadata(line),
        raw=line,
    )


def parse_world_fact(line: str) -> WorldFact:
    conf_match = CONFIDENCE_RE.search(line)
    src_match = SOURCES_RE.search(line)
    return WorldFact(
        entities=parse_entities(line),
        text=strip_metadata(line),
        confidence=float(conf_match.group(1)) if conf_match else None,
        sources=int(src_match.group(1)) if src_match else None,
        raw=line,
    )


def parse_belief(line: str) -> Belief:
    conf_match = CONFIDENCE_RE.search(line)
    formed_match = FORMED_RE.search(line)
    updated_match = UPDATED_RE.search(line)
    return Belief(
        entities=parse_entities(line),
        text=strip_metadata(line),
        confidence=float(conf_match.group(1)) if conf_match else None,
        formed=formed_match.group(1) if formed_match else None,
        updated=updated_match.group(1) if updated_match else None,
        raw=line,
    )


def parse_memory_file(path: Path) -> MemoryBank:
    if not path.exists():
        return MemoryBank()

    content = path.read_text(encoding="utf-8")
    bank = MemoryBank()

    current_section = None
    summary_name = None
    summary_lines: list[str] = []

    in_comment = False

    for line in content.splitlines():
        stripped = line.strip()

        if in_comment:
            if "-->" in stripped:
                in_comment = False
            continue
        if stripped.startswith("<!--"):
            if "-->" not in stripped:
                in_comment = True
            continue

        if stripped == "## Experiences":
            _flush_summary(bank, summary_name, summary_lines)
            summary_name = None
            summary_lines = []
            current_section = "experiences"
            continue
        elif stripped == "## World Knowledge":
            _flush_summary(bank, summary_name, summary_lines)
            summary_name = None
            summary_lines = []
            current_section = "world_knowledge"
            continue
        elif stripped == "## Beliefs":
            _flush_summary(bank, summary_name, summary_lines)
            summary_name = None
            summary_lines = []
            current_section = "beliefs"
            continue
        elif stripped == "## Entity Summaries":
            _flush_summary(bank, summary_name, summary_lines)
            summary_name = None
            summary_lines = []
            current_section = "entity_summaries"
            continue
        elif stripped.startswith("## "):
            _flush_summary(bank, summary_name, summary_lines)
            summary_name = None
            summary_lines = []
            current_section = None
            continue

        if not stripped:
            continue

        if current_section == "experiences" and stripped.startswith("- "):
            bank.experiences.append(parse_experience(stripped))
        elif current_section == "world_knowledge" and stripped.startswith("- "):
            bank.world_knowledge.append(parse_world_fact(stripped))
        elif current_section == "beliefs" and stripped.startswith("- "):
            bank.beliefs.append(parse_belief(stripped))
        elif current_section == "entity_summaries":
            heading_match = SUMMARY_HEADING_RE.match(stripped)
            if heading_match:
                _flush_summary(bank, summary_name, summary_lines)
                summary_name = heading_match.group(1).strip()
                summary_lines = []
            elif summary_name is not None:
                summary_lines.append(stripped)

    _flush_summary(bank, summary_name, summary_lines)
    return bank


def _flush_summary(bank: MemoryBank, name: Optional[str], lines: list[str]):
    if name and lines:
        text = " ".join(l for l in lines if l).strip()
        if text:
            bank.entity_summaries.append(EntitySummary(
                name=name,
                text=text,
                raw=f"### {name}\n" + "\n".join(lines),
            ))


def collect_all_entities(bank: MemoryBank) -> dict[str, list[str]]:
    """Return a map of entity -> list of sections it appears in."""
    index: dict[str, set[str]] = {}
    for exp in bank.experiences:
        for e in exp.entities:
            index.setdefault(e, set()).add("experiences")
    for wf in bank.world_knowledge:
        for e in wf.entities:
            index.setdefault(e, set()).add("world_knowledge")
    for b in bank.beliefs:
        for e in b.entities:
            index.setdefault(e, set()).add("beliefs")
    for es in bank.entity_summaries:
        index.setdefault(es.name, set()).add("entity_summaries")
    return {k: sorted(v) for k, v in sorted(index.items())}


def recall(
    bank: MemoryBank,
    *,
    keyword: Optional[str] = None,
    entity: Optional[str] = None,
    since: Optional[str] = None,
    until: Optional[str] = None,
    section: Optional[str] = None,
    cross_section: bool = False,
) -> dict:
    """Filter memories by keyword, entity, date, or section.

    If cross_section is True and entity is specified, returns all
    memories across all sections that mention that entity.
    """
    results: dict = {
        "experiences": [],
        "world_knowledge": [],
        "beliefs": [],
        "entity_summaries": [],
    }

    sections_to_search = SECTION_NAMES
    if section and not cross_section:
        sections_to_search = (section,)
    if cross_section and entity:
        sections_to_search = SECTION_NAMES

    since_date = _parse_date(since) if since else None
    until_date = _parse_date(until) if until else None
    kw_lower = keyword.lower() if keyword else None
    ent_lower = entity.lower() if entity else None

    if "experiences" in sections_to_search:
        for exp in bank.experiences:
            if kw_lower and kw_lower not in exp.text.lower() and kw_lower not in exp.raw.lower():
                continue
            if ent_lower and not _entity_matches(exp.entities, ent_lower):
                continue
            if since_date and exp.date and _parse_date(exp.date) < since_date:
                continue
            if until_date and exp.date and _parse_date(exp.date) > until_date:
                continue
            results["experiences"].append(exp.raw)

    if "world_knowledge" in sections_to_search:
        for wf in bank.world_knowledge:
            if kw_lower and kw_lower not in wf.text.lower() and kw_lower not in wf.raw.lower():
                continue
            if ent_lower and not _entity_matches(wf.entities, ent_lower):
                continue
            results["world_knowledge"].append(wf.raw)

    if "beliefs" in sections_to_search:
        for b in bank.beliefs:
            if kw_lower and kw_lower not in b.text.lower() and kw_lower not in b.raw.lower():
                continue
            if ent_lower and not _entity_matches(b.entities, ent_lower):
                continue
            results["beliefs"].append(b.raw)

    if "entity_summaries" in sections_to_search:
        for es in bank.entity_summaries:
            if kw_lower and kw_lower not in es.text.lower() and kw_lower not in es.name.lower():
                continue
            if ent_lower and ent_lower not in es.name.lower():
                continue
            results["entity_summaries"].append(es.raw)

    return {k: v for k, v in results.items() if v}


def stats(bank: MemoryBank, label: Optional[str] = None) -> dict:
    entity_index = collect_all_entities(bank)
    result = {
        "counts": {
            "experiences": len(bank.experiences),
            "world_knowledge": len(bank.world_knowledge),
            "beliefs": len(bank.beliefs),
            "entity_summaries": len(bank.entity_summaries),
            "total": (
                len(bank.experiences) + len(bank.world_knowledge)
                + len(bank.beliefs) + len(bank.entity_summaries)
            ),
        },
        "unique_entities": len(entity_index),
        "entities": entity_index,
    }
    if label:
        result["scope"] = label
    return result


def merge_banks(banks: list[tuple[str, MemoryBank]]) -> MemoryBank:
    """Merge multiple labeled banks into one."""
    merged = MemoryBank()
    for _, bank in banks:
        merged.experiences.extend(bank.experiences)
        merged.world_knowledge.extend(bank.world_knowledge)
        merged.beliefs.extend(bank.beliefs)
        merged.entity_summaries.extend(bank.entity_summaries)
    return merged


def recall_multi(
    banks: list[tuple[str, MemoryBank]],
    **kwargs,
) -> dict:
    """Run recall across multiple scoped banks, tagging results by source."""
    combined: dict = {}
    for label, bank in banks:
        result = recall(bank, **kwargs)
        for section_name, items in result.items():
            tagged = [f"[{label}] {item}" for item in items]
            combined.setdefault(section_name, []).extend(tagged)
    return {k: v for k, v in combined.items() if v}


def stats_multi(banks: list[tuple[str, MemoryBank]]) -> dict:
    """Compute stats per scope and combined totals."""
    per_scope = []
    for label, bank in banks:
        per_scope.append(stats(bank, label=label))

    merged = merge_banks(banks)
    combined = stats(merged)
    combined["scope"] = "combined"
    combined["per_scope"] = per_scope
    return combined


def digest(
    banks: list[tuple[str, MemoryBank]],
    *,
    last: int = 5,
    days: Optional[int] = None,
) -> str:
    """Produce a human-readable memory digest for context injection.

    Shows world knowledge, beliefs, entity summaries, and recent
    experiences (bounded by count or day range). Each entry is tagged
    with its source scope.
    """
    lines: list[str] = []

    for label, bank in banks:
        has_content = (
            bank.world_knowledge or bank.beliefs
            or bank.entity_summaries or bank.experiences
        )
        if not has_content:
            continue

        lines.append(f"### [{label}] memory")
        lines.append("")

        if bank.world_knowledge:
            lines.append("**World Knowledge:**")
            for wf in bank.world_knowledge:
                conf = f" ({wf.confidence})" if wf.confidence is not None else ""
                lines.append(f"- {wf.text}{conf}")
            lines.append("")

        if bank.beliefs:
            lines.append("**Beliefs:**")
            for b in bank.beliefs:
                conf = f" ({b.confidence})" if b.confidence is not None else ""
                lines.append(f"- {b.text}{conf}")
            lines.append("")

        if bank.entity_summaries:
            lines.append("**Entity Summaries:**")
            for es in bank.entity_summaries:
                lines.append(f"- **{es.name}**: {es.text}")
            lines.append("")

        exps = _filter_experiences(bank.experiences, last=last, days=days)
        if exps:
            range_desc = f"last {days} days" if days else f"last {len(exps)}"
            lines.append(f"**Recent Experiences ({range_desc}):**")
            for exp in exps:
                date_str = exp.date or "unknown"
                ctx = f" [{exp.context}]" if exp.context else ""
                lines.append(f"- {date_str}{ctx}: {exp.text}")
            lines.append("")

    if not lines:
        return "(no memories found)"
    return "\n".join(lines).rstrip()


def _filter_experiences(
    experiences: list[Experience],
    *,
    last: int = 5,
    days: Optional[int] = None,
) -> list[Experience]:
    """Return recent experiences by count or day range."""
    if days is not None:
        cutoff = (datetime.now().date() - __import__("datetime").timedelta(days=days))
        return [
            e for e in experiences
            if e.date and _parse_date(e.date) >= cutoff
        ]
    return experiences[:last]


def _entity_matches(entities: list[str], query: str) -> bool:
    return any(query in e.lower() for e in entities)


def _parse_date(s: str) -> date:
    return datetime.strptime(s, "%Y-%m-%d").date()


def main():
    parser = argparse.ArgumentParser(
        description="Structured recall over MEMORY.md (user + project)"
    )
    parser.add_argument("--file", type=Path, default=None,
                        help="Explicit path to a single MEMORY.md (overrides --scope)")
    parser.add_argument("--scope", choices=list(SCOPES), default="both",
                        help="Memory scope: user, project, or both (default: both)")
    parser.add_argument("--keyword", "-k", help="Keyword substring filter")
    parser.add_argument("--entity", "-e", help="Entity name filter")
    parser.add_argument("--since", help="Start date filter (YYYY-MM-DD)")
    parser.add_argument("--until", help="End date filter (YYYY-MM-DD)")
    parser.add_argument("--section", "-s",
                        choices=list(SECTION_NAMES),
                        help="Limit to one section")
    parser.add_argument("--cross-section", "-x", action="store_true",
                        help="With --entity, search all sections")
    parser.add_argument("--show", action="store_true",
                        help="Show a digest: world knowledge, beliefs, entity summaries, and recent experiences")
    parser.add_argument("--last", type=int, default=5,
                        help="With --show, number of recent experiences to include (default: 5)")
    parser.add_argument("--days", type=int, default=None,
                        help="With --show, include experiences from the last N days instead of --last count")
    parser.add_argument("--stats", action="store_true",
                        help="Print memory statistics instead of recall")
    parser.add_argument("--json", action="store_true",
                        help="Output as JSON")

    args = parser.parse_args()

    if args.file:
        banks = [("file", parse_memory_file(args.file))]
    else:
        paths = resolve_memory_paths(args.scope)
        banks = [(label, parse_memory_file(path)) for label, path in paths]

    if args.show:
        output = digest(banks, last=args.last, days=args.days)
        print(output)
        return

    if args.stats:
        if len(banks) == 1:
            result = stats(banks[0][1], label=banks[0][0])
        else:
            result = stats_multi(banks)
        print(json.dumps(result, indent=2))
        return

    if not any([args.keyword, args.entity, args.since, args.until, args.section]):
        parser.error("Specify at least one filter (--keyword, --entity, --since, --until, --section) or --stats")

    kwargs = dict(
        keyword=args.keyword,
        entity=args.entity,
        since=args.since,
        until=args.until,
        section=args.section,
        cross_section=args.cross_section,
    )

    if len(banks) == 1:
        result = recall(banks[0][1], **kwargs)
    else:
        result = recall_multi(banks, **kwargs)

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        for section_name, items in result.items():
            print(f"\n=== {section_name} ({len(items)}) ===")
            for item in items:
                print(item)

    if not result:
        print("No memories matched the query.", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
