use core::{fmt, hash::Hash, mem::transmute};

use crate::{
    CmpFn, CmpFnTyped, DebugFn, DebugFnTyped, DisplayFn, DisplayFnTyped, Facet, HashFn,
    HashFnTyped, HasherProxy, MarkerTraits, PartialEqFn, PartialEqFnTyped, PartialOrdFn,
    PartialOrdFnTyped, PointerType, Shape, Type, TypeParam, ValuePointerType, ValueVTable,
};

macro_rules! impl_facet_for_pointer {
    ($variant:ident: $type:ty => $shape:expr => $vtable_builder:expr => $ptr_type:ident, $mutable:expr) => {
        unsafe impl<'a, T: Facet<'a> + ?Sized> Facet<'a> for $type {
            const VTABLE: &'static ValueVTable = &const {
                $vtable_builder
                    .type_name(|f, opts| {
                        if let Some(opts) = opts.for_children() {
                            if stringify!($ptr_type) == "Raw" {
                                if $mutable {
                                    write!(f, "*mut ")?;
                                } else {
                                    write!(f, "*const ")?;
                                }
                            } else {
                                write!(f, "&")?;
                                if $mutable {
                                    write!(f, "mut ")?;
                                }
                            }
                            (T::VTABLE.type_name)(f, opts)
                        } else {
                            if stringify!($ptr_type) == "Raw" {
                                if $mutable {
                                    write!(f, "*mut ⋯")
                                } else {
                                    write!(f, "*const ⋯")
                                }
                            } else {
                                write!(f, "&")?;
                                if $mutable {
                                    write!(f, "mut ⋯")
                                } else {
                                    write!(f, "⋯")
                                }
                            }
                        }
                    })
                    .build()
            };

            const SHAPE: &'static Shape<'static> = &const {
                $shape
                    .type_params(&[TypeParam {
                        name: "T",
                        shape: || T::SHAPE,
                    }])
                    .ty({
                        let is_wide =
                            ::core::mem::size_of::<$type>() != ::core::mem::size_of::<*const ()>();
                        let vpt = ValuePointerType {
                            mutable: $mutable,
                            wide: is_wide,
                            target: || T::SHAPE,
                        };

                        Type::Pointer(PointerType::$ptr_type(vpt))
                    })
                    .build()
            };
        }
    };
}

// *const pointers
impl_facet_for_pointer!(
    Raw: *const T
        => Shape::builder_for_sized::<Self>()
            .inner(|| T::SHAPE)
        => ValueVTable::builder::<Self>()
            .marker_traits(
                MarkerTraits::EQ
                    .union(MarkerTraits::COPY)
                    .union(MarkerTraits::UNPIN),
            )
            .debug(fmt::Debug::fmt)
            .clone_into(|src, dst| unsafe { dst.put(*src) })
            .eq(|left, right| left.cast::<()>().eq(&right.cast::<()>()))
            .partial_ord(|&left, &right| {
                left.cast::<()>().partial_cmp(&right.cast::<()>())
            })
            .ord(|&left, &right| left.cast::<()>().cmp(&right.cast::<()>()))
            .hash(|value, hasher_this, hasher_write_fn| {
                value.hash(&mut unsafe {
                    HasherProxy::new(hasher_this, hasher_write_fn)
                })
            })
        => Raw, false
);

// *mut pointers
impl_facet_for_pointer!(
    Raw: *mut T
        => Shape::builder_for_sized::<Self>()
            .inner(|| T::SHAPE)
        => ValueVTable::builder::<Self>()
            .marker_traits(
                MarkerTraits::EQ
                    .union(MarkerTraits::COPY)
                    .union(MarkerTraits::UNPIN),
            )
            .debug(fmt::Debug::fmt)
            .clone_into(|src, dst| unsafe { dst.put(*src) })
            .eq(|left, right| left.cast::<()>().eq(&right.cast::<()>()))
            .partial_ord(|&left, &right| {
                left.cast::<()>().partial_cmp(&right.cast::<()>())
            })
            .ord(|&left, &right| left.cast::<()>().cmp(&right.cast::<()>()))
            .hash(|value, hasher_this, hasher_write_fn| {
                value.hash(&mut unsafe {
                    HasherProxy::new(hasher_this, hasher_write_fn)
                })
            })
        => Raw, true
);

// &T references
impl_facet_for_pointer!(
    Reference: &'a T
        => Shape::builder_for_sized::<Self>()
        => {
            let mut marker_traits = MarkerTraits::COPY.union(MarkerTraits::UNPIN);
            if T::SHAPE.vtable.marker_traits.contains(MarkerTraits::EQ) {
                marker_traits = marker_traits.union(MarkerTraits::EQ);
            }
            if T::SHAPE.vtable.marker_traits.contains(MarkerTraits::SYNC) {
                marker_traits = marker_traits.union(MarkerTraits::SEND).union(MarkerTraits::SYNC);
            }

            let mut builder = ValueVTable::builder::<Self>()
                .marker_traits(marker_traits)
                .clone_into(|src, dst| unsafe { dst.put(core::ptr::read(src)) });

            // Forward trait methods to the underlying type if it implements them
            if T::VTABLE.debug.is_some() {
                builder = builder.debug(|value, f| {
                    let debug_fn = unsafe { transmute::<DebugFn, DebugFnTyped<T>>(T::VTABLE.debug.unwrap()) };
                    debug_fn(*value, f)
                });
            }

            if T::VTABLE.display.is_some() {
                builder = builder.display(|value, f| {
                    let display_fn = unsafe { transmute::<DisplayFn, DisplayFnTyped<T>>(T::VTABLE.display.unwrap()) };
                    display_fn(*value, f)
                });
            }

            if T::VTABLE.eq.is_some() {
                builder = builder.eq(|a, b| {
                    let eq_fn = unsafe { transmute::<PartialEqFn, PartialEqFnTyped<T>>(T::VTABLE.eq.unwrap()) };
                    eq_fn(*a, *b)
                });
            }

            if T::VTABLE.partial_ord.is_some() {
                builder = builder.partial_ord(|a, b| {
                    let partial_ord_fn = unsafe { transmute::<PartialOrdFn, PartialOrdFnTyped<T>>(T::VTABLE.partial_ord.unwrap()) };
                    partial_ord_fn(*a, *b)
                });
            }

            if T::VTABLE.ord.is_some() {
                builder = builder.ord(|a, b| {
                    let ord_fn = unsafe { transmute::<CmpFn, CmpFnTyped<T>>(T::VTABLE.ord.unwrap()) };
                    ord_fn(*a, *b)
                });
            }

            if T::VTABLE.hash.is_some() {
                builder = builder.hash(|value, hasher_this, hasher_write_fn| {
                    let hash_fn = unsafe { transmute::<HashFn, HashFnTyped<T>>(T::VTABLE.hash.unwrap()) };
                    hash_fn(*value, hasher_this, hasher_write_fn)
                });
            }

            builder
        }
        => Reference, false
);

// &mut T references
impl_facet_for_pointer!(
    Reference: &'a mut T
        => Shape::builder_for_sized::<Self>()
        => {
            let mut marker_traits = MarkerTraits::UNPIN;
            if T::SHAPE.vtable.marker_traits.contains(MarkerTraits::EQ) {
                marker_traits = marker_traits.union(MarkerTraits::EQ);
            }
            if T::SHAPE.vtable.marker_traits.contains(MarkerTraits::SEND) {
                marker_traits = marker_traits.union(MarkerTraits::SEND);
            }
            if T::SHAPE.vtable.marker_traits.contains(MarkerTraits::SYNC) {
                marker_traits = marker_traits.union(MarkerTraits::SYNC);
            }

            let mut builder = ValueVTable::builder::<Self>()
                .marker_traits(marker_traits);

            // Forward trait methods to the underlying type if it implements them
            if T::VTABLE.debug.is_some() {
                builder = builder.debug(|value, f| {
                    let debug_fn = unsafe { transmute::<DebugFn, DebugFnTyped<T>>(T::VTABLE.debug.unwrap()) };
                    debug_fn(*value, f)
                });
            }

            if T::VTABLE.display.is_some() {
                builder = builder.display(|value, f| {
                    let display_fn = unsafe { transmute::<DisplayFn, DisplayFnTyped<T>>(T::VTABLE.display.unwrap()) };
                    display_fn(*value, f)
                });
            }

            if T::VTABLE.eq.is_some() {
                builder = builder.eq(|a, b| {
                    let eq_fn = unsafe { transmute::<PartialEqFn, PartialEqFnTyped<T>>(T::VTABLE.eq.unwrap()) };
                    eq_fn(*a, *b)
                });
            }

            if T::VTABLE.partial_ord.is_some() {
                builder = builder.partial_ord(|a, b| {
                    let partial_ord_fn = unsafe { transmute::<PartialOrdFn, PartialOrdFnTyped<T>>(T::VTABLE.partial_ord.unwrap()) };
                    partial_ord_fn(*a, *b)
                });
            }

            if T::VTABLE.ord.is_some() {
                builder = builder.ord(|a, b| {
                    let ord_fn = unsafe { transmute::<CmpFn, CmpFnTyped<T>>(T::VTABLE.ord.unwrap()) };
                    ord_fn(*a, *b)
                });
            }

            if T::VTABLE.hash.is_some() {
                builder = builder.hash(|value, hasher_this, hasher_write_fn| {
                    let hash_fn = unsafe { transmute::<HashFn, HashFnTyped<T>>(T::VTABLE.hash.unwrap()) };
                    hash_fn(*value, hasher_this, hasher_write_fn)
                });
            }

            builder
        }
        => Reference, true
);
