use super::PokeValueUninit;

/// Represents an enum before a variant has been selected
pub struct PokeEnumNoVariant<'mem> {
    /// underlying value
    pub(crate) value: PokeValueUninit<'mem>,

    /// definition for this enum
    pub(crate) def: EnumDef,
}

impl<'mem> PokeEnumNoVariant<'mem> {
    /// Shape getter
    #[inline(always)]
    pub fn shape(&self) -> &'static Shape {
        self.value.shape
    }

    /// Enum definition getter
    #[inline(always)]
    pub fn def(&self) -> &EnumDef {
        &self.def
    }

    /// Sets the variant of an enum by name.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - No variant with the given name exists.
    pub fn set_variant_by_name(
        self,
        variant_name: &str,
    ) -> Result<PokeEnumUninit<'mem>, FieldError> {
        let variant_index = self
            .def
            .variants
            .iter()
            .enumerate()
            .find(|(_, v)| v.name == variant_name)
            .map(|(i, _)| i)
            .ok_or(FieldError::NoSuchField)?;

        self.set_variant_by_index(variant_index)
    }

    /// Sets the variant of an enum by index.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The index is out of bounds.
    pub fn set_variant_by_index(
        self,
        variant_index: usize,
    ) -> Result<PokeEnumUninit<'mem>, FieldError> {
        if variant_index >= self.def.variants.len() {
            return Err(FieldError::IndexOutOfBounds);
        }

        // Get the current variant info
        let variant = &self.def.variants[variant_index];

        // Prepare memory for the enum
        unsafe {
            // Zero out the memory first to ensure clean state
            core::ptr::write_bytes(self.data.as_mut_bytes(), 0, self.shape.layout.size());

            // Set up the discriminant (tag)
            // For enums in Rust, the first bytes contain the discriminant
            let discriminant_value = match &variant.discriminant {
                // If we have an explicit discriminant, use it
                Some(discriminant) => *discriminant,
                // Otherwise, use the variant index directly
                None => variant_index as i64,
            };

            // Write the discriminant value based on the representation
            match self.def.repr {
                EnumRepr::U8 => {
                    let tag_ptr = self.data.as_mut_bytes();
                    *tag_ptr = discriminant_value as u8;
                }
                EnumRepr::U16 => {
                    let tag_ptr = self.data.as_mut_bytes() as *mut u16;
                    *tag_ptr = discriminant_value as u16;
                }
                EnumRepr::U32 => {
                    let tag_ptr = self.data.as_mut_bytes() as *mut u32;
                    *tag_ptr = discriminant_value as u32;
                }
                EnumRepr::U64 => {
                    let tag_ptr = self.data.as_mut_bytes() as *mut u64;
                    *tag_ptr = discriminant_value as u64;
                }
                EnumRepr::USize => {
                    let tag_ptr = self.data.as_mut_bytes() as *mut usize;
                    *tag_ptr = discriminant_value as usize;
                }
                EnumRepr::I8 => {
                    let tag_ptr = self.data.as_mut_bytes() as *mut i8;
                    *tag_ptr = discriminant_value as i8;
                }
                EnumRepr::I16 => {
                    let tag_ptr = self.data.as_mut_bytes() as *mut i16;
                    *tag_ptr = discriminant_value as i16;
                }
                EnumRepr::I32 => {
                    let tag_ptr = self.data.as_mut_bytes() as *mut i32;
                    *tag_ptr = discriminant_value as i32;
                }
                EnumRepr::I64 => {
                    let tag_ptr = self.data.as_mut_bytes() as *mut i64;
                    *tag_ptr = discriminant_value;
                }
                EnumRepr::ISize => {
                    let tag_ptr = self.data.as_mut_bytes() as *mut isize;
                    *tag_ptr = discriminant_value as isize;
                }
                _ => {
                    panic!("Unsupported enum representation: {:?}", self.def.repr);
                }
            }
        }

        // Create PokeEnum with the selected variant
        Ok(PokeEnumUninit {
            data: self.data,
            iset: Default::default(),
            shape: self.shape,
            def: self.def,
            selected_variant: variant_index,
        })
    }

    /// Try to find the variant by name.
    pub fn variant_by_name(&self, name: &str) -> Option<&Variant> {
        self.def
            .variants
            .iter()
            .find(|variant| variant.name == name)
    }

    /// Whether the enum has a variant.
    pub fn contains_variant_with_name(&self, name: &str) -> bool {
        self.def.variants.iter().any(|variant| variant.name == name)
    }
}
