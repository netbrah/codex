---
name: ontap-dev-guide
description: ONTAP development patterns and codebase navigation. Use when working with ONTAP C/C++ source code, SMF iterators, keymanager subsystem, or writing unit tests.
metadata:
  short-description: ONTAP codebase development guidance
---

# ONTAP Development Guide

## Codebase Navigation

Use the MCP tools (`analyze_iterator`, `call_graph_fast`, `trace_call_chain`, `analyze_symbol_ast`) to explore the ONTAP codebase. These are your primary instruments.

### Key Patterns

1. **SMF Iterators** — Data access objects generated from `.smf` schema files. Use `analyze_iterator` to understand fields, callers, and REST mappings.

2. **CLI → NACL → Iterator chain** — Every CLI command maps to a NACL method which instantiates an iterator. Use `trace_call_chain` to walk this.

3. **Unit tests** — Use `generate_test_plan` or `prepare_unit_test_context` to understand fixtures, mockers, and FIJI fault handles before writing tests.

## Workflow

When asked to investigate or modify ONTAP code:
1. Start with `analyze_symbol_ast` on the function of interest
2. Use `call_graph_fast` to understand who calls it (upstream) 
3. Use `trace_call_chain` to find the CLI entry point and tables touched
4. Check for existing CITs with `find_cits`
5. Check Jira for related bugs with `ask_jira`
