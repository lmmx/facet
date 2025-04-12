use facet_core::{Facet, FieldError, Shape, StructDef};

use crate::ReflectError;

use super::{Guard, ISet, PokeStruct, PokeValue, PokeValueUninit};

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// Allows gradually initializing a struct (setting fields, etc.)
///
/// This also works for tuples, and tuple structs.
pub struct PokeStructUninit<'mem> {
    /// underlying value
    pub(crate) value: PokeValueUninit<'mem>,

    /// fields' shape, etc.
    pub(crate) def: StructDef,

    /// tracks initialized fields
    pub(crate) iset: ISet,
}

impl<'mem> PokeStructUninit<'mem> {
    /// Shape getter
    #[inline(always)]
    pub fn shape(&self) -> &'static Shape {
        self.value.shape()
    }

    /// Gets the struct definition
    #[inline(always)]
    pub fn def(&self) -> StructDef {
        self.def
    }

    pub(crate) fn assert_all_fields_initialized(&self) -> Result<(), ReflectError> {
        for (i, field) in self.def.fields.iter().copied().enumerate() {
            if !self.iset.has(i) {
                return Err(ReflectError::PartiallyInitialized { field });
            }
        }
        Ok(())
    }

    /// Asserts that every field has been initialized and gives a [`PokeStruct`]
    ///
    /// If one of the field was not initialized, all fields will be dropped in place.
    pub fn build_in_place(self) -> Result<PokeStruct<'mem>, ReflectError> {
        self.assert_all_fields_initialized()?;

        let data = unsafe { self.value.data.assume_init() };
        let shape = self.value.shape;
        let def = self.def;

        // prevent field drops
        core::mem::forget(self);

        Ok(PokeStruct {
            def,
            value: PokeValue { data, shape },
        })
    }

    /// Builds a value of type `T` from the PokeStruct, then deallocates the memory
    /// that this PokeStruct was pointing to.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - Not all the fields have been initialized.
    /// - The generic type parameter T does not match the shape that this PokeStruct is building.
    pub fn build<T: Facet>(self, guard: Option<Guard>) -> Result<T, ReflectError> {
        // change drop order: we want to drop guard _after_ this.
        let (mut guard, this) = (guard, self);

        this.shape().assert_type::<T>();
        if let Some(guard) = &guard {
            guard.shape.assert_type::<T>();
        }

        let ps = this.build_in_place()?;
        let t = unsafe { ps.value.data.read::<T>() };
        ps.value.data();
        Ok(t)
    }

    /// Build that PokeStruct into a boxed completed shape.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - Not all the fields have been initialized.
    /// - The generic type parameter T does not match the shape that this PokeStruct is building.
    #[cfg(feature = "alloc")]
    pub fn build_boxed<T: Facet>(self) -> Box<T> {
        self.assert_all_fields_initialized();
        self.shape().assert_type::<T>();

        let boxed = unsafe { Box::from_raw(self.value.data.as_mut_bytes() as *mut T) };
        core::mem::forget(self);
        boxed
    }

    /// Gets a field, by name
    pub fn field_by_name(&self, name: &str) -> Result<(usize, PokeValueUninit<'mem>), FieldError> {
        let index = self
            .def
            .fields
            .iter()
            .position(|f| f.name == name)
            .ok_or(FieldError::NoSuchField)?;
        Ok((index, self.field(index)?))
    }

    /// Get a field writer for a field by index.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The shape doesn't represent a struct.
    /// - The index is out of bounds.
    pub fn field(&self, index: usize) -> Result<crate::PokeUninit<'mem>, FieldError> {
        if index >= self.def.fields.len() {
            return Err(FieldError::IndexOutOfBounds);
        }

        let field = &self.def.fields[index];

        // Get the field's address
        let field_addr = unsafe { self.value.data.field_uninit(field.offset) };
        let field_shape = field.shape;

        let poke = unsafe { crate::PokeUninit::unchecked_new(field_addr, field_shape) };
        Ok(poke)
    }

    unsafe fn unchecked_set(&mut self, index: usize, value: OpaqueConst) -> Result<(), FieldError> {
        if index >= self.def.fields.len() {
            return Err(FieldError::IndexOutOfBounds);
        }
        let field = &self.def.fields[index];
        let field_shape = field.shape;

        unsafe {
            core::ptr::copy_nonoverlapping(
                value.as_ptr(),
                self.value.data.field_uninit(field.offset).as_mut_bytes(),
                field_shape.layout.size(),
            );
            self.iset.set(index);
        }

        Ok(())
    }

    /// Sets a field's value by its index in a type-safe manner.
    ///
    /// This method takes ownership of the value and ensures proper memory management.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The index is out of bounds
    /// - The field shapes don't match
    pub fn set<T: Facet>(&mut self, index: usize, value: T) -> Result<(), FieldError> {
        let field_shape = self
            .def
            .fields
            .get(index)
            .ok_or(FieldError::IndexOutOfBounds)?
            .shape;
        if !field_shape.is_type::<T>() {
            return Err(FieldError::TypeMismatch {
                expected: field_shape,
                actual: T::SHAPE,
            });
        }

        unsafe {
            let opaque = OpaqueConst::new(&value);
            let result = self.unchecked_set(index, opaque);
            if result.is_ok() {
                core::mem::forget(value);
            }
            result
        }
    }

    /// Sets a field's value by its name in a type-safe manner.
    ///
    /// This method takes ownership of the value and ensures proper memory management.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The field name doesn't exist
    /// - The field shapes don't match
    pub fn set_by_name<T: Facet>(&mut self, name: &str, value: T) -> Result<(), FieldError> {
        let index = self
            .def
            .fields
            .iter()
            .position(|f| f.name == name)
            .ok_or(FieldError::NoSuchField)?;

        self.set(index, value)
    }

    /// Marks a field as initialized.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the field is initialized. Only call this after writing to
    /// an address gotten through [`Self::field`] or [`Self::field_by_name`].
    pub unsafe fn assume_field_init(&mut self, index: usize) {
        // TODO: retire — use `Slot` system instead
        self.iset.set(index);
    }
}

impl Drop for PokeStructUninit<'_> {
    fn drop(&mut self) {
        for (i, field) in self.def.fields.iter().enumerate() {
            // for every set field...
            if self.iset.has(i) {
                // that has a drop function...
                if let Some(drop_fn) = field.shape.vtable.drop_in_place {
                    unsafe {
                        // call it
                        drop_fn(self.value.data.field_init(field.offset));
                    }
                }
            }
        }
    }
}
