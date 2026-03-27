# Script Reference

## memory-recall.py

```bash
python3 skills/memory/scripts/memory-recall.py --help

# Show digest (world knowledge + beliefs + summaries + recent experiences):
python3 skills/memory/scripts/memory-recall.py --show
python3 skills/memory/scripts/memory-recall.py --show --scope user
python3 skills/memory/scripts/memory-recall.py --show --scope project
python3 skills/memory/scripts/memory-recall.py --show --last 10
python3 skills/memory/scripts/memory-recall.py --show --days 7

# Search both scopes (default):
python3 skills/memory/scripts/memory-recall.py --keyword "database" --json

# User scope only:
python3 skills/memory/scripts/memory-recall.py --keyword "database" --scope user --json

# Project scope only:
python3 skills/memory/scripts/memory-recall.py --entity "api-gateway" --scope project --cross-section --json

# Temporal:
python3 skills/memory/scripts/memory-recall.py --since 2026-03-01 --json

# Section filter:
python3 skills/memory/scripts/memory-recall.py --section beliefs --json

# Stats (combined + per-scope):
python3 skills/memory/scripts/memory-recall.py --stats
python3 skills/memory/scripts/memory-recall.py --stats --scope user
```

## memory-manage.py

```bash
python3 skills/memory/scripts/memory-manage.py --help

# Initialize user memory:
python3 skills/memory/scripts/memory-manage.py init-user

# Validate structure:
python3 skills/memory/scripts/memory-manage.py validate
python3 skills/memory/scripts/memory-manage.py validate --scope user

# Check for duplicates (cross-scope):
python3 skills/memory/scripts/memory-manage.py check-duplicate --section experiences --candidate "text" --cross-scope

# Screen text before storing:
python3 skills/memory/scripts/memory-manage.py screen-text --text "some text"

# Append a new entry through the guarded writer:
python3 skills/memory/scripts/memory-manage.py append-entry --section experiences --scope user --date 2026-03-27 --context testing --entities "integration-tests,port-5432" --text "the memory text"

# Update confidence (user scope by default):
python3 skills/memory/scripts/memory-manage.py update-confidence --section beliefs --index 0 --delta 0.1
python3 skills/memory/scripts/memory-manage.py update-confidence --section beliefs --index 0 --delta 0.1 --scope project

# Extract entities from text:
python3 skills/memory/scripts/memory-manage.py extract-entities --text "some text"

# Find beliefs below threshold:
python3 skills/memory/scripts/memory-manage.py prune-beliefs --threshold 0.2

# Suggest entities needing summaries:
python3 skills/memory/scripts/memory-manage.py suggest-summaries

# Detect contradictions between beliefs:
python3 skills/memory/scripts/memory-manage.py check-conflicts
python3 skills/memory/scripts/memory-manage.py check-conflicts --scope user

# Promote from user to project:
python3 skills/memory/scripts/memory-manage.py promote --section experiences --index 0 --allow-project-promotion

# Forget — step 1: fuzzy-find matching memories:
python3 skills/memory/scripts/memory-manage.py find-matches --query "port conflict"
python3 skills/memory/scripts/memory-manage.py find-matches --query "dev server" --threshold 0.3

# Forget — step 2: delete a confirmed entry (after user approval):
python3 skills/memory/scripts/memory-manage.py delete-entry --section experiences --index 0
python3 skills/memory/scripts/memory-manage.py delete-entry --section beliefs --index 1
```
