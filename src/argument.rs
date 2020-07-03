pub mod arg {
    use enum_dispatch::enum_dispatch;
    use std::{any::Any, ffi::CString, marker::PhantomData};

    use crate::*;

    #[inline]
    pub(crate) fn get_c_param_vec(args_in: &ArgSlice) -> Vec<nsi_sys::NSIParam_t> {
        let mut args_out = Vec::<nsi_sys::NSIParam_t>::with_capacity(args_in.len());
        for arg_in in args_in {
            args_out.push(nsi_sys::NSIParam_t {
                name: arg_in.name.as_ptr(),
                data: arg_in.data.as_c_ptr(),
                type_: arg_in.data.type_() as i32,
                arraylength: arg_in.array_length as i32,
                count: (arg_in.data.len() / arg_in.array_length) as u64,
                flags: arg_in.flags as std::os::raw::c_int,
            });
        }
        args_out
    }

    /// A slice of (optional) [`Context`] method arguments.
    pub type ArgSlice<'a, 'b> = [Arg<'a, 'b>];

    /// A vector of (optional) [`Context`] method arguments.
    pub type ArgVec<'a, 'b> = Vec<Arg<'a, 'b>>;

    pub struct Arg<'a, 'b> {
        pub(crate) name: CString,
        pub(crate) data: ArgData<'a, 'b>,
        // length of each element if an array type
        pub(crate) array_length: usize,
        // number of elements
        pub(crate) flags: u32,
    }

    impl<'a, 'b> Arg<'a, 'b> {
        #[inline]
        pub fn new(name: &str, data: ArgData<'a, 'b>) -> Self {
            Arg {
                name: CString::new(name).unwrap(),
                data,
                array_length: 1,
                flags: 0,
            }
        }

        #[inline]
        pub fn array_len(mut self, length: usize) -> Self {
            self.array_length = length;
            self.flags |= nsi_sys::NSIParamIsArray;
            self
        }

        #[inline]
        pub fn per_face(mut self) -> Self {
            self.flags |= nsi_sys::NSIParamPerFace;
            self
        }

        #[inline]
        pub fn per_vertex(mut self) -> Self {
            self.flags |= nsi_sys::NSIParamPerVertex;
            self
        }

        #[inline]
        pub fn linear_interpolation(mut self) -> Self {
            self.flags |= nsi_sys::NSIParamInterpolateLinear;
            self
        }
    }

    #[enum_dispatch(ArgData)]
    pub(crate) trait ArgDataMethods {
        //const TYPE: Type;
        fn type_(&self) -> Type;
        fn len(&self) -> usize;
        fn as_c_ptr(&self) -> *const std::ffi::c_void;
    }

    // Lifetime 'a is for any tuple or array type
    // Lifetime 'b is for the pointer type. This is pegged to the lifetime of the [`Context`]
    #[enum_dispatch]
    pub enum ArgData<'a, 'b> {
        /// Single [`f32`) value.
        Float,
        Floats(Floats<'a>),
        /// Single [`f64`] value.
        Double,
        Doubles(Doubles<'a>),
        /// Single [`i32`] value.
        Integer,
        /// An [[`i32`]] array.
        Integers(Integers<'a>),
        /// Single [`i32`] value.
        Unsigned,
        /// An [[`i32`]] array.
        Unsigneds(Unsigneds<'a>),
        /// A [`String`].
        String(String),
        /// A [[`String`]] array.
        Strings(Strings),
        /// Color in linear space, given as a red, green, blue triplet
        /// of [`f32`] values; usually in the range `0..1`.
        Color(Color<'a>),
        /// An arry of colors.
        Colors(Colors<'a>),
        /// Point, given as three [`f32`] values.
        Point(Point<'a>),
        Points(Points<'a>),
        /// Vector, given as three [`f32`] values.
        Vector(Vector<'a>),
        Vectors(Vectors<'a>),
        /// Normal vector, given as three [`f32`] values.
        /// values.
        Normal(Normal<'a>),
        Normasl(Normals<'a>),
        /// Transformation matrix, given as 16 [`f32`] floating
        /// point datas.
        Matrix(Matrix<'a>),
        Matrices(Matrices<'a>),
        /// Transformation matrix, given as 16 [`f64`] floating
        /// point datas.
        DoubleMatrix(DoubleMatrix<'a>),
        DoubleMatrices(DoubleMatrices<'a>),
        /// Reference to arbitrary data.
        Reference(Reference<'b>),
        References(References<'b>),
        /// Raw (`*const T`) pointer.
        Pointer,
        Pointers(Pointers<'a>),
    }

    macro_rules! nsi_data_def {
        ($type: ty, $name: ident, $nsi_type: expr) => {
            pub struct $name {
                data: $type,
            }

            impl $name {
                pub fn new(data: $type) -> Self {
                    Self { data }
                }
            }

            impl ArgDataMethods for $name {
                fn type_(&self) -> Type {
                    $nsi_type
                }

                fn len(&self) -> usize {
                    1
                }

                fn as_c_ptr(&self) -> *const std::ffi::c_void {
                    &self.data as *const $type as _
                }
            }
        };
    }

    macro_rules! nsi_data_array_def {
        ($type: ty, $name: ident, $nsi_type: expr) => {
            pub struct $name<'a> {
                data: &'a [$type],
            }

            impl<'a> $name<'a> {
                pub fn new(data: &'a [$type]) -> Self {
                    debug_assert!(data.len() % $nsi_type.element_size() == 0);
                    Self { data }
                }
            }

            impl<'a> ArgDataMethods for $name<'a> {
                fn type_(&self) -> Type {
                    $nsi_type
                }

                fn len(&self) -> usize {
                    self.data.len() / $nsi_type.element_size()
                }

                fn as_c_ptr(&self) -> *const std::ffi::c_void {
                    self.data.as_ptr() as _
                }
            }
        };
    }

    macro_rules! nsi_tuple_data_def {
        ($type: tt, $len: expr, $name: ident, $nsi_type: expr) => {
            pub struct $name<'a> {
                data: &'a [$type; $len],
            }

            impl<'a> $name<'a> {
                pub fn new(data: &'a [$type; $len]) -> Self {
                    Self { data }
                }
            }

            impl<'a> ArgDataMethods for $name<'a> {
                fn type_(&self) -> Type {
                    $nsi_type
                }

                fn len(&self) -> usize {
                    1
                }

                fn as_c_ptr(&self) -> *const std::ffi::c_void {
                    self.data.as_ptr() as _
                }
            }
        };
    }

    nsi_data_def!(f32, Float, Type::Float);
    nsi_data_def!(f64, Double, Type::Double);
    nsi_data_def!(i32, Integer, Type::Integer);
    nsi_data_def!(u32, Unsigned, Type::Integer);
    //nsi_data_def!(*const std::ffi::c_void, Pointer, Type::Pointer);

    /// Reference type *with* lifetime guaratees.
    ///
    /// Prefer this over using a raw [`Pointer`]
    /// as it allows the compiler to check that
    /// the data you reference outlives the
    /// [`Context`] you eventually send it to.
    ///
    /// This gets converted to a raw pointer when passed
    /// through the FFI boundary.
    /// ```
    /// struct Payload {
    ///     some_data: u32,
    /// }
    ///
    /// let ctx = nsi::Context::new(&[]).unwrap();
    ///
    /// // Lots of scene setup omitted ...
    ///
    /// // Setup a custom output driver and send
    /// // a payload to it through the FFI boundary
    /// ctx.create("driver", nsi::NodeType::OutputDriver, &[]);
    /// ctx.connect("driver", "", "beauty", "outputdrivers", &[]);
    /// let payload = Payload {
    ///     some_data: 42,
    /// };
    /// ctx.set_attribute(
    ///     "driver",
    ///     &[
    ///         nsi::string!("drivername", "custom_driver"),
    ///         // Payload gets sent as raw pointer through
    ///         // the FFI boundary.
    ///         nsi::reference!("payload", Some(&payload)),
    ///     ],
    /// );
    ///
    /// // We need to explicitly call drop here as
    /// // ctx's lifetime is pegged to that of payload.
    /// drop(ctx);
    /// ```
    pub struct Reference<'a> {
        data: *const std::ffi::c_void,
        _marker: PhantomData<&'a ()>,
    }

    impl<'a> Reference<'a> {
        pub fn new(data: Option<&'a dyn Any>) -> Self {
            Self {
                data: data
                    .map(|p| p as *const _ as *const std::ffi::c_void)
                    .unwrap_or(std::ptr::null()),
                _marker: PhantomData,
            }
        }
    }

    impl<'a> ArgDataMethods for Reference<'a> {
        fn type_(&self) -> Type {
            Type::Pointer
        }

        fn len(&self) -> usize {
            1
        }

        fn as_c_ptr(&self) -> *const std::ffi::c_void {
            self.data
        }
    }

    /// Raw pointer type *without* lifietime guaratees.
    ///
    /// This can't guarantee that the data this
    /// points to outlives the [`Context`] you
    /// eventually send this to. This is the user's
    /// responsibility.
    ///
    /// If you need to send a pointer a better
    /// alternative is the [`Reference`] type
    /// that allows the compiler to check that
    /// the the data outlives the [`Context`].
    pub struct Pointer {
        data: *const std::ffi::c_void,
    }

    impl Pointer {
        pub unsafe fn new(data: *const std::ffi::c_void) -> Self {
            Self { data }
        }
    }

    impl ArgDataMethods for Pointer {
        fn type_(&self) -> Type {
            Type::Pointer
        }

        fn len(&self) -> usize {
            1
        }

        fn as_c_ptr(&self) -> *const std::ffi::c_void {
            self.data
        }
    }

    pub struct String {
        _data: CString,
        // The FFI API needs a pointer to a C string
        pointer: *const std::ffi::c_void,
    }

    impl String {
        pub fn new<T: Into<Vec<u8>>>(data: T) -> Self {
            let _data = CString::new(data).unwrap();
            let pointer = _data.as_ptr() as _;

            String { _data, pointer }
        }
    }

    impl ArgDataMethods for String {
        fn type_(&self) -> Type {
            Type::String
        }

        fn len(&self) -> usize {
            1
        }

        fn as_c_ptr(&self) -> *const std::ffi::c_void {
            unsafe { std::mem::transmute(&self.pointer) }
        }
    }

    nsi_data_array_def!(f32, Floats, Type::Float);
    nsi_data_array_def!(f64, Doubles, Type::Double);
    nsi_data_array_def!(i32, Integers, Type::Integer);
    nsi_data_array_def!(u32, Unsigneds, Type::Integer);
    nsi_data_array_def!(f32, Colors, Type::Color);
    nsi_data_array_def!(f32, Points, Type::Point);
    nsi_data_array_def!(f32, Vectors, Type::Vector);
    nsi_data_array_def!(f32, Normals, Type::Normal);
    nsi_data_array_def!(f32, Matrices, Type::Matrix);
    nsi_data_array_def!(f64, DoubleMatrices, Type::DoubleMatrix);

    /// Reference array type *with* lifetime guarantees.
    ///
    /// Prefer this over using a raw [`Pointers`]
    /// as it allows the compiler to check that
    /// the data you reference outlives the
    /// [`Context`] you eventually send it to.
    ///
    /// This gets converted to am array of raw pointers when
    /// passed through the FFI boundary.
    pub struct References<'a> {
        data: Vec<*const std::ffi::c_void>,
        _marker: PhantomData<&'a ()>,
    }

    impl<'a> References<'a> {
        pub fn new(data: &'a [Option<&'a dyn Any>]) -> Self {
            debug_assert!(data.len() % Type::Pointer.element_size() == 0);

            let mut c_data = Vec::<*const std::ffi::c_void>::with_capacity(data.len());

            for e in data {
                c_data.push(
                    e.map(|p| p as *const _ as *const std::ffi::c_void)
                        .unwrap_or(std::ptr::null()),
                );
            }

            Self {
                data: c_data,
                _marker: PhantomData,
            }
        }
    }

    impl<'a> ArgDataMethods for References<'a> {
        fn type_(&self) -> Type {
            Type::Pointer
        }

        fn len(&self) -> usize {
            self.data.len() / Type::Pointer.element_size()
        }

        fn as_c_ptr(&self) -> *const std::ffi::c_void {
            self.data.as_ptr() as _
        }
    }

    /// Raw pointer array type *without* lifietime guaratees.
    ///
    /// This can't guarantee that the data this points to
    /// outlives the [`Context`] you eventually send this
    /// to. This is your responsibility.
    ///
    /// If you need to send pointers a better alternative
    /// is the [`References`] type that allows the compiler
    /// to check that the the referenced data outlives the
    /// [`Context`].
    pub struct Pointers<'a> {
        data: &'a [*const std::ffi::c_void],
    }

    impl<'a> Pointers<'a> {
        /// This is marked unsafe because the responsibility
        /// to ensure the pointer can be safely dereferenced
        /// after the function has returned lies with the user.
        ///
        /// [`References`] is as a *safe* alternative.
        pub unsafe fn new(data: &'a [*const std::ffi::c_void]) -> Self {
            Self { data }
        }
    }

    impl<'a> ArgDataMethods for Pointers<'a> {
        fn type_(&self) -> Type {
            Type::Pointer
        }

        fn len(&self) -> usize {
            self.data.len() / Type::Pointer.element_size()
        }

        fn as_c_ptr(&self) -> *const std::ffi::c_void {
            self.data.as_ptr() as _
        }
    }

    pub struct Strings {
        _data: Vec<CString>,
        pointer: Vec<*const std::ffi::c_void>,
    }

    impl Strings {
        pub fn new<T: Into<Vec<u8>> + Copy>(data: &[T]) -> Self {
            let mut _data = Vec::<CString>::with_capacity(data.len());
            let mut pointer = Vec::<*const std::ffi::c_void>::with_capacity(data.len());

            data.iter().for_each(|s| {
                _data.push(CString::new(*s).unwrap());
                pointer.push(_data.last().unwrap().as_ptr() as _);
            });

            Strings { _data, pointer }
        }
    }

    impl ArgDataMethods for Strings {
        fn type_(&self) -> Type {
            Type::String
        }

        fn len(&self) -> usize {
            self.pointer.len()
        }

        fn as_c_ptr(&self) -> *const std::ffi::c_void {
            self.pointer.as_ptr() as _
        }
    }

    nsi_tuple_data_def!(f32, 3, Color, Type::Color);
    nsi_tuple_data_def!(f32, 3, Point, Type::Point);
    nsi_tuple_data_def!(f32, 3, Vector, Type::Vector);
    nsi_tuple_data_def!(f32, 3, Normal, Type::Normal);
    nsi_tuple_data_def!(f32, 16, Matrix, Type::Matrix);
    nsi_tuple_data_def!(f64, 16, DoubleMatrix, Type::DoubleMatrix);

    /// Identifies an [`Arg`]’s data type.
    #[derive(Copy, Clone, Debug, PartialEq)]
    #[repr(i32)]
    pub(crate) enum Type {
        /// A single 32-bit ([`f32`]) floating point value.
        Float = nsi_sys::NSIType_t_NSITypeFloat as i32,
        /// A single 64-bit ([`f64`]) floating point value.
        Double = nsi_sys::NSIType_t_NSITypeDouble as i32,
        /// Single 32-bit ([`i32`]) integer data.
        Integer = nsi_sys::NSIType_t_NSITypeInteger as i32,
        /// A [`String`].
        String = nsi_sys::NSIType_t_NSITypeString as i32,
        /// Color, given as three 32-bit ([`i32`]) floating point datas,
        /// usually in the range `0..1`. Red would e.g. be `[1.0, 0.0,
        /// 0.0]`
        Color = nsi_sys::NSIType_t_NSITypeColor as i32,
        /// Point, given as three 32-bit ([`f32`])floating point datas.
        Point = nsi_sys::NSIType_t_NSITypePoint as i32,
        /// Vector, given as three 32-bit ([`f32`]) floating point datas.
        Vector = nsi_sys::NSIType_t_NSITypeVector as i32,
        /// Normal vector, given as three 32-bit ([`f32`]) floating point
        /// datas.
        Normal = nsi_sys::NSIType_t_NSITypeNormal as i32,
        /// Transformation matrix, given as 16 32-bit ([`f32`]) floating
        /// point datas.
        Matrix = nsi_sys::NSIType_t_NSITypeMatrix as i32,
        /// Transformation matrix, given as 16 64-bit ([`f64`]) floating
        /// point datas.
        DoubleMatrix = nsi_sys::NSIType_t_NSITypeDoubleMatrix as i32,
        /// Raw (`*const T`) pointer.
        Pointer = nsi_sys::NSIType_t_NSITypePointer as i32,
    }

    impl Type {
        /// Returns the size of the resp. type in bytes.
        #[inline]
        pub(crate) fn element_size(&self) -> usize {
            match self {
                Type::Float => 1,
                Type::Double => 1,
                Type::Integer => 1,
                Type::String => 1,
                Type::Color => 3,
                Type::Point => 3,
                Type::Vector => 3,
                Type::Normal => 3,
                Type::Matrix => 16,
                Type::DoubleMatrix => 16,
                Type::Pointer => 1,
            }
        }
    }

    /*
    #[macro_export]
    macro_rules! arg {
        ($name:expr, $value:expr) => {
            nsi::arg::Arg::new($name, $value)
        };
    }


    #[macro_export]
    macro_rules! arg_data {
        ($name: ident, $value: expr) => {
            nsi::arg::ArgData::from(nsi::arg::$name::new($value))
        };
    }
    */
}

#[macro_export]
macro_rules! float {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new($name, nsi::arg::ArgData::from(nsi::arg::Float::new($value)))
    };
}

#[macro_export]
macro_rules! floats {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Floats::new($value)),
        )
    };
}

#[macro_export]
macro_rules! double {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Double::new($value)),
        )
    };
}

#[macro_export]
macro_rules! doubles {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Doubles::new($value)),
        )
    };
}

#[macro_export]
macro_rules! integer {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Integer::new($value)),
        )
    };
}

#[macro_export]
macro_rules! integers {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Integers::new($value)),
        )
    };
}

#[macro_export]
macro_rules! unsigned {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Unsigned::new($value)),
        )
    };
}

#[macro_export]
macro_rules! unsigneds {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Unsigneds::new($value)),
        )
    };
}

#[macro_export]
macro_rules! color {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new($name, nsi::arg::ArgData::from(nsi::arg::Color::new($value)))
    };
}

#[macro_export]
macro_rules! colors {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Colors::new($value)),
        )
    };
}

#[macro_export]
macro_rules! point {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new($name, nsi::arg::ArgData::from(nsi::arg::Point::new($value)))
    };
}

#[macro_export]
macro_rules! points {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Points::new($value)),
        )
    };
}

#[macro_export]
macro_rules! vector {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Vector::new($value)),
        )
    };
}

#[macro_export]
macro_rules! vectors {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Vectors::new($value)),
        )
    };
}

#[macro_export]
macro_rules! normal {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Normal::new($value)),
        )
    };
}

#[macro_export]
macro_rules! normals {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Normals::new($value)),
        )
    };
}

#[macro_export]
macro_rules! matrix {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Matrix::new($value)),
        )
    };
}

#[macro_export]
macro_rules! matrices {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Matrices::new($value)),
        )
    };
}

/// A macro to create a double precision 4×4 matrix argument.
/// # Example
/// ```
/// // create rendering context.
/// let ctx = nsi::Context::new(&[]).unwrap();
///
/// // Setup a transform node.
/// ctx.create("xform", nsi::NodeType::Transform, &[]);
/// ctx.connect("xform", "", ".root", "objects", &[]);
///
/// // Translate 5 units along z-axis.
/// ctx.set_attribute(
///     "xform",
///     &[nsi::double_matrix!(
///         "transformationmatrix",
///         &[1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 5., 1.,]
///     )],
/// );
/// ```
#[macro_export]
macro_rules! double_matrix {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::DoubleMatrix::new($value)),
        )
    };
}

/// A macro to create a double precision 4×4 matrix array argument.
#[macro_export]
macro_rules! double_matrices {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::DoubleMatrices::new($value)),
        )
    };
}

/// A macro to create a string argument.
/// # Example
/// ```
/// // Create rendering context.
/// let ctx = nsi::Context::new(&[nsi::string!(
///     "streamfilename",
///     "stdout"
/// )])
/// .expect("Could not create NSI context.");
/// ```
#[macro_export]
macro_rules! string {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::String::new($value)),
        )
    };
}

/// A macro to create a string array argument.
/// # Example
/// ```
/// // Create rendering context.
/// let ctx = nsi::Context::new(&[]).unwrap();
/// // One of these is not an actor:
/// ctx.create("dummy", nsi::NodeType::Attributes, &[
///    nsi::strings!("actors", &["Klaus Kinski", "Giorgio Moroder", "Rainer Brandt", "Helge Schneider"]).array_len(2)
/// ]);
/// ```
#[macro_export]
macro_rules! strings {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Strings::new($value)),
        )
    };
}

#[macro_export]
macro_rules! reference {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Reference::new($value)),
        )
    };
}

#[macro_export]
macro_rules! references {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::References::new($value)),
        )
    };
}

#[macro_export]
macro_rules! pointer {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Pointer::new($value)),
        )
    };
}

#[macro_export]
macro_rules! pointers {
    ($name: tt, $value: expr) => {
        nsi::arg::Arg::new(
            $name,
            nsi::arg::ArgData::from(nsi::arg::Pointers::new($value)),
        )
    };
}
