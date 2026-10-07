# Dragon Ball Arena

A native, original 2D arena fighter living inside the Bevy fork. It uses only Bevy-drawn shapes and text; it does not bundle game rips, extracted assets, or proprietary code.

## Run

From this directory:

```sh
cargo run
```

## Controls

- `1` / `Enter`: start a solo match against the CPU
- `2`: start a local two-player match
- `Enter`: play again after the result
- `Escape`: pause or resume
- Player 1: `A` / `D` move, `Space` jump, `J` strike, `K` ki blast, `Left Shift` dash
- Player 2: arrow keys move and jump, `N` strike, `M` ki blast, `Right Shift` dash

A local CPU rival pursues the player, attacks at close range, and throws ki blasts from a distance. Local two-player matches use the same combat loop with keyboard controls for both fighters. Matches use a best-of-three format. A round ends when one fighter's health reaches zero; the next round begins after a short result screen. The opening menu, pause screen, and match result are all rendered by Bevy's native UI.
