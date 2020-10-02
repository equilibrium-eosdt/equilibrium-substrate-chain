use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};
impl crate::WeightInfo for () {
	fn vest_locked(l: u32, ) -> Weight {
		(24210000 as Weight)
			.saturating_add((0 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(l as Weight)))
			.saturating_add(DbWeight::get().writes(0 as Weight))
			.saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(l as Weight)))
	}
	fn vest_unlocked(l: u32, ) -> Weight {
		(90417000 as Weight)
			.saturating_add((0 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(l as Weight)))
			.saturating_add(DbWeight::get().writes(6 as Weight))
			.saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(l as Weight)))
	}
	fn vest_other_locked(l: u32, ) -> Weight {
		(23787000 as Weight)
			.saturating_add((0 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(l as Weight)))
			.saturating_add(DbWeight::get().writes(0 as Weight))
			.saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(l as Weight)))
	}
	fn vest_other_unlocked(l: u32, ) -> Weight {
		(89240000 as Weight)
			.saturating_add((5000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(8 as Weight))
			.saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(l as Weight)))
			.saturating_add(DbWeight::get().writes(6 as Weight))
			.saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(l as Weight)))
	}
	fn vested_transfer(l: u32, ) -> Weight {
		(133267000 as Weight)
			.saturating_add((0 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(9 as Weight))
			.saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(l as Weight)))
			.saturating_add(DbWeight::get().writes(6 as Weight))
			.saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(l as Weight)))
	}
}
