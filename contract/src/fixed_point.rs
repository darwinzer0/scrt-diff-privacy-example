use cosmwasm_std::{
    StdError, StdResult,
};
use substrate_fixed::types::{I32F32,I64F64};
use std::str::FromStr;
use std::convert::TryInto;

pub fn byte_vec_to_fixed(
    byte_vec: Vec<u8>,
) -> StdResult<I32F32> {
    let bytes: [u8; 8] = match byte_vec.as_slice().try_into() {
        Ok(mean) => mean,
        Err(err) => { return Err(StdError::generic_err(format!("{:?}", err))) },
    };
    Ok(I32F32::from_be_bytes(bytes))
}

pub fn byte_vec_to_fixed64(
    byte_vec: Vec<u8>,
) -> StdResult<I64F64> {
    let bytes: [u8; 16] = match byte_vec.as_slice().try_into() {
        Ok(mean) => mean,
        Err(err) => { return Err(StdError::generic_err(format!("{:?}", err))) },
    };
    Ok(I64F64::from_be_bytes(bytes))
}

pub fn fixed_from_str(string: String) -> StdResult<I32F32> {
    let value = match I32F32::from_str(&string) {
        Ok(value) => value,
        _ =>  { return Err(StdError::generic_err("Invalid fixed point number string notation")); }
    };
    Ok(value)
}