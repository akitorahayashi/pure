use byte_unit::{Byte, UnitType};

/// Format bytes into a human-readable string.
pub fn format_bytes(size: u64) -> String {
    if size == 0 {
        "0 B".to_string()
    } else {
        let adjusted = Byte::from_u64(size).get_appropriate_unit(UnitType::Decimal);
        format!("{adjusted:#.2}")
    }
}
