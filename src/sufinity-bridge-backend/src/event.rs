use std::borrow::Cow;

use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{
    memory_manager::VirtualMemory, storable::Bound, DefaultMemoryImpl, Storable,
};
use serde::Deserialize;

#[derive(CandidType, PartialEq, Deserialize)]
pub struct Event {
    pub timestamp: u64,
    pub tx_digest: String,
    pub from: String,
    pub minter_address: String,
    pub principal_address: String,
    pub value: String,
}

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct KeyName(pub(crate) String);

impl Storable for KeyName {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        self.0.to_bytes()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(String::from_bytes(bytes))
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub struct KeyValue(pub(crate) String);

impl Storable for KeyValue {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        self.0.to_bytes()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(String::from_bytes(bytes))
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for Event {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}
