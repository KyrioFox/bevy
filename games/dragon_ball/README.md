# Dragon Ball Arena

A native, original 2D arena fighter living inside the Bevy fork. It uses only Bevy-drawn shapes and text; it does not bundle game rips, extracted assets, or proprietary code.

## Run

From this directory:

```sh
cargo run
```

## Controls

- `A` / `D`: move
- `Space`: jump
- `J`: close-range strike
- `K`: spend ki on a projectile
- `Left Shift`: dash

A local CPU rival pursues the player, attacks at close range, and throws ki blasts from a distance. A round ends when one fighter's health reaches zero and resets automatically after two seconds.
