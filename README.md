# tpick

Tpick is a tui color picker because I was not satisfied with the available color pickers.

None of them have a way to select with HSL except for [this one](https://github.com/uga-rosa/ccc.nvim) but that's a neovim extension which is clunky.

# Usage

Currently there are no cli options.

## Terms

- Input format, the format used for inputting a color
- Selection format, the way that selecting a color is done, eg: rgb, hsl
- Output format, the color format used for outputting

## Controls

| Control | action                                                   |
| ------- | -------------------------------------------------------- |
| l       | increase current value                                   |
| h       | decrease current value                                   |
| 1-9     | set current value to 10%-90%                             |
| o       | cycle output                                             |
| y       | copy output                                              |
| Y       | copy ONLY the output value with no extra formatting      |
| p       | Paste color from clipboard (can be any supported format) |
| a       | Enable/disable alpha channel                             |

## Variable Key
- 0 <= R <= 255 (red)
- 0 <= G <= 255 (green)
- 0 <= B <= 255 (blue)

- 0 <= A <= 255 (alpha)

- 0 <= H <= 255 (hue)
- 0 <= S <= 100 (saturation)
- 0 <= L <= 100 (lightness)

## Supported Input formats

- `rgb(R, G, B)`
- `rgba(R, G, B, A)`
- `hsl(H, S, L)`
- `hsla(H, S, L, A)`
- `\x1b[38;2;R;G;Bm`
- `R;G;B`
- `#RGB` `#RGBA` `#RRGGBB` `#RRGGBBAA`

## Supported Selection Formats

- `hsl`

## Supported Output Formats

- `rgb(R, G, B)`
- `rgba(R, G, B, A)`
- `hsl(H, S, L)`
- `hsla(H, S, L, A)`
- `#RRGGBB`
- `#RRGGBBAA`
- `\x1b[38;2;R;G;Bm`

# Goals

- [x] HSL selection
- [ ] RGB selection
- [x] HSL output
- [x] RGB output
- [x] Hex output
- [x] Enable/disable alpha channel
  - [x] Modify alpha when enabled
- [x] Swap between different outputs
- [ ] Swap between different selections
- [ ] Specify starting value with cli option
- [x] Copy output
- [ ] Convert any supported format to any other supported format via cli.

## Maybe goals

- [ ] Saving colors

## Non Goals

- supporting non 24bit (truecolor) terminals.
