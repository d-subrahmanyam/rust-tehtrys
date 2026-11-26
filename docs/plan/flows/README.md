# Flows

Detailed flow documentation. All reference the **2009 Tetris Design Guideline**
(see [README](../README.md)); core behavior in [03-game-engine](../03-game-engine.md).
Diagrams are Mermaid (rendered by GitHub).

| # | Flow | What it covers |
|---|------|----------------|
| 01 | [game-flow](01-game-flow.md) | Full app/game flowchart (§2 + §12 endings) |
| 02 | [engine-phases](02-engine-phases.md) | The eight Tetris Engine phases (A1.1/A1.2) |
| 03 | [frame-and-game-loop](03-frame-and-game-loop.md) | SDL2 loop, fixed timestep, accumulator, render |
| 04 | [input-flow](04-input-flow.md) | key→command; DAS/ARR; soft/hard; pause; focus |
| 05 | [piece-lifecycle](05-piece-lifecycle.md) | spawn→fall→lock→pattern→…→completion→next; hold |
| 06 | [lock-down](06-lock-down.md) | lock-down FSM; timer; 3 rulesets; A1.2.3 lift-off |
| 07 | [line-clear-and-scoring](07-line-clear-and-scoring.md) | pattern→hit list→animate→eliminate→scoring; B2B; combos |
| 08 | [t-spin-detection](08-t-spin-detection.md) | §9.1 recognition; corners; Point-5; mini vs full |
| 09 | [hold-and-next](09-hold-and-next.md) | hold swap; once-per-lock; next queue; 7-bag feed |
| 10 | [level-progression](10-level-progression.md) | goals; level-up; fall speed; scoring level |
| 11 | [game-over](11-game-over.md) | lock out / block out / top out; buffer; endings; events |
| 12 | [rendering](12-rendering.md) | snapshot→layout→draw; animation; events→audio seam |
| 13 | [pause](13-pause.md) | pause/resume; hide matrix; focus |
| 14 | [state-machine](14-state-machine.md) | application state machine (screens) |

See also engine contract in [02-architecture](../02-architecture.md).
