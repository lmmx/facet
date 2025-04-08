use facet_opaque::{Opaque, OpaqueConst, OpaqueUninit};

/// Initialize a list in place with a given capacity
///
/// # Safety
///
/// The `list` parameter must point to uninitialized memory of sufficient size.
/// The function must properly initialize the memory.
pub type ListInitInPlaceWithCapacityFn =
    unsafe fn(list: OpaqueUninit, capacity: usize) -> Result<Opaque, ()>;

/// Push an item to the list
///
/// # Safety
///
/// The `list` parameter must point to aligned, initialized memory of the correct type.
/// `item` is moved out of (with [`std::ptr::read`]) — it should be deallocated
/// afterwards but NOT dropped.
pub type ListPushFn = unsafe fn(list: Opaque, item: Opaque);
// FIXME: this forces allocating item separately, copying it, and then dropping it — it's not great.

/// Get the number of items in the list
///
/// # Safety
///
/// The `list` parameter must point to aligned, initialized memory of the correct type.
pub type ListLenFn = unsafe fn(list: OpaqueConst) -> usize;

/// Get pointer to the item at the given index. Panics if out of bounds.
///
/// # Safety
///
/// The `list` parameter must point to aligned, initialized memory of the correct type.
pub type ListGetItemPtrFn = unsafe fn(list: OpaqueConst, index: usize) -> OpaqueConst;

/// Virtual table for a list-like type (like `Vec<T>`,
/// but also `HashSet<T>`, etc.)
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListVTable {
    /// cf. [`ListInitInPlaceWithCapacityFn`]
    pub init_in_place_with_capacity: ListInitInPlaceWithCapacityFn,

    /// cf. [`ListPushFn`]
    pub push: ListPushFn,

    /// cf. [`ListLenFn`]
    pub len: ListLenFn,

    /// cf. [`ListGetItemPtrFn`]
    pub get_item_ptr: ListGetItemPtrFn,
}

impl ListVTable {
    /// Returns a builder for ListVTable
    pub const fn builder() -> ListVTableBuilder {
        ListVTableBuilder::new()
    }
}

/// Builds a [`ListVTable`]
pub struct ListVTableBuilder {
    init_in_place_with_capacity: Option<ListInitInPlaceWithCapacityFn>,
    push: Option<ListPushFn>,
    len: Option<ListLenFn>,
    get_item_ptr: Option<ListGetItemPtrFn>,
}

impl ListVTableBuilder {
    /// Creates a new [`ListVTableBuilder`] with all fields set to `None`.
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            init_in_place_with_capacity: None,
            push: None,
            len: None,
            get_item_ptr: None,
        }
    }

    /// Sets the init_in_place_with_capacity field
    pub const fn init_in_place_with_capacity(mut self, f: ListInitInPlaceWithCapacityFn) -> Self {
        self.init_in_place_with_capacity = Some(f);
        self
    }

    /// Sets the push field
    pub const fn push(mut self, f: ListPushFn) -> Self {
        self.push = Some(f);
        self
    }

    /// Sets the len field
    pub const fn len(mut self, f: ListLenFn) -> Self {
        self.len = Some(f);
        self
    }

    /// Sets the get_item_ptr field
    pub const fn get_item_ptr(mut self, f: ListGetItemPtrFn) -> Self {
        self.get_item_ptr = Some(f);
        self
    }

    /// Builds the [`ListVTable`] from the current state of the builder.
    ///
    /// # Panics
    ///
    /// This method will panic if any of the required fields are `None`.
    pub const fn build(self) -> ListVTable {
        ListVTable {
            init_in_place_with_capacity: self.init_in_place_with_capacity.unwrap(),
            push: self.push.unwrap(),
            len: self.len.unwrap(),
            get_item_ptr: self.get_item_ptr.unwrap(),
        }
    }
}
