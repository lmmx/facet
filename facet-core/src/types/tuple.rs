use super::Field;
use crate::ptr::PtrConst;

/// Definition for tuple types
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct TupleDef {
    /// VTable for interacting with the tuple
    pub vtable: &'static TupleVTable,

    /// Elements of the tuple - each potentially of a different type
    pub elements: &'static [Field],
}

impl TupleDef {
    /// Returns a builder for TupleDef
    pub const fn builder() -> TupleDefBuilder {
        TupleDefBuilder::new()
    }

    /// Returns the number of elements in the tuple
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns whether the tuple is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

/// Builder for TupleDef
pub struct TupleDefBuilder {
    vtable: Option<&'static TupleVTable>,
    elements: &'static [Field],
}

impl TupleDefBuilder {
    /// Creates a new TupleDefBuilder
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            vtable: None,
            elements: &[],
        }
    }

    /// Sets the vtable for the TupleDef
    pub const fn vtable(mut self, vtable: &'static TupleVTable) -> Self {
        self.vtable = Some(vtable);
        self
    }

    /// Sets the elements for the TupleDef
    pub const fn elements(mut self, elements: &'static [Field]) -> Self {
        self.elements = elements;
        self
    }

    /// Builds the TupleDef
    pub const fn build(self) -> TupleDef {
        TupleDef {
            vtable: self.vtable.unwrap(),
            elements: self.elements,
        }
    }
}

/// Get pointer to the element at the given index
///
/// # Safety
///
/// The `tuple` parameter must point to aligned, initialized memory of the correct type.
pub type TupleGetElementPtrFn = unsafe fn(tuple: PtrConst, index: usize) -> PtrConst;

/// Virtual table for tuple operations
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[repr(C)]
#[non_exhaustive]
pub struct TupleVTable {
    /// Get a pointer to an element at the given index
    pub get_element_ptr: TupleGetElementPtrFn,
}

impl TupleVTable {
    /// Returns a builder for TupleVTable
    pub const fn builder() -> TupleVTableBuilder {
        TupleVTableBuilder::new()
    }
}

/// Builds a [`TupleVTable`]
pub struct TupleVTableBuilder {
    get_element_ptr: Option<TupleGetElementPtrFn>,
}

impl TupleVTableBuilder {
    /// Creates a new [`TupleVTableBuilder`] with all fields set to `None`.
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            get_element_ptr: None,
        }
    }

    /// Sets the get_element_ptr field
    pub const fn get_element_ptr(mut self, f: TupleGetElementPtrFn) -> Self {
        self.get_element_ptr = Some(f);
        self
    }

    /// Builds the [`TupleVTable`] from the current state of the builder.
    ///
    /// # Panics
    ///
    /// This method will panic if any of the required fields are `None`.
    pub const fn build(self) -> TupleVTable {
        TupleVTable {
            get_element_ptr: self.get_element_ptr.unwrap(),
        }
    }
}
