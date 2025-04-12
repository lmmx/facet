use facet_core::{Facet, Opaque, OpaqueConst, OpaqueUninit, OptionDef, OptionVTable, Shape};

use crate::Guard;

/// Allows poking an option (setting Some/None)
pub struct PokeOption<'mem> {
    data: Opaque<'mem>,
    shape: &'static Shape,
    def: OptionDef,
}

impl<'mem> PokeOption<'mem> {
    /// Creates a new option poke
    ///
    /// # Safety
    ///
    /// `data` must be properly aligned and sized for this shape.
    pub(crate) unsafe fn new(data: Opaque<'mem>, shape: &'static Shape, def: OptionDef) -> Self {
        Self { data, shape, def }
    }

    /// Returns the shape of this option
    pub fn shape(&self) -> &'static Shape {
        self.shape
    }

    /// Returns the option definition
    pub fn def(&self) -> OptionDef {
        self.def
    }

    /// Returns the option vtable
    pub fn vtable(&self) -> &'static OptionVTable {
        self.def.vtable
    }

    /// Replace the current value with None
    pub fn replace_with_none(self) -> Self {
        unsafe { (self.vtable().replace_with_fn)(self.data, None) };
        self
    }

    /// Replace the current value with Some
    pub fn replace_with_some<T>(self, value: T) -> Self {
        let value_opaque = OpaqueConst::new(&raw const value);
        core::mem::forget(value);
        self.replace_with_some_opaque(value_opaque)
    }

    /// Replace the current value with some type-erased inner value.
    pub fn replace_with_some_opaque(self, value: OpaqueConst<'mem>) -> Self {
        unsafe { (self.vtable().replace_with_fn)(self.data, Some(value)) };
        self
    }

    /// Get a reference to the underlying value
    #[inline(always)]
    pub fn into_value(self) -> crate::PokeValueUninit<'mem> {
        unsafe {
            crate::PokeValueUninit::new(OpaqueUninit::new(self.data.as_mut_byte_ptr()), self.shape)
        }
    }

    /// Takes ownership of this `PokeOption` and returns the underlying data.
    #[inline]
    pub fn build_in_place(self) -> Opaque<'mem> {
        self.data
    }

    /// Builds an `Option<T>` from the PokeOption, then deallocates the memory
    /// that this PokeOption was pointing to.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The generic type parameter T does not match the shape that this PokeOption is building.
    pub fn build<T: Facet>(self, guard: Option<Guard>) -> Option<T> {
        let mut guard = guard;
        let this = self;
        // this changes drop order: guard must be dropped _after_ this.

        this.shape.assert_type::<Option<T>>();
        if let Some(guard) = &guard {
            guard.shape.assert_type::<Option<T>>();
        }

        let result = unsafe {
            let ptr = this.data.as_ref::<Option<T>>();
            core::ptr::read(ptr)
        };
        guard.take(); // dealloc
        result
    }
}
