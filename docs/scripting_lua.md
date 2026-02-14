# Lua Scripting Guide

## Engine API

- `Transform`: `position()`, `set_position(x, y, z)`, `forward()`, etc.
- `Input`: `is_key_pressed("Space")`, `get_axis("Horizontal")`
- `World`: `spawn()`, `despawn(id)`

## Hooks

- `on_start()`: Called when script starts.
- `on_update()`: Called every frame.

## Hot Reload

Edit files in `assets/scripts` while engine is running. State is preserved via automatic copying of table fields.
