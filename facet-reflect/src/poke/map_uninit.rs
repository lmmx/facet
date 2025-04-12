use facet_core::{MapDef, OpaqueUninit, Shape};

use super::{PokeMap, PokeValueUninit};

/// Allows initializing an uninitialized map
pub struct PokeMapUninit<'mem> {
    pub(crate) value: PokeValueUninit<'mem>,
    pub(crate) def: MapDef,
}

impl<'mem> PokeMapUninit<'mem> {
    #[inline(always)]
    /// Shape getter
    pub fn shape(&self) -> &'static Shape {
        self.shape
    }

    /// Initializes the map with an optional size hint
    pub fn init(self, size_hint: Option<usize>) -> Result<PokeMap<'mem>, OpaqueUninit<'mem>> {
        let res = if let Some(capacity) = size_hint {
            let init_in_place_with_capacity = self.def.vtable.init_in_place_with_capacity_fn;
            unsafe { init_in_place_with_capacity(self.data, capacity) }
        } else {
            let pv = unsafe { PokeValueUninit::new(self.data, self.shape) };
            pv.default_in_place().map_err(|_| ())
        };
        let data = res.map_err(|_| self.data)?;
        Ok(unsafe { PokeMap::new(data, self.shape, self.def) })
    }
}
