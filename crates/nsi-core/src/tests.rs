#[allow(unused_imports)]
use crate as nsi;

#[cfg(test)]
#[test]
fn test_dodecahedron() {
    let ctx = nsi::Context::new(None).expect("Could not create NSI context.");

    // Create a dodecahedron.
    let face_index: [i32; 60] =
    // 12 regular pentagon faces.
    [
         0, 16,  2, 10,  8,  0,  8,  4, 14, 12,
        16, 17,  1, 12,  0,  1,  9, 11,  3, 17,
         1, 12, 14,  5,  9,  2, 13, 15,  6, 10,
        13,  3, 17, 16,  2,  3, 11,  7, 15, 13,
         4,  8, 10,  6, 18, 14,  5, 19, 18,  4,
         5, 19,  7, 11,  9, 15,  7, 19, 18,  6,
    ];
    let positions: [f32; 60] =
    // 20 points @ 3 vertices.
    [
         1.,  1.,  1.,  1. , 1., -1.,  1., -1.,  1.,
         1., -1., -1., -1.,  1.,  1., -1.,  1., -1.,
        -1., -1.,  1., -1., -1., -1.,
         0.,     0.618,  1.618,   0.,     0.618, -1.618,
         0.,    -0.618,  1.618,   0.,    -0.618, -1.618,
         0.618,  1.618,  0.,      0.618, -1.618,  0.,
        -0.618,  1.618,  0.,     -0.618, -1.618,  0.,
         1.618,  0.,     0.618,   1.618,  0.,    -0.618,
        -1.618,  0.,     0.618,  -1.618,  0.,    -0.618,
    ];

    // Create a new mesh node and call it 'dodecahedron'.
    ctx.create("dodecahedron", nsi::NodeType::Mesh, None);

    use rayon::prelude::*;

    (0..100)
        .into_par_iter()
        .for_each(|_| ctx.connect("dodecahedron", "", ".root", "objects", None));

    // Connect the 'dodecahedron' node to the scene's root.
    ctx.connect("dodecahedron", "", ".root", "objects", None);

    // Define the geometry of the 'dodecahedron' node.
    ctx.set_attribute(
        "dodecahedron",
        &[
            nsi::points!("P", &positions),
            nsi::integers!("P.indices", &face_index),
            // 5 vertices per each face.
            nsi::integers!("nvertices", &[5; 12]),
            // Render this as a subdivison surface.
            nsi::string!("subdivision.scheme", "catmull-clark"),
            // Crease each of our 30 edges a bit.
            nsi::integers!("subdivision.creasevertices", &face_index),
            nsi::floats!("subdivision.creasesharpness", &[10.; 30]),
        ],
    );
}

#[cfg(test)]
#[test]
fn test_reference() {
    let ctx = nsi::Context::new(Some(&[nsi::string!("streamfilename", "stdout")]))
        .expect("Could not create NSI context.");

    let foo = 42u64;
    // Setup an output driver.
    ctx.create("driver1", nsi::NodeType::OutputDriver, None);
    ctx.connect("driver1", "", "beauty", "outputdrivers", None);
    ctx.set_attribute(
        "driver1",
        &[
            nsi::string!("drivername", "idisplay"),
            // Pass a pointer to foo to NSI
            nsi::reference!("_foo", Some(&foo)),
        ],
    );

    drop(ctx);
}

#[cfg(test)]
#[test]
fn live_edit() {
    // # Compile the shaders.
    // "oslc emitter.osl")
    // "oslc matte.osl")
    // "oslc waves.osl")

    // create rendering context.
    let c = nsi::Context::new(Some(&[nsi::string!("streamfilename", "stdout")]))
        .expect("Could not create NSI context.");

    // Setup a camera transform.
    c.create("cam1_trs", nsi::NodeType::Transform, None);
    c.connect("cam1_trs", "", ".root", "objects", None);

    c.set_attribute(
        "cam1_trs",
        &[nsi::double_matrix!(
            "transformationmatrix",
            &[1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 5., 1.,]
        )],
    );

    // Setup a camera.
    c.create("cam1", nsi::NodeType::PerspectiveCamera, None);

    c.set_attribute("cam1", &[nsi::float!("fov", 35.)]);
    c.connect("cam1", "", "cam1_trs", "objects", None);

    // Setup a screen.
    c.create("s1", nsi::NodeType::Screen, None);
    c.connect("s1", "", "cam1", "screens", None);
    c.set_attribute(
        "s1",
        &[
            nsi::integers!("resolution", &[1280, 720]).array_len(2),
            nsi::integer!("oversampling", 16),
        ],
    );

    // Setup an output layer.
    c.create("beauty", nsi::NodeType::OutputLayer, None);
    c.set_attribute(
        "beauty",
        &[
            nsi::string!("variablename", "Ci"),
            nsi::integer!("withalpha", 1),
            nsi::string!("scalarformat", "half"),
            nsi::color!("some_color", &[0.1f32, 0.2, 0.3]),
        ],
    );
    c.connect("beauty", "", "s1", "outputlayers", None);

    // Setup an output driver.
    c.create("driver1", nsi::NodeType::OutputDriver, None);
    c.connect("driver1", "", "beauty", "outputdrivers", None);
    c.set_attribute("driver1", &[nsi::string!("drivername", "idisplay")]);

    // Add a plane.
    c.create("mesh1", nsi::NodeType::Mesh, None);
    c.connect("mesh1", "", ".root", "objects", None);
    c.set_attribute(
        "mesh1",
        &[
            nsi::integer!("nvertices", 4),
            nsi::points!(
                "P",
                &[-1.0f32, -0.8, -1., -1., -0.8, 1., 1., -0.8, 1., 1., -0.8, -1.0]
            ),
        ],
    );

    c.create("plane_attribs", nsi::NodeType::Attributes, None);
    c.connect("plane_attribs", "", "mesh1", "geometryattributes", None);

    // Add a basic shader for the plane.
    c.create("shader1", nsi::NodeType::Shader, None);
    c.set_attribute("shader1", &[nsi::string!("shaderfilename", "matte")]);
    c.connect("shader1", "", "plane_attribs", "surfaceshader", None);

    // Add a triangular light, with shader.
    c.create("light1_trs", nsi::NodeType::Transform, None);
    c.connect("light1_trs", "", ".root", "objects", None);

    c.set_attribute(
        "light1_trs",
        &[nsi::double_matrix!(
            "transformationmatrix",
            &[0.1f64, 0., 0., 0., 0., 0.1, 0., 0., 0., 0., 0.1, 0., 0., 4., 0., 1.,]
        )],
    );

    c.create("light1", nsi::NodeType::Mesh, None);
    // This one is connected to the transform instead of the mesh
    // itself. Because we can.
    c.connect("light1", "", "light1_trs", "objects", None);
    c.set_attribute(
        "light1",
        &[
            nsi::integer!("nvertices", 3),
            nsi::points!("P", &[-1., 0., 0., 0., 0., 1., 1., 0., 0.]),
        ],
    );

    c.create("light1_shader", nsi::NodeType::Shader, None);
    c.set_attribute(
        "light1_shader",
        &[
            nsi::string!("shaderfilename", "emitter"),
            nsi::float!("power", 80.),
        ],
    );

    c.create("light1_attribs", nsi::NodeType::Attributes, None);
    c.connect(
        "light1_attribs",
        "",
        "light1_trs",
        "geometryattributes",
        None,
    );
    c.connect("light1_shader", "", "light1_attribs", "surfaceshader", None);

    // Start interactive render.
    c.render_control(&[nsi::string!("action", "start")]); //, interactive=1)

    // Let it render a while.
    //thread::sleep(time::Duration::from_secs(5));

    c.render_control(&[nsi::string!("action", "wait")]);
}

// FIXME: port rest of live_edit example from Python
