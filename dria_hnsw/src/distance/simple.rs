#[cfg(target_arch = "x86_64")]
const MIN_DIM_SIZE_AVX: usize = 32;

#[cfg(any(
    target_arch = "x86",
    target_arch = "x86_64",
    all(target_arch = "aarch64", target_feature = "neon")
))]
const MIN_DIM_SIZE_SIMD: usize = 16;

pub fn euclid_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    -v1.iter().zip(v2).map(|(a, b)| (a - b).powi(2)).sum::<f32>()
}

pub fn manhattan_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    -v1.iter().zip(v2).map(|(a, b)| (a - b).abs()).sum::<f32>()
}

pub fn cosine_preprocess(vector: &[f32]) -> Vec<f32> {
    let mut length: f32 = vector.iter().map(|x| x * x).sum();
    if length < f32::EPSILON {
        return vector.to_vec();
    }
    length = length.sqrt();
    vector.iter().map(|x| x / length).collect()
}

pub fn dot_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    v1.iter().zip(v2).map(|(a, b)| a * b).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_preprocessing() {
        let v = vec![0.0, 0.0, 0.0, 0.0];
        let res = cosine_preprocess(v.as_slice());
        assert_eq!(res, vec![0.0, 0.0, 0.0, 0.0]);
    }
}
