#![cfg(test)]

use crate::mock;
use crate::mock::{
    force_new_session, initialize_block, new_test_ext, session_changed, validators,
    ErrorSessionManager, MockSessionKeys, ModuleSessionManager, Origin, Session,
};
use frame_support::assert_err;
use sp_runtime::testing::UintAuthorityId;

fn sorted<T: Clone + Ord>(v: Vec<T>) -> Vec<T> {
    let mut w = v.clone();
    w.sort();
    w
}

fn register_validator(id: u64) {
    Session::set_keys(
        Origin::signed(id),
        MockSessionKeys::from(UintAuthorityId::from(id)),
        vec![],
    )
    .unwrap();
}

#[test]
fn initial_validators() {
    new_test_ext().execute_with(|| {
        let actual = <ModuleSessionManager as pallet_session::SessionManager<u64>>::new_session(0)
            .map(|x| sorted(x));
        let expected = Some(mock::initial_validators());
        assert_eq!(expected, actual);
    });
}

#[test]
fn add_validators() {
    new_test_ext().execute_with(|| {
        <ModuleSessionManager as pallet_session::SessionManager<u64>>::new_session(0);
        register_validator(333);
        register_validator(444);
        ModuleSessionManager::add_validator(system::RawOrigin::Root.into(), 333).unwrap();
        ModuleSessionManager::add_validator(system::RawOrigin::Root.into(), 444).unwrap();
        let actual = <ModuleSessionManager as pallet_session::SessionManager<u64>>::new_session(1)
            .map(|x| sorted(x));
        let expected = Some(vec![111, 222, 333, 444]);
        assert_eq!(expected, actual);
    });
}

#[test]
fn remove_validators() {
    new_test_ext().execute_with(|| {
        <ModuleSessionManager as pallet_session::SessionManager<u64>>::new_session(0);
        ModuleSessionManager::remove_validator(system::RawOrigin::Root.into(), 111).unwrap();
        let actual = <ModuleSessionManager as pallet_session::SessionManager<u64>>::new_session(1)
            .map(|x| sorted(x));
        let expected = Some(vec![222]);
        assert_eq!(expected, actual);
    });
}

#[test]
fn validators_stay_unchanged() {
    new_test_ext().execute_with(|| {
        <ModuleSessionManager as pallet_session::SessionManager<u64>>::new_session(0);
        let actual = <ModuleSessionManager as pallet_session::SessionManager<u64>>::new_session(1)
            .map(|x| sorted(x));
        let expected = None;
        assert_eq!(expected, actual);
    });
}

#[test]
fn several_sessions() {
    new_test_ext().execute_with(|| {
        <ModuleSessionManager as pallet_session::SessionManager<u64>>::new_session(0);

        register_validator(333);
        ModuleSessionManager::add_validator(system::RawOrigin::Root.into(), 333).unwrap();
        let actual = <ModuleSessionManager as pallet_session::SessionManager<u64>>::new_session(1)
            .map(|x| sorted(x));
        let expected = Some(vec![111, 222, 333]);
        assert_eq!(expected, actual);

        let actual = <ModuleSessionManager as pallet_session::SessionManager<u64>>::new_session(2)
            .map(|x| sorted(x));
        let expected = None;
        assert_eq!(expected, actual);

        ModuleSessionManager::remove_validator(system::RawOrigin::Root.into(), 111).unwrap();
        let actual = <ModuleSessionManager as pallet_session::SessionManager<u64>>::new_session(3)
            .map(|x| sorted(x));
        let expected = Some(vec![222, 333]);
        assert_eq!(expected, actual);

        let actual = <ModuleSessionManager as pallet_session::SessionManager<u64>>::new_session(4)
            .map(|x| sorted(x));
        let expected = None;
        assert_eq!(expected, actual);
    });
}

#[test]
fn first_session_no_validators() {
    new_test_ext().execute_with(|| {
        force_new_session();
        initialize_block(1);

        let actual: Vec<u64> = sorted(validators());
        let expected: Vec<u64> = vec![];
        assert_eq!(expected, actual);
    });
}

fn second_session_validators_from_config() {
    new_test_ext().execute_with(|| {
        force_new_session();
        initialize_block(1);

        force_new_session();
        initialize_block(2);

        let actual: Vec<u64> = sorted(validators());
        let expected: Vec<u64> = mock::initial_validators();
        assert_eq!(expected, actual);
    });
}

#[test]
fn session_no_validator_changes() {
    new_test_ext().execute_with(|| {
        force_new_session();
        initialize_block(1);

        force_new_session();
        initialize_block(2);

        force_new_session();
        initialize_block(3);

        let actual: Vec<u64> = sorted(validators());
        let expected: Vec<u64> = mock::initial_validators();
        assert_eq!(expected, actual);
    });
}

#[test]
fn session_validators_added() {
    new_test_ext().execute_with(|| {
        force_new_session();
        initialize_block(1);

        register_validator(333);
        register_validator(444);
        ModuleSessionManager::add_validator(system::RawOrigin::Root.into(), 333).unwrap();
        ModuleSessionManager::add_validator(system::RawOrigin::Root.into(), 444).unwrap();

        force_new_session();
        initialize_block(2);

        let actual: Vec<u64> = sorted(validators());
        let expected: Vec<u64> = mock::initial_validators();
        assert_eq!(expected, actual);

        force_new_session();
        initialize_block(3);

        let actual: Vec<u64> = sorted(validators());
        let expected: Vec<u64> = vec![111, 222, 333, 444];
        assert_eq!(expected, actual);
    });
}

#[test]
fn session_validator_removed() {
    new_test_ext().execute_with(|| {
        force_new_session();
        initialize_block(1);

        ModuleSessionManager::remove_validator(system::RawOrigin::Root.into(), 111).unwrap();

        force_new_session();
        initialize_block(2);

        let actual: Vec<u64> = sorted(validators());
        let expected: Vec<u64> = mock::initial_validators();
        assert_eq!(expected, actual);

        force_new_session();
        initialize_block(3);

        let actual: Vec<u64> = sorted(validators());
        let expected: Vec<u64> = vec![222];
        assert_eq!(expected, actual);
    });
}

#[test]
fn session_several_sessions() {
    new_test_ext().execute_with(|| {
        force_new_session();
        initialize_block(1);
        register_validator(333);
        register_validator(444);
        ModuleSessionManager::add_validator(system::RawOrigin::Root.into(), 333).unwrap();
        ModuleSessionManager::add_validator(system::RawOrigin::Root.into(), 444).unwrap();

        force_new_session();
        initialize_block(2);
        let actual: Vec<u64> = sorted(validators());
        let expected: Vec<u64> = mock::initial_validators();
        assert_eq!(expected, actual);

        force_new_session();
        initialize_block(3);
        let a: Vec<u64> = sorted(validators());
        assert_eq!(vec![111, 222, 333, 444], a);

        force_new_session();
        initialize_block(4);
        let a: Vec<u64> = sorted(validators());
        assert_eq!(vec![111, 222, 333, 444], a);
        ModuleSessionManager::remove_validator(system::RawOrigin::Root.into(), 333).unwrap();

        force_new_session();
        initialize_block(5);
        let a: Vec<u64> = sorted(validators());
        assert_eq!(vec![111, 222, 333, 444], a);
        register_validator(555);
        ModuleSessionManager::add_validator(system::RawOrigin::Root.into(), 555).unwrap();
        ModuleSessionManager::remove_validator(system::RawOrigin::Root.into(), 222).unwrap();

        force_new_session();
        initialize_block(6);
        let a: Vec<u64> = sorted(validators());
        assert_eq!(vec![111, 222, 444], a);

        force_new_session();
        initialize_block(6);
        let a: Vec<u64> = sorted(validators());
        assert_eq!(vec![111, 444, 555], a);

        force_new_session();
        initialize_block(7);
        let a: Vec<u64> = sorted(validators());
        assert_eq!(vec![111, 444, 555], a);
    });
}

#[test]
fn session_existing_validator_added() {
    new_test_ext().execute_with(|| {
        force_new_session();
        initialize_block(1);

        register_validator(333);
        register_validator(444);
        ModuleSessionManager::add_validator(system::RawOrigin::Root.into(), 333).unwrap();
        ModuleSessionManager::add_validator(system::RawOrigin::Root.into(), 444).unwrap();
        let actual = ModuleSessionManager::add_validator(system::RawOrigin::Root.into(), 333);
        let expected = ErrorSessionManager::AlreadyAdded;
        assert_err!(actual, expected);
    });
}

#[test]
fn session_nonexistent_validator_removed() {
    new_test_ext().execute_with(|| {
        force_new_session();
        initialize_block(1);

        let actual = ModuleSessionManager::remove_validator(system::RawOrigin::Root.into(), 999);
        let expected = ErrorSessionManager::AlreadyRemoved;
        assert_err!(actual, expected);
    });
}

#[test]
fn session_change_flag() {
    new_test_ext().execute_with(|| {
        force_new_session();
        initialize_block(1);

        force_new_session();
        initialize_block(2);

        force_new_session();
        initialize_block(3);
        register_validator(333);
        register_validator(444);
        ModuleSessionManager::add_validator(system::RawOrigin::Root.into(), 333).unwrap();
        ModuleSessionManager::add_validator(system::RawOrigin::Root.into(), 444).unwrap();
        assert_eq!(false, session_changed());

        force_new_session();
        initialize_block(4);
        assert_eq!(false, session_changed());

        force_new_session();
        initialize_block(5);
        assert_eq!(true, session_changed());

        force_new_session();
        initialize_block(6);
        assert_eq!(false, session_changed());
    });
}

#[test]
fn session_unregistred_validator_added() {
    new_test_ext().execute_with(|| {
        force_new_session();
        initialize_block(1);

        let actual = ModuleSessionManager::add_validator(system::RawOrigin::Root.into(), 333);
        let expected = ErrorSessionManager::NotRegistred;
    });
}
