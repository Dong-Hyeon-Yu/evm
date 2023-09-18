use core::str::FromStr;
use std::collections::BTreeMap;
use ethers_core::types::{H160, U256, Address};
use crate::{backend::{MemoryBackend, Apply, Log, ApplyBackend, Backend, MemoryVicinity, MemoryAccount}, executor::stack::{PrecompileFn, StackExecutor, MemoryStackState, StackSubstateMetadata}, Config};

const DEFAULT_TX_GAS_LIMIT: u64 = u64::MAX;
pub const CONTRACT_BYTECODE: &str = include_str!("contracts/SmallBank.bin");

pub struct ExecutionResult {
    pub logs: Vec<Log>,
    pub effects: Vec::<Apply>,
}

pub trait ExecutionBackend {

    fn config(&self) -> &Config;

    fn precompiles(&self) -> &BTreeMap<H160, PrecompileFn>;

    fn code(&self, address: Address) -> Vec<u8>;

    fn apply_local_effect(&mut self, effect: Vec<Apply>, log: Vec<Log>);

    fn backend(&self) -> &MemoryBackend;
}

/// This storage is used for evm global state.
#[derive(Clone)]
pub struct MemoryStorage {
    pub backend: MemoryBackend,  //TODO: change to MutexTable for concurrent execution.
    precompiles: BTreeMap<H160, PrecompileFn>,
    config: Config,
    // checkpoint:  ArcSwap<BTreeMap<H160, MemoryAccount>>?
    // mutex_table: MutexTable<TransactionDigest>, // TODO MutexTable for transaction locks (prevent concurrent execution of same transaction)
}

impl MemoryStorage {
    pub fn new(backend: MemoryBackend, precompiles: BTreeMap<H160, PrecompileFn>) -> Self {
        
        let config = Config::istanbul();

        Self { 
            backend,
            precompiles,
            config
        }
    }

    pub fn default() -> Self {
        let vicinity = MemoryVicinity { 
            gas_price: U256::zero(), 
            origin: H160::default(), 
            chain_id: U256::one(), 
            block_hashes: Vec::new(), 
            block_number: Default::default(), 
            block_coinbase: Default::default(), 
            block_timestamp: Default::default(), 
            block_difficulty: Default::default(), 
            block_gas_limit: Default::default(), 
            block_base_fee_per_gas: U256::zero(), //Gwei 
            block_randomness: None
        };
        let mut state = BTreeMap::new();
        state.insert(
            H160::from_str("0x1000000000000000000000000000000000000000").unwrap(),
            MemoryAccount {
                nonce: U256::one(),
                balance: U256::from(10000000),
                storage: BTreeMap::new(),
                code: hex::decode(CONTRACT_BYTECODE).unwrap(),
            }
        );
        state.insert(
            H160::from_str("0xe14de1592b52481b94b99df4e9653654e14fffb6").unwrap(),
            MemoryAccount {
                nonce: U256::one(),
                balance: U256::from(10000000),
                storage: BTreeMap::new(),
                code: Vec::new(),
            },
        );

        MemoryStorage::new(
            MemoryBackend::new(vicinity, state),
            BTreeMap::new(),
        )
    }

    pub fn executor(&self, simulation: bool) -> StackExecutor<MemoryStackState<MemoryBackend>, BTreeMap<H160, PrecompileFn>> {

        StackExecutor::new_with_precompiles(
            MemoryStackState::new(StackSubstateMetadata::new(DEFAULT_TX_GAS_LIMIT, self.config()), &self.backend),
            self.config(),
            self.precompiles(),
            simulation
        )
    }
}

impl ExecutionBackend for MemoryStorage {

    fn config(&self) -> &Config {
        &self.config
    }

    fn precompiles(&self) -> &BTreeMap<H160, PrecompileFn> {
        &self.precompiles
    }

    fn code(&self, address: Address) -> Vec<u8> {
        self.backend.code(address)
    }

    fn apply_local_effect(&mut self, effect: Vec<Apply>, log: Vec<Log>) {
        self.backend.apply(effect, log, false);    
    }

    fn backend(&self) -> &MemoryBackend {
        &self.backend
    }
}
