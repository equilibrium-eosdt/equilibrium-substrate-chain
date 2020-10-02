use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};
impl crate::WeightInfo for () {
    fn transfer(b: u32) -> Weight {
        (94651000 as Weight)
            .saturating_add((0 as Weight).saturating_mul(b as Weight))
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(b as Weight)))
            .saturating_add(DbWeight::get().writes(5 as Weight))
            .saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(b as Weight)))
    }
    fn deposit(b: u32) -> Weight {
        (42443000 as Weight)
            .saturating_add((0 as Weight).saturating_mul(b as Weight))
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(b as Weight)))
            .saturating_add(DbWeight::get().writes(2 as Weight))
            .saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(b as Weight)))
    }
    fn burn(b: u32) -> Weight {
        (42414000 as Weight)
            .saturating_add((0 as Weight).saturating_mul(b as Weight))
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(b as Weight)))
            .saturating_add(DbWeight::get().writes(2 as Weight))
            .saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(b as Weight)))
    }
}
