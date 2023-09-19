use std::{str::FromStr, collections::BTreeMap};
use ethers_core::types::transaction::eip2718::TypedTransaction;
use evm::{
	executor::stack::StackState, 
	tracing::{EventListener, Event}, 
	execution_storage::{MemoryStorage, ExecutionBackend, CONTRACT_BYTECODE}, backend::{MemoryVicinity, MemoryAccount, MemoryBackend}};
use primitive_types::{U256, H160};
use rlp::Rlp;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
use tracing::subscriber::set_global_default;
use futures::future::join_all;

#[tokio::main]
async fn main() {
	let _ = &DebugEventListener::new();
	
	tx_simulation_serial();

	// tx_execution_serial();

	// tx_execution_async().await;

	// contract_deploy();
}

#[allow(dead_code)]
async fn tx_execution_async() {
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

	let backend = MemoryBackend::new(vicinity, state);
	let mut handles = vec![];

	for i in 0..100 {
		let backend = backend.clone();

		handles.push(
			tokio::spawn(async move {
				let storage = MemoryStorage::new(backend, BTreeMap::new());
				let mut executor = storage.executor(false);


				let (reason, _) = executor.transact_call(
					H160::from_str("0xe14de1592b52481b94b99df4e9653654e14fffb6").unwrap(),
					H160::from_str("0x1000000000000000000000000000000000000000").unwrap(),
					U256::zero(),
					hex::decode("870187eb0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000053434373336000000000000000000000000000000000000000000000000000000")
						.unwrap(),
					// hex::decode("0f14a4060000000000000000000000000000000000000000000000000000000000002ee0").unwrap(),
					50002,
					Vec::new(),
				);
		
				info!("[thread-{i}]::{reason:?}");
				info!("[thread-{i}]::gas snapshot: {:?}", executor.state().metadata().gasometer().snapshot());
		}));
	}

	join_all(handles).await;
}

#[allow(dead_code)]
pub fn tx_execution_serial() {
	let mut execution_state = MemoryStorage::default();

	for _ in 0..10 {
		let mut executor = execution_state.executor(false);
		
		let raw_tx = hex::decode("02f8f509887d0b53721cd770f6808088ffffffffffffffff94100000000000000000000000000000000000000080b8840be8374d0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000053930383531000000000000000000000000000000000000000000000000000000c080a0bde42a3e09ccdc41d2729fc9d2ae0d54418dab8fa6e513984323e74b94a57b4ba07258cb57ad993e0048b0e05e8c2997d9b25f4a3df391738db275271a8a532485").unwrap();
		
		let tx = validate(raw_tx.as_slice());

		let (reason, _) = executor.transact_call(
			H160::from_str("0xe14de1592b52481b94b99df4e9653654e14fffb6").unwrap(),
			tx.to_addr().unwrap().to_owned(),
			tx.value().unwrap().to_owned(),
			tx.data().unwrap().to_owned().to_vec(),
			tx.gas().unwrap().to_owned().as_u64(),
			Vec::new(),
		);

		info!("{reason:?}");
		let (effects, logs) = executor.into_state().deconstruct();
		info!("{:?}\n\n", effects);
		execution_state.apply_local_effect(effects, logs);
	}
}

#[allow(dead_code)]
pub fn contract_deploy() {
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
		H160::from_str("0xe14de1592b52481b94b99df4e9653654e14fffb6").unwrap(),
		MemoryAccount {
			nonce: U256::one(),
			balance: U256::from(10000000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);
	let execution_state = MemoryStorage::new(MemoryBackend::new(vicinity, state), BTreeMap::new());

	let mut executor = execution_state.executor(false);
	let (reason, res) = executor.transact_create(
		H160::from_str("0xe14de1592b52481b94b99df4e9653654e14fffb6").unwrap(), 
		U256::zero(), 
		hex::decode(CONTRACT_BYTECODE).unwrap(), 
		u64::MAX, 
		Vec::new()
	);

	info!("{:?}", reason);
	info!("gas snapshot: {:?}", executor.state().metadata().gasometer().snapshot());
	info!("return: {}", hex::encode(res));
	let (effects, logs) = executor.into_state().deconstruct();
	info!("{:?}", effects);
	info!("{:?}", logs);


}


#[allow(dead_code)]
pub fn tx_simulation_serial() {
	let mut execution_state = MemoryStorage::default();

	let raw_tx = hex::decode("02f8f509887d0b53721cd770f6808088ffffffffffffffff94100000000000000000000000000000000000000080b8840be8374d0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000053930383531000000000000000000000000000000000000000000000000000000c080a0bde42a3e09ccdc41d2729fc9d2ae0d54418dab8fa6e513984323e74b94a57b4ba07258cb57ad993e0048b0e05e8c2997d9b25f4a3df391738db275271a8a532485").unwrap();
	let tx = validate(raw_tx.as_slice());

	for _ in 0..10 {
		let mut executor = execution_state.executor(true);

		let (reason, _) = executor.transact_call(
			H160::from_str("0xe14de1592b52481b94b99df4e9653654e14fffb6").unwrap(),
			tx.to_addr().unwrap().to_owned(),
			tx.value().unwrap().to_owned(),
			tx.data().unwrap().to_owned().to_vec(),
			tx.gas().unwrap().to_owned().as_u64(),
			Vec::new(),
		);

		info!("{reason:?}");
		info!("{:?}\n\n", executor.rw_set());
		let (effects, logs) = executor.into_state().deconstruct();
		info!("{:?}\n\n", effects);
		execution_state.apply_local_effect(effects, logs);
	}
}

/// Determines if a transaction valid for the worker to consider putting in a batch
fn validate(t: &[u8]) -> TypedTransaction { 

	let rlp = Rlp::new(t);
	
	match TypedTransaction::decode_signed(&rlp) {
		Ok((tx, sig)) => {
			if let Err(e) = sig.verify(tx.sighash(), *tx.from().unwrap()) {
				panic!("invalid tx signature: {:?}", e);
			}
			info!("tx: {:?}\n", tx);
			tx
		},
		Err(e) => {
			panic!("invalid tx: {:?}", e);
		}
	}
}


struct DebugEventListener;

impl DebugEventListener {
	pub fn new() -> Self {
		// let custom_directive = "evm=trace";
		let filter = EnvFilter::builder()
			.with_default_directive(LevelFilter::INFO.into())
			.parse(format!(
				"debug"
			)).expect("fail to parse env for log filter");
	
		let env_filter = EnvFilter::try_from_default_env().unwrap_or(filter);
	
		let timer = tracing_subscriber::fmt::time::UtcTime::rfc_3339();
		let subscriber_builder = tracing_subscriber::fmt::Subscriber::builder()
			.with_env_filter(env_filter)
			.with_timer(timer)
			.with_ansi(false);
		let subscriber = subscriber_builder.with_writer(std::io::stderr).finish();
		set_global_default(subscriber).expect("Failed to set subscriber");
		Self {}
	}
}

impl EventListener for DebugEventListener {

	fn event(&mut self, event: Event<'_>) {
		info!("{:?}", event);
	}
}