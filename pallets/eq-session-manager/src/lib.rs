#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
	storage::IterableStorageMap, traits::ValidatorRegistration, Parameter,
};
use pallet_session::SessionManager;
use sp_runtime::traits::{MaybeSerializeDeserialize, Member};
use sp_staking::SessionIndex;
use sp_std::prelude::*;
use system as frame_system;
use system::ensure_root;

mod mock;
mod tests;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type ValidatorId: Member + Parameter + MaybeSerializeDeserialize;
	type RegistrationChecker: ValidatorRegistration<Self::ValidatorId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as EqSessionManager {
		pub Validators get(fn validators): map hasher(blake2_128_concat) T::ValidatorId => bool;
		pub IsChanged get(fn is_changed): bool;
	}
	add_extra_genesis {
		config(validators): Vec<T::ValidatorId>;

		build(|config: &GenesisConfig<T>| {
			for &ref validator in config.validators.iter() {
				<Validators<T>>::insert(validator, true);
			}
			IsChanged::put(true);
		});
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T>
	where
		ValidatorId = <T as Trait>::ValidatorId,
	{
		ValidatorAdded(ValidatorId),
		ValidatorRemoved(ValidatorId),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		AlreadyAdded,
		AlreadyRemoved,
		NotRegistred,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// Adds validator. Only root can add validator.
		#[weight = 10_000]
		pub fn add_validator(origin, validator_id: T::ValidatorId) -> DispatchResult
		{
			ensure_root(origin)?;

			let is_registered = T::RegistrationChecker::is_registered(&validator_id);
			ensure!(is_registered, Error::<T>::NotRegistred);

			let validator = <Validators<T>>::get(&validator_id);
			ensure!(!validator, Error::<T>::AlreadyAdded);

			<Validators<T>>::insert(&validator_id, true);

			IsChanged::put(true);

			debug::warn!("Validator {:?} added", validator_id);

			Self::deposit_event(RawEvent::ValidatorAdded(validator_id));

			Ok(())
		}

		/// Removes validator. Only root can remove validator.
		#[weight = 10_000]
		pub fn remove_validator(origin, validator_id: T::ValidatorId) -> DispatchResult
		{
			ensure_root(origin)?;

			let validator = <Validators<T>>::get(&validator_id);
			ensure!(validator, Error::<T>::AlreadyRemoved);

			<Validators<T>>::remove(&validator_id);

			IsChanged::put(true);

			debug::warn!("Validator {:?} removed", validator_id);

			Self::deposit_event(RawEvent::ValidatorRemoved(validator_id));

			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	fn commit() {
		IsChanged::put(false);
	}
}

impl<T: Trait> SessionManager<T::ValidatorId> for Module<T> {
	fn new_session(_: SessionIndex) -> Option<Vec<T::ValidatorId>> {
		let result = if IsChanged::get() {
			Some(<Validators<T>>::iter().map(|(k, _v)| k).collect())
		} else {
			None
		};

		Self::commit();

		result
	}
	fn start_session(_: SessionIndex) {}
	fn end_session(_: SessionIndex) {}
}
