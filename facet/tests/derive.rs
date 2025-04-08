use facet::{Def, Facet, FieldFlags, StructDef, StructKind};
use std::{fmt::Debug, mem::offset_of};

#[test]
fn simple_struct() {
    #[derive(Debug, Facet)]
    struct Blah {
        foo: u32,
        bar: String,
    }

    if !cfg!(miri) {
        let shape = Blah::SHAPE;

        // Check the name using Display
        assert_eq!(format!("{}", shape), "Blah");

        assert_eq!(shape.layout.size(), 32);
        assert_eq!(shape.layout.align(), 8);

        if let Def::Struct(StructDef { kind, fields, .. }) = shape.def {
            assert_eq!(kind, StructKind::Struct);
            assert_eq!(fields.len(), 2);

            let foo_field = &fields[0];
            assert_eq!(foo_field.name, "foo");
            assert_eq!(foo_field.shape.layout.size(), 4);
            assert_eq!(foo_field.shape.layout.align(), 4);
            assert_eq!(foo_field.offset, offset_of!(Blah, foo));

            let bar_field = &fields[1];
            assert_eq!(bar_field.name, "bar");
            assert_eq!(bar_field.shape.layout.size(), 24);
            assert_eq!(bar_field.shape.layout.align(), 8);
            assert_eq!(bar_field.offset, offset_of!(Blah, bar));
        } else {
            panic!("Expected Struct innards");
        }
    }
}

#[test]
fn struct_with_sensitive_field() {
    #[derive(Debug, Facet)]
    struct Blah {
        foo: u32,
        #[facet(sensitive)]
        bar: String,
    }

    if !cfg!(miri) {
        let shape = Blah::SHAPE;

        if let Def::Struct(StructDef { fields, .. }) = shape.def {
            let bar_field = &fields[1];
            assert_eq!(bar_field.name, "bar");
            match shape.def {
                Def::Struct(struct_def) => {
                    assert!(!struct_def.fields[0].flags.contains(FieldFlags::SENSITIVE));
                    assert!(struct_def.fields[1].flags.contains(FieldFlags::SENSITIVE));
                }
                _ => panic!("Expected struct"),
            }
        } else {
            panic!("Expected Struct innards");
        }
    }
}

#[test]
fn struct_repr_c() {
    #[derive(Clone, Hash, PartialEq, Eq, ::facet::Facet)]
    #[repr(C)]
    struct Blah {
        foo: u32,
        bar: String,
    }
}

#[test]
fn struct_doc_comment() {
    #[derive(Clone, Hash, PartialEq, Eq, ::facet::Facet)]
    /// yes
    struct Foo {}
}

#[test]
fn struct_field_doc_comment() {
    #[derive(Clone, Hash, PartialEq, Eq, ::facet::Facet)]
    struct Foo {
        /// This field has a doc comment
        bar: u32,
    }
}

#[test]
fn struct_with_pub_field() {
    #[derive(Clone, Hash, PartialEq, Eq, ::facet::Facet)]
    struct Foo {
        /// This is a public field
        pub bar: u32,
    }
}

#[test]
fn tuple_struct_repr_transparent() {
    #[derive(Clone, Hash, PartialEq, Eq, ::facet::Facet)]
    #[repr(transparent)]
    struct Blah(u32);
}

#[test]
fn tuple_struct_doc_comment() {
    #[derive(Clone, Hash, PartialEq, Eq, ::facet::Facet)]
    #[repr(transparent)]
    /// This is a struct for sure
    struct Blah(u32);
}

#[test]
fn tuple_struct_field_doc_comment() {
    #[derive(Clone, Hash, PartialEq, Eq, ::facet::Facet)]
    #[repr(transparent)]
    /// This is a struct for sure
    struct Blah(
        /// and this is a field
        u32,
    );
}

#[test]
fn tuple_struct_with_pub_field() {
    #[derive(Clone, Hash, PartialEq, Eq, ::facet::Facet)]
    #[repr(transparent)]
    /// This is a struct for sure
    struct Blah(
        /// and this is a field
        pub u32,
    );
}

#[test]
fn cfg_attrs() {
    #[derive(Facet)]
    #[cfg_attr(feature = "testfeat", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "testfeat", serde(deny_unknown_fields))]
    pub struct CubConfig {}
}

#[test]
fn cfg_attrs2() {
    #[derive(Facet)]
    #[cfg_attr(feature = "testfeat", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "testfeat", serde(deny_unknown_fields))]
    pub struct CubConfig {
        /// size the disk cache is allowed to use
        #[cfg_attr(feature = "testfeat", serde(skip_serializing))]
        #[cfg_attr(
            feature = "testfeat",
            serde(default = "serde_defaults::default_disk_cache_size")
        )]
        pub disk_cache_size: String,
    }
}

#[test]
fn struct_with_std_string() {
    #[derive(Clone, Hash, PartialEq, Eq, ::facet::Facet)]
    struct FileInfo {
        path: std::string::String,
        size: u64,
    }
}

#[test]
fn derive_real_life_cub_config() {
    #[derive(Facet)]
    #[cfg_attr(feature = "testfeat", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "testfeat", serde(deny_unknown_fields))]
    pub struct CubConfig {
        /// size the disk cache is allowed to use
        #[cfg_attr(feature = "testfeat", serde(skip_serializing))]
        #[cfg_attr(
            feature = "testfeat",
            serde(default = "serde_defaults::default_disk_cache_size")
        )]
        pub disk_cache_size: String,

        /// Listen address without http, something like "127.0.0.1:1111"
        #[cfg_attr(feature = "testfeat", serde(default = "serde_defaults::address"))]
        pub address: std::string::String,

        /// Something like `http://localhost:1118`
        /// or `http://mom.svc.cluster.local:1118`, never
        /// a trailing slash.
        #[cfg_attr(feature = "testfeat", serde(default = "serde_defaults::mom_base_url"))]
        pub mom_base_url: String,

        /// API key used to talk to mom
        #[cfg_attr(feature = "testfeat", serde(default = "serde_defaults::mom_api_key"))]
        pub mom_api_key: String,
    }
}

// #[test]
// fn struct_with_tuple() {
//     #[derive(Debug, ::facet::Facet)]
//     struct TupleContainer {
//         data: (u32, String, bool),
//     }

//     if !cfg!(miri) {
//         let shape = TupleContainer::SHAPE;

//         assert_eq!(format!("{}", shape), "TupleContainer");

//         if let Def::Struct(StructDef { fields }) = shape.def {
//             assert_eq!(fields.len(), 1);

//             let data_field = &fields[0];
//             assert_eq!(data_field.name, "data");

//             // Get the layout from the tuple type itself
//             let tuple_layout = std::alloc::Layout::new::<(u32, String, bool)>();

//             assert_eq!(data_field.shape.layout.size(), tuple_layout.size());
//             assert_eq!(data_field.shape.layout.align(), tuple_layout.align());
//             assert_eq!(data_field.offset, offset_of!(TupleContainer, data));
//         } else {
//             panic!("Expected Struct innards");
//         }
//     }
// }

// #[test]
// fn struct_with_vec() {
//     #[derive(Debug, ::facet::Facet)]
//     struct VecContainer {
//         data: Vec<u32>,
//     }
// }

// // #[test]
// // fn struct_with_generic() {
// //     #[derive(Debug, ::facet::Facet)]
// //     struct Generic<T> {
// //         data: T,
// //     }

// //     let shape = Generic::<u32>::shape();
// //     assert_eq!(format!("{}", shape), "Generic<u32>");
// // }

// // #[test]
// // fn struct_with_lifetime() {
// //     #[derive(Debug, ::facet::Facet)]
// //     struct WithLifetime<'a> {
// //         reference: &'a str,
// //     }

// //     let shape = WithLifetime::shape();
// //     assert_eq!(format!("{}", shape), "WithLifetime");
// // }

// #[test]
// fn tuple_struct() {
//     #[derive(Debug, ::facet::Facet)]
//     struct Point(f32, f32);

//     let shape = Point::SHAPE;
//     assert_eq!(format!("{}", shape), "Point");
//     if let Def::Struct(StructDef { fields }) = shape.def {
//         assert_eq!(fields.len(), 2);
//         assert_eq!(fields[0].name, "0");
//         assert_eq!(fields[1].name, "1");
//     } else {
//         panic!("Expected TupleStruct innards");
//     }
// }

// // #[test]
// // fn unit_struct() {
// //     /// A unit struct with documentation
// //     #[derive(Debug, ::facet::Facet)]
// //     struct Unit;

// //     let shape = Unit::shape();
// //     assert_eq!(format!("{}", shape), "Unit");
// //     assert!(matches!(shape.innards, facet::Innards::Struct { fields } if fields.is_empty()));
// // }

// // // #[test]
// // // fn struct_with_attributes() {
// // //     #[derive(Debug, ::facet::Facet)]
// // //     #[repr(C, packed)]
// // //     struct Packed {
// // //         a: u8,
// // //         b: u32,
// // //     }

// // //     let shape = Packed::shape();
// // //     assert_eq!(shape.layout.size(), 5);
// // //     assert_eq!(shape.layout.align(), 1);
// // // }

// // // #[test]
// // // fn enum_test() {
// // //     #[derive(Debug, ::facet::Facet)]
// // //     enum MyEnum {
// // //         A,
// // //         B(i32),
// // //         C { x: f64, y: f64 },
// // //     }

// // //     let shape = MyEnum::shape();
// // //     assert_eq!(format!("{}", shape), "MyEnum");
// // //     if let facet::Innards::Enum { variants, .. } = shape.innards {
// // //         assert_eq!(variants.len(), 3);
// // //         assert_eq!(variants[0].name, "A");
// // //         assert_eq!(variants[1].name, "B");
// // //         assert_eq!(variants[2].name, "C");
// // //     } else {
// // //         panic!("Expected Enum innards");
// // //     }
// // // }
