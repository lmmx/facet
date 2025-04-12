use facet_core::Field;

use crate::ReflectError;

use super::{PokeStructUninit, PokeValueUninit};

/// We're essentially building a graph of slots we're initializing —
/// when we're done initializing something, we need to be able to go back
/// to the parent.
enum Parent<'mem> {
    Slot(Box<Slot<'mem>>),
    StructSlot(Box<StructSlot<'mem>>), // TODO: we can probably avoid box allocations by using arenas 🤷
                                       // also, let's worry about enums later
}

/// The memory location for a struct or enum field.
///
/// Setting it will mark it as initialized, and will allow us to resume access to the parent.
///
/// Maybe that slot is also a struct itself, in which case we'll need to nest deeper.
pub struct Slot<'mem> {
    pub(crate) parent: Parent<'mem>,
    pub(crate) value: PokeValueUninit<'mem>,
    pub(crate) field: Field,
    pub(crate) index: usize,
}

impl<'mem> Slot<'mem> {
    /// Assign this field, get back the parent with the field marked as initialized.
    pub fn set<T>(self, t: T) -> Parent<'mem> {
        self.value.set(t);
        unsafe { self.assume_init() }
    }

    #[inline(always)]
    unsafe fn assume_init(self) -> Parent<'mem> {
        unsafe {
            self.parent.assume_field_init(self.index);
        }
        self.parent
    }

    pub fn into_struct(self) -> Result<StructSlot<'mem>, Self> {
        if let Some(storage) = self.value.into_struct() {
            Ok(StructSlot {
                slot: self,
                storage,
            })
        } else {
            Err(self)
        }
    }
}

/// A partially-initialized struct within a slot
pub struct StructSlot<'mem> {
    /// the thing that we'll need to mark as initialized when we're done.
    slot: Slot<'mem>,

    /// what we're actually initializing
    storage: PokeStructUninit<'mem>,
}

impl<'mem> StructSlot<'mem> {
    pub fn slot_by_name(self, name: &str) -> Option<Slot<'mem>> {
        self.storage.field_by_name(name).map(|field| {
            let index = self.storage.field_index(field);
            let value = unsafe { self.storage.field_uninit(index)? };
            Slot {
                parent: Parent::StructSlot(self),
                value,
                field,
                index,
            }
        })
    }

    /// Finish this struct
    pub fn finish(self) -> Result<Parent<'mem>, ReflectError> {
        self.storage.build_in_place()?;
        unsafe { Ok(self.slot.assume_init()) }
    }
}
