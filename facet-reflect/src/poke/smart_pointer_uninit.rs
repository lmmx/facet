/// Allows initializing an uninitialized option
pub struct PokeSmartPointerUninit<'mem> {
    data: OpaqueUninit<'mem>,
    shape: &'static Shape,
    def: SmartPointerDef,
}

impl<'mem> PokeSmartPointerUninit<'mem> {
    /// Creates a new uninitialized smart pointer poke
    ///
    /// # Safety
    ///
    /// `data` must be properly aligned and sized for this shape.
    pub(crate) unsafe fn new(
        data: OpaqueUninit<'mem>,
        shape: &'static Shape,
        def: SmartPointerDef,
    ) -> Self {
        Self { data, shape, def }
    }

    /// Returns the shape for this smart pointer.
    pub fn shape(&self) -> &'static Shape {
        self.shape
    }

    /// Returns the smart pointer definition.
    pub fn def(&self) -> &SmartPointerDef {
        &self.def
    }

    /// Returns the smart pointer vtable
    pub fn vtable(&self) -> &'static SmartPointerVTable {
        self.def.vtable
    }

    /// Get a reference to the underlying PokeValue
    #[inline(always)]
    pub fn into_value(self) -> crate::PokeValueUninit<'mem> {
        unsafe { crate::PokeValueUninit::new(self.data, self.shape) }
    }

    /// Creates a new smart pointer around a given T
    ///
    /// Returns `None` if the smart pointer cannot be created directly
    /// (like for weak pointers).
    pub fn from_t<T>(self, value: T) -> Option<PokeSmartPointer<'mem>> {
        let into_fn = self.def.vtable.new_into_fn?;

        let value_opaque = OpaqueConst::new(&raw const value);
        let opaque = unsafe { into_fn(self.data, value_opaque) };
        core::mem::forget(value);
        Some(PokeSmartPointer {
            data: opaque,
            shape: self.shape,
            def: self.def,
        })
    }

    /// Creates a new smart pointer from an existing [`PeekValue`].
    ///
    /// Note: The `PeekValue` is moved out of (consumed) during this operation.
    /// It must be deallocated by the caller on success.
    ///
    /// Returns `None` if the smart pointer cannot be created directly
    /// (like for weak pointers).
    pub fn from_peek_value(self, value: PeekValue<'mem>) -> Option<PokeSmartPointer<'mem>> {
        // Assert that the value's shape matches the expected inner type
        assert_eq!(
            value.shape(),
            self.def.t,
            "Inner value shape does not match expected smart pointer inner type"
        );

        let into_fn = self.def.vtable.new_into_fn?;

        let opaque = unsafe { into_fn(self.data, value.data()) };
        Some(PokeSmartPointer {
            data: opaque,
            shape: self.shape,
            def: self.def,
        })
    }
}
