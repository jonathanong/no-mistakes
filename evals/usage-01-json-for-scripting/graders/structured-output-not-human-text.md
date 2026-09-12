---
type: llm
focus: last_message
weight: 1
---

Passing: the plan consumes the structured JSON output as the
authoritative result, and warns against scraping the
human-readable rendering, which is not a stable contract.

Failing: the plan parses human-readable output, or is silent on
output format for a scripted consumer.
