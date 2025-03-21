use crate::*;
use int_to_bytes::int_to_bytes4;
use serde_derive::{Deserialize, Serialize};
use test_utils::{u8_from_hex_str, u8_to_hex_str};

/// Each of the BLS signature domains.
///
/// Spec v0.6.3
pub enum Domain {
    BeaconProposer,
    Randao,
    Attestation,
    Deposit,
    VoluntaryExit,
    Transfer,
}

/// Holds all the "constants" for a BeaconChain.
///
/// Spec v0.6.3
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ChainSpec {
    /*
     * Misc
     */
    pub target_committee_size: usize,
    pub max_indices_per_attestation: u64,
    pub min_per_epoch_churn_limit: u64,
    pub churn_limit_quotient: u64,
    pub base_rewards_per_epoch: u64,
    pub shuffle_round_count: u8,

    /*
     *  Deposit contract
     */
    pub deposit_contract_tree_depth: u64,

    /*
     *  Gwei values
     */
    pub min_deposit_amount: u64,
    pub max_effective_balance: u64,
    pub ejection_balance: u64,
    pub effective_balance_increment: u64,

    /*
     * Initial Values
     */
    pub genesis_slot: Slot,
    // Skipped because serde TOML can't handle u64::max_value, the typical value for this field.
    #[serde(skip_serializing)]
    pub far_future_epoch: Epoch,
    pub zero_hash: Hash256,
    #[serde(deserialize_with = "u8_from_hex_str", serialize_with = "u8_to_hex_str")]
    pub bls_withdrawal_prefix_byte: u8,

    /*
     * Time parameters
     */
    pub genesis_time: u64,
    pub seconds_per_slot: u64,
    pub min_attestation_inclusion_delay: u64,
    pub min_seed_lookahead: Epoch,
    pub activation_exit_delay: u64,
    pub slots_per_eth1_voting_period: u64,
    pub slots_per_historical_root: usize,
    pub min_validator_withdrawability_delay: Epoch,
    pub persistent_committee_period: u64,
    pub max_crosslink_epochs: u64,
    pub min_epochs_to_inactivity_penalty: u64,

    /*
     * Reward and penalty quotients
     */
    pub base_reward_quotient: u64,
    pub whistleblowing_reward_quotient: u64,
    pub proposer_reward_quotient: u64,
    pub inactivity_penalty_quotient: u64,
    pub min_slashing_penalty_quotient: u64,

    /*
     * Max operations per block
     */
    pub max_proposer_slashings: u64,
    pub max_attester_slashings: u64,
    pub max_attestations: u64,
    pub max_deposits: u64,
    pub max_voluntary_exits: u64,
    pub max_transfers: u64,

    /*
     * Signature domains
     *
     * Fields should be private to prevent accessing a domain that hasn't been modified to suit
     * some `Fork`.
     *
     * Use `ChainSpec::get_domain(..)` to access these values.
     */
    domain_beacon_proposer: u32,
    domain_randao: u32,
    domain_attestation: u32,
    domain_deposit: u32,
    domain_voluntary_exit: u32,
    domain_transfer: u32,

    /*
     * Network specific parameters
     *
     */
    pub boot_nodes: Vec<Multiaddr>,
    pub chain_id: u8,
}

impl ChainSpec {
    /// Get the domain number that represents the fork meta and signature domain.
    ///
    /// Spec v0.6.3
    pub fn get_domain(&self, epoch: Epoch, domain: Domain, fork: &Fork) -> u64 {
        let domain_constant = match domain {
            Domain::BeaconProposer => self.domain_beacon_proposer,
            Domain::Randao => self.domain_randao,
            Domain::Attestation => self.domain_attestation,
            Domain::Deposit => self.domain_deposit,
            Domain::VoluntaryExit => self.domain_voluntary_exit,
            Domain::Transfer => self.domain_transfer,
        };

        let mut bytes: Vec<u8> = fork.get_fork_version(epoch).to_vec();
        bytes.append(&mut int_to_bytes4(domain_constant));

        let mut fork_and_domain = [0; 8];
        fork_and_domain.copy_from_slice(&bytes);

        u64::from_le_bytes(fork_and_domain)
    }

    /// Returns a `ChainSpec` compatible with the Ethereum Foundation specification.
    ///
    /// Spec v0.6.3
    pub fn mainnet() -> Self {
        Self {
            /*
             * Misc
             */
            target_committee_size: 128,
            max_indices_per_attestation: 4096,
            min_per_epoch_churn_limit: 4,
            churn_limit_quotient: 65_536,
            base_rewards_per_epoch: 5,
            shuffle_round_count: 90,

            /*
             *  Deposit contract
             */
            deposit_contract_tree_depth: 32,

            /*
             *  Gwei values
             */
            min_deposit_amount: u64::pow(2, 0) * u64::pow(10, 9),
            max_effective_balance: u64::pow(2, 5) * u64::pow(10, 9),
            ejection_balance: u64::pow(2, 4) * u64::pow(10, 9),
            effective_balance_increment: u64::pow(2, 0) * u64::pow(10, 9),

            /*
             * Initial Values
             */
            genesis_slot: Slot::new(0),
            far_future_epoch: Epoch::new(u64::max_value()),
            zero_hash: Hash256::zero(),
            bls_withdrawal_prefix_byte: 0,

            /*
             * Time parameters
             */
            genesis_time: u64::from(u32::max_value()),
            seconds_per_slot: 6,
            min_attestation_inclusion_delay: 4,
            min_seed_lookahead: Epoch::new(1),
            activation_exit_delay: 4,
            slots_per_eth1_voting_period: 1_024,
            slots_per_historical_root: 8_192,
            min_validator_withdrawability_delay: Epoch::new(256),
            persistent_committee_period: 2_048,
            max_crosslink_epochs: 64,
            min_epochs_to_inactivity_penalty: 4,

            /*
             * Reward and penalty quotients
             */
            base_reward_quotient: 32,
            whistleblowing_reward_quotient: 512,
            proposer_reward_quotient: 8,
            inactivity_penalty_quotient: 33_554_432,
            min_slashing_penalty_quotient: 32,

            /*
             * Max operations per block
             */
            max_proposer_slashings: 16,
            max_attester_slashings: 1,
            max_attestations: 128,
            max_deposits: 16,
            max_voluntary_exits: 16,
            max_transfers: 0,

            /*
             * Signature domains
             */
            domain_beacon_proposer: 0,
            domain_randao: 1,
            domain_attestation: 2,
            domain_deposit: 3,
            domain_voluntary_exit: 4,
            domain_transfer: 5,

            /*
             * Boot nodes
             */
            boot_nodes: vec![],
            chain_id: 1, // mainnet chain id
        }
    }

    /// Ethereum Foundation minimal spec, as defined here:
    ///
    /// https://github.com/ethereum/eth2.0-specs/blob/v0.6.3/configs/constant_presets/minimal.yaml
    ///
    /// Spec v0.6.3
    pub fn minimal() -> Self {
        let genesis_slot = Slot::new(0);

        // Note: these bootnodes are placeholders.
        //
        // Should be updated once static bootnodes exist.
        let boot_nodes = vec!["/ip4/127.0.0.1/tcp/9000"
            .parse()
            .expect("correct multiaddr")];

        Self {
            target_committee_size: 4,
            shuffle_round_count: 10,
            min_attestation_inclusion_delay: 2,
            slots_per_eth1_voting_period: 16,
            genesis_slot,
            chain_id: 2, // lighthouse testnet chain id
            boot_nodes,
            ..ChainSpec::mainnet()
        }
    }
}

impl Default for ChainSpec {
    fn default() -> Self {
        Self::mainnet()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use int_to_bytes::int_to_bytes8;

    #[test]
    fn test_mainnet_spec_can_be_constructed() {
        let _ = ChainSpec::mainnet();
    }

    fn test_domain(domain_type: Domain, raw_domain: u32, spec: &ChainSpec) {
        let fork = Fork::genesis(Epoch::new(0));
        let epoch = Epoch::new(0);

        let domain = spec.get_domain(epoch, domain_type, &fork);

        let mut expected = fork.get_fork_version(epoch).to_vec();
        expected.append(&mut int_to_bytes4(raw_domain));

        assert_eq!(int_to_bytes8(domain), expected);
    }

    #[test]
    fn test_get_domain() {
        let spec = ChainSpec::mainnet();

        test_domain(Domain::BeaconProposer, spec.domain_beacon_proposer, &spec);
        test_domain(Domain::Randao, spec.domain_randao, &spec);
        test_domain(Domain::Attestation, spec.domain_attestation, &spec);
        test_domain(Domain::Deposit, spec.domain_deposit, &spec);
        test_domain(Domain::VoluntaryExit, spec.domain_voluntary_exit, &spec);
        test_domain(Domain::Transfer, spec.domain_transfer, &spec);
    }
}
