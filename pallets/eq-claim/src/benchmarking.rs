#![cfg(feature = "runtime-benchmarks")]
use super::*;
use crate::secp_utils::*;
use crate::Module as Claim;
use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::DispatchResult;

const SEED: u32 = 0;

const MAX_CLAIMS: u32 = 10_000;
const VALUE: u32 = 1_000_000;

fn create_claim<T: Trait>(input: u32) -> DispatchResult {
    let secret_key = secp256k1::SecretKey::parse(&keccak_256(&input.encode())).unwrap();
    let eth_address = eth(&secret_key);
    let vesting = Some((100_000.into(), 1_000.into(), 100.into()));
    super::Module::<T>::mint_claim(
        RawOrigin::Root.into(),
        eth_address,
        VALUE.into(),
        vesting,
        false,
    )?;
    Ok(())
}

fn create_claim_attest<T: Trait>(input: u32) -> DispatchResult {
    let secret_key = secp256k1::SecretKey::parse(&keccak_256(&input.encode())).unwrap();
    let eth_address = eth(&secret_key);
    let vesting = Some((100_000.into(), 1_000.into(), 100.into()));
    super::Module::<T>::mint_claim(
        RawOrigin::Root.into(),
        eth_address,
        VALUE.into(),
        vesting,
        true,
    )?;
    Ok(())
}

benchmarks! {
    _ {
        // Create claims in storage. Two are created at a time!
        let c in 0 .. MAX_CLAIMS / 2 => {
            create_claim::<T>(c)?;
            create_claim_attest::<T>(u32::max_value() - c)?;
        };
    }

    // Benchmark `claim` for different users.
    claim {
        let u in 0 .. 1000;
        let secret_key = secp256k1::SecretKey::parse(&keccak_256(&u.encode())).unwrap();
        let eth_address = eth(&secret_key);
        let account: T::AccountId = account("user", u, SEED);
        let vesting = Some((100_000.into(), 1_000.into(), 100.into()));
        let signature = sig::<T>(&secret_key, &account.encode(), &[][..]);
        super::Module::<T>::mint_claim(RawOrigin::Root.into(), eth_address, VALUE.into(), vesting, false)?;
        assert_eq!(Claims::<T>::get(eth_address), Some(VALUE.into()));
    }: _(RawOrigin::None, account, signature)
    verify {
        assert_eq!(Claims::<T>::get(eth_address), None);
    }

    // Benchmark `mint_claim` when there already exists `c` claims in storage.
    mint_claim {
        let c in ...;
        let eth_address = account("eth_address", c, SEED);
        let vesting = Some((100_000.into(), 1_000.into(), 100.into()));
        let statement = true;
    }: _(RawOrigin::Root, eth_address, VALUE.into(), vesting, statement)
    verify {
        assert_eq!(Claims::<T>::get(eth_address), Some(VALUE.into()));
    }

    // Benchmark `claim_attest` for different users.
    claim_attest {
        let u in 0 .. 1000;
        let attest_u = u32::max_value() - u;
        let secret_key = secp256k1::SecretKey::parse(&keccak_256(&attest_u.encode())).unwrap();
        let eth_address = eth(&secret_key);
        let account: T::AccountId = account("user", u, SEED);
        let vesting = Some((100_000.into(), 1_000.into(), 100.into()));
        let statement = true;
        let signature = sig::<T>(&secret_key, &account.encode(), get_statement_text());
        super::Module::<T>::mint_claim(RawOrigin::Root.into(), eth_address, VALUE.into(), vesting, statement)?;
        assert_eq!(Claims::<T>::get(eth_address), Some(VALUE.into()));
    }: _(RawOrigin::None, account, signature, get_statement_text().to_vec())
    verify {
        assert_eq!(Claims::<T>::get(eth_address), None);
    }

    // Benchmark `attest` for different users.
    attest {
        let u in 0 .. 1000;
        let attest_u = u32::max_value() - u;
        let secret_key = secp256k1::SecretKey::parse(&keccak_256(&attest_u.encode())).unwrap();
        let eth_address = eth(&secret_key);
        let account: T::AccountId = account("user", u, SEED);
        let vesting = Some((100_000.into(), 1_000.into(), 100.into()));
        let statement = true;
        let signature = sig::<T>(&secret_key, &account.encode(), get_statement_text());
        super::Module::<T>::mint_claim(RawOrigin::Root.into(), eth_address, VALUE.into(), vesting, statement)?;
        Preclaims::<T>::insert(&account, eth_address);
        assert_eq!(Claims::<T>::get(eth_address), Some(VALUE.into()));
    }: _(RawOrigin::Signed(account), get_statement_text().to_vec())
    verify {
        assert_eq!(Claims::<T>::get(eth_address), None);
    }

    // Benchmark the time it takes to execute `validate_unsigned` for `claim`
    validate_unsigned_claim {
        let c in ...;
        // Crate signature
        let secret_key = secp256k1::SecretKey::parse(&keccak_256(&c.encode())).unwrap();
        let account: T::AccountId = account("user", c, SEED);
        let signature = sig::<T>(&secret_key, &account.encode(), &[][..]);
        let call = Call::<T>::claim(account, signature);
        let source = sp_runtime::transaction_validity::TransactionSource::External;
    }: {
        super::Module::<T>::validate_unsigned(source, &call)?
    }

    // Benchmark the time it takes to execute `validate_unsigned` for `claim_attest`
    validate_unsigned_claim_attest {
        let c in ...;
        // Crate signature
        let attest_c = u32::max_value() - c;
        let secret_key = secp256k1::SecretKey::parse(&keccak_256(&attest_c.encode())).unwrap();
        let account: T::AccountId = account("user", c, SEED);
        let signature = sig::<T>(&secret_key, &account.encode(), get_statement_text());
        let call = Call::<T>::claim_attest(account, signature, get_statement_text().to_vec());
        let source = sp_runtime::transaction_validity::TransactionSource::External;
    }: {
        super::Module::<T>::validate_unsigned(source, &call)?
    }

    validate_prevalidate_attests {
        let c in ...;
        let attest_c = u32::max_value() - c;
        let secret_key = secp256k1::SecretKey::parse(&keccak_256(&attest_c.encode())).unwrap();
        let eth_address = eth(&secret_key);
        let account: T::AccountId = account("user", c, SEED);
        Preclaims::<T>::insert(&account, eth_address);
        let call = super::Call::attest(get_statement_text().to_vec());
        // We have to copy the validate statement here because of trait issues... :(
        let validate = |who: &T::AccountId, call: &super::Call<T>| -> DispatchResult {
            if let Call::attest(attested_statement) = call {
                let signer = Preclaims::<T>::get(who).ok_or("signer has no claim")?;
                let s = Signing::get(signer);
                if s == true {
                    ensure!(&attested_statement[..] == get_statement_text(), "invalid statement");
                }
            }
            Ok(())
        };
    }: {
        validate(&account, &call)?
    }

    // Benchmark the time it takes to do `repeat` number of keccak256 hashes
    keccak256 {
        let i in 0 .. 10_000;
        let bytes = (i).encode();
    }: {
        for index in 0 .. i {
            let _hash = keccak_256(&bytes);
        }
    }

    // Benchmark the time it takes to do `repeat` number of `eth_recover`
    eth_recover {
        let i in 0 .. 1_000;
        // Crate signature
        let secret_key = secp256k1::SecretKey::parse(&keccak_256(&i.encode())).unwrap();
        let account: T::AccountId = account("user", i, SEED);
        let signature = sig::<T>(&secret_key, &account.encode(), &[][..]);
        let data = account.using_encoded(to_ascii_hex);
        let extra = get_statement_text();
    }: {
        for _ in 0 .. i {
            assert!(super::Module::<T>::eth_recover(&signature, &data, extra).is_some());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claims::tests::{new_test_ext, Test};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        new_test_ext().execute_with(|| {
            assert_ok!(test_benchmark_claim::<Test>());
            assert_ok!(test_benchmark_mint_claim::<Test>());
            assert_ok!(test_benchmark_claim_attest::<Test>());
            assert_ok!(test_benchmark_attest::<Test>());
            assert_ok!(test_benchmark_validate_unsigned_claim::<Test>());
            assert_ok!(test_benchmark_validate_unsigned_claim_attest::<Test>());
            assert_ok!(test_benchmark_validate_prevalidate_attests::<Test>());
            assert_ok!(test_benchmark_keccak256::<Test>());
            assert_ok!(test_benchmark_eth_recover::<Test>());
        });
    }
}
