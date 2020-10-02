#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::{Currency, ExistenceRequirement, Get, VestingSchedule};
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, ensure, weights::Weight,
};
use frame_system::{ensure_root, ensure_signed};
use sp_runtime::{
    traits::{
        AccountIdConversion, AtLeast32BitUnsigned, Convert, MaybeSerializeDeserialize, Saturating,
        StaticLookup, Zero,
    },
    DispatchResult, ModuleId, RuntimeDebug,
};
use sp_std::fmt::Debug;
use sp_std::prelude::*;

mod benchmarking;
mod benchmarks;
mod mock;
mod tests;
use eq_utils::log::eq_log;

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait WeightInfo {
    fn vest_locked(l: u32) -> Weight;
    fn vest_unlocked(l: u32) -> Weight;
    fn vest_other_locked(l: u32) -> Weight;
    fn vest_other_unlocked(l: u32) -> Weight;
    fn vested_transfer(l: u32) -> Weight;
}

pub trait Trait: frame_system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type ModuleId: Get<ModuleId>;

    /// The currency adapter trait.
    type Currency: Currency<Self::AccountId>;

    /// Convert the block number into a balance.
    type BlockNumberToBalance: Convert<Self::BlockNumber, BalanceOf<Self>>;

    /// The minimum amount transferred to call `vested_transfer`.
    type MinVestedTransfer: Get<BalanceOf<Self>>;

    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;
}

/// Struct to encode the vesting schedule of an individual account.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct VestingInfo<Balance, BlockNumber> {
    /// Locked amount at genesis.
    pub locked: Balance,
    /// Amount that gets unlocked every block after `starting_block`.
    pub per_block: Balance,
    /// Starting block for unlocking(vesting).
    pub starting_block: BlockNumber,
}

impl<Balance: AtLeast32BitUnsigned + Copy, BlockNumber: AtLeast32BitUnsigned + Copy>
    VestingInfo<Balance, BlockNumber>
{
    /// Amount locked at block `n`.
    pub fn locked_at<BlockNumberToBalance: Convert<BlockNumber, Balance>>(
        &self,
        n: BlockNumber,
    ) -> Balance {
        // Number of blocks that count toward vesting
        // Saturating to 0 when n < starting_block
        let vested_block_count = n.saturating_sub(self.starting_block);
        let vested_block_count = BlockNumberToBalance::convert(vested_block_count);
        // Return amount that is still locked in vesting
        let maybe_balance = vested_block_count.checked_mul(&self.per_block);
        if let Some(balance) = maybe_balance {
            self.locked.saturating_sub(balance)
        } else {
            Zero::zero()
        }
    }

    pub fn unlocked_at<BlockNumberToBalance: Convert<BlockNumber, Balance>>(
        &self,
        n: BlockNumber,
    ) -> Balance {
        // Number of blocks that count toward vesting
        // Saturating to 0 when n < starting_block
        let vested_block_count = n.saturating_sub(self.starting_block);
        let vested_block_count = BlockNumberToBalance::convert(vested_block_count);
        // Return amount that is still locked in vesting
        let maybe_balance = vested_block_count.checked_mul(&self.per_block);
        if let Some(balance) = maybe_balance {
            balance.min(self.locked)
        } else {
            self.locked
        }
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Vesting {
        /// Information regarding the vesting of a given account.
        pub Vesting get(fn vesting):
            map hasher(blake2_128_concat) T::AccountId
            => Option<VestingInfo<BalanceOf<T>, T::BlockNumber>>;

        pub Vested get(fn vested):
            map hasher(blake2_128_concat) T::AccountId
            => Option<BalanceOf<T>>;
    }
    add_extra_genesis {
        config(vesting): Vec<(T::AccountId, T::BlockNumber, T::BlockNumber, BalanceOf<T>)>;
        build(|config: &GenesisConfig<T>| {
            use sp_runtime::traits::Saturating;
            // Generate initial vesting configuration
            // * who - Account which we are generating vesting configuration for
            // * begin - Block when the account will start to vest
            // * length - Number of blocks from `begin` until fully vested
            // * liquid - Number of units which can be spent before vesting begins
            for &(ref who, begin, length, liquid) in config.vesting.iter() {
                let balance = T::Currency::free_balance(who);
                assert!(!balance.is_zero(), "Currencies must be initiated before vesting");
                // Total genesis `balance` minus `liquid` equals funds locked for vesting
                let locked = balance.saturating_sub(liquid);
                let length_as_balance = T::BlockNumberToBalance::convert(length);
                let per_block = locked / length_as_balance.max(sp_runtime::traits::One::one());

                Vesting::<T>::insert(who, VestingInfo {
                    locked: locked,
                    per_block: per_block,
                    starting_block: begin
                });
                // let reasons = WithdrawReason::Transfer | WithdrawReason::Reserve;
            }
        })
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>,
    {
        /// The amount vested has been updated. This could indicate more funds are available. The
        /// balance given is the amount which is left unvested (and thus locked).
        /// [account, unvested]
        VestingUpdated(AccountId, Balance),
        /// An [account] has become fully vested. No further vesting can happen.
        VestingCompleted(AccountId),
    }
);

decl_error! {
    /// Error for the vesting module.
    pub enum Error for Module<T: Trait> {
        /// The account given is not vesting.
        NotVesting,
        /// An existing vesting schedule already exists for this account that cannot be clobbered.
        ExistingVestingSchedule,
        /// Amount being transferred is too low to create a vesting schedule.
        AmountLow,
    }
}

decl_module! {
    /// Vesting module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        /// The minimum amount to be transferred to create a new vesting schedule.
        const MinVestedTransfer: BalanceOf<T> = T::MinVestedTransfer::get();

        fn deposit_event() = default;

        /// Unlock any vested funds of the sender account.
        ///
        /// The dispatch origin for this call must be _Signed_ and the sender must have funds still
        /// locked under this module.
        ///
        /// Emits either `VestingCompleted` or `VestingUpdated`.
        ///
        /// # <weight>
        /// - `O(1)`.
        /// - DbWeight: 2 Reads, 2 Writes
        ///     - Reads: Vesting Storage, Balances Locks, [Sender Account]
        ///     - Writes: Vesting Storage, Balances Locks, [Sender Account]
        /// - Benchmark:
        ///     - Unlocked: 48.76 + .048 * l µs (min square analysis)
        ///     - Locked: 44.43 + .284 * l µs (min square analysis)
        /// - Using 50 µs fixed. Assuming less than 50 locks on any user, else we may want factor in number of locks.
        /// # </weight>
        #[weight = T::WeightInfo::vest_locked(20).max(
            T::WeightInfo::vest_unlocked(20))
        ]
        fn vest(origin) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::update_lock(who)
        }

        /// Unlock any vested funds of a `target` account.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// - `target`: The account whose vested funds should be unlocked. Must have funds still
        /// locked under this module.
        ///
        /// Emits either `VestingCompleted` or `VestingUpdated`.
        ///
        /// # <weight>
        /// - `O(1)`.
        /// - DbWeight: 3 Reads, 3 Writes
        ///     - Reads: Vesting Storage, Balances Locks, Target Account
        ///     - Writes: Vesting Storage, Balances Locks, Target Account
        /// - Benchmark:
        ///     - Unlocked: 44.3 + .294 * l µs (min square analysis)
        ///     - Locked: 48.16 + .103 * l µs (min square analysis)
        /// - Using 50 µs fixed. Assuming less than 50 locks on any user, else we may want factor in number of locks.
        /// # </weight>
        #[weight = T::WeightInfo::vest_other_locked(20).max(
            T::WeightInfo::vest_other_unlocked(20))
        ]
        fn vest_other(origin, target: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            ensure_signed(origin)?;
            Self::update_lock(T::Lookup::lookup(target)?)
        }

        /// Create a vested transfer.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// - `target`: The account that should be transferred the vested funds.
        /// - `amount`: The amount of funds to transfer and will be vested.
        /// - `schedule`: The vesting schedule attached to the transfer.
        ///
        /// Emits `VestingCreated`.
        ///
        /// # <weight>
        /// - `O(1)`.
        /// - DbWeight: 3 Reads, 3 Writes
        ///     - Reads: Vesting Storage, Balances Locks, Target Account, [Sender Account]
        ///     - Writes: Vesting Storage, Balances Locks, Target Account, [Sender Account]
        /// - Benchmark: 100.3 + .365 * l µs (min square analysis)
        /// - Using 100 µs fixed. Assuming less than 50 locks on any user, else we may want factor in number of locks.
        /// # </weight>
        #[weight = T::WeightInfo::vested_transfer(20)]
        pub fn vested_transfer(
            origin,
            target: <T::Lookup as StaticLookup>::Source,
            schedule: VestingInfo<BalanceOf<T>, T::BlockNumber>,
        ) -> DispatchResult {
            let transactor = ensure_signed(origin)?;
            ensure!(schedule.locked >= T::MinVestedTransfer::get(), Error::<T>::AmountLow);

            let who = T::Lookup::lookup(target)?;
            ensure!(!Vesting::<T>::contains_key(&who), Error::<T>::ExistingVestingSchedule);

            T::Currency::transfer(&transactor, &Self::account_id(), schedule.locked, ExistenceRequirement::AllowDeath)?;

            Self::add_vesting_schedule(&who, schedule.locked, schedule.per_block, schedule.starting_block)
                .expect("user does not have an existing vesting schedule; q.e.d.");

            Ok(())
        }

        /// Force a vested transfer.
        ///
        /// The dispatch origin for this call must be _Root_.
        ///
        /// - `source`: The account whose funds should be transferred.
        /// - `target`: The account that should be transferred the vested funds.
        /// - `amount`: The amount of funds to transfer and will be vested.
        /// - `schedule`: The vesting schedule attached to the transfer.
        ///
        /// Emits `VestingCreated`.
        ///
        /// # <weight>
        /// - `O(1)`.
        /// - DbWeight: 4 Reads, 4 Writes
        ///     - Reads: Vesting Storage, Balances Locks, Target Account, Source Account
        ///     - Writes: Vesting Storage, Balances Locks, Target Account, Source Account
        /// - Benchmark: 100.3 + .365 * l µs (min square analysis)
        /// - Using 100 µs fixed. Assuming less than 50 locks on any user, else we may want factor in number of locks.
        /// # </weight>
        #[weight = T::WeightInfo::vested_transfer(20)]
        pub fn force_vested_transfer(
            origin,
            source: <T::Lookup as StaticLookup>::Source,
            target: <T::Lookup as StaticLookup>::Source,
            schedule: VestingInfo<BalanceOf<T>, T::BlockNumber>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(schedule.locked >= T::MinVestedTransfer::get(), Error::<T>::AmountLow);

            let target = T::Lookup::lookup(target)?;
            let source = T::Lookup::lookup(source)?;
            ensure!(!Vesting::<T>::contains_key(&target), Error::<T>::ExistingVestingSchedule);

            T::Currency::transfer(&source, &Self::account_id(), schedule.locked, ExistenceRequirement::AllowDeath)?;

            Self::add_vesting_schedule(&target, schedule.locked, schedule.per_block, schedule.starting_block)
                .expect("user does not have an existing vesting schedule; q.e.d.");

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn account_id() -> T::AccountId {
        T::ModuleId::get().into_account()
    }
    /// (Re)set or remove the module's currency lock on `who`'s account in accordance with their
    /// current unvested amount.
    fn update_lock(who: T::AccountId) -> DispatchResult {
        let vesting = Self::vesting(&who).ok_or(Error::<T>::NotVesting)?;
        let now = <frame_system::Module<T>>::block_number();
        let unlocked_now = vesting.unlocked_at::<T::BlockNumberToBalance>(now);
        let vested = Self::vested(&who).unwrap_or(BalanceOf::<T>::zero());
        let to_vest = unlocked_now.saturating_sub(vested);
        eq_log!(
            "vest() who: {:?}, to_vest: {:?}, now: {:?}",
            &who,
            &to_vest,
            &now
        );

        #[allow(unused_must_use)]
        if to_vest > BalanceOf::<T>::zero() {
            T::Currency::transfer(
                &Self::account_id(),
                &who,
                to_vest,
                ExistenceRequirement::KeepAlive,
            );

            if unlocked_now == vesting.locked {
                Vesting::<T>::remove(&who);
                Vested::<T>::remove(&who);
                Self::deposit_event(RawEvent::VestingCompleted(who));
            } else {
                Vested::<T>::insert(&who, unlocked_now);
                Self::deposit_event(RawEvent::VestingUpdated(who, to_vest));
            }
        };
        Ok(())
    }
}

impl<T: Trait> VestingSchedule<T::AccountId> for Module<T>
where
    BalanceOf<T>: MaybeSerializeDeserialize + Debug,
{
    type Moment = T::BlockNumber;
    type Currency = T::Currency;

    /// Get the amount that is currently being vested and cannot be transferred out of this account.
    fn vesting_balance(who: &T::AccountId) -> Option<BalanceOf<T>> {
        if let Some(v) = Self::vesting(who) {
            let now = <frame_system::Module<T>>::block_number();
            let locked_now = v.locked_at::<T::BlockNumberToBalance>(now);
            Some(T::Currency::free_balance(who).min(locked_now))
        } else {
            None
        }
    }

    /// Adds a vesting schedule to a given account.
    ///
    /// If there already exists a vesting schedule for the given account, an `Err` is returned
    /// and nothing is updated.
    ///
    /// On success, a linearly reducing amount of funds will be locked. In order to realise any
    /// reduction of the lock over time as it diminishes, the account owner must use `vest` or
    /// `vest_other`.
    ///
    /// Is a no-op if the amount to be vested is zero.
    fn add_vesting_schedule(
        who: &T::AccountId,
        locked: BalanceOf<T>,
        per_block: BalanceOf<T>,
        starting_block: T::BlockNumber,
    ) -> DispatchResult {
        if locked.is_zero() {
            return Ok(());
        }
        if Vesting::<T>::contains_key(who) {
            Err(Error::<T>::ExistingVestingSchedule)?
        }
        let vesting_schedule = VestingInfo {
            locked,
            per_block,
            starting_block,
        };
        Vesting::<T>::insert(who, vesting_schedule);
        // it can't fail, but even if somehow it did, we don't really care.
        let _ = Self::update_lock(who.clone());
        Ok(())
    }

    /// Remove a vesting schedule for a given account.
    fn remove_vesting_schedule(who: &T::AccountId) {
        Vesting::<T>::remove(who);
        // it can't fail, but even if somehow it did, we don't really care.
        let _ = Self::update_lock(who.clone());
    }
}

impl<T: Trait> eq_primitives::AccountGetter<T::AccountId> for Module<T> {
    fn get_account_id() -> T::AccountId {
        Self::account_id()
    }
}
