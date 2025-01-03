extern crate hashmap;
use hashmap::*;


fn main() {
    // type inference lets us omit an explicit type signature (which
    // would be `Hashmap<&str, u8>` in this example).
    let mut player_stats = Hashmap::new();

    fn random_stat_buff() -> u8 {
        // could actually return some random value here - let's just return
        // some fixed value for now
        42
    }

    // insert a key only if it doesn't already exist
    player_stats.entry("health").or_insert(100);

    // update a key, guarding against the key possibly not being set
    let stat = player_stats.entry("attack").or_insert(100);
    *stat += random_stat_buff();

}
