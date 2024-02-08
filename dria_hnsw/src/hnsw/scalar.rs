use std::collections::HashMap;
extern crate core;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use tdigest::TDigest;

#[derive(Debug, Clone)]
pub struct ScalarQuantizer {
    levels: usize,
    quantiles: Vec<f64>,
    t_digest: TDigest,
    dim: usize,
}

impl ScalarQuantizer {
    pub fn new(levels: usize, size: usize, dim: usize) -> Self {
        let quantiles = vec![];
        let t_digest = TDigest::new_with_size(size);
        ScalarQuantizer {
            levels,
            quantiles,
            t_digest,
            dim,
        }
    }

    pub fn merge(&mut self, matrix: Vec<Vec<f64>>) {
        let flattened: Vec<f64> = matrix.into_iter().flatten().collect();
        self.t_digest = self.t_digest.merge_unsorted(flattened); // Pass a reference to flattened
        self.quantiles = vec![]; //reset
        for i in 0..self.levels {
            self.quantiles.push(
                self.t_digest
                    .estimate_quantile((i as f64) / (self.levels.clone() as f64)),
            );
        }
    }

    fn __quantize_scalar(&self, scalar: &f64) -> usize {
        let ax = &self.quantiles;
        ax.into_iter()
            .enumerate()
            .find(|&(_, q)| scalar < q)
            .map_or(256 - 1, |(i, _)| i)
    }

    pub fn quantize_vectors(&self, vecs: Vec<Vec<f64>>) -> Vec<Vec<usize>> {
        let single: Vec<f64> = vecs.into_iter().flatten().collect();
        let quantized = self.quantize(single.as_slice());
        quantized
            .chunks(self.dim.clone())
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    pub fn dequantize(&self, qv: &[usize]) -> Vec<f64> {
        qv.iter().map(|&val| self.quantiles[val.min(255)]).collect()
    }

    pub fn quantize(&self, v: &[f64]) -> Vec<usize> {
        v.iter()
            .map(|value| self.__quantize_scalar(value))
            .collect()
    }
}
