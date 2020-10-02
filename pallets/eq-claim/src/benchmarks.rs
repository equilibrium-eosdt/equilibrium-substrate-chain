use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};
impl crate::WeightInfo for () {
	fn claim(u: u32, ) -> Weight {
		(805531000 as Weight)
			.saturating_add((0 as Weight).saturating_mul(u as Weight))
			.saturating_add(DbWeight::get().reads(27 as Weight))
			.saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(u as Weight)))
			.saturating_add(DbWeight::get().writes(10 as Weight))
			.saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(u as Weight)))
	}
	fn mint_claim(c: u32, ) -> Weight {
		(18042000 as Weight)
			.saturating_add((0 as Weight).saturating_mul(c as Weight))
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(c as Weight)))
			.saturating_add(DbWeight::get().writes(4 as Weight))
			.saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(c as Weight)))
	}
	fn claim_attest(u: u32, ) -> Weight {
		(459289000 as Weight)
			.saturating_add((18000 as Weight).saturating_mul(u as Weight))
			.saturating_add(DbWeight::get().reads(27 as Weight))
			.saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(u as Weight)))
			.saturating_add(DbWeight::get().writes(10 as Weight))
			.saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(u as Weight)))
	}
	fn attest(u: u32, ) -> Weight {
		(290295000 as Weight)
			.saturating_add((1000 as Weight).saturating_mul(u as Weight))
			.saturating_add(DbWeight::get().reads(28 as Weight))
			.saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(u as Weight)))
			.saturating_add(DbWeight::get().writes(11 as Weight))
			.saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(u as Weight)))
	}
	fn validate_unsigned_claim(c: u32, ) -> Weight {
		(194116000 as Weight)
			.saturating_add((0 as Weight).saturating_mul(c as Weight))
			.saturating_add(DbWeight::get().reads(2 as Weight))
			.saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(c as Weight)))
			.saturating_add(DbWeight::get().writes(0 as Weight))
			.saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(c as Weight)))
	}
	fn validate_unsigned_claim_attest(c: u32, ) -> Weight {
		(195930000 as Weight)
			.saturating_add((0 as Weight).saturating_mul(c as Weight))
			.saturating_add(DbWeight::get().reads(2 as Weight))
			.saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(c as Weight)))
			.saturating_add(DbWeight::get().writes(0 as Weight))
			.saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(c as Weight)))
	}
	fn validate_prevalidate_attests(c: u32, ) -> Weight {
		(13604000 as Weight)
			.saturating_add((0 as Weight).saturating_mul(c as Weight))
			.saturating_add(DbWeight::get().reads(2 as Weight))
			.saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(c as Weight)))
			.saturating_add(DbWeight::get().writes(0 as Weight))
			.saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(c as Weight)))
	}
	fn keccak256(i: u32, ) -> Weight {
		(57946000 as Weight)
			.saturating_add((895000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(0 as Weight))
			.saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(i as Weight)))
			.saturating_add(DbWeight::get().writes(0 as Weight))
			.saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(i as Weight)))
	}
	fn eth_recover(i: u32, ) -> Weight {
		(0 as Weight)
			.saturating_add((193474000 as Weight).saturating_mul(i as Weight))
			.saturating_add(DbWeight::get().reads(0 as Weight))
			.saturating_add(DbWeight::get().reads((0 as Weight).saturating_mul(i as Weight)))
			.saturating_add(DbWeight::get().writes(0 as Weight))
			.saturating_add(DbWeight::get().writes((0 as Weight).saturating_mul(i as Weight)))
	}
}
