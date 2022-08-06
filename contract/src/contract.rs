use cosmwasm_std::{
    entry_point, to_binary, Binary, Env,
    StdError, StdResult, DepsMut, MessageInfo, 
    Response, Deps,
};
use substrate_fixed::types::{I32F32, I64F64};
use rand_chacha::ChaChaRng;
use crate::msg::{ExecuteMsg, ExecuteAnswer, InstantiateMsg, QueryMsg, QueryAnswer, ResponseStatus::Success};
use crate::state::{set_stats, get_stats, Stats, get_config, set_config, Config, set_budget_remaining, get_budget_remaining,};
use crate::fixed_point::{fixed_from_str};
use crate::random::{supply_more_entropy, get_random_number_generator};
use crate::dp::{laplace};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let epsilon = fixed_from_str(msg.epsilon.clone())?;
    let privacy_budget = fixed_from_str(msg.privacy_budget.clone())?;

    set_config(deps.storage, deps.api, Config{
        admin: info.sender.clone(),
        epsilon,
        privacy_budget,
    })?;

    set_budget_remaining(deps.storage, privacy_budget)?;

    set_stats(deps.storage, Stats{
        count: 0_u32,
        sum: I64F64::from_num(0_u32),
        upper_bound: I32F32::from_num(0_u32),
        lower_bound: I32F32::from_num(u32::MAX),
    })?;

    // includes sent entropy from msg
    let mut fresh_entropy = to_binary(&msg)?.0;
    fresh_entropy.extend(to_binary(&env)?.0);
    fresh_entropy.extend(to_binary(&info)?.0);
    supply_more_entropy(deps.storage, fresh_entropy.as_slice())?;

    Ok(Response::new().add_attribute("init", "😎"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut, 
    env: Env, 
    info: MessageInfo, 
    msg: ExecuteMsg
) -> StdResult<Response> {
    let mut fresh_entropy = to_binary(&msg)?.0;
    fresh_entropy.extend(to_binary(&env)?.0);
    fresh_entropy.extend(to_binary(&info)?.0);
    supply_more_entropy(deps.storage, fresh_entropy.as_slice())?;
    let mut rng = get_random_number_generator(deps.storage);

    match msg {
        ExecuteMsg::AddObservation { value, .. } => try_add_observation(deps, value,),
        ExecuteMsg::FuzzyCount { .. } => try_fuzzy_count(deps, info, &mut rng, ),
        ExecuteMsg::FuzzyMean { .. } => try_fuzzy_mean(deps, info, &mut rng, ),
        ExecuteMsg::Reset { .. } => try_reset(deps, info, ),
    }
}

pub fn try_add_observation(
    deps: DepsMut,
    value: String,
) -> StdResult<Response> {
    let fixed_value: I32F32 = fixed_from_str(value)?;

    let mut stats = get_stats(deps.storage)?;

    if stats.count == 0 {
        stats = Stats {
            count: 1_u32,
            sum: I64F64::from_num(fixed_value),
            lower_bound: fixed_value,
            upper_bound: fixed_value,
        };
    } else {
        // increment count
        stats.count = stats.count + 1;

        // add to sum
        stats.sum = stats.sum + I64F64::from_num(fixed_value);
        //stats.mean = (fixed_value - stats.mean) / I32F32::from_num(stats.count) + stats.mean;

        if fixed_value > stats.upper_bound {
            stats.upper_bound = fixed_value;
        }
        if fixed_value < stats.lower_bound {
            stats.lower_bound = fixed_value;
        }
    }

    set_stats(deps.storage, stats)?;

    let mut resp = Response::default();
    resp.data = Some(to_binary(&ExecuteAnswer::AddObservation {
        status: Success,
    })?);
    Ok(resp)
}

pub fn try_fuzzy_count(
    deps: DepsMut,
    info: MessageInfo,
    rng: &mut ChaChaRng,
) -> StdResult<Response> {
    let config = get_config(deps.storage, deps.api)?;

    if info.sender != config.admin {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let stats = get_stats(deps.storage)?;

    if stats.count == 0 {
        return Err(StdError::generic_err("No data"));
    }

    let budget_remaining = get_budget_remaining(deps.storage)?;
    if budget_remaining <= I32F32::from_num(0_u32) {
        return Err(StdError::generic_err("Privacy budget exhausted"));
    }

    // sensitivity is always 1 for count queries
    let sensitivity = I32F32::from_num(1_u32);
    let epsilon: I32F32 = config.epsilon;
    
    let scale: I32F32 = sensitivity / epsilon;
    
    let noise = laplace(rng, scale);
    let real_count = I32F32::from_num(stats.count);
    let fuzzy_count = real_count + noise;

    set_budget_remaining(deps.storage, budget_remaining - epsilon)?;

    let mut resp = Response::default();
    resp.data = Some(to_binary(&ExecuteAnswer::FuzzyCount {
        count: fuzzy_count.to_string()
    })?);
    Ok(resp)
}

pub fn try_fuzzy_mean(
    deps: DepsMut,
    info: MessageInfo,
    rng: &mut ChaChaRng,
) -> StdResult<Response> {
    let config = get_config(deps.storage, deps.api)?;

    if info.sender != config.admin {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let stats = get_stats(deps.storage)?;

    if stats.count == 0 {
        return Err(StdError::generic_err("No data"));
    }

    let budget_remaining = get_budget_remaining(deps.storage)?;
    if budget_remaining <= I32F32::from_num(0_u32) {
        return Err(StdError::generic_err("Privacy budget exhausted"));
    }

    // using a bounded sensitivity for sum (not always best approach)
    let sensitivity = stats.upper_bound - stats.lower_bound;
    let epsilon = config.epsilon; 

    let scale = sensitivity / epsilon;
    
    let sum_noise = laplace(rng, scale);
    let fuzzy_sum = stats.sum + I64F64::from_num(sum_noise);

    // calculate fuzzy count
    let sensitivity = I32F32::from_num(1_u32);
    let scale = sensitivity / epsilon;

    let noise = laplace(rng, scale);
    let real_count = I32F32::from_num(stats.count);
    let fuzzy_count = I64F64::from_num(real_count + noise);

    let fuzzy_mean = I32F32::from_num(fuzzy_sum / fuzzy_count);

    // sequential queries for sum + count
    set_budget_remaining(deps.storage, budget_remaining - (2 * epsilon))?;

    let mut resp = Response::default();
    resp.data = Some(to_binary(&ExecuteAnswer::FuzzyMean {
        mean: fuzzy_mean.to_string(),
    })?);
    Ok(resp)
}

pub fn try_reset(
    deps: DepsMut,
    info: MessageInfo,
) -> StdResult<Response> {
    let config = get_config(deps.storage, deps.api)?;

    if info.sender != config.admin {
        return Err(StdError::generic_err("Unauthorized"));
    }

    set_budget_remaining(deps.storage, config.privacy_budget)?;

    set_stats(deps.storage, Stats{
        count: 0_u32,
        sum: I64F64::from_num(0_u32),
        upper_bound: I32F32::from_num(0_u32),
        lower_bound: I32F32::from_num(u32::MAX),
    })?;

    let mut resp = Response::default();
    resp.data = Some(to_binary(&ExecuteAnswer::Reset {
        status: Success,
    })?);
    Ok(resp)
}

#[entry_point]
pub fn query(
    deps: Deps, 
    _env: Env, 
    msg: QueryMsg
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetEpsilon {} => to_binary(&query_get_epsilon(deps)?),
    }
}

fn query_get_epsilon(
    deps: Deps,
) -> StdResult<Binary> {
    // return the epsilon
    let epsilon = get_config(deps.storage, deps.api)?.epsilon.to_string();
    to_binary(&QueryAnswer::GetEpsilon { epsilon, })
}
