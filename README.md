# Why another CHIP-8 Emulator?

Because it's fun, and I wanted to make one no_std compatible

# How to use

See the [sdl example](examples/sdl.rs).

In general, you need to do the following in order to have a full working
emulator :

- Draw to the screen
- Handle keyboard events, and route them to the cpu
- Supply a RNG function

# License

MIT
