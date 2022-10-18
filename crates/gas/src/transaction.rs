//! This module defines all the gas parameters for transactions, along with their initial values
//! in the genesis and a mapping between the Rust representation and the on-chain gas schedule.

use crate::algebra::{GasScalingFactor, GasUnit};
// use aptos_types::{state_store::state_key::StateKey, write_set::WriteOp};
use move_deps::move_core_types::{
    effects::Op,
    gas_algebra::{
        InternalGas, InternalGasPerArg, InternalGasPerByte, InternalGasUnit, NumArgs, NumBytes,
        ToUnitFractionalWithParams, ToUnitWithParams,
    },
};
use nova_types::access_path::AccessPath;

crate::params::define_gas_parameters!(
    TransactionGasParameters,
    "txn",
    [
        // The flat minimum amount of gas required for any transaction.
        // Charged at the start of execution.
        [
            min_transaction_gas_units: InternalGas,
            "min_transaction_gas_units",
            5_000 * 10_000 // 5_000 SDK gas cost per execute
        ],
        // Any transaction over this size will be charged an additional amount per byte.
        [
            large_transaction_cutoff: NumBytes,
            "large_transaction_cutoff",
            600 // 600 bytes
        ],
        // The units of gas that to be charged per byte over the `large_transaction_cutoff` in addition to
        // `min_transaction_gas_units` for transactions whose size exceeds `large_transaction_cutoff`.
        [
            intrinsic_gas_per_byte: InternalGasPerByte,
            "intrinsic_gas_per_byte",
            10_000 * 2 / 10 // 0.2 SDK gas per bytes
        ],
        // The scaling factor is used to scale up the passed `CosmosSDK.GasLimit`
        // i.e. The gas cost defined vm will be scale down with this value,
        // when we return used gas to chain.
        [
            gas_unit_scaling_factor: GasScalingFactor,
            "gas_unit_scaling_factor",
            10_000
        ],
        // Gas Parameters for reading data from storage.
        [
            load_data_base: InternalGas,
            "load_data.base",
            1_000 * 10_000 // sdk.ReadCostFlat = 1_000
        ],
        [
            load_data_per_byte: InternalGasPerByte,
            "load_data.per_byte",
            3 * 10_000 // sdk.ReadCostPerByte = 3
        ],
        [load_data_failure: InternalGas, "load_data.failure", 0],
        // Gas parameters for writing data to storage.
        [
            write_data_per_op: InternalGasPerArg,
            "write_data.per_op",
            2_000 * 10_000 // sdk.WriteCostFlat = 2_000
        ],
        [
            write_data_per_byte_in_key: InternalGasPerByte,
            "write_data.per_byte_in_key",
            30 * 10_000 // sdk.WriteCostPerByte = 3
        ],
        [
            write_data_per_byte_in_val: InternalGasPerByte,
            "write_data.per_byte_in_val",
            30 * 10_000 // sdk.WriteCostPerByte = 3
        ],
    ]
);

impl TransactionGasParameters {
    // TODO(Gas): Right now we are relying on this to avoid div by zero errors when using the all-zero
    //            gas parameters. See if there's a better way we can handle this.
    fn scaling_factor(&self) -> GasScalingFactor {
        match u64::from(self.gas_unit_scaling_factor) {
            0 => 1.into(),
            x => x.into(),
        }
    }

    /// Calculate the intrinsic gas for the transaction based upon its size in bytes.
    pub fn calculate_intrinsic_gas(&self, transaction_size: NumBytes) -> InternalGas {
        let min_transaction_fee = self.min_transaction_gas_units;

        if transaction_size > self.large_transaction_cutoff {
            let excess = transaction_size
                .checked_sub(self.large_transaction_cutoff)
                .unwrap();
            min_transaction_fee + (excess * self.intrinsic_gas_per_byte)
        } else {
            min_transaction_fee
        }
    }

    pub fn calculate_write_set_gas<'a>(
        &self,
        ops: impl IntoIterator<Item = (&'a AccessPath, &'a Op<Vec<u8>>)>, // ops: impl IntoIterator<Item = (&'a AccessPath, &'a Op<Vec<u8>>)>,
    ) -> InternalGas {
        use Op::*;

        // Counting
        let mut num_ops = NumArgs::zero();
        let mut num_bytes_key = NumBytes::zero();
        let mut num_bytes_val = NumBytes::zero();

        for (key, op) in ops.into_iter() {
            num_ops += 1.into();

            if self.write_data_per_byte_in_key > 0.into() {
                // TODO(Gas): Are we supposed to panic here?;
                num_bytes_key += NumBytes::new(
                    bcs::to_bytes(key)
                        .expect("Should be able to serialize AccessPath")
                        .len() as u64,
                );
            }

            match op {
                New(data) => {
                    num_bytes_val += NumBytes::new(data.len() as u64);
                }
                Modify(data) => {
                    num_bytes_val += NumBytes::new(data.len() as u64);
                }
                Delete => (),
            }
        }

        // Calculate the costs
        let cost_ops = self.write_data_per_op * num_ops;
        let cost_bytes = self.write_data_per_byte_in_key * num_bytes_key
            + self.write_data_per_byte_in_val * num_bytes_val;

        cost_ops + cost_bytes
    }
}

impl ToUnitWithParams<InternalGasUnit> for GasUnit {
    type Params = TransactionGasParameters;

    fn multiplier(params: &Self::Params) -> u64 {
        params.scaling_factor().into()
    }
}

impl ToUnitFractionalWithParams<GasUnit> for InternalGasUnit {
    type Params = TransactionGasParameters;

    fn ratio(params: &Self::Params) -> (u64, u64) {
        (1, params.scaling_factor().into())
    }
}
