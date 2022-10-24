use crate::gas_params::table::*;
use crate::meter::EXECUTION_GAS_MULTIPLIER as MUL;

crate::natives::define_gas_parameters_for_natives!(GasParameters, "table", [
    // Note(Gas): These are legacy parameters for loading from storage so they do not
    //            need to be multiplied.
    [.common.load_base, "common.load.base", 8000],
    [.common.load_per_byte, "common.load.per_byte", 1000],
    [.common.load_failure, "common.load.failure", 0],

    [.new_table_handle.base, "new_table_handle.base", 1000 * MUL],

    [.add_box.base, "add_box.base", 1200 * MUL],
    [.add_box.per_byte_serialized, "add_box.per_byte_serialized", 10 * MUL],

    [.borrow_box.base, "borrow_box.base", 1200 * MUL],
    [.borrow_box.per_byte_serialized, "borrow_box.per_byte_serialized", 10 * MUL],

    [.contains_box.base, "contains_box.base", 1200 * MUL],
    [.contains_box.per_byte_serialized, "contains_box.per_byte_serialized", 10 * MUL],

    [.remove_box.base, "remove_box.base", 1200 * MUL],
    [.remove_box.per_byte_serialized, "remove_box.per_byte_serialized", 10 * MUL],

    [.destroy_empty_box.base, "destroy_empty_box.base", 1200 * MUL],

    [.drop_unchecked_box.base, "drop_unchecked_box.base", 100 * MUL],
]);

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub common: CommonGasParameters,
    pub new_table_handle: NewTableHandleGasParameters,
    pub set_table_payer: SetTablePayerGasParameters,
    pub add_box: AddBoxGasParameters,
    pub borrow_box: BorrowBoxGasParameters,
    pub contains_box: ContainsBoxGasParameters,
    pub remove_box: RemoveGasParameters,
    pub destroy_empty_box: DestroyEmptyBoxGasParameters,
    pub drop_unchecked_box: DropUncheckedBoxGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            common: CommonGasParameters {
                load_base: 0.into(),
                load_per_byte: 0.into(),
                load_failure: 0.into(),
            },
            new_table_handle: NewTableHandleGasParameters { base: 0.into() },
            set_table_payer: SetTablePayerGasParameters { base: 0.into() },
            add_box: AddBoxGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            borrow_box: BorrowBoxGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            contains_box: ContainsBoxGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            remove_box: RemoveGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            destroy_empty_box: DestroyEmptyBoxGasParameters { base: 0.into() },
            drop_unchecked_box: DropUncheckedBoxGasParameters { base: 0.into() },
        }
    }
}
