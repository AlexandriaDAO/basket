use ic_stable_structures::{
    DefaultMemoryImpl, 
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    StableBTreeMap, Storable,
};
use candid::Principal;
use std::cell::RefCell;
use std::borrow::Cow;

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

// Wrapper for Principal to implement Storable
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StorablePrincipal(pub Principal);

// ===== Storage Implementation =====
// Make StorablePrincipal storable for stable structures
impl Storable for StorablePrincipal {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(self.0.as_slice().to_vec())
    }
    
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        StorablePrincipal(Principal::from_slice(&bytes))
    }
    
    const BOUND: ic_stable_structures::storable::Bound = 
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 29,
            is_fixed_size: false,
        };
}

// Embed the lock canister WASM at compile time
// This requires building lock_canister FIRST, then building factory
pub const LOCK_CANISTER_WASM: &[u8] = include_bytes!("../../../../target/wasm32-unknown-unknown/release/lock_canister.wasm");

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = 
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    
    // Permanent user → lock canister mapping  
    pub static USER_LOCK_CANISTERS: RefCell<StableBTreeMap<StorablePrincipal, StorablePrincipal, Memory>> = 
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        ));
}