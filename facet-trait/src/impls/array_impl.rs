use crate::*;
use std::alloc::Layout;

unsafe impl<T> Facet for [T; 1]
where
    T: Facet,
{
    const DUMMY: Self = [T::DUMMY; 1];
    const SHAPE: &'static Shape = &const {
        Shape::builder()
            .layout(Layout::new::<[T; 1]>())
            .vtable(
                &const {
                    ValueVTable {
                        type_name: |f, opts| {
                            if let Some(opts) = opts.for_children() {
                                write!(f, "[")?;
                                (T::SHAPE.vtable.type_name)(f, opts)?;
                                write!(f, "; 1]")
                            } else {
                                write!(f, "[⋯; 1]")
                            }
                        },
                        display: None,
                        debug: if T::SHAPE.vtable.debug.is_some() {
                            Some(|value, f| {
                                let value = unsafe { value.as_ref::<[T; 1]>() };
                                write!(f, "[")?;
                                unsafe {
                                    (T::SHAPE.vtable.debug.unwrap_unchecked())(
                                        OpaqueConst::from_ref(&value[0]),
                                        f,
                                    )?;
                                }
                                write!(f, "]")
                            })
                        } else {
                            None
                        },
                        default_in_place: if T::SHAPE.vtable.default_in_place.is_some() {
                            Some(|target| unsafe {
                                let t_dip = T::SHAPE.vtable.default_in_place.unwrap_unchecked();
                                (t_dip)(target.field_uninit(0))
                            })
                        } else {
                            None
                        },
                        clone_into: if T::SHAPE.vtable.clone_into.is_some() {
                            Some(|src, dst| unsafe {
                                let t_cip = T::SHAPE.vtable.clone_into.unwrap_unchecked();
                                (t_cip)(
                                    OpaqueConst::from_ref(&src.as_ref::<[T; 1]>()[0]),
                                    dst.field_uninit(0),
                                )
                            })
                        } else {
                            None
                        },
                        marker_traits: T::SHAPE.vtable.marker_traits,
                        eq: T::SHAPE.vtable.eq,
                        partial_ord: if T::SHAPE.vtable.partial_ord.is_some() {
                            Some(|a, b| {
                                let a = unsafe { a.as_ref::<[T; 1]>() };
                                let b = unsafe { b.as_ref::<[T; 1]>() };
                                unsafe {
                                    (T::SHAPE.vtable.partial_ord.unwrap_unchecked())(
                                        OpaqueConst::from_ref(&a[0]),
                                        OpaqueConst::from_ref(&b[0]),
                                    )
                                }
                            })
                        } else {
                            None
                        },
                        ord: T::SHAPE.vtable.ord,
                        hash: if T::SHAPE.vtable.hash.is_some() {
                            Some(|value, state, hasher| {
                                let value = unsafe { value.as_ref::<[T; 1]>() };
                                unsafe {
                                    (T::SHAPE.vtable.hash.unwrap_unchecked())(
                                        OpaqueConst::from_ref(&value[0]),
                                        state,
                                        hasher,
                                    )
                                }
                            })
                        } else {
                            None
                        },
                        drop_in_place: Some(|value| unsafe {
                            std::ptr::drop_in_place(value.as_mut::<[T; 1]>());
                        }),
                        parse: None,
                        try_from: None,
                    }
                },
            )
            .def(Def::List(ListDef {
                vtable: &const {
                    ListVTable::builder()
                        .init_in_place_with_capacity(|_, _| Err(()))
                        .push(|_, _| {
                            panic!("Cannot push to [T; 1]");
                        })
                        .len(|_| 1)
                        .get_item_ptr(|ptr, index| unsafe {
                            if index >= 1 {
                                panic!(
                                    "Index out of bounds: the len is 1 but the index is {index}"
                                );
                            }
                            OpaqueConst::new_unchecked(ptr.as_ptr::<[T; 1]>())
                        })
                        .build()
                },
                t: T::SHAPE,
            }))
            .build()
    };
}
