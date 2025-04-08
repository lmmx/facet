use crate::{Facet, Shape};

#[doc(hidden)]
pub const fn shape_of<TStruct, TField: Facet>(_f: &dyn Fn(TStruct) -> TField) -> &'static Shape {
    TField::SHAPE
}

#[doc(hidden)]
#[macro_export]
macro_rules! struct_field {
    ($struct:ty, $field:tt) => {
        $crate::Field {
            name: stringify!($field),
            shape: $crate::shape_of(&|s: $struct| s.$field),
            offset: ::std::mem::offset_of!($struct, $field),
            flags: $crate::FieldFlags::EMPTY,
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! struct_fields {
    ($struct:ty, ($($field:tt),*)) => {{
        static FIELDS: &[$crate::Field] = &[ $($crate::struct_field!($struct, $field)),* ];
        FIELDS
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! enum_unit_variant {
    ($enum:ty, $variant:ident) => {
        $crate::Variant {
            name: stringify!($variant),
            discriminant: None,
            kind: $crate::VariantKind::Unit,
        }
    };
    ($enum:ty, $variant:ident, $discriminant:expr) => {
        $crate::Variant {
            name: stringify!($variant),
            discriminant: Some($discriminant),
            kind: $crate::VariantKind::Unit,
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! enum_tuple_variant {
    ($enum:ty, $variant:ident, [$($field_type:ty),*]) => {{
        static FIELDS: &[$crate::Field] = &[
            $(
                $crate::Field {
                    name: concat!("_", stringify!($field_type)),
                    shape_fn: <$field_type>::shape,
                    offset: 0, // Will be calculated at runtime
                    flags: $crate::FieldFlags::EMPTY,
                }
            ),*
        ];

        $crate::Variant {
            name: stringify!($variant),
            discriminant: None,
            kind: $crate::VariantKind::Tuple { fields: FIELDS },
        }
    }};
    ($enum:ty, $variant:ident, [$($field_type:ty),*], $discriminant:expr) => {{
        static FIELDS: &[$crate::Field] = &[
            $(
                $crate::Field {
                    name: concat!("_", stringify!($field_type)),
                    shape_fn: <$field_type>::shape,
                    offset: 0, // Will be calculated at runtime
                    flags: $crate::FieldFlags::EMPTY,
                }
            ),*
        ];

        $crate::Variant {
            name: stringify!($variant),
            discriminant: Some($discriminant),
            kind: $crate::VariantKind::Tuple { fields: FIELDS },
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! enum_struct_variant {
    ($enum:ty, $variant:ident, {$($field:ident: $field_type:ty),*}) => {{
        static FIELDS: &[$crate::Field] = &[
            $(
                $crate::Field {
                    name: stringify!($field),
                    shape_fn: <$field_type>::shape,
                    offset: 0, // Will be calculated at runtime
                    flags: $crate::FieldFlags::EMPTY,
                }
            ),*
        ];

        $crate::Variant {
            name: stringify!($variant),
            discriminant: None,
            kind: $crate::VariantKind::Struct { fields: FIELDS },
        }
    }};
    ($enum:ty, $variant:ident, {$($field:ident: $field_type:ty),*}, $discriminant:expr) => {{
        static FIELDS: &[$crate::Field] = &[
            $(
                $crate::Field {
                    name: stringify!($field),
                    shape_fn: <$field_type>::shape,
                    offset: 0, // Will be calculated at runtime
                    flags: $crate::FieldFlags::EMPTY,
                }
            ),*
        ];

        $crate::Variant {
            name: stringify!($variant),
            discriminant: Some($discriminant),
            kind: $crate::VariantKind::Struct { fields: FIELDS },
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! enum_variants {
    ($enum:ty, [$($variant:expr),*]) => {{
        static VARIANTS: &[$crate::Variant] = &[ $($variant),* ];
        VARIANTS
    }};
}

/// Creates a `ValueVTable` for a given type.
///
/// This macro generates a `ValueVTable` with implementations for various traits
/// (Display, Debug, PartialEq, Eq, PartialOrd, Ord, Hash) if they are implemented for the given type.
///
/// # Arguments
///
/// * `$type_name:ty` - The type for which to create the `ValueVTable`.
/// * `$type_name_fn:expr` - A function that writes the type name to a formatter.
///
/// # Example
///
/// ```
/// use facet_trait::value_vtable;
/// use std::fmt::{self, Formatter};
/// use facet_types::TypeNameOpts;
///
/// let vtable = value_vtable!(String, |f: &mut Formatter<'_>, _opts: TypeNameOpts| write!(f, "String"));
/// ```
///
/// This cannot be used for a generic type because the `impls!` thing depends on type bounds.
/// If you have a generic type, you need to do specialization yourself, like we do for slices,
/// arrays, etc. — essentially, this macro is only useful for 1) scalars, 2) inside a derive macro
#[macro_export]
macro_rules! value_vtable {
    ($type_name:ty, $type_name_fn:expr) => {
        &const {
            let mut builder = $crate::ValueVTable::builder()
                .type_name($type_name_fn)
                .drop_in_place(|data| unsafe { data.drop_in_place::<$type_name>() });

            if $crate::facet_spez::impls!($type_name: std::fmt::Display) {
                builder = builder.display(|data, f| {
                    use $crate::facet_spez::*;
                    (&&Spez(unsafe { data.as_ref::<$type_name>() })).spez_display(f)
                });
            }

            if $crate::facet_spez::impls!($type_name: std::fmt::Debug) {
                builder = builder.debug(|data, f| {
                    use $crate::facet_spez::*;
                    (&&Spez(unsafe { data.as_ref::<$type_name>() })).spez_debug(f)
                });
            }

            if $crate::facet_spez::impls!($type_name: std::default::Default) {
                builder = builder.default_in_place(|target| {
                    use $crate::facet_spez::*;
                    (&&Spez(<$type_name as $crate::Facet>::DUMMY)).spez_default_in_place(target)
                });
            }

            if $crate::facet_spez::impls!($type_name: std::clone::Clone) {
                builder = builder.clone_into(|src, dst| {
                    use $crate::facet_spez::*;
                    (&&Spez(unsafe { src.as_ref::<$type_name>() })).spez_clone_into(dst)
                });
            }

            {
                let mut traits = $crate::MarkerTraits::empty();
                if $crate::facet_spez::impls!($type_name: std::cmp::Eq) {
                    traits = traits.union($crate::MarkerTraits::EQ);
                }
                if $crate::facet_spez::impls!($type_name: std::marker::Send) {
                    traits = traits.union($crate::MarkerTraits::SEND);
                }
                if $crate::facet_spez::impls!($type_name: std::marker::Sync) {
                    traits = traits.union($crate::MarkerTraits::SYNC);
                }
                if $crate::facet_spez::impls!($type_name: std::marker::Copy) {
                    traits = traits.union($crate::MarkerTraits::COPY);
                }
                builder = builder.marker_traits(traits);
            }

            if $crate::facet_spez::impls!($type_name: std::cmp::PartialEq) {
                builder = builder.eq(|left, right| {
                    use $crate::facet_spez::*;
                    (&&Spez(unsafe { left.as_ref::<$type_name>() }))
                        .spez_eq(&&Spez(unsafe { right.as_ref::<$type_name>() }))
                });
            }

            if $crate::facet_spez::impls!($type_name: std::cmp::PartialOrd) {
                builder = builder.partial_ord(|left, right| {
                    use $crate::facet_spez::*;
                    (&&Spez(unsafe { left.as_ref::<$type_name>() }))
                        .spez_partial_cmp(&&Spez(unsafe { right.as_ref::<$type_name>() }))
                });
            }

            if $crate::facet_spez::impls!($type_name: std::cmp::Ord) {
                builder = builder.ord(|left, right| {
                    use $crate::facet_spez::*;
                    (&&Spez(unsafe { left.as_ref::<$type_name>() }))
                        .spez_cmp(&&Spez(unsafe { right.as_ref::<$type_name>() }))
                });
            }

            if $crate::facet_spez::impls!($type_name: std::hash::Hash) {
                builder = builder.hash(|value, hasher_this, hasher_write_fn| {
                    use $crate::facet_spez::*;
                    use $crate::HasherProxy;
                    (&&Spez(unsafe { value.as_ref::<$type_name>() }))
                        .spez_hash(&mut unsafe { HasherProxy::new(hasher_this, hasher_write_fn) })
                });
            }

            builder.build()
        }
    };
}
