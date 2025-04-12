use facet_core::Field;

use super::{PokeEnumUninit, PokeStructUninit, PokeValueUninit};

trait StructLike {
    /// Mark a field as initialized
    unsafe fn assume_field_init(&mut self, index: usize);
}

impl StructLike for PokeStructUninit<'_> {
    unsafe fn assume_field_init(&mut self, index: usize) {
        unsafe {
            self.assume_field_init(index);
        }
    }
}

impl StructLike for PokeEnumUninit<'_> {
    unsafe fn assume_field_init(&mut self, index: usize) {
        unsafe {
            self.storage.assume_field_init(index);
        }
    }
}

/// A slot that can be assigned to
pub struct Slot<'mem, Parent>
where
    Parent: StructLike,
{
    pub(crate) parent: Parent,
    pub(crate) value: PokeValueUninit<'mem>,
    pub(crate) field: Field,
    pub(crate) index: usize,
}

impl<'mem, Parent> Slot<'mem, Parent>
where
    Parent: StructLike,
{
    /// Assign this field, get back the parent with the field marked as initialized.
    pub fn set<T>(self, t: T) -> Parent {
        self.value.set(t);
        unsafe {
            self.parent.assume_field_init(self.index);
        }
        self.parent
    }

    pub fn into_struct(self) -> SlottedStructUninit<'mem, Parent> {
        SlottedStructUninit { parent: self }
    }
}

/// A partially-initialized struct within a slot
pub struct SlottedStructUninit<'mem, Parent>
where
    Parent: StructLike,
{
    parent: Slot<'mem, Parent>,
}
