# GeBemula

Emulator for GameBoy made in Rust.

## TODO

* factor instructions taking advantage of how they're organized like in the table: https://clrhome.org/table/
    - pay attention to patterns in terms of what register is being used;
    - use such pattern in the `match` (same stuff going to the same code/match arm).

* **fix**: better opcode dispatch.

* have a priority queue (sorted vector, probably enough), keyed by the cycle when the event happens on every iteration;
    - check the next event, check how many cycles remain until it happens and then execute enough instructions to use up those cycles (advancing time) and then handle that event. Rinse and repeat.
    - 'events' in this case will be probably interurpts and maybe some periodic thing to syntthetize and output sound as well as a vblank event, which is when you'll display a frame (and also frame-limit your emulator so it runs in realtime speed).

    - imagine you have a timeline where events happen, like vblank (once every 1/60s or w/e is the exact frequency), hblank (once per scanline), etc or even for one-off events like a DMA which will take some amount of time and then finish, you basically have a queue with those events and then fill in the spots inbetween by running the CPU.
    - this approach should be enough to get you started but I don't know if it'll scale later, you may need to replace it if you end up needing ithgter synchronization (it would still work, it would just be kinda awkward and slow)
    - you basicallyneed to interleave execution of the cput with the other hardware in the console.
    - and for the timing you don't sleep all the time like that, rather, you execute as fast as you can and then sleep for a long time after displaying a frame or reading input or something, so that in the long run things run at the correct speed (or if you want to fast-forward you just don't sleep).

## Tips

* Don't take enum params by reference;
* Prefer using fixed vectors instead of resizable ones (e.g., `[u8; 6]`);
    - or slices (which are pointers + size).
* Use `match` whenever possible instead of `if else`;
