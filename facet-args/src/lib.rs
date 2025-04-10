use facet_poke::Poke;
use facet_trait::Facet;
use facet_trait::OpaqueConst;
use facet_trait::ShapeExt;

pub fn from_slice<T: Facet>(s: &[&str]) -> T {
    log::trace!("Entering from_slice function");
    let mut s = s;
    let (poke, guard) = Poke::alloc::<T>();
    log::trace!("Allocated Poke for type T");
    let ps = poke.into_struct();
    log::trace!("Converted Poke into struct");

    while let Some(token) = s.first() {
        log::trace!("Processing token: {}", token);
        s = &s[1..];

        if let Some(key) = token.strip_prefix("--") {
            log::trace!("Found named argument: {}", key);
            let (_field_index, field) = ps.field_by_name(key).unwrap();
            let field_shape = field.shape();
            log::trace!("Field shape: {:?}", field_shape);
            if field_shape.is_type::<bool>() {
                log::trace!("Boolean field detected, setting to true");
                unsafe { field.into_value().put(OpaqueConst::from_ref(&true)) };
            } else {
                let value = s.first().unwrap();
                log::trace!("Field value: {}", value);
                s = &s[1..];

                let parse = field_shape.vtable.parse.unwrap_or_else(|| {
                    log::trace!("No parse function found for shape {}", field.shape());
                    panic!("shape {} does not support parse", field.shape())
                });
                log::trace!("Parsing field value");
                unsafe { (parse)(value, field.into_value().data()) }.unwrap_or_else(|e| {
                    log::trace!("Failed to parse field: {}", e);
                    panic!(
                        "Failed to parse field '{}' of shape {}: {}",
                        key, field_shape, e
                    )
                });
            }
        } else {
            log::trace!("Encountered positional argument: {}", token);
            // Handle positional argument
        }
    }
    ps.build(Some(guard))
}
