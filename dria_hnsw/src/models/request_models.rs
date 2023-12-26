use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use core::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct InsertModel{
    pub vector: Vec<f32>,
    pub metadata: Option<Value>,
    pub contract_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InsertBatchModel{
    pub data: String,
    pub contract_id: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct FetchModel{
    pub id: Vec<u32>,
    pub contract_id: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct QueryModel {
    pub vector: Vec<f32>,
    pub top_n: usize,
    pub contract_id: String,
    pub level: Option<usize>,
}

impl QueryModel {
    pub fn new(vector: Vec<f32>, top_n: usize, contract_id: String, level: Option<usize>) -> Result<Self, ValidationError> {
        Self::validate_top_n(top_n)?;
        Self::validate_level(level)?;

        Ok(QueryModel {
            vector,
            top_n,
            contract_id,
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

        if level.is_some(){
            match level.unwrap() {
                1 | 2 | 3 | 4 => Ok(()),
                _ => Err(ValidationError("Level should be 1, 2, 3, or 4.".to_string())),
            }
        }
        else{
            Ok(())
        }

    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchModel {
    pub query: String,
    pub top_n: usize,
    pub field: Option<String>, //default value text
    pub model: Option<String>, // "jina-embeddings-v2-base-en"
    pub contract_id: String,
    pub rerank: Option<bool>,
    pub level: Option<usize>,  //default value: 1
}

impl SearchModel {
    pub fn new(query: String, top_n: usize, field: Option<String>, model: Option<String>, contract_id: String, rerank: Option<bool>, level: Option<usize>) -> Result<Self, ValidationError> {
        Self::check_query_length(&query)?;
        Self::check_top_n(top_n)?;
        Self::check_level(level)?;
        Self::check_model(model.clone())?;

        Ok(SearchModel {
            query,
            top_n,
            field,
            model,
            contract_id,
            rerank,
            level
        })
    }

    fn check_query_length(query: &str) -> Result<(), ValidationError> {
        if query.len() > 500 {
            Err(ValidationError("Query cannot be more than 500 chars.".to_string()))
        } else {
            Ok(())
        }
    }

    fn check_top_n(top_n: usize) -> Result<(), ValidationError> {
        if top_n > 20 {
            Err(ValidationError("Top N cannot be more than 20.".to_string()))
        } else {
            Ok(())
        }
    }

    fn check_level(level: Option<usize>) -> Result<(), ValidationError> {
        if level.is_none(){
            return Ok(());
        }
        let l = level.unwrap();
        match l {
            1 | 2 | 3 | 4 => Ok(()),
            _ => Err(ValidationError("Level should be 1, 2, 3, or 4.".to_string())),
        }
    }

    fn check_model(model: Option<String>) -> Result<(), ValidationError> {
        if model.is_none() {
            return Ok(());
        }
        let m = model.clone().unwrap();
        match m.as_str() {
            "jina-embeddings-v2-base-en" | "text-embedding-ada-002" | "jina-embeddings-v2-small-en" => Ok(()),
            _ => Err(ValidationError("Model must be either jina-embeddings-v2-base-en, jina-embeddings-v2-small-en or text-embedding-ada-002.".to_string())),
        }
    }
}

#[derive(Debug)]
pub struct ValidationError(String);

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ValidationError {}
