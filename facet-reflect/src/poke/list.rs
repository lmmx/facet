use crate::PokeValueUninit;
use facet_core::{ListDef, ListVTable, Opaque, OpaqueConst, OpaqueUninit, Shape};

/// Allows poking a list (appending, etc.)
pub struct PokeList<'mem> {
    data: Opaque<'mem>,
    #[allow(dead_code)]
    shape: &'static Shape,
    def: ListDef,
}

impl<'mem> PokeList<'mem> {
    /// Creates a new list write-proxy
    ///
    /// # Safety
    ///
    /// The data buffer must match the size and alignment of the shape.
    pub(crate) unsafe fn new(data: Opaque<'mem>, shape: &'static Shape, def: ListDef) -> Self {
        Self { data, shape, def }
    }

    #[inline(always)]
    /// Shape getter
    pub fn shape(&self) -> &'static Shape {
        self.shape
    }

    /// Gets the vtable for the list
    #[inline(always)]
    fn list_vtable(&self) -> &'static ListVTable {
        self.def.vtable
    }

    /// Pushes an item to the list
    ///
    /// # Safety
    ///
    /// `item` is moved out of (with [`core::ptr::read`]) — it should be deallocated
    /// afterwards but NOT dropped.
    pub unsafe fn push(&mut self, item: Opaque<'_>) {
        unsafe { (self.list_vtable().push)(self.data, item) }
    }

    /// Gets the number of items in the list
    pub fn len(&self) -> usize {
        unsafe { (self.list_vtable().len)(self.data.as_const()) }
    }

    /// Returns true if the list is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets a pointer to the item at the given index
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn get_item_ptr(&self, index: usize) -> OpaqueConst {
        unsafe { (self.list_vtable().get_item_ptr)(self.data.as_const(), index) }
    }

    /// Takes ownership of this `PokeList` and returns the underlying data.
    pub fn build_in_place(self) -> Opaque<'mem> {
        self.data
    }

    /// Gets the def for that list
    pub fn def(&self) -> &ListDef {
        &self.def
    }
}
