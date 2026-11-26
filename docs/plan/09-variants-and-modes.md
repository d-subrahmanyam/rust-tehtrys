# 09 — Variants & Modes

## Main variants (§12) — pick one in menu; **Marathon default** (§2.3.2)

| Variant | Objective | Ends when | Levels | Ref |
|---------|-----------|-----------|--------|-----|
| **Marathon** | maximize score over 15 levels | level 15 complete / game over | 1→15, level-up | §12.1 |
| **Sprint / 40 Lines** | clear N lines (typ **40**) fastest | N lines / game over | no level-up; choose start level | §12.2 |
| **Ultra** | max score **or** lines in time | time (**2:00** v1; 2–3 min) / game over | no level-up | §12.3 |

- Starting level 1–15 where applicable (§2.3.3, §12.2).
- Game name "Tetris…" for licensed release; variants needn't (§12 note).
- Sprint timer counts up; Ultra counts down; Marathon time counts up. Score/lines
  always shown. Ultra "score OR lines" — pick score-primary (track both; §12.3).
- Marathon at level 15: continue clearing but no further level-up; ends per §12.1
  decision (we end when level-15 goal met → "game complete"). *(Guideline says game
  ends at 15 levels; exact end trigger pinned in tests.)*

## Lock Down ruleset (§2.5.4, §5.7–5.9)

Selectable; **Extended Placement default**; multiplayer/arcade **require Extended**
(§5.7); hard drop always immediate. See [03](03-game-engine.md#lock-down-§5457959-a123).

- **Extended** (default) · **Infinite** · **Classic**.

## Goal system (§6) — see [05](05-scoring-and-progression.md#goal-systems-§6--pick-one-engine-supports-both)

**Fixed** (10/level) default v1; **Variable** supported. Sprint ignores goals;
Ultra ignores; Marathon uses.

## Options (§2.5)

- Next Queue length 1–6 (default 6). Hold on/off (default on). Ghost on/off (default
  on). Lock Down mode. BGM/SFX on/off+volume **[later]**. Key remap. Starting Blocks
  **[later]**. Goal system. (Also key config & sound §2.3.)

## Starting Blocks (§2.4.6) **[later]**

Pre-fill N bottom rows (broken/random) at start.

## Scope decisions

**v1 builds:**
- Single player. Three variants. Extended lock default. 7-bag. SRS. Hold + Ghost +
  Next. Full scoring/T-Spin/B2B. Levels/goals. Game-over. HUD. Pause. Keyboard.
- (Phase 4) persistence + high scores; (Phase 5) sound/music.

**Out of scope / non-goals:**
- **Multiplayer** (Appendix B) — not built; seams kept. Top Out only relevant there
  (§10.5). Line Attack/Counter tables documented in guideline; not implemented.
- **Custom variants**; Iterate phase unused (A1.2.5); other authorized variants
  (§12 note; "Tetris Variant Concepts" separate doc).
- **Licensing/marketing**: official Tetris logo (§2.2), Legal screen wording (§2.1),
  "Tetris" name (§12), protected artwork, **Korobeiniki** licensed audio (§13.3),
  in-app credits for a *released* product. This is a learning project named `tehtrys`.
- **Platforms**: keyboard desktop v1; touch/mobile/joystick schemes (§4.1) later.

## Multiplayer note (Appendix B) — not v1, for future

Line attacks (Double/Triple/Tetris/T-Spin S/D/T; B2B +1; §B1.3.5); counter attacks
(§B1.3.4/§B1.3.6); broken vs solid lines; gap randomness per 8; Receiving Queue;
Top Out (§10.5); Extended lock required (§5.7); shared piece order (§B1.2.1 note);
practice mode (§B1.2.3); multiplayer UI (§B1.4). Our `EngineEvent::LinesCleared` +
clean `GameCommand` keep this addable.
