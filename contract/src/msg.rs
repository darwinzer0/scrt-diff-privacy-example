use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub epsilon: String,
    pub privacy_budget: String,
    pub entropy: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddObservation { 
        value: String, 
        padding: String, 
    },
    FuzzyCount {
        padding: String,
    },
    FuzzyMean {
        padding: String,
    },
    Reset {
        padding: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteAnswer {
    AddObservation {
        status: ResponseStatus,
    },
    FuzzyCount {
        count: String,
    },
    FuzzyMean {
        mean: String,
    },
    Reset {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Get epsilon value set for contract
    GetEpsilon { },
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    GetEpsilon {
        epsilon: String,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}