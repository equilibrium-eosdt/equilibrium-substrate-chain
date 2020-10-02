#![cfg_attr(not(feature = "std"), no_std)]

use eq_primitives::AccountGetter;
use frame_support::{decl_module, decl_storage, traits::Get};
use sp_runtime::{traits::AccountIdConversion, ModuleId};

pub trait Trait<I = DefaultInstance>: system::Trait {
    type ModuleId: Get<ModuleId>;
}

decl_storage! {
    trait Store for Module<T: Trait<I>, I: Instance=DefaultInstance> as EqDistribution {}
}

decl_module! {
    pub struct Module<T: Trait<I>, I: Instance=DefaultInstance> for enum Call where origin: T::Origin {
        const ModuleId: ModuleId = T::ModuleId::get();
    }
}

impl<T: Trait<I>, I: Instance> Module<T, I> {
    pub fn account_id() -> T::AccountId {
        T::ModuleId::get().into_account()
    }
}

impl<T: Trait<I>, I: Instance> AccountGetter<T::AccountId> for Module<T, I> {
    fn get_account_id() -> T::AccountId {
        Self::account_id()
    }
}
