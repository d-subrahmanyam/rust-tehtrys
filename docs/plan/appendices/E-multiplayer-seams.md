# Appendix E — Multiplayer seams (Phase 6, document-only, **not v1**)

**Phase 6 is document-only.** `tehtrys` is **single-player**; **nothing here is
built**. This appendix records (a) the official **2009 Tetris Design Guideline
Appendix B — Multiplayer** rules *as restated in my local reference*, and (b) the
**seams already present** in `tehtrys`'s pure `core` that would let multiplayer be
added later **without rewriting** the engine. 

> **Source/status.** The authoritative source is the 2009 Guideline (© Blue Planet
> Software, CONFIDENTIAL, used for reference, **not distributed**; local-only +
> git-ignored). This page **restates** facts in my own words; it is **not** the
> original Appendix B, and **no** original text/art/audio is reproduced. My local
> restatement numbers Appendix B as **§16**; original PDF numbering may be **§B1**.
> References below cite both. **Not legal advice.** "Tetris®" trademark; not
> affiliated with The Tetris Company. 

---

## B1. Terms (Guideline Appendix B / §16.1)

| Term | Meaning |
|------|---------|
| **Line Attack** | garbage lines sent to opponents via Doubles/Triples/Tetrises/T-Spin Singles/Doubles/Triples (+B2B bonus). |
| **Broken Line** | garbage row with **exactly one gap**; clearable. standard choice. |
| **Solid Line** | garbage row with **no gap**; usually can't be cleared. |
| **Receiving Queue** | meter by the Matrix showing **incoming garbage**; added to **bottom** at next **Lock Down**, before next piece spawns. |
| **Counter Attack** | your attack first **cancels** lines in your own Receiving Queue; only **leftover** is sent. |
| **Starting Lines** | preset or random garbage rows at game start. |
| **Target Mark** | movable marker choosing which opponent receives your attack; visible only to you. |
| **Top Out** | **multiplayer-only**: garbage pushes Blocks past the Top Out Line. |

(Also reuse single-player terms: Matrix, Buffer Zone, Skyline, Top Out Line,
Lock Down, Back-to-Back, 7-bag, SRS, Hold, Ghost, Next. See appendix D.)

## B2. Game types (§B1.2 / §16.2)

- **Competitive:** force opponents (human or AI) to top out. Solo or teams
  (typically 2 teams of 2–3). Usually Line Attack; sometimes speeds up over time
  or piece count. **Unless the variant says otherwise, all players get the same
  piece sequence** (random each game). e.g. 1v1, 1vCPU (adjustable AI), FFA
  humans/CPUs. 
- **Cooperative:** teams toward a shared goal (beat a CPU; clear N lines fastest).
  Teams may include CPU teammates. team vs team / team vs CPU / multi-team. 
- **Practice mode:** if a player must wait for others, they can **practice** at
  current level meanwhile (restart on top out), then rejoin. **"Practice" shown
  prominently** in the Matrix area. 

## B3. Garbage rules (§B1.3 / §16.3)

- Lines sent to **multiple opponents are identical**; rise from **bottom**. 
- **Broken Lines:** gap stays in the **same random column for 8 lines that
  actually appear in the Matrix** (`GAP_PERIOD = 8`; lines still in the Receiving
  Queue **don't count**), then a new random column for the next 8. 
- **Incoming lines enter only after the receiving player's piece Locks Down.** 
- **Extended Placement Lock Down is required** for multiplayer (§5.7); our
  `LockRuleset::Extended` default matches. 
- **Top Out** when garbage added pushes blocks past the top-out line (§10.5). 

## B4. Lines sent per action (§B1.3.5 / §16.4)

| Action | Lines sent |
|--------|-----------|
| Single | 0 |
| Double | 1 |
| Triple | 2 |
| Tetris | 4 |
| T-Spin / Mini T-Spin / Mini T-Spin Single | 0 |
| T-Spin Single | 2 |
| T-Spin Double | 4 |
| T-Spin Triple | 6 |
| **Back-to-Back** (any qualifying clear in chain, **incl. first B2B**) | **+1** |

So **B2B Tetris = 5**, **B2B T-Spin Single = 3**, **B2B T-Spin Double = 5**,
**B2B T-Spin Triple = 7**. B2B T-Spin / Mini-T-Spin with no lines (or Mini-T-Spin
Single) still sends **0**. (No *combo* scoring/attack in 2009; matches our
scoring §8.) 

## B5. Counter Attack rule (§B1.3.6 / §16.5)

```
attack   = lines_sent_for_action      # table above
queue    = lines currently in your Receiving Queue
sent     = max(0, attack - queue)     # goes to opponent(s)
remaining= max(0, queue - attack)     # added to bottom of your Matrix after this Lock Down
```

**Counter Attack table** (S = sent to opponent, R = remaining in your queue +
added to your Matrix):

| Your action | Q=0 | Q=1 | Q=2 | Q=3 | Q=4 | Q=5 |
|-------------|-----|-----|-----|-----|-----|-----|
| Single (0) | 0 | 1R | 2R | 3R | 4R | 5R |
| Double (1) | 1S | 0 | 1R | 2R | 3R | 4R |
| Triple (2) | 2S | 1S | 0 | 1R | 2R | 3R |
| Tetris (4) | 4S | 3S | 2S | 1S | 0 | 1R |
| T-Spin Single (2) | 2S | 1S | 0 | 1R | 2R | 3R |
| T-Spin Double (4) | 4S | 3S | 2S | 1S | 0 | 1R |
| T-Spin Triple (6) | 6S | 5S | 4S | 3S | 2S | 1S |
| B2B Tetris (5) | 5S | 4S | 3S | 2S | 1S | 0 |
| B2B T-Spin Triple (7) | 7S | 6S | 5S | 4S | 3S | 2S |

## B6. Multiplayer UI (§B1.4 / §16.6)

Per-player screen (suggestion, not requirement): (1) player Matrix, (2) player
Receiving Queue, (3) chat, (4) opponent Matrix (smaller), (5) opponent Receiving
Queue, (6) avatar, (7) Target Mark on targeted opponent; teammates' avatars/
Matrices, chat bubbles, timer, usual Hold/Next. Shared-screen multiplayer per
guideline §4.5. (Our `flows/12` + §2.4 UI is single-player; no multiplayer UI.) 

---

## Seams already present in `tehtrys` (pure `core`; **not wired for multiplayer**)

These are **not** a multiplayer implementation; they're the **minimal seams** the
design intentionally keeps (flows/11; `GameOverReason::TopOut`; no logic added):

| Seam | Where | What it enables |
|------|-------|-----------------|
| **`GameOverReason::TopOut`** | `core::types::GameOverReason` | multiplayer top-out; currently **never emitted**; single-player uses `LockOut`/`BlockOut`. Hook for "garbage pushed past top-out line." |
| **`LockRuleset::Extended` default + present** | `core::types::LockRuleset` | §5.7 Extended **required** for multiplayer; default matches; `Infinite`/`Classic` available. |
| **10×20 + 20-row buffer; top-out line** | `core::matrix::Matrix` (`MATRIX_HEIGHT_TOTAL=40`, `MATRIX_HEIGHT_VISIBLE=20`); `all_cells_above_skyline` | room for garbage; Buffer Zone; Lock Out/Block Out checks already. |
| **pure `GameCommand` in / `EngineEvent` out** | `core::types` | presentation-agnostic; a network layer can feed commands + broadcast events; no SDL in `core`. |
| **deterministic 7-bag + seedable PRNG** | `core::bag::Bag`, `core::rng::SplitMix64`, `Game::new(seed,…)` | **shared tetrimino order** §16.2 by sharing `seed`; replay already uses this. |
| **scoring exposes line-clear + B2B** | `core::scoring::Scoring`, `ScoreAction::{is_b2b_qualifying,clears_line,breaks_b2b}`; `EngineEvent::Scored{action,b2b,…}` | **Line Attack** lines (B4) + **Counter Attack** (B5) computable purely from `Scored`. |
| **`LockedDown` hook point** | `core::engine` `lock_now` → before next spawn | where **incoming garbage** would be inserted at bottom after Lock Down (B3). Not implemented. |
| **`Game` variant shell; `Variant`** | `core::game::{Game,Variant}` | a future `Multiplayer` variant / wrapper; `core` `Game` unaffected. |
| **`#![forbid(unsafe_code)]`, no wall-clock, integer-ms, injectable time** | `core` | transport/authoritative-server determinism; no hidden nondeterminism. |

### Not built (Phase 6 scope)

- No **garbage** insertion / **Receiving Queue** / **Counter Attack** execution. 
- No **line-attack** sending between players; no `sent`/`remaining` computation wired. 
- No **`insert_garbage`**; no **broken/solid line** generation; no `GAP_PERIOD=8`. 
- No **`TopOut`** emission; no multiplayer **`GameOver`**; no **targeting**. 
- No **shared-seed** synchronization beyond what `Game::new(seed,…)` allows. 
- No **networking**, no **AI opponent**, no **practice mode**, no **teams**, no
  multiplayer **UI** (`AppState` is single-player). 
- No **`Multiplayer` `Variant`**. 

### Possible future implementation order (not v1)

1. **Pure helpers** (no engine change): `line_attack(action,b2b)`, `counter_attack(attack,queue)`,
   `garbage_row(gap_col)`, `broken_line_stream` (`GAP_PERIOD=8`), tested vs B4/B5. 
2. **Engine seam**: `insert_garbage(rows)` before spawn after `lock_now`; emit `TopOut`. 
3. **A `Multiplayer` wrapper** around `Game` + shared `seed`; receiving queue;
   target; `Scored`→attack routing. 
4. **UI** per B6 + practice. **Networking** last. 

---

## Source-section cross-reference (original PDF ↔ local restatement)

Original guideline numbers my plan cites as **§B1***; my local restatement uses
**§16***. Content matches (restated, not copied):

| Concept | Original | Local restatement |
|---------|----------|-------------------|
| Terms | §B1.1 | §16.1 |
| Game types / shared order / practice | §B1.2 | §16.2 |
| Garbage rules | §B1.3 / §B1.3.x | §16.3 |
| Lines sent per action | §B1.3.5 | §16.4 |
| Counter Attack | §B1.3.6 | §16.5 |
| Multiplayer UI | §B1.4 | §16.6 |
| Top Out | §10.5 | §10.5 |
| Extended required | §5.7 | §5.7 |

*(My appendix **B** = piece data; appendix **A**=SRS kicks; **C**=fall speed;
**D**=glossary; this appendix **E**=multiplayer seams.)*
