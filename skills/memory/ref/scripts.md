# Script Reference

## memory-recall.py

```bash
python3 scripts/memory-recall.py --help

# Show digest (world knowledge + beliefs + summaries + recent experiences):
python3 scripts/memory-recall.py --show
python3 scripts/memory-recall.py --show --scope user
python3 scripts/memory-recall.py --show --scope project
python3 scripts/memory-recall.py --show --last 10
python3 scripts/memory-recall.py --show --days 7

# Search both scopes (default):
python3 scripts/memory-recall.py --keyword "database" --json

# User scope only:
python3 scripts/memory-recall.py --keyword "database" --scope user --json

# Project scope only:
python3 scripts/memory-recall.py --entity "api-gateway" --scope project --cross-section --json

# Temporal:
python3 scripts/memory-recall.py --since 2026-03-01 --json

# Section filter:
python3 scripts/memory-recall.py --section beliefs --json

# Stats (combined + per-scope):
python3 scripts/memory-recall.py --stats
python3 scripts/memory-recall.py --stats --scope user
```

## memory-manage.py

```bash
python3 scripts/memory-manage.py --help

# Initialize user memory:
python3 scripts/memory-manage.py init-user

# Validate structure:
python3 scripts/memory-manage.py validate
python3 scripts/memory-manage.py validate --scope user

# Check for duplicates (cross-scope):
python3 scripts/memory-manage.py check-duplicate --section experiences --candidate "text" --cross-scope

# Update confidence (user scope by default):
python3 scripts/memory-manage.py update-confidence --section beliefs --index 0 --delta 0.1
python3 scripts/memory-manage.py update-confidence --section beliefs --index 0 --delta 0.1 --scope project

# Extract entities from text:
python3 scripts/memory-manage.py extract-entities --text "some text"

# Find beliefs below threshold:
python3 scripts/memory-manage.py prune-beliefs --threshold 0.2

# Suggest entities needing summaries:
python3 scripts/memory-manage.py suggest-summaries

# Promote from user to project:
python3 scripts/memory-manage.py promote --section experiences --index 0
```
