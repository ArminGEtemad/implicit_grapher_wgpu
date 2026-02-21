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

## First Focus

- [ ] Basic WGPU setup
- [ ] Raymarcher prototype
- [ ] Camera Setup
