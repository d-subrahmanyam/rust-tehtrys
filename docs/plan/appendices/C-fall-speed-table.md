# Appendix C — Fall Speed, Drop, Timing Constants

§7 + §5.2/§5.4 + A1.2.1. Integer ms. 

## Fall speed (§7)

```
seconds_per_line(level) = (0.8 - (level - 1) · 0.007)^(level - 1)     // level 1..15
fall_ms(level) = round(seconds_per_line(level) · 1000) ; clamp >= 1
```
Once generated, piece falls at normal speed **even while manipulated**; generated
piece drops one row immediately if clear (§3.4, §7). On **Level Up**, recompute.

### Table (levels 1–15) — §7 + computed

| Level | sec/line | fall_ms |
|------:|---------:|--------:|
| 1 | 1.000 | 1000 |
| 2 | 0.793 | 793 |
| 3 | 0.618 | 618 |
| 4 | 0.473 | 473 |
| 5 | 0.355 | 355 |
| 6 | 0.262 | 262 |
| 7 | 0.190 | 190 |
| 8 | 0.135 | 135 |
| 9 | 0.094 | 94 |
| 10 | 0.064 | 64 |
| 11 | 0.043 | 43 |
| 12 | 0.028 | 28 |
| 13 | 0.018 | 18 |
| 14 | 0.011 | 11 |
| 15 | 0.007 | 7 |

(Guideline prints seconds; our `fall_ms` rounded. We verify formula + table in
tests. `fall_ms≥1`.)

## Drop speeds (§7.1, §5.4, §5.5)

- **Soft Drop** `SOFT_DROP_MULTIPLIER = 20`: `fall_ms/20` while held; `+1` score/
  cell. `clamp(fall_ms/20, >=1)`. No auto-repeat issue. 
- **Hard Drop** `HARD_DROP_MS = 0.0001` (§5.4/§7.1) → **instant** in engine
  (rounds to 0); `+2` score/cell; immediate Lock. No auto-repeat. 

## Timing constants

| Constant | Value | Ref |
|----------|-------|-----|
| `MATRIX_WIDTH` | 10 | §1.4/§2.4.1 |
| `MATRIX_HEIGHT_VISIBLE` | 20 | §1.4 |
| `BUFFER_HEIGHT` | 20 | §1.4/§10 |
| `LOCK_DOWN_MS` | **500** | §5.4/§5.7/§5.8/§5.9/A1.2.3 |
| `GENERATION_MS` | **200** (0.2 s) | A1.2.1 |
| `HARD_DROP_MS` | 0.0001 → instant | §5.4/§7.1 |
| `SOFT_DROP_MULTIPLIER` | **20** | §5.5/§7.1 |
| `DAS_MS` | **300** (~0.3 s) | §5.2 |
| `ARR_MS` | **≈33.3** (10 cells / 0.5 s) | §5.2 |
| `EXTENDED_MOVE_CAP` | **15** | §5.7 |
| `TICK_MS` | 1000/60 (fixed) | ours (flows/03) |
| `LOCK_RESET` ruleset | Extended/Infinite/Classic | §2.5.4/§5.7–5.9 |
| `LEVEL_MIN/MAX` | 1 / 15 | §6 |
| `FALL_BASE/A` | 0.8 / 0.007 | §7 |
| scoring table | [05](../05-scoring-and-progression.md) | §8 |
| `B2B_BONUS` | 0.5× action total | §8 |
| `SOFT_PTS`/`HARD_PTS` | 1 / 2 per cell | §8 |

## Variant clocks (§12)
Marathon time up; Sprint time up (+target lines); Ultra countdown **2:00** v1;
paused time excluded. Sprint no level-up; Ultra no level-up. 

## Scoring level
`points = base·level + b2b(0.5·base·level)`; drops not scaled. Integer `u64`. §8.
