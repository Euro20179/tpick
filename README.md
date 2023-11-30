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

| Control    | action                                                   |
| ---------- | -------------------------------------------------------- |
| l          | increase current value                                   |
| L          | increase current value by 10                             |
| h          | decrease current value                                   |
| H          | decrease current value by 10                             |
| 0-9        | set current value to 0%-90%                              |
| $          | set current value to 100%                                |
| o          | cycle output format                                      |
| O          | Various output format options                            |
| i          | cycle input format                                       |
| I          | Input a number for the selected slider                   |
| y          | copy output                                              |
| Y          | copy ONLY the output value with no extra formatting      |
| p          | Paste color from clipboard (can be any supported format) |
| a          | Enable/disable alpha channel                             |
| n          | Input a new color                                        |
| q          | quit                                                     |
| &lt;CR&gt; | quit and copy to clipboard                               |

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

## Configuration
See the [example config](./example-config.toml)

# Goals

- [x] Swap between different selections
  - [x] HSL
  - [x] RGB
  - [x] CYMK
  - [x] ANSI 256 color selection pannel
- [x] Ability to input number for slider
- [x] Swap between different outputs
  - [x] Let user pick from some kind of menu instead of cycling (or both)
  - [x] Option to display all outputs at once
  - [x] Custom output formats
  - [x] <del>Show multiple different custom output formats</del>
    - This is unecessary as different formats can be seperated with %n (a new line)
- [ ] Swap between different output cycles
- [x] Specify starting value with cli option
  - [x] Read starting value from stdin
- [x] Copy output
- [x] Convert any supported format to any other supported format via cli.
- [x] Pressing a key to input a color
- [x] Cli options for setting default Selection/Output formats
- [ ] Event system
  - would write to some kind of fifo/socket file that a shell script can read from
  - events would include:
    - color change
    - on color copy
    - selection type changed
- [x] Config file
  - [x] Custom output formats
  - [x] Keybindings
- [ ] Multiple color previews to allow color comparisons

## Maybe goals

- [ ] Saving colors
- [ ] Using the kitty keyboard input standard

## Non Goals

- supporting non 24bit (truecolor) terminals.
