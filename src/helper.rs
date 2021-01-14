// `@ud` ~1970.1.1
static DA_UNIX_EPOCH: u128 = 170141184475152167957503069145530368000;
// `@ud` ~s1
static DA_SECOND: u128 = 18446744073709551616;

// Convert from Unix time to Urbit `@da` time
pub fn unix_time_to_da(unix_time: u64) -> u128 {
    let time_since_epoch = (unix_time as u128 * DA_SECOND) / 1000;
    DA_UNIX_EPOCH + time_since_epoch
}
