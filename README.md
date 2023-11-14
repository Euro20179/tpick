# tpick

Tpick is a tui color picker because I was not satisfied with the available color pickers.

None of them has a way to select with HSL except for [this one](https://github.com/uga-rosa/ccc.nvim) but that's a neovim extension which is clunky.

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
* [ ] RGB output
* [ ] Hex output
* [ ] Ability to swap between different selections/outputs
* [ ] Ability to specify starting value with cli option
* [ ] Ability to copy output
* [ ] Ability to convert any supported format to any other supported format via cli.

## Maybe goals
* [ ] Saving colors

## Non Goals
* supporting non 24bit (truecolor) terminals.
