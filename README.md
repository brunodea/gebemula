# GeBemula

Emulator for GameBoy made in Rust.

## TODO

* factor instructions taking advantage of how they're organized like in the table: https://clrhome.org/table/
    - pay attention to patterns in terms of what register is being used;
    - use such pattern in the `match` (same stuff going to the same code/match arm).

## Tips

* Don't take enum params by reference;
* Prefer using fixed vectors instead of resizable ones (e.g., `[u8; 6]`);
    - or slices (which are pointers + size).
* Use `match` whenever possible instead of `if else`;
