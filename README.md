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
| 0-9     | set current value to 0%-90%                              |
| $       | set current value to 100%                                |
| o       | cycle output format                                      |
| i       | cycle input format                                       |
| y       | copy output                                              |
| Y       | copy ONLY the output value with no extra formatting      |
| p       | Paste color from clipboard (can be any supported format) |
| a       | Enable/disable alpha channel                             |

## Variables

- 0 <= R <= 255 (red)
- 0 <= G <= 255 (green)
- 0 <= B <= 255 (blue)

- 0 <= A <= 255 (alpha)

- 0 <= H <= 360 (hue)
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
- `rgb`

## Supported Output Formats

- `rgb(R, G, B)`
- `rgba(R, G, B, A)`
- `hsl(H, S, L)`
- `hsla(H, S, L, A)`
- `#RRGGBB`
- `#RRGGBBAA`
- `\x1b[38;2;R;G;Bm`

# Goals

- [x] Swap between different selections
    - [x] HSL
    - [x] RGB
    - [ ] CYMK
    - [x] ANSI 256 color selection pannel
- [x] Ability to input number for slider
- [x] Swap between different outputs
  - [ ] Let user pick from some kind of menu instead of cycling (or both)
  - [ ] Option to display all outputs at once
  - [x] Custom output formats
- [ ] Specify starting value with cli option
- [x] Copy output
- [ ] Convert any supported format to any other supported format via cli.
- [x] Pressing a key to input a color
- [ ] Cli options for setting default Input/Selection/Output formats

## Maybe goals

- [ ] Saving colors

## Non Goals

- supporting non 24bit (truecolor) terminals.
