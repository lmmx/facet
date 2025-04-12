use facet_core::{Facet, Field, FieldError, OpaqueConst, Shape, StructDef};

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use super::{Guard, ISet, PokeValue, PokeValueUninit};

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Uninitialized / partially-initialized struct
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Allows gradually initializing a struct (setting fields, etc.)
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

    pub(crate) fn assert_all_fields_initialized(&self) -> Result<(), StructBuildError> {
        for (i, field) in self.def.fields.iter().copied().enumerate() {
            if !self.iset.has(i) {
                return Err(StructBuildError::FieldNotInitialized { field });
            }
        }
        Ok(())
    }

    /// Asserts that every field has been initialized and gives a [`PokeStruct`]
    ///
    /// If one of the field was not initialized, all fields will be dropped in place.
    pub fn build_in_place(self) -> Result<PokeStruct<'mem>, StructBuildError> {
        self.assert_all_fields_initialized()?;

        let data = unsafe { self.value.data().assume_init() };

        // prevent field drops when the PokeStruct is dropped
        core::mem::forget(self);

        Ok(PokeStruct {
            def: self.def,
            value: self.value.assume_init(),
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
    pub fn build<T: Facet>(self, guard: Option<Guard>) -> T {
        let mut guard = guard;
        let this = self;
        // this changes drop order: guard must be dropped _after_ this.

        this.assert_all_fields_initialized();
        this.shape().assert_type::<T>();
        if let Some(guard) = &guard {
            guard.shape.assert_type::<T>();
        }

        let result = unsafe {
            let ptr = this.value.data.as_mut_bytes() as *const T;
            core::ptr::read(ptr)
        };
        guard.take(); // dealloc
        core::mem::forget(this);
        result
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
    pub fn field_by_name(
        &self,
        name: &str,
    ) -> Result<(usize, crate::PokeUninit<'mem>), FieldError> {
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
    pub unsafe fn mark_initialized(&mut self, index: usize) {
        self.iset.set(index);
    }
}

impl Drop for PokeStructUninit<'_> {
    fn drop(&mut self) {
        self.def
            .fields
            .iter()
            .enumerate()
            .filter_map(|(i, field)| {
                if self.iset.has(i) {
                    Some((field, field.shape.vtable.drop_in_place?))
                } else {
                    None
                }
            })
            .for_each(|(field, drop_fn)| unsafe {
                drop_fn(self.data.field_init(field.offset));
            });
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Fully-initialized struct
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Allows mutating a fully-initialized struct
pub struct PokeStruct<'mem> {
    /// pointer to the partially-initialized struct
    value: PokeValue<'mem>,

    /// field list, with offsets and shapes
    def: StructDef,
    // no need to track initialized fields — we're not
    // uninitializing some fields
}

impl<'mem> PokeStruct {
    /// Coerce back into a value
    #[inline(always)]
    pub fn into_value(self) -> PokeValue<'mem> {
        self.value
    }

    /// Shape getter
    #[inline(always)]
    pub fn shape(&self) -> &'static Shape {
        self.value.shape()
    }

    /// Gets the struct definition
    pub fn def(&self) -> StructDef {
        self.def
    }
}

pub enum StructBuildError {
    FieldNotInitialized { field: Field },
}
