# Performance

The workspace keeps two plugin representations:

- JSON for inspectability and human review
- FST for compact lookup-heavy runtime usage

The current implementation favors deterministic payload generation and predictable precedence over aggressive micro-optimizations. For large prepared lexicons, prefer FST output.
