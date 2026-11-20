# Fluxxy's PvP Pack

A Horizon mod (WebAssembly, client side) with three small PvP helpers. Each one has its own switch in
**Options > Mods > Fluxxy's PvP Pack > Config**.

- **Low fire**: while you burn, the fire overlay is drawn lower on the screen, so it hides less of the view. Three
  heights: Low, Lower (default), Lowest. Off: vanilla's fire.
- **Food info**: your saturation as a gold outline over the hunger bar (2 points an icon, as the food icons), and a
  thin bar above the row that fills as the next hunger point gets close to going (exhaustion, 0 to 4). Only in
  survival and adventure, and not while a mount's hearts take the row's place.
- **Elytra pitch**: while gliding, the angle you look at, under the crosshair: `+` above the horizon, `-` below.

It only draws on your screen: nothing changes how the game plays, and nothing gives an edge a server's rules would
refuse ([mod policy](https://github.com/horizon-rust/horizon-mods/blob/main/MOD-POLICY.md#cheat-clients)).

## Credits

The ideas come from Java Edition mods and packs players know: AppleSkin (saturation and exhaustion on the hunger bar),
"low fire" resource packs, and elytra HUD mods. No code, texture or other file of theirs is in this mod: everything is
drawn by its own code with the game's own fire sprite.

## Install

In the Horizon launcher: **Mods**, find Fluxxy's PvP Pack, **Install**. It is a WebAssembly mod, so nothing is built on
your computer.

## Known limits

- The fire is drawn with the game's `block/fire_1` sprite. If a resource pack has no such sprite, the mod leaves
  vanilla's fire as it is.
- Food info draws on vanilla's place for the food row; a mod that moves that row would leave the outline behind.

## Build

With [`cargo horizon-mod`](https://github.com/horizon-rust/horizon/tree/main/docs/modding):

```sh
cargo test                       # the fire's place, the food info's rectangles, the pitch text, the settings
cargo horizon-mod build --wasm   # target/horizon-mod/fluxxy_pvp.hmod
cargo horizon-mod check
```

## Licence

MIT, see [LICENSE](LICENSE).
