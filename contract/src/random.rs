use cosmwasm_std::{StdResult, Storage,};
use cosmwasm_storage::{ReadonlySingleton, Singleton};

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaChaRng;
use sha2::{Digest, Sha256};
use substrate_fixed::types::{I32F32,I64F64};

static KEY_ENTROPY_POOL: &[u8] = b"entropy_pool";

fn get_current_entropy_pool(storage: &dyn Storage) -> [u8; 32] {
    ReadonlySingleton::new(storage, KEY_ENTROPY_POOL)
        .load()
        .or::<[u8; 32]>(Ok([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]))
        .unwrap()
}

pub fn supply_more_entropy(
    storage: &mut dyn Storage,
    additional_entropy: &[u8],
) -> StdResult<()> {
    let current_entropy_pool = get_current_entropy_pool(storage);
    let mut new_entropy_source = Vec::from(current_entropy_pool);
    new_entropy_source.extend(additional_entropy);
    let new_entropy_pool: [u8; 32] = Sha256::digest(&new_entropy_source).into();
    Singleton::new(storage, KEY_ENTROPY_POOL).save(&new_entropy_pool)
}

pub fn get_random_number_generator(storage: &dyn Storage) -> ChaChaRng {
    let entropy_pool = get_current_entropy_pool(storage);
    ChaChaRng::from_seed(entropy_pool)
}

// returns a random fixed point number between 0..1 as I32F32
pub fn get_random_fixed_unit_interval(rng: &mut ChaChaRng) -> I32F32 {
    //let entropy_pool = get_current_entropy_pool(storage);
    //let mut rng = ChaChaRng::from_seed(entropy_pool);
    let numerator = rng.next_u32();
    let ratio = I64F64::from_num(numerator) / I64F64::from_num(u32::MAX);
    I32F32::from_num(ratio)
}

