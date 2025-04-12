use crate::{ScalarType, peek::Peek};
use facet_core::{Facet, OpaqueConst, OpaqueUninit, Shape, TryFromError, ValueVTable};

use super::PokeValue;

/// Allows initializing/setting a value
pub struct PokeValueUninit<'mem> {
    /// pointer to the value (not initialized, or partially initialized)
    pub(crate) data: OpaqueUninit<'mem>,

    /// shape of the value
    pub(crate) shape: &'static Shape,
}

impl<'mem> PokeValueUninit<'mem> {
    /// Shape getter
    #[inline(always)]
    pub fn shape(&self) -> &'static Shape {
        self.shape
    }

    /// Gets the vtable for the value
    #[inline(always)]
    fn vtable(&self) -> &'static ValueVTable {
        self.shape.vtable
    }

    /// Attempts to convert a value from another type into this one
    ///
    /// Returns `Ok(Opaque)` if the conversion was successful, `Err((Self, TryFromError))` otherwise.
    pub fn try_from<'src>(
        self,
        source: OpaqueConst<'src>,
    ) -> Result<PokeValue<'mem>, (Self, TryFromError)> {
        if let Some(try_from_fn) = self.vtable().try_from {
            match unsafe { try_from_fn(source, self.data) } {
                Ok(data) => Ok(PokeValue {
                    shape: self.shape,
                    data,
                }),
                Err(err) => Err((self, err)),
            }
        } else {
            let shape = self.shape;
            Err((self, TryFromError::Unimplemented(shape)))
        }
    }

    /// Attempts to parse a string into this value
    ///
    /// Returns `Ok(Opaque)` if parsing was successful, `Err(Self)` otherwise.
    pub fn parse(self, s: &str) -> Result<PokeValue<'mem>, Self> {
        if let Some(parse_fn) = self.vtable().parse {
            match unsafe { parse_fn(s, self.data) } {
                Ok(data) => Ok(PokeValue {
                    shape: self.shape,
                    data,
                }),
                Err(_) => Err(self),
            }
        } else {
            Err(self)
        }
    }

    /// Place a value in the space provided. See also [`Self::typed`], which
    /// is panic-free.
    ///
    /// This function places a value of type T into the destination space,
    /// checking that T exactly matches the expected shape.
    pub fn put<'src, T>(self, value: T) -> PokeValue<'mem>
    where
        T: Facet + 'src,
    {
        self.shape.assert_type::<T>();
        PokeValue {
            data: unsafe { self.data.put(value) },
            shape: self.shape,
        }
    }

    /// Attempts to set the value to its default
    ///
    /// Returns `Ok(PokeValue)` if setting to default was successful, `Err(Self)` otherwise.
    pub fn default_in_place(self) -> Result<PokeValue<'mem>, Self> {
        if let Some(default_in_place_fn) = self.vtable().default_in_place {
            Ok(PokeValue {
                data: unsafe { default_in_place_fn(self.data) },
                shape: self.shape,
            })
        } else {
            Err(self)
        }
    }

    /// Attempts to clone `source` into this value
    ///
    /// Returns `Ok(PokeValue)` if cloning was successful, `Err(Self)` otherwise.
    pub fn clone_from<'src>(self, source: Peek<'src>) -> Result<PokeValue<'mem>, Self> {
        if let Some(clone_fn) = self.vtable().clone_into {
            // Safe because the function will initialize our data if it returns Some
            Ok(PokeValue {
                data: unsafe { clone_fn(source.data(), self.data) },
                shape: self.shape,
            })
        } else {
            Err(self)
        }
    }

    /// Tries to identify this value's type as a [`ScalarType`] — returns
    /// `None` if the value isn't a scalar, or is a scalar not listed in [`ScalarType`]
    pub fn scalar_type(&self) -> Option<ScalarType> {
        ScalarType::try_from_shape(self.shape)
    }
}
