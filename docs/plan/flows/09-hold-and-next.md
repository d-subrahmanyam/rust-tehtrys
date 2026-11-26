# Flow 09 — Hold & Next

Hold §2.4.8/§2.5.2/§5.6; Next §2.4.3/§2.5.1/§3.3; 7-bag §3.3/A1.2.1. Hold off if
disabled.

```mermaid
flowchart TD
  %% NEXT / BAG
  Bag[7-bag §3.3\nshuffle 7; deal all 7; refill+reshuffle when empty] --> Queue[Next Queue\n1..6 visible (default 6) §2.4.3/§2.5.1\nnext-to-play first; North-facing]
  Queue --> Gen[Generation pops front;\nbag tail pushed; queue advances by 1 §3.3]
  Gen --> Active[active piece]
  %% HOLD
  Active -->|Hold pressed §5.6| Check{hold used this piece?\nhold feature on?}
  Check -->|used already / off| NoOp[no-op]
  Check -->|no| Swap[put current in hold (North)\nincoming = previously held (if any) ELSE pop Next-from-bag\ngenerated piece = incoming §5.6]
  Swap --> Fresh[fresh spawn North §3.4; drop-1; ghost;\nlock state reset; hold_used=true;\nBlock-Out check §10.7]
  Fresh --> Active2[active piece incoming]
  Active2 -->|lock must occur before holding again §5.6| AfterLock[on Lock/Pattern/... Completion\nhold_used resets for NEXT piece]
```

## Next Queue (§2.4.3, §2.5.1, §3.3, A1.2.1)

- **North-facing** shown. Located top-right. Next-to-play first (closest to top for
  vertical). **1–6** visible; default **6**; can't see bag order beyond queue.
- Fed by **7-bag**: every generated piece pushes bag-front to **tail** of queue;
  bag refilled+reshuffled when empty; equal distribution (§3.3). 
- (Holding pulls from bag when hold empty.)

## Hold (§2.4.8, §5.6)

- **Once per lock**; lock must occur between holds; can't hold same piece twice
  in a row (since you can't hold again until it locks, and it won't be re-held
  without a lock between). 
- Current piece → hold, **North-facing**; previously held swaps back at spawn;
  if hold empty, **next-from-bag** comes in; Next Queue advances accordingly.
- Incoming piece: fresh spawn, North, drop-1-if-clear, **Block-Out** check; lock
  timer/cap reset; can act immediately. 
- **Doesn't break B2B** (§8.1); doesn't affect scoring; "lock without clear" n/a.
- Feature toggle (default **on** §2.5.2).
- Hold UI top-left; used-this-piece dimmed.

## 7-bag exactness (§3.3)

```
bag = [I,O,T,L,J,S,Z]; shuffle(rng); output until empty; then refill+reshuffle.
```
- Reproducible with seed. `Rng` internal; no external crate. 
- "The bag is refilled and reshuffled once it is empty." We may pre-shuffle next bag
  while current drains (implementation); order unaffected. 

## Interaction with DAS (§5.2)

DAS auto-repeat **continues across lock-down & hold-swap** to the next piece while
button held; hold itself doesn't reset DAS. (Our DAS lives in input; core just gets
moves.) 

## Events
`PieceGenerated`, `HoldUsed{swapped}`, queue updates; UI reads snapshot `next`/`hold`.
