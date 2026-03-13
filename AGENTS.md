## Hard-Cut Product Policy
- This application currently has no external installed user base; optimize for one canonical current-state implementation, not compatibility with historical local states.
- Do not preserve or introduce compatibility bridges, migration shims, fallback paths, compact adapters, or dual behavior for old local states unless the user explicitly asks for that support.
- Prefer:
- one canonical current-state codepath
- fail-fast diagnostics
- explicit recovery steps over:
- automatic migration
- compatibility glue
- silent fallbacks
- "temporary" second paths
- If temporary migration or compatibility code is introduced for debugging or a narrowly scoped transition, it must be called out in the same diff with:
- why it exists
- why the canonical path is insufficient
- exact deletion criteria
- the ADR/task that tracks its removal
- Default stance across the app: delete old-state compatibility code rather than carrying it forward.

## Rust Style

Write Rust that is minimal, concrete, and type-driven.

- Prefer the smallest correct design.
- Encode invariants in types when it makes the code simpler and clearer.
- Prefer concrete types over traits and generics unless abstraction is needed by current callers.
- Prefer small functions and small modules over type-heavy designs.
- Prefer plain parameters over builders or config structs unless there is real call-site complexity.
- Prefer compile-time guarantees over runtime validation.
- Prefer straightforward ownership and data flow over clever abstractions.
- Use `?` and narrow error propagation; avoid elaborate error layering unless the caller truly benefits.
- Avoid defensive branches for states that should be impossible if the types are correct.
- Avoid adding flexibility “for the future”; optimize for the current use case.
- If two designs are both correct, choose the one with fewer types, fewer moving parts, and less indirection.

## Rust Anti-Patterns

Avoid these unless there is a clear, present need:

- Traits with only one implementation
- Builders for simple internal types
- Config structs used only once
- Generic type parameters added only for flexibility
- Custom error enums when an existing error type is sufficient
- Repeated runtime validation for invariants that could live in the type system
- `Arc`, `Mutex`, `RwLock`, `Box`, or cloning introduced without a concrete need
- New wrappers, helpers, or abstractions that do not reduce real duplication

## Rust Planning Requirements

When creating plans for Rust changes:

- Include a "Rust Style Constraints" section.
- Prefer type-driven designs that make invalid states unrepresentable.
- Prefer the smallest correct implementation over extensible abstractions.
- Avoid introducing traits, builders, config structs, or generic parameters unless current callers require them.
- Call out any runtime validation that should instead be encoded in types.
- When proposing alternatives, prefer the option with fewer types, less indirection, and clearer data flow.
- Reference existing simple patterns in this repo and follow them.

## Rust Review Heuristics

Before finalizing Rust changes, simplify once:

- Remove any abstraction not required by current callers.
- Check whether a type can eliminate a branch or validation.
- Check whether a helper struct can become a function.
- Check whether a trait can become a concrete type.
- Check whether the error handling can be made smaller.
- Check whether the implementation matches the simplest existing patterns in the repo.

When unsure, prefer code that is easier to read, easier to delete, and harder to misuse.
