use std::time::{SystemTime, UNIX_EPOCH};

// `@ud` ~1970.1.1
static DA_UNIX_EPOCH: u128 = 170141184475152167957503069145530368000;
// `@ud` ~s1
static DA_SECOND: u128 = 18446744073709551616;

/// Convert from Unix time in milliseconds to Urbit `@da` time
pub fn unix_time_to_da(unix_time: u64) -> u128 {
    let time_since_epoch = (unix_time as u128 * DA_SECOND) / 1000;
    DA_UNIX_EPOCH + time_since_epoch
}

/// Acquire the current time as u64
pub fn get_current_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Acquire the current time in Urbit `@da` encoding
pub fn get_current_da_time() -> u128 {
    let unix_time = get_current_time();
    unix_time_to_da(unix_time)
}

/// Encode an index path into urbit ud format
/// /12345678901234/1/10987654321 -> /12.345.678.901.234/1/10.987.654.321
pub fn index_dec_to_ud(index: &str) -> String {
    // Split the index
    let index_split: Vec<&str> = index.split("/").collect();
    let mut udindex = String::new();
    // Handle each segment
    for i in 0..index_split.len() {
        if index_split[i].len() > 0 {
            let mut rev: String = index_split[i].chars().rev().collect();
            let mut out = String::new();
            while rev.len() >= 3 {
                let chunk: String = rev.drain(..3).collect();
                out += &chunk;
                if rev.len() > 0 {
                    out += ".";
                }
            }
            out += &rev;
            let seg: String = out.chars().rev().collect();
            udindex += &format!("/{}", &seg);
        }
    }
    udindex
}
