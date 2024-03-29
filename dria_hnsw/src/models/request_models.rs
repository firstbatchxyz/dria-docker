use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::errors::ValidationError;

#[derive(Serialize, Deserialize, Debug)]
pub struct InsertModel {
    pub vector: Vec<f32>,
    pub metadata: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InsertBatchModel {
    pub data: Vec<InsertModel>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FetchModel {
    pub id: Vec<u32>, // TODO: rename this to `ids`
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryModel {
    pub vector: Vec<f32>,
    pub top_n: usize,
    pub query: Option<String>,
    pub level: Option<usize>,
}

impl QueryModel {
    pub fn new(
        vector: Vec<f32>,
        top_n: usize,
        query: Option<String>,
        level: Option<usize>,
    ) -> Result<Self, ValidationError> {
        Self::validate_top_n(top_n)?;
        Self::validate_level(level)?;

        Ok(QueryModel {
            vector,
            top_n,
            query,
            level,
        })
    }

    fn validate_top_n(top_n: usize) -> Result<(), ValidationError> {
        if top_n > 20 {
            Err(ValidationError("Top N cannot be more than 20.".to_string()))
        } else {
            Ok(())
        }
    }

    fn validate_level(level: Option<usize>) -> Result<(), ValidationError> {
        if level.is_some() {
            match level.unwrap() {
                1 | 2 | 3 | 4 => Ok(()),
                _ => Err(ValidationError(
                    "Level should be 1, 2, 3, or 4.".to_string(),
                )),
            }
        } else {
            Ok(())
        }
    }
}
