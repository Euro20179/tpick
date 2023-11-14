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

## Supported Input formats

- `rgb(r, g, b)` Max: (255, 255, 255), Min: (0, 0, 0)
- `rgba(r, g, b, a)` Max: (255, 255, 255, 255), Min: (0, 0, 0, 0)
- `hsl(h, s, l)` Max: (360, 1, 1), Min: (0, 0, 0)
- `hsla(h, s, l, a)` Max: (360, 1, 1, 255), Min: (0, 0, 0, 0)
- `\x1b[38;2;R;G;Bm`
- `R;G;B`

## Supported Selection Formats

- `hsl`

## Supported Output Formats

- `rgb(r, g, b)`
- `hsl(h, s, l)`
- `#RRGGBB`
- `\x1b[38;2;R;G;Bm`

# Goals

- [x] HSL selection
- [ ] RGB selection
- [x] HSL output
- [x] RGB output
- [x] Hex output
- [x] Enable/disable alpha channel
  - [ ] Modify alpha when enabled
- [x] Swap between different outputs
- [ ] Swap between different selections
- [ ] Specify starting value with cli option
- [x] Copy output
- [ ] Convert any supported format to any other supported format via cli.

## Maybe goals

- [ ] Saving colors

## Non Goals

- supporting non 24bit (truecolor) terminals.
