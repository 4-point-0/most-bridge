use candid::CandidType;
use ic_stable_structures::{
    memory_manager::VirtualMemory, storable::Bound, DefaultMemoryImpl, Storable,
};
use serde::Deserialize;

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

#[derive(CandidType, Deserialize)]
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
