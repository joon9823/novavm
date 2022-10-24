use std::{collections::VecDeque, sync::Arc, u8, str::{self, FromStr}};
use anyhow::{anyhow, bail};
use bigdecimal::{BigDecimal, FromPrimitive};

use smallvec::smallvec;
use move_deps::{move_core_types::gas_algebra::{InternalGas, InternalGasPerByte}, move_vm_runtime::native_functions::{NativeContext, NativeFunction}, move_vm_types::{loaded_data::runtime_types::Type, values::Value, natives::function::NativeResult, pop_arg}, move_binary_format::errors::PartialVMResult};


pub mod status {
    pub const INVALID_INPUT: u64 = 0x1;
}


/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub base: NumberGasParameters,
}

/***************************************************************************************************
 * native public fun from_bytes_le(
 *     input: vector<u8>,
 * )
 * 
 * gas cost: base_cost + unit_cost * bytes_len
 *
 * _and_
 *
 * native public fun from_bytes_be(
 *     input: vector<u8>,
 * )
 * 
 * gas cost: base_cost + unit_cost * bytes_len
 *
 **************************************************************************************************/
#[derive(Clone, Debug)]
pub struct NumberGasParameters {
    pub base: InternalGas,
    pub unit: InternalGasPerByte,
}

fn native_from_bytes_le(
    gas_params: &NumberGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let mut cost = gas_params.base;

    let bytes = pop_arg!(arguments, Vec<u8>);

    let number = Number::from_str(str::from_utf8(&bytes).unwrap());
    match number {
        Ok(n) => {
            let val = Value::vector_u64(number);
            Ok(NativeResult::ok(cost, smallvec![val]))
        },
        Err(_) =>  Ok(NativeResult::err(cost, status::INVALID_INPUT))
    }
}


pub fn make_native_from_bytes_le(gas_params: NumberGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| native_from_bytes_le(&gas_params, context, ty_args, args))
}



pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        ("from_bytes_le", make_native_from_bytes_le(gas_params.base))
    ];

    crate::helpers::make_module_natives(natives)
}

#[derive(Clone, Debug, Eq)]
pub struct Number(BigDecimal);

impl Number {
    pub fn new(uval: u128) -> Number {
        Number(BigDecimal::from_u128(uval).unwrap())
    }

    fn foo(&self) {
        self.into_
    }


    pub fn checked_add(&self, other: &Self) -> Number {
        let res = self.0.clone() + other.0.clone();
        
        Number(res)
    }
}

impl From<BigDecimal> for Number {
    fn from(val: BigDecimal) -> Self {
        Number(val)
    }
}

impl FromStr for Number {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
       let dec = BigDecimal::from_str(s).unwrap();
       
       Ok(Number(dec))
    }
}

impl PartialEq for Number{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

#[cfg(test)] 
mod tests {
    use bigdecimal::num_bigint::BigInt;

    use super::*;

    #[test]
    fn test_new_integer_from_str() {
        let expected = "831204";
        assert_eq!(Number::from_str(expected).unwrap(), Number::new(831204u128));
    }

    #[test]
    fn test_new_number_from_str() {
        let expected = "831204.850816";
        let number = Number::from_str(expected).unwrap();
        let (bi, exp) = number.0.into_bigint_and_exponent();
        assert_eq!(bi, BigInt::from_i64(831204850816).unwrap());
        assert_eq!(exp, 6);
    }
}

//
// Number implementation
//
/* 
pub struct Number(Vec<u64>);

impl Number {
    const DECIMAL_FRACTIONAL: U256 = UInt::from_be_slice(
            &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 224, 182,
            179, 167, 100, 0, 0,]);

    const DECIMAL_FRACTIONAL_SQUARED: U256 = UInt::from_be_slice(
            &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 151, 206, 123, 201, 7, 21, 179,
            75, 159, 16, 0, 0, 0, 0,]);
    
    /// The number of decimal places. Since decimal types are fixed-point rather than
    /// floating-point, this is a constant.
    pub const DECIMAL_PLACES: u8 = 18;
    pub const DECIMAL_PLACES_OBJ: U256 = UInt::from_u8(Self::DECIMAL_PLACES);

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> anyhow::Result<Self> {
        bail!("unimplemented")
    }

    pub fn from_str(input: &str) -> anyhow::Result<Self> {
        let mut parts_iter = input.split('.');

        let whole_part = parts_iter.next().unwrap(); // split always returns at least one element
        let whole = U256::from_le_hex(whole_part);

        let atomics = whole.checked_mul(&Self::DECIMAL_FRACTIONAL);
        if atomics.is_none().into() {
            bail!("value too big")
        }
        let atomics = atomics.unwrap();

        if let Some(fractional_part) = parts_iter.next() {
            let fractional = U256::from_le_hex(fractional_part);

            let exp = Self::DECIMAL_PLACES_OBJ.checked_sub(&U256::from_u8(fractional_part.len() as u8));
            if exp.is_none().into() {
                bail!("cannot parse more than {} fractional digits", Self::DECIMAL_PLACES)
            }
            let exp = exp.unwrap() as u8;

            debug_assert!(exp <= Self::DECIMAL_PLACES);
            let factor = 10u128.pow(exp.into());
            let fractional_factor = U256::from(factor);
            atomics = match atomics.checked_add(&fractional.checked_mul(&fractional_factor).unwrap()).into() as Option<U256> {
                Some(val) => val,
                None =>  bail!("value too big")
            };
        }

        if parts_iter.next().is_some() {
            bail!("Unexpected number of dots");
        }

        Ok(Self(atomics.to_words().into_iter().map(|w| w.to_le()).collect()))
    }


}
*/