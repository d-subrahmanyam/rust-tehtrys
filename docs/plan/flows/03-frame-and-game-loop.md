# Flow 03 — Frame & Game Loop

SDL2 app loop + **fixed timestep** core; deterministic. Cosmetic rendering separate.
(flows/12 for render; flows/04 input.)

## Structure

```
const TICK_MS = 1000 / 60;      // 60 Hz fixed update
const MAX_FRAME_MS = 250;       // clamp "spiral of death"

fn run() {
  let mut accumulator_ms = 0u64;
  let mut last = sdl_ticks();
  loop {
    // 1. INPUT (drain SDL event pump fully)
    pump_events(&mut input);                 // key down/up, quit, focus, resize
    let commands = input.poll_commands();    // incl. DAS/ARR emissions for this frame
    for cmd in commands { app.dispatch(cmd); }

    // 2. TIME
    let now = sdl_ticks();
    let mut frame_ms = now - last; last = now;
    frame_ms = frame_ms.min(MAX_FRAME_MS);
    accumulator_ms += frame_ms;

    // 3. UPDATE (fixed steps; paused => skip core ticks)
    while accumulator_ms >= TICK_MS {
      app.tick(TICK_MS);                     // core.tick(dt_ms) advances timers/gravity
      accumulator_ms -= TICK_MS;
    }

    // 4. RENDER (interpolated optional; reads snapshot)
    render(app.snapshot(), accumulator_ms as f32 / TICK_MS as f32);
    canvas.present();

    // 5. PACE
    cap_framerate();                         // vsync or sleep to ~60fps
  }
}
```

## Guarantees

- **Core never reads wall-clock**; only `tick(dt_ms)`. Same `dt` sequence ⇒ same
  result (NFR-1). 
- DAS/ARR also advanced on `TICK_MS` in input layer with integer ms.
- Paused ⇒ `app.tick` skipped; accumulator reset on resume to avoid a burst.
- Focus loss ⇒ auto-pause + clear held keys (flows/13/04).
- `MAX_FRAME_MS` prevents catch-up explosion; we don't drop logic, just clamp input
  frame time.
- Render is a pure function of `RenderSnapshot` (+ anim state). No logic in render.

## Ordering per frame

Input (events) → dispatch commands → fixed-step update → collect events → route
events (render anim, audio, stats) → snapshot → draw → present. Events are processed
before render so a piece locked this frame shows correctly.

## Clock injection

`Clock` trait; `SdlClock` in app; `ManualClock` in tests. Core `tick(dt_ms)`.
Variant clock (Ultra) advanced by `tick`; paused-time excluded.

## Replay

Record `(start_ms, GameCommand)` + seed; re-run `core` with `ManualClock` ⇒ identical
events (NFR-1). (Phase 5.)
