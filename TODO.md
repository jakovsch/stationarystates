
## Performance

- [] instance attribs: use vec3 instead of mat4
- [] scale factor (radius) as uniform
- [] minimal camera far field
- [] normal buffer: use packed ints instead of floats
  - [] interleave vertex/normals
- [] minimize inversions/transpositions in shaders
- [] cull failed samples before sending to shaders
- [] dynamic_draw vs stream_draw
- [] canvas image-rendering: pixelated

## Features

- [] dynamic sample count (switch to heap array)
- [] distribution that samples around center more often
- [] SSAO and deferred rendering
- [] Simulate electron trajectories (guiding eq./probability density)
- [] Website UI and controls
