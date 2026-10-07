# Dragon Ball Arena

A native, original 2D arena fighter living inside the Bevy fork. It uses only Bevy-drawn shapes and text; it does not bundle game rips, extracted assets, or proprietary code.

## Run

From this directory:

```sh
cargo run
```

## Controls

- `Enter`: start a match or play again after the result
- `Escape`: pause or resume
- `A` / `D`: move
- `Space`: jump
- `J`: close-range strike
- `K`: spend ki on a projectile
- `Left Shift`: dash

A local CPU rival pursues the player, attacks at close range, and throws ki blasts from a distance. Matches use a best-of-three format. A round ends when one fighter's health reaches zero; the next round begins after a short result screen. The opening menu, pause screen, and match result are all rendered by Bevy's native UI.
