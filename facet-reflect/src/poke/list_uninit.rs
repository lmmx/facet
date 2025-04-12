/// Allows initializing an uninitialized list
pub struct PokeListUninit<'mem> {
    pub(crate) value: PokeValueUninit<'mem>,
    pub(crate) def: ListDef,
}

impl<'mem> PokeListUninit<'mem> {
    #[inline(always)]
    /// Coerce back into a `PokeValue`
    pub fn into_value(self) -> PokeValueUninit<'mem> {
        unsafe { PokeValueUninit::new(self.data, self.shape) }
    }

    #[inline(always)]
    /// Shape getter
    pub fn shape(&self) -> &'static Shape {
        self.shape
    }

    /// Initializes the list with an optional size hint
    pub fn init(self, size_hint: Option<usize>) -> Result<PokeList<'mem>, OpaqueUninit<'mem>> {
        let res = if let Some(capacity) = size_hint {
            let init_in_place_with_capacity = self.def.vtable.init_in_place_with_capacity;
            unsafe { init_in_place_with_capacity(self.data, capacity) }
        } else {
            let pv = unsafe { PokeValueUninit::new(self.data, self.shape) };
            pv.default_in_place().map_err(|_| ())
        };
        let data = res.map_err(|_| self.data)?;
        Ok(unsafe { PokeList::new(data, self.shape, self.def) })
    }
}
