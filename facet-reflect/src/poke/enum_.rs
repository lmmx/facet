use core::ptr::NonNull;
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use facet_core::{
    EnumDef, EnumRepr, Facet, FieldError, Opaque, OpaqueUninit, Shape, Variant, VariantKind,
};

use crate::Guard;

use super::{ISet, PokeValueUninit};
