# Things to do

## Performance

- [] buffers
  - [x] instance attribs: use vec3 instead of mat4
  - [x] scale factor (radius) as uniform
  - [] normal buffer: use packed ints instead of floats
  - [] interleave vertex/normals?
  - [] dynamic_draw vs stream_draw
- [] minimal camera far field
- [x] minimize inversions/transpositions in shaders
- [] CPU-side optimizations
  - [x] vget_unchecked
  - [] emit-asm, codegen-units
  - [] simd
  - [] threads?
- [x] canvas devicePixelRatio (high-dpi)

## Features

- [] wavefunction sampling
  - [x] dynamic sample count (switch to heap array)
  - [x] distribution that samples around origin more often
  - [] accelerating interpolation func. instead of tanh()
  - [] sampling regions OR integrate density -> distribution
  - [] derive color, sample and camera parameters from n,l,m
- [] occlusion and deferred rendering
  - [x] render-to-texture, blending
  - [x] viewspace position from normals
  - [] fix MSAA
  - [] actual SSAO impl.
- [] simulate electron trajectories (Bohm guiding eq./probability current)
- [] visualize complex phase?
- [] website UI and controls
