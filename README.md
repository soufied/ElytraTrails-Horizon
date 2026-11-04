# ElytraTrails (Horizon native mod)

![ElytraTrails icon](icons/elytratrails.png)

A native mod for Horizon. The idea comes from [ElytraTrails](https://github.com/dbrighthd/elytratrails) by dbrighthd.

![ElytraTrails preview](icons/elytratrails.gif)

## What it does

Lines stream from the outer tip of each wing while you glide and fade out over time. Only players get trails.

- Colors: ice, gold, rose, violet, white, rainbow, or speed (cold blue when slow, orange and thicker when fast).
- Patterns: solid, dashed or wavy. Built in only.
- Pulse: gently breathing brightness.
- Night: lines are brighter at night and dimmer in the day.
- Boost: a sudden speed jump, such as a firework, lights the whole trail up for two seconds.
- Formation: gliding players close to each other are joined by white lines between their wingtips.
- Lines taper toward the tail, the wingtip has a soft glow, and fading eases in and out.
- Singleplayer: without LAN or a server, all code for other players is skipped.
- Sharing: your color and pattern are sent through the server, so other players see them (single player, LAN and servers).

## Settings

Open Options > Mods > ElytraTrails:

- **Trails**: turns every line on or off.
- **Your trail**: your own lines.
- **Your trail in first person**: on by default. Turn it off to see your lines only in third person (F5).
- **Other players' trails**: the lines of other players within 48 blocks.
- **Color**: see the list above.
- **Duration**: how long a line lasts, 1 to 12 seconds (default 5).
- **Width**: thin, normal or wide.
- **Pattern**: solid, dashed or wavy.
- **Pulse**: on or off.
- **Day/night brightness**: off by default (same brightness at any time). On: lines are brighter at night and dimmer in the day.
- **Optimization** (submenu):
  - **Your wings**: dynamic (the lines follow the real wing movement) or fixed (the line is computed once when gliding starts and then only moves with your body, which is cheaper).
  - **Other players' wings**: dynamic or fixed, the same for other players.
  - **Line detail**: full or half (half draws every second segment, about half the lines).
  - **Other players' range**: 24, 48 or 96 blocks.
  - **Formation lines**: on or off.
  - **Boost flash**: on or off.

## Build

```sh
horizon-launcher native-build . --trust-native-code
```

## Credits

- ElytraTrails by [dbrighthd](https://github.com/dbrighthd/elytratrails) (MPL-2.0), the original mod. The idea of wing trails comes from it.
- This is a separate implementation for Horizon, written from scratch in Rust. It does not contain code from the original mod.

## License

MIT. See [LICENSE](LICENSE).
