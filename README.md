# ɴsɪ

High level Rust bindings for Illumination Research’s Nodal Scene
Interface – [ɴsɪ](https://nsi.readthedocs.io/).

[![Build](https://github.com/virtualritz/nsi/workflows/Build/badge.svg)](https://github.com/virtualritz/nsi/actions)
[![Documentation](https://docs.rs/nsi/badge.svg)](https://docs.rs/nsi)
[![Crate](https://img.shields.io/crates/v/nsi.svg)](https://crates.io/crates/nsi)
[![Chat](https://badges.gitter.im/n-s-i/community.svg)](https://gitter.im/n-s-i/community)

This puts one of the most advanced 3D production offline renderers at
your fingertips in Rust – [3Delight](https://www.3delight.com/).

![Moana Island, rendered with 3Delight|ɴsɪ](moana_island.jpg)
*[The Moana Island Scene](https://www.technology.disneyanimation.com/islandscene),
provided courtesy of Walt Disney Pictures, rendered with 3Delight|ɴsɪ.*

This is a huge scene (72GB of data) made of 31 million instances,
78 million polygons defining subdivision surface geometry and 2,300
[Ptex](http://ptex.us/) textures. The above image was rendered in less
than two minutes (wall time) using
[3Delight Cloud](https://documentation.3delightcloud.com/display/3DLC/Cloud+Rendering+Speed).

## Example

```rust
// Create a context to send the scene to.
let ctx = nsi::Context::new(None)
    .expect("Could not create NSI context.");

// Create a dodecahedron.

// 12 regular pentagon faces.
let face_index: [i32; 60] =
    [
         0, 16,  2, 10,  8,  0,  8,  4, 14, 12,
        16, 17,  1, 12,  0,  1,  9, 11,  3, 17,
         1, 12, 14,  5,  9,  2, 13, 15,  6, 10,
        13,  3, 17, 16,  2,  3, 11,  7, 15, 13,
         4,  8, 10,  6, 18, 14,  5, 19, 18,  4,
         5, 19,  7, 11,  9, 15,  7, 19, 18,  6,
    ];

// Golden ratio.
let phi: f32 = 0.5 * (1.0 + 5_f32.sqrt());

// Golden ratio conjugate.
let phi_c: f32 = phi - 1.0;

// 20 points @ 3 vertices.
let positions: [f32; 60] =
    [
         1.,     1.,     1.,     1.,     1.,    -1.,
         1.,    -1.,     1.,     1.,    -1.,    -1.,
        -1.,     1.,     1.,    -1.,     1.,    -1.,
        -1.,    -1.,     1.,    -1.,    -1.,    -1.,
         0.,     phi_c,  phi,    0.,     phi_c, -phi,
         0.,    -phi_c,  phi,    0.,    -phi_c, -phi,
         phi_c,  phi,    0.,     phi_c, -phi,    0.,
        -phi_c,  phi,    0.,    -phi_c, -phi,    0.,
         phi,    0.,     phi_c,  phi,    0.,    -phi_c,
        -phi,    0.,     phi_c, -phi,    0.,    -phi_c,
    ];

// Create a new mesh node and call it 'dodecahedron'.
ctx.create("dodecahedron", nsi::MESH, None);

// Connect the 'dodecahedron' node to the scene's root.
ctx.connect("dodecahedron", None, nsi::ROOT, "objects", None);

// Define the geometry of the 'dodecahedron' node.
ctx.set_attribute(
    "dodecahedron",
    &[
        nsi::points!("P", &positions),
        nsi::unsigneds!("P.indices", &face_index),
        // 5 vertices per each face.
        nsi::unsigneds!("nvertices", &[5; 12]),
        // Render this as a subdivison surface.
        nsi::string!("subdivision.scheme",
            "catmull-clark"
        ),
        // Crease each of our 30 edges a bit.
        nsi::unsigneds!("subdivision.creasevertices",
            &face_index
        ),
        nsi::floats!(
            "subdivision.creasesharpness",
            &[10.; 30]
        ),
    ],
);
```

Also check out my
[Diffusion Limited Aggregation play-thingy](https://github.com/virtualritz/rust-diffusion-limited-aggregation)
or [Polyhedron Operators](https://github.com/virtualritz/polyhedron-operators)
for more example code (demonstrates render settings, sending meshes
to the renderer, instancing, particle rendering, [OSL](https://github.com/imageworks/OpenShadingLanguage)
shaders, environment (lights) and dumping a scene description to disk).

PRs are most welcome!

## Getting Pixels

The crate has support for streaming pixels from the renderer, via
callbacks (i.e. closures) during and/or after rendering via the
`output` module. This module is enabled through the feature of the
same name.

There is a full example showing color conversion and writing data
out to 8bit/channel PNG and 32bit/channel (float) OpenEXR formats.

## Dependencies

This crate depends on `nsi-core` which in term requires a renderer that
implements the ɴsɪ API to generate images.

Currently the only renderer that does is 3Delight which, though
commercial, has been and is free for personal use since over twenty
years.

> ***Note:*** The free version of 3Delight will render with up to 12
cores on your machine. For crazier projects you can use their cheap
cloud rendering service that gives you access to unlimited CPU cores.
When you register you get 1,000 cloud minutes for free which ain’t too
shabby.

That being said – I hope this crate serves as inspiration for other
people writing renderers, particularly in Rust, to adopt this API for
scene description.

## Prerequisites

Before you start it is suggested that you [download a 3Delight
package](https://www.3delight.com/download) for your platform & install
it (supported: Linux, macOS, Windows). This will allow you to create
pretty pictures with the examples straight away.

It will also install 3Delight Display which you can render to as an
alternative to writing images to disk. When working with this crate
this is quite handy.

You can skip this step and build with the `download_3delight` feature.
However, this will download an older version of 3Delight so this is
not suggested.

## Cargo Features

* `output` – Add support for streaming pixels from the renderer
  to the calling context via closures.

* `jupyter` – Add support for rendering to Jupyter notebooks (when
  using a [Rust Jupyter kernel](https://github.com/google/evcxr)).

* `toolbelt` – Add convenience methods to `Context`.

* [`delight`](crate::delight) – Add some nodes & shaders specifi to
  3Delight.

* `nightly` – Enable some unstable features (suggested if you build
  with a `nightly` toolchain)

* `ustr_handles` – use [`ustr`](https://crates.io/crates/ustr) for
  node handles. This will give a you a speed boost if your node names
  aren't changing while an app using ɴsɪ is running but is not advised
  otherwise (`ustr` are never freed).

* `download_lib3delight` & `link_lib3delight` – See next section.

## Linking Style

The 3Delight dynamic library (`lib3delight`) can either be linked to
during build or loaded at runtime.

By default the lib is *loaded at runtime*.

* Load `lib3delight` at runtime (default). This has several advantages:

  1. If you ship your application or library you can ship it without
     the library. It can still run and will print an informative error
     if the library can not be loaded.

  2. A user can install an updated version of the renderer and stuff
     will ‘just work’.

* Dynamically link against `lib3delight`.

  * `lib3delight` becomes a dependency. If it can not be found your
    lib/app will not load/start.

  * The feature is called `link_lib3delight`.

  * You should disable default features (`default-features = false`).

  * in this case:

    ```toml
    [dependencies]
    nsi = { version = "0.7", default-features = false, features = ["link_lib3delight"] }
    ```

* Download `lib3delight` during build.

  * `lib3delight` is downloaded during build. Note that this may be
    an outdated version. This feature mainly exists for CI purposes.

  * The feature is called `download_lib3delight`.

## Documentation

Crate documentation can be found at [docs.rs](https://docs.rs/nsi/).

Docs for the C, C++, Lua & Python bindings as well as an introduction
and deep dive into the API [can be found here](https://nsi.readthedocs.io/).

## Getting Help

I hang out on the [3Delight Discord](https://discord.gg/9J9WwqNj) (I
have the same user name as on GitHub). Ping me in the `#lobby` channel
or send me a PM.

There is also a [3Delight Slack](https://join.slack.com/t/3delight/shared_invite/zt-eipakj10-lK84ZzUzWgDw0qJ3Z3KuOg)
which has a dedicated, invitation only channel about ɴsɪ.

If you have more advanced questions or want to add support for the ɴsɪ
API/export to ɴsɪ to your renderer/DCC app/whatever ping me and I get
you an invite.
