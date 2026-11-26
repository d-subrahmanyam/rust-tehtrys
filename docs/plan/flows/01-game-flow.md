# Flow 01 — Game Flow

Guideline **§2 Basic Game Flowchart** + **§12** + **§11**. App-level (see also
[14-state-machine](14-state-machine.md)).

```mermaid
flowchart TD
  PowerOn([Power On]) --> Legal[Legal Screen §2.1\n(optional for learning build)]
  Legal --> Title[Title Screen & Logo §2.2]
  Title --> Menu[Main Menu §2.3]
  Menu -->|New Game| SelectPlayers[Select # Players §2.3.1\nv1: 1]
  Menu -->|Options| Options[Options §2.5\nnext/hold/ghost/lock/sound/keys]
  Menu -->|Help| Help[Help §2.6]
  Menu -->|High Score| HiScore[High Score Table §2.3.4/§11.2]
  Options --> Menu
  Help --> Menu
  HiScore --> Menu
  SelectPlayers --> SelectVar[Select Game Variation §2.3.2\nMarathon default · Sprint · Ultra]
  SelectVar --> SelectLevel[Select Starting Level §2.3.3\n1..15 default 1]
  SelectLevel --> Begin[Begin Game §2.4\nNew Game & Interface]
  Begin --> Game[Game Running\neight-phase engine, flows/02]
  Game <-->|Pause §13.1.6| Paused[Paused\nmatrix/next/hold hidden]
  Game -->|Game Over §10 or Variant End §12| Over[Game Over §11]
  Over -->|End Rewards §11.1| Rewards[Congratulations / effects]
  Rewards --> HiScoreEntry
  Over --> HiScoreEntry[High Score entry §11.2\nif qualified]
  HiScoreEntry --> Menu
```

## Marathon flow (§12.1)

```
begin level L (1..15)
loop: clear lines; on goal -> Level Up; level 1..15
end when: game over (§10) OR level 15 goal complete -> Game Over/End -> §11
scoring depends on level; speed increases.
```

## Sprint / 40 Lines (§12.2)

```
choose start level; no level-up
end when: lines_cleared >= N (40) -> success (time result)
        OR game over (§10)
```

## Ultra (§12.3)

```
no level-up; timer T (2:00 v1)
end when: time expires -> end (report score/lines)
        OR game over (§10)
goal: maximize score OR lines.
```

## Game-over events (§11)

- **Game Over** screen after §10 condition / variant end.
- **End Rewards** if above-average score / high level (§11.1).
- **High Score Table**: enter name/initials if qualified; per variant; persisted;
  accessible from menu (§11.2, A1.2.7). 
- On Game Over → back to **Main Menu**; restart shortcut.

## Per-game reset

New game: fresh matrix; clear buffer; new seed; **7-bag refilled** (§3.3); hold
cleared; next queue rebuilt; score/level/lines/goal reset; B2B reset; combo reset;
time reset; lock/generation timers cleared; variant clock reset; starting blocks
applied (§2.4.6, later).
