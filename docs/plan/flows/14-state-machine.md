# Flow 14 — Application State Machine

Screens/states around the pure engine. §2 + §11 + §12. Engine itself has its own
phase machine (flows/02).

```mermaid
stateDiagram-v2
  [*] --> PowerOn
  PowerOn --> Legal : boot
  Legal --> Title : [optional §2.1]
  Title --> MainMenu
  MainMenu --> NewGame : New Game §2.3
  MainMenu --> Options : Options §2.5
  MainMenu --> Help : Help §2.6
  MainMenu --> HighScore : High Score §2.3.4/§11.2
  NewGame --> SelectPlayers : v1=1 §2.3.1
  SelectPlayers --> SelectVariation : §2.3.2 Marathon/Sprint/Ultra
  SelectVariation --> SelectStartLevel : §2.3.3
  SelectStartLevel --> Playing : Begin Game §2.4
  Options --> MainMenu
  Help --> MainMenu
  HighScore --> MainMenu
  Playing --> Paused : Pause §13.1.6
  Paused --> Playing : Resume
  Playing --> GameOver : §10 Game Over
  Playing --> VariantEnd : §12 goal/time reached
  GameOver --> EndRewards : §11.1 [optional]
  VariantEnd --> EndRewards : [optional]
  EndRewards --> HighScoreEntry : §11.2
  GameOver --> HighScoreEntry
  VariantEnd --> HighScoreEntry
  HighScoreEntry --> MainMenu
  Playing --> MainMenu : quit to menu
  [*] --> [*]
```

## States

`PowerOn → Legal → Title → MainMenu → {NewGame(SelectPlayers→SelectVariation→
SelectStartLevel)→Playing | Options | Help | HighScore} → Playing ⇄ Paused →
{GameOver|VariantEnd} → {EndRewards→}HighScoreEntry → MainMenu`. 

Plus in-game transient engine phases inside **Playing** (flows/02), independent of
app screens. **Paused** only reachable from Playing. 

## Notes
- **Playing** delegates to pure `core::TetrisEngine`; app routes commands & renders
  snapshot. 
- **Legal** (§2.1) + **Title logo** (§2.2) required for *licensed* release; optional
  simplified for learning. 
- **HighScoreEntry** then **MainMenu**; table persisted (§11.2). 
- v1 simplifies SelectPlayers=1; keeps structure for later multiplayer (Appendix B).

## Phase-5 additions (implemented)

States added to flows/14 `AppState`: **KeyMap** (key remapping capture; §4.1),
**Help** (§2.6 + scoring demo + credits), **NameEntry** (§11.2 high-score name
entry; on qualify after GameOver-finish), **Demo** (attract mode). 
GameOver → (NameEntry if qualifies) → GameOver. Options extended (sound volume/
toggles, seed, remap). Pause auto on focus-loss. Sound (§13.2/§13.3) is
presentation-only, driven by `core` `EngineEvent`s; `core` stays pure.
Replay recorded (`tehtrys_replay.txt`); deterministic restart (same seed).
