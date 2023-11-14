# tpick

Tpick is a tui color picker because I was not satisfied with the available color pickers.

None of them have a way to select with HSL except for [this one](https://github.com/uga-rosa/ccc.nvim) but that's a neovim extension which is clunky.

# Usage

Currently there are no cli options.

## Controls

| Control | action                       |
| ------- | ---------------------------- |
| l       | increase current value       |
| h       | decrease current value       |
| 1-9     | set current value to 10%-90% |


# Goals

* [x] HSL selection
* [ ] RGB selection
* [x] HSL output
* [x] RGB output
* [x] Hex output
* [ ] Ability to enable/disable alpha channel
* [x] Ability to swap between different outputs
* [ ] Ability to swap between different selections
* [ ] Ability to specify starting value with cli option
* [ ] Ability to copy output
* [ ] Ability to convert any supported format to any other supported format via cli.

## Maybe goals
* [ ] Saving colors

## Non Goals
* supporting non 24bit (truecolor) terminals.
