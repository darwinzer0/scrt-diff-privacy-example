use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use std::any::type_name;
use cosmwasm_std::{
    StdError, StdResult, Storage, Addr, CanonicalAddr, Api,
};
use substrate_fixed::types::{I32F32, I64F64};
use crate::fixed_point::{byte_vec_to_fixed, byte_vec_to_fixed64};

pub static CONFIG_KEY: &[u8] = b"config";
pub static STATS_KEY: &[u8] = b"stats";
pub static BUDGET_REMAINING_KEY: &[u8] = b"rem";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StoredConfig {
    pub admin: CanonicalAddr,
    pub epsilon: Vec<u8>,
    pub privacy_budget: Vec<u8>,
}

impl StoredConfig {
    pub fn into_humanized(self, api: &dyn Api) -> StdResult<Config> {
        let config = Config {
            admin: api.addr_humanize(&self.admin)?,
            epsilon: byte_vec_to_fixed(self.epsilon)?,
            privacy_budget: byte_vec_to_fixed(self.privacy_budget)?,
        };
        Ok(config)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub admin: Addr,
    pub epsilon: I32F32,
    pub privacy_budget: I32F32,
}

impl Config {
    pub fn into_stored(self, api: &dyn Api) -> StdResult<StoredConfig> {
        let stored_config = StoredConfig {
            admin: api.addr_canonicalize(self.admin.as_str())?,
            epsilon: self.epsilon.to_be_bytes().to_vec(),
            privacy_budget: self.privacy_budget.to_be_bytes().to_vec(),
        };
        Ok(stored_config)
    }
}

pub fn set_config(
    storage: &mut dyn Storage,
    api: &dyn Api,
    config: Config,
) -> StdResult<()> {
    set_bin_data(storage, CONFIG_KEY, &config.into_stored(api)?)
}

pub fn get_config(
    storage: &dyn Storage,
    api: &dyn Api, 
) -> StdResult<Config> {
    let stored_config: StoredConfig = get_bin_data(storage, CONFIG_KEY)?;
    let config = stored_config.into_humanized(api)?;
    Ok(config)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StoredStats {
    pub count: u32,
    pub sum: Vec<u8>,
    pub upper_bound: Vec<u8>,
    pub lower_bound: Vec<u8>,
}

impl StoredStats {
    pub fn into_humanized(self) -> StdResult<Stats> {
        let stats = Stats {
            count: self.count,
            sum: byte_vec_to_fixed64(self.sum)?,
            upper_bound: byte_vec_to_fixed(self.upper_bound)?,
            lower_bound: byte_vec_to_fixed(self.lower_bound)?,
        };
        Ok(stats)
    }
}

//
// Record of actual stats
//

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    pub count: u32,
    pub sum: I64F64,
    pub upper_bound: I32F32,
    pub lower_bound: I32F32,
}

impl Stats {
    pub fn into_stored(self) -> StdResult<StoredStats> {
        let stored_stats = StoredStats {
            count: self.count,
            sum: self.sum.to_be_bytes().to_vec(),
            upper_bound: self.upper_bound.to_be_bytes().to_vec(),
            lower_bound: self.lower_bound.to_be_bytes().to_vec(),
        };
        Ok(stored_stats)
    }
}

pub fn set_stats(
    storage: &mut dyn Storage,
    stats: Stats,
) -> StdResult<()> {
    set_bin_data(storage, STATS_KEY, &stats.into_stored()?)
}

pub fn get_stats(
    storage: &dyn Storage,
) -> StdResult<Stats> {
    let stored_stats: StoredStats = get_bin_data(storage, STATS_KEY)?;
    let stats = stored_stats.into_humanized()?;
    Ok(stats)
}

pub fn set_budget_remaining(
    storage: &mut dyn Storage,
    remaining: I32F32,
) -> StdResult<()> {
    set_bin_data(storage, BUDGET_REMAINING_KEY, &remaining.to_be_bytes().to_vec())
}

pub fn get_budget_remaining(
    storage: &dyn Storage,
) -> StdResult<I32F32> {
    let stored_remaining: Vec<u8> = get_bin_data(storage, BUDGET_REMAINING_KEY)?;
    byte_vec_to_fixed(stored_remaining)
}

//
// Bin data storage setters and getters
//

pub fn set_bin_data<T: Serialize>(
    storage: &mut dyn Storage,
    key: &[u8],
    data: &T,
) -> StdResult<()> {
    let bin_data =
        bincode2::serialize(&data).map_err(|e| StdError::serialize_err(type_name::<T>(), e))?;
    storage.set(key, &bin_data);
    Ok(())
}

pub fn get_bin_data<T: DeserializeOwned>(
    storage: &dyn Storage,
    key: &[u8],
) -> StdResult<T> {
    let bin_data = storage.get(key);
    match bin_data {
        None => Err(StdError::not_found("Key not found in storage")),
        Some(bin_data) => Ok(bincode2::deserialize::<T>(&bin_data)
            .map_err(|e| StdError::serialize_err(type_name::<T>(), e))?),
    }
}
