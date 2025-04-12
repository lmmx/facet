use facet_core::{Field, Shape, StructDef};

use super::PokeValue;

/// Allows mutating a fully-initialized struct
pub struct PokeStruct<'mem> {
    /// pointer to the partially-initialized struct
    pub(crate) value: PokeValue<'mem>,

    /// field list, with offsets and shapes
    pub(crate) def: StructDef,
}

impl<'mem> PokeStruct<'mem> {
    /// Shape getter
    #[inline(always)]
    pub fn shape(&self) -> &'static Shape {
        self.value.shape()
    }

    /// Gets the struct definition
    pub fn def(&self) -> StructDef {
        self.def
    }

    /// Coerce back into a value
    #[inline(always)]
    pub fn into_value(self) -> PokeValue<'mem> {
        self.value
    }
}
