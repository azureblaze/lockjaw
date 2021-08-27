# Cubit / color_cycle

This is an example of a "game" that only loops colored background, using
a preliminary game engine, with a DirectX 12 Windows implementation. This
example showcases how Lockjaw can be used in a *real environment*

## Project structure

### ./

The interface and platform independent implementations of the `Cubit` game engine.

### cubit-win64/

The implementation of the game engine using DirectX 12 on Windows.

### color_cycle/

The game binary.

## Dependency chain

```
color_cycle -> cubit-win64 -> cubit 
        \---------------------/^
```