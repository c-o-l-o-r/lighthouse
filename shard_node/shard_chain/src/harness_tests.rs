use crate::harness::ShardChainHarness;
use lmd_ghost::ThreadSafeReducedTree;
use shard_lmd_ghost::ThreadSafeReducedTree as ShardThreadSafeReducedTree;
use shard_store::MemoryStore as ShardMemoryStore;
use store::MemoryStore;
use types::{MinimalEthSpec, MinimalShardSpec};

pub const VALIDATOR_COUNT: usize = 24;

pub type TestBeaconForkChoice = ThreadSafeReducedTree<MemoryStore, MinimalEthSpec>;
pub type TestShardForkChoice = ShardThreadSafeReducedTree<ShardMemoryStore, MinimalShardSpec>;

fn get_harness(
    validator_count: usize,
) -> ShardChainHarness<TestBeaconForkChoice, MinimalEthSpec, TestShardForkChoice, MinimalShardSpec>
{
    let harness = ShardChainHarness::new(validator_count);

    // Move past the zero slot
    harness.advance_beacon_slot();
    harness.advance_shard_slot();

    harness
}

#[test]
fn advance_shard_slot() {
    // let code = load_file("../../eth2/shard_state_processing/execution_environments/sheth.wasm");

    let harness = get_harness(VALIDATOR_COUNT);
    let num_blocks_produced =
        MinimalEthSpec::slots_per_epoch() * harness.beacon_spec.phase_1_fork_epoch;

    harness.extend_beacon_chain((num_blocks_produced + 1) as usize);

    let beacon_slot = harness.beacon_chain.current_state().slot;
    let shard_slot = harness.shard_chain.current_state().slot;

    harness
        .shard_chain
        .process_body(hex::decode("48656c6c6f20776f726c6421").unwrap());
    harness.extend_shard_chain(1);

    for i in 0..30 {
        harness.advance_beacon_slot();
        harness.advance_shard_slot();
        harness.extend_beacon_chain(1);
        harness.extend_shard_chain(1);
        harness.advance_shard_slot();
        harness.extend_shard_chain(1);
    }
}
