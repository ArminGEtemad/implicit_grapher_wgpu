# Implicit Grapher WGPU

A real-time 3D implicit surface grapher written in Rust using `wgpu` + `wgsl`.

The goal of this project is to render surfaces defined by implicit equations  
of the form

```math
f(x, y, z) = 0
```

in real time using GPU raymarching.

This project is my second large learning project journey into GPU programming.

I am planning to re-use the small render graph I make for my first WGPU project, which was [reaction_diffusion_wgpu](https://github.com/ArminGEtemad/reaction_diffusion_wgpu).

## Milestones

### First Focus

- [x] Basic WGPU setup
- [x] Raymarcher prototype
- [x] Camera Setup

## Interactivity

| Key         | What it does                                        |
| ----------- | --------------------------------------------------- |
| W           | Rotate camera upwards vertically                    |
| A           | Rotate camera to left horizontally                  |
| S           | Rotate camera downwards vertically                  |
| D           | Rotate camera to right horizontally                 |
| Arrow Up    | Slide camera along positive y axis                  |
| Arrow Down  | Slide camera along negative y axis                  |
| Arrow Right | Slide camera along positive x axis                  |
| Arrow Left  | Slide camera along negative x axis                  |
| Mouse wheel | Camera zoom in and out                              |
| O           | Camera points towards the origin of the coordinates |

## Screen Shots and GIFs

For now the program has a basic 3D axes for the coordinate sytem.

<div style="display: flex; gap: 100px; align-items: flex-start;">

  <div>
    <img src="docs/Pics/Coordinates.png" width="1000"/>
  </div>

</div>

And a functioning camera

### Rotate WASD

<div style="display: flex; gap: 100px; align-items: flex-start;">

  <div>
    <img src="docs/GIFs/WASD.gif" width="1000"/>
  </div>

</div>

### Zoom in and out MouseWheel

<div style="display: flex; gap: 100px; align-items: flex-start;">

  <div>
    <img src="docs/GIFs/MouseWheel.gif" width="1000"/>
  </div>

</div>

### Slide Along axis and back to origin

<div style="display: flex; gap: 100px; align-items: flex-start;">

  <div>
    <img src="docs/GIFs/SlideAndKeyO.gif" width="1000"/>
  </div>

</div>
