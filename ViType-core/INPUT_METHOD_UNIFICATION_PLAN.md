# Input Method Unification Plan (Telex + VNI)

Goal: keep both Telex and VNI modes with identical user-visible behavior, while sharing ~90% of transformation logic so changes to rules/fixes are implemented once.

Non-goal: introduce a hybrid mode that accepts both Telex and VNI keys simultaneously (this would change UX and increase accidental transforms in IDs, numbers, etc.).

## Findings (current code)

### Already shared
- **Tone application is shared** via `VitypeEngine::apply_tone_mark_internal()` in `src/lib.rs`.
- **VNI tone digits are mapped to Telex internal tone keys** (`1→s`, `2→f`, `3→r`, `4→x`, `5→j`, `0→z`) in `src/vni.rs` (`vni_tone_key_to_internal`), then routed to the shared `apply_tone_mark_internal`.
- Tone escape behavior is shared via `VitypeEngine::try_escape_repeated_tone_key()` in `src/lib.rs`.

### Major duplication areas
- **Vowel shape transforms + escape tables**
  - Telex: `src/telex.rs` (`VOWEL_TRANSFORMS`, `VOWEL_UNTRANSFORMS`)
  - VNI: `src/vni.rs` (`VNI_VOWEL_TRANSFORMS`, `VNI_VOWEL_UNTRANSFORMS`)
- **Scanning for transform targets**
  - Telex: `find_last_matching_vowel_index`, `find_last_transformable_vowel`, `find_last_untransformable_vowel` in `src/telex.rs`
  - VNI: `find_last_vni_transformable_vowel`, `find_last_vni_untransformable_vowel` in `src/vni.rs`
- **D-stroke**
  - Telex: `dd → đ` in `src/telex.rs`
  - VNI: `d9 → đ` in `src/vni.rs`
  - Both implement the same idea (scan backward within 4 chars), but with different trigger keys.
- **Compound horn transforms**
  - Telex uses `w` and has a set of compound transforms in `src/telex.rs`.
  - VNI uses `7` and implements the same patterns in `src/vni.rs`.
- **State naming mismatch**
  - The engine uses `last_w_transform_kind: WTransformKind` for both Telex and VNI compound/horn transforms, even though VNI’s trigger key is `7`.

## Mapping sanity notes

Canonical tone mapping (Telex ↔ VNI):
- `s` ↔ `1` (sắc / acute)
- `f` ↔ `2` (huyền / grave)
- `r` ↔ `3` (hỏi / hook)
- `x` ↔ `4` (ngã / tilde)
- `j` ↔ `5` (nặng / dot below)
- `z` ↔ `0` (remove tone)

Vowel shape / consonant transforms:
- Circumflex: Telex `aa/ee/oo` ↔ VNI `6`
- Horn: Telex `w` (on `o/u`) ↔ VNI `7`
- Breve: Telex `w` (on `a`) ↔ VNI `8`
- D-stroke: Telex `dd` ↔ VNI `d9`

## Proposed direction: “actions” + shared algorithms

Treat both modes as thin keymaps that decide:
1) does this keystroke represent a transform action or a literal character?
2) if it’s a transform action, which *canonical action* is it?

Then implement actions once.

Suggested canonical actions:
- `Tone(s|f|r|x|j)` and `RemoveTone` (already mostly shared)
- `Circumflex`, `Horn`, `Breve`
- `DStroke`

## Core refactor idea: algorithmic vowel-shape transforms (shared)

Instead of maintaining per-mode “from→to” tables for every toned variant, implement shape transforms by:
1) read the current vowel and split into `(base_vowel, optional_tone)` using `TONED_TO_BASE` from `src/common.rs`
2) apply a shape-change rule to the **base** (including override rules)
3) re-apply the same tone using `VOWEL_TO_TONED` from `src/common.rs`

Shape-change rules needed:
- Circumflex:
  - `a/ă → â`
  - `e → ê`
  - `o/ơ → ô`
- Horn:
  - `o/ô → ơ`
  - `u/ư → ư`
- Breve:
  - `a/â → ă`

Escape (“undo shape”) should also be algorithmic:
- For a given transformed vowel, map it back to its unshaped counterpart while preserving tone.
  - Example: `ấ` (â+s) escapes to `á` (a+s), `ờ` (ơ+f) escapes to `ò` (o+f), etc.

This should eliminate the biggest maintenance burden: huge mapping tables duplicated across Telex/VNI.

## Plan (incremental, test-safe)

### Step 0 — Add a design test matrix (no behavior changes)
- Write down a small list of must-not-break cases:
  - tones on single vowels, multi-vowel placement, tone repositioning
  - shape overrides (`ă + a → â`, `ơ + o → ô`, etc.)
  - compound horn transforms (`uo?`, `uoi?`, “final consonant then horn-key”)
  - escape sequences (repeating the transform key)
- Keep this as a checklist; do not change tests yet.

### Step 1 — Introduce shared “diacritics” helpers
- Add a new module (e.g. `src/diacritics.rs`) with private helpers:
  - `split_vowel_and_tone(ch) -> (base, Option<tone>)`
  - `apply_tone(base, tone) -> toned_char`
  - `apply_shape(base, kind) -> new_base`
  - `apply_shape_preserving_tone(ch, kind) -> new_char`
  - `escape_shape_preserving_tone(ch, kind) -> new_char` (or inverse mapping per kind)
- No Telex/VNI code changes yet, just helper module + unit tests for the helpers if appropriate.

### Step 2 — Unify VNI vowel transforms to use shared helpers
- Replace `VNI_VOWEL_TRANSFORMS` usage in `try_vni_vowel_transform` with the algorithmic `apply_shape_preserving_tone`.
- Replace `VNI_VOWEL_UNTRANSFORMS` usage in `try_vni_escape_sequence` with algorithmic escape.
- Keep VNI’s key behavior unchanged (`6/7/8` are still the trigger keys and emitted on escape).
- Run `cargo test` to ensure all VNI tests still pass (especially `src/tests/vni_legacy_tests_do_not_edit_or_update`).

### Step 3 — Unify Telex vowel transforms to use shared helpers
- Replace `VOWEL_TRANSFORMS` usage for `a/e/o` doubling with algorithmic `apply_shape_preserving_tone` (circumflex kind).
- Keep Telex-specific `w` behavior:
  - compound transforms (for now)
  - standalone `w → ư`
  - `ww` escape
- Replace `VOWEL_UNTRANSFORMS` for Telex escape with algorithmic escape.
- Run `cargo test`.

### Step 4 — Shared scanning helpers
- Extract a shared scanner in `src/lib.rs` or a small module:
  - “scan backward up to N chars for a target that matches predicate”
  - “stop on intervening vowels except the immediate-glide exception”
- Update Telex and VNI vowel-shape transforms to call the shared scanner.
- Run `cargo test`.

### Step 5 — Shared D-stroke
- Add a shared method on `VitypeEngine`:
  - `try_d_stroke(trigger_key, store_last_key)`
- Telex calls it on second `d`, VNI calls it on `9`.
- Run `cargo test`.

### Step 6 — Shared compound horn transforms
- Move the shared logic for `uo`, `uoi`, “final consonant then horn key”, etc. into one implementation, parameterized by:
  - trigger key (`w` vs `7`)
  - what to store in `last_transform_key`
  - which `WTransformKind` variant to set
- Keep Telex-only `w` standalone behavior separate.
- Run `cargo test`.

## Risk areas / regression hotspots
- Escape behavior: both modes rely on “repeat key to escape” but the emitted literal differs (`aa` vs `a6`, `w` vs `7`).
- Free-transform stopping rules (intervening vowels, glide exception) must remain identical to current behavior.
- Compound horn transforms interact with tone repositioning; preserve exact order of operations.
- Word boundaries: Telex treats digits as boundaries; VNI does not. Any shared scanner must not change boundary behavior.

## Success criteria
- All existing tests pass (`cargo test`).
- TELEX and VNI behavior unchanged for existing rule docs (`TELEX_RULES.md`, `VNI_RULES.md`) and “legacy do not edit” test suite remains valid.
- Vowel-shape mapping tables are gone or drastically smaller, and both modes share the same vowel-shape algorithm.

