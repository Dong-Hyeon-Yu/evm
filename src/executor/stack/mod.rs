//! A stack-based executor with customizable state.
//! A memory-based state is provided, but can replaced by a custom
//! implementation, for exemple one interacting with a database.

mod executor;
mod memory;
mod precompile;
mod tagged_runtime;

pub use self::executor::{
	Accessed, StackExecutor, StackExitKind, StackState, StackSubstateMetadata, RwSet, Simulatable, MultiversionStackExecutor, MultiversionView
};
pub use self::memory::{MemoryStackAccount, MemoryStackState, MemoryStackSubstate};
pub use self::precompile::{
	IsPrecompileResult, PrecompileFailure, PrecompileFn, PrecompileHandle, PrecompileOutput,
	PrecompileSet,
};
pub use ethereum::Log;

