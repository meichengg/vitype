# CLAUDE.md

## Build & Test Commands
```bash
cargo build                        # Build
cargo test                         # All tests
cargo test test_name_substring     # Filter tests by name substring
```

## Build Verification
After modifying code, always run a build to check for compile errors:
```bash
cargo build
```
Fix any compilation errors before proceeding. Never leave the codebase in a broken state.

## Code Style

- **Naming**: PascalCase for types (`VitypeEngine`), snake_case for functions/fields (`delete_count`)
- **Types**: Use `struct` for value types, `enum` for variants; keep data structures small and focused
- **Access**: Use `private` for implementation details; minimal public API surface
- **Optionals**: Use early returns with `Option`/`Result` and `?` for propagation
- **Modules**: Keep helpers in `src/lib.rs` or small modules under `src/` with clear responsibilities

## Testing

- After updating tests, ALWAYS run `cargo test` to verify they pass (do not ask for confirmation)
- Unit tests live in `src/tests/` and are run via `cargo test`

## Documentation & Rule Changes

When modifying Telex input rules or transformation logic in the implementation:

1. **Update `TELEX_RULES.md` and `VNI_RULES.md` **: Document any new or changed rules, including:
   - New transformation patterns
   - Modified behavior for existing rules
   - New examples showing the expected input/output
   - Edge cases or special handling

2. **Add corresponding tests**: Every rule change must have test coverage:
   - Add unit tests in `vnkeyTests/` to verify the new behavior
   - Include tests for edge cases and boundary conditions
   - Test both the happy path and escape sequences if applicable
