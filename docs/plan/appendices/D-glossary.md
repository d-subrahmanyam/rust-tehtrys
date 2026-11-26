# Appendix D — Glossary

Tetris terminology (guideline §1.4 + §14; note §2.6: use correct terms — call it
**Matrix**, not "Grid"; **Tetrimino**, and reserve **Block** for a locked cell).

| Term | Meaning |
|------|---------|
| **Mino** | single square block; 4 make a Tetrimino. |
| **Tetrimino** | 4-mino connected shape; 7 kinds (I,O,T,L,J,S,Z). |
| **Block** | a mino locked in a matrix cell ("existing Block"). |
| **Matrix** | playfield; standard 10×20. |
| **Skyline** | top boundary of visible matrix. |
| **Buffer Zone** | invisible 10×20 above skyline; detects Lock Out/Block Out/Top Out. |
| **Top Out Line** | ceiling of buffer; 20 above skyline. |
| **Line Clear** | full row removed; above shifts down. |
| **Single/Double/Triple/Tetris** | 1/2/3/4-line clear. |
| **Back-to-Back (B2B)** | consecutive qualifying actions (Tetris/T-Spin/Mini-T line clears) w/o Single/Double/Triple between. |
| **T-Slot** | formation with 3-of-4 diagonal corners occupied for T. |
| **T-Spin** | T spun into T-Slot before lock. |
| **Mini T-Spin** | easier T-Spin (C&D+one / holes; not Point-5 full). |
| **Lock Down** | piece can no longer be manipulated; ~0.5 s. |
| **Lock Down Timer** | 0.5 s countdown to lock. |
| **Lock Out** | game over; whole piece locked above skyline. |
| **Block Out** | game over; new piece's start cell blocked. |
| **Top Out** | game over; blocks pushed past top of buffer (multiplayer). |
| **Ghost Piece** | translucent copy showing landing position. |
| **Next Queue** | upcoming tetriminos (1–6); North-facing. |
| **Hold Queue** | one stored tetrimino; once per lock; North-facing. |
| **7-bag / Random Generation** | shuffled bag of 7; deal all before refill. |
| **Super Rotation System (SRS)** | 5-point kick rotation. |
| **Facing** | North/East/South/West orientation. |
| **Fall Speed** | seconds per line; `(0.8-(L-1)·0.007)^(L-1)`. |
| **Soft Drop** | 20× fall while held; +1/cell. |
| **Hard Drop** | instant to surface + lock; +2/cell. |
| **DAS / ARR** | auto-repeat delay / rate (~0.3 s / ~0.5 s width). |
| **Level / Goal** | 1–15; lines to level-up; Fixed/Variable. |
| **Marathon / Sprint / Ultra** | 15-level / 40-lines / time-limited variants. |
| **Combo** | consecutive clears (stat only in 2009). |
| **Pause** | stop; hide matrix/next/hold. |
| **Starting Blocks** | pre-filled rows at start (optional). |
| **Generation Phase** etc. | eight engine phases A1.2. |

Full glossary: guideline §14. 
