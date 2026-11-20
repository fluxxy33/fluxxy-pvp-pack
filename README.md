# Fluxxy's PvP Pack

A Horizon mod (WebAssembly, client side) with the PvP helpers players install one by one on Java Edition. Each one has
its own switch in **Options > Mods > Fluxxy's PvP Pack > Config**.

- **Low fire**: while you burn, the fire overlay is drawn lower on the screen, so it hides less of the view. Three
  heights: Low, Lower (default), Lowest. Off: vanilla's fire.
- **Food info**: AppleSkin's saturation outlines over the hunger bar (2 points an icon, in quarters) and its exhaustion
  bar under the row, filling from the right as the next hunger point gets close to going. With food in hand, the hunger,
  saturation and hearts eating it would give flash over the bars; a food's tooltip says its hunger and saturation. Only in survival and
  adventure, and not while a mount's hearts take the row's place.
- **Elytra pitch**: while gliding, the angle you look at under the crosshair, in whole degrees: positive looking up,
  negative looking down.
- **Armor HUD**: the boots, leggings, chestplate and helmet you wear, with their durability bars, in slots left of the
  off hand's slot: a slot shows only for a piece worn. They are only drawn: nothing can be clicked or moved there.
- **Item counters**: how many golden apples, ender pearls, totems, arrows, potions and cobwebs you carry (off hand
  included), in a column of slots in the bottom right corner. A kind you don't carry takes no slot.
- **Damage indicator**: how many hearts a hit took, in red above your hearts, for a moment and a half; hits close
  together add up.

It only draws on your screen: nothing changes how the game plays, and nothing gives an edge a server's rules would
refuse ([mod policy](https://github.com/horizon-rust/horizon-mods/blob/main/MOD-POLICY.md#cheat-clients)).

## Credits

- **Food info is ported from [AppleSkin](https://github.com/squeek502/AppleSkin)** by squeek502, released into the
  public domain (the Unlicense): its saturation and exhaustion pixels (`src/appleskin_icons.rs`, generated from its
  `textures/icons.png`) and the way it picks and places them (`HUDOverlayHandler`).
- The other helpers are this mod's own code, on ideas from PVP Essentials (armour HUD, item counters, damage indicator,
  low fire), "low fire" resource packs and elytra HUD mods; no code or texture of theirs is in it. They draw with the
  game's own fire and hotbar sprites.

## Install

In the Horizon launcher: **Mods**, find Fluxxy's PvP Pack, **Install**. It is a WebAssembly mod, so nothing is built on
your computer.

## Known limits

- The fire is drawn with the game's `block/fire_1` sprite. If a resource pack has no such sprite, the mod leaves
  vanilla's fire as it is.
- Food info draws on vanilla's place for the food row; a mod that moves that row would leave the outline behind.
- The exhaustion bar needs this mod on the server too, as AppleSkin's does: vanilla sends a client its food and
  saturation but never its exhaustion, so the server's part sends it. In single player the game's own server runs the
  mod; on a server without it, the bar stays hidden (the saturation outlines still show).
- Potions are counted together, whatever their effect: the mod doesn't read a potion's contents.
- The damage indicator shows the damage you take; the damage you deal isn't shown yet.
- Hiding potion and explosion particles, as some PvP mods do, isn't there: the mod API has no hook for it on the
  client.

## Build

With [`cargo horizon-mod`](https://github.com/horizon-rust/horizon/tree/main/docs/modding):

```sh
cargo test                       # each helper's layout and numbers, and the settings
cargo horizon-mod build --wasm   # target/horizon-mod/fluxxy_pvp.hmod
cargo horizon-mod check
```

## Licence

MIT, see [LICENSE](LICENSE).
