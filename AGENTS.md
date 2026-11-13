# AGENTS.md
- **The directory structure is the architecture.** Simple, clear directory and module naming >= module dependency relationships > concrete implementation details; communicate with the user using directory trees.
- Do not preserve backward compatibility. Remove obsolete paths instead of adding compatibility layers, fallbacks, or migrations.
- Choose the simplest implementation that fully meets the current requirements. Avoid speculative abstractions, configuration, and indirection.
- Grow the system in layers. Start from the smallest version that works end to end, and add each new capability on top of a product that already works. Never trade a working product for unfinished complexity.
- Keep components modular and concerns clearly separated.
- Prefer established, well-maintained libraries when they reduce overall complexity or improve reliability. Do not reimplement common functionality without a clear reason.
- Lean on the dependencies already in the project before writing your own implementation or adding packages. Do not assume a library lacks a capability without checking its documentation and types.
- Make architectural decisions for the long term. Do not accept a stopgap that only works for now and is meant to be replaced later.

## Communication

- Lead with the conclusion. Keep responses direct and omit filler, repeated context, generic explanations, and narration of obvious steps.
- Match the level requested by the user: discuss architecture as architecture, behavior as behavior, and code details only when they materially support the answer.
- For architecture, refactoring, and module-boundary discussions, communicate primarily with annotated directory trees and ASCII architecture/data-flow diagrams instead of long prose.
- In directory trees, annotate every relevant directory and file with its single responsibility. When discussing code size, include line counts in the comments:
  - use measured line counts for existing code;
  - use clearly marked approximate targets such as `≈300 lines` for proposed code;
  - state whether tests, generated code, and blank lines are excluded.
- Show the complete main execution path, including inputs, ownership boundaries, runtime loops, persistence, outputs, and extension points. Make the direction of data and control flow explicit.
- Clearly separate the current structure from the proposed structure. Explicitly list modules that move, merge, split, or are deleted.
- Use the user's vocabulary consistently. Do not introduce new domain terms when existing plain-language terms are sufficient; when an implementation name must be mentioned, distinguish it from the architecture concept.
- For stateful or concurrent behavior, show timing, ownership, state boundaries, and before/during/after behavior explicitly. Distinguish continuation within the same lifecycle from creation of a new lifecycle.
- Prefer compact trees, flow diagrams, state matrices, and mappings when they communicate the structure more clearly than paragraphs.
- Tie recommendations to the actual repository structure and code. Do not present a speculative target architecture as if it already exists.

## Changelog

- Keep `README.md` (`## Changelog`) and `README-CN.md` (`## 更新日志`) in sync. Every user-visible change adds one entry to both, same date, same order, same facts; translate, do not paraphrase.
- Group entries under `### YYYY-MM-DD`. Newest date first; append to the existing date section when the change lands the same day. Never edit or re-date an already published entry except to correct a factual error.
- One entry per change. Use a bold user-facing outcome as the title, then exactly two sub-bullets:
  - `**Change:**` / `**改动**`: what changed, which setting path, endpoint, platform, or behavior is affected, and any concrete numbers (counts, limits, versions) the change depends on. State the action the user must take, such as restarting Cursor or pressing **Re-apply**, when one is required.
  - `**Root cause:**` / `**根本原因**` — or `**Why it was needed:**` / `**背景**` for additions: the problem this solves, from the user's or the protocol's perspective.
- Write release notes, not commits: no file-by-file lists, no refactor narration, no "update code" phrasing, no internal names that a reader cannot search for.
- Follow the existing entries in `README.md` as the canonical format example. If a change has no user-visible effect, it does not get an entry.
