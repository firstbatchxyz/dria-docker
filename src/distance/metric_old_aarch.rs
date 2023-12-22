use rand::Rng;
use std::arch::aarch64::*;


pub fn l2(x: &[f32], y: &[f32]) -> f32 {
    x.iter()
        .zip(y.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
}

pub fn dot_product(x: &[f32], y: &[f32]) -> f32 {
    x.iter()
        .zip(y.iter())
        .map(|(a, b)| a * b)
        .sum::<f32>()
}


#[inline]
pub fn l2_f32_apple(from: &[f32], to: &[f32]) -> f32 {
    unsafe {
        let len = from.len() / 4 * 4;
        let buf = [0.0_f32; 4];
        let mut sum = vld1q_f32(buf.as_ptr());
        for i in (0..len).step_by(4) {
            let left = vld1q_f32(from.as_ptr().add(i));
            let right = vld1q_f32(to.as_ptr().add(i));
            let sub = vsubq_f32(left, right);
            sum = vfmaq_f32(sum, sub, sub);
        }
        let mut sum = vaddvq_f32(sum);
        sum += l2(&from[len..], &to[len..]);
        sum
    }
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn distance_aarch(x: &[f32], y: &[f32]) -> f32 {
    use std::arch::aarch64::*;

    let len = x.len().min(y.len());
    let mut pp = 0.0_f32; // Sum of squares for x
    let mut qq = 0.0_f32; // Sum of squares for y
    let mut pq = 0.0_f32; // Dot product of x and y

    let zero = vdupq_n_f32(0.0); // Set all elements to 0.0
    let mut sum_pp = zero;
    let mut sum_qq = zero;
    let mut sum_pq = zero;

    // SIMD loop
    let simd_len = len / 4 * 4;
    for i in (0..simd_len).step_by(4) {
        let x_simd = vld1q_f32(x.as_ptr().add(i));
        let y_simd = vld1q_f32(y.as_ptr().add(i));

        sum_pp = vfmaq_f32(sum_pp, x_simd, x_simd); // sum_pp += x[i] * x[i];
        sum_qq = vfmaq_f32(sum_qq, y_simd, y_simd); // sum_qq += y[i] * y[i];
        sum_pq = vfmaq_f32(sum_pq, x_simd, y_simd); // sum_pq += x[i] * y[i];
    }

    pp += vaddvq_f32(sum_pp); // Horizontal add for sum_pp
    qq += vaddvq_f32(sum_qq); // Horizontal add for sum_qq
    pq += vaddvq_f32(sum_pq); // Horizontal add for sum_pq

    // Handle remaining elements
    for i in simd_len..len {
        pp += x[i] * x[i];
        qq += y[i] * y[i];
        pq += x[i] * y[i];
    }

    let ppqq = pp * qq;
    if ppqq > 0.0 {
        2.0 - 2.0 * pq / ppqq.sqrt()
    } else {
        2.0
    }
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn cosine_distance_aarch(x: &[f32], y: &[f32]) -> f32 {
    use std::arch::aarch64::*;

    let len = x.len().min(y.len());
    let mut pp = 0.0_f32; // Sum of squares for x
    let mut qq = 0.0_f32; // Sum of squares for y
    let mut pq = 0.0_f32; // Dot product of x and y

    let zero = vdupq_n_f32(0.0); // Set all elements to 0.0
    let mut sum_pp = zero;
    let mut sum_qq = zero;
    let mut sum_pq = zero;

    // SIMD loop
    let simd_len = len / 4 * 4;
    for i in (0..simd_len).step_by(4) {
        let x_simd = vld1q_f32(x.as_ptr().add(i));
        let y_simd = vld1q_f32(y.as_ptr().add(i));

        sum_pp = vfmaq_f32(sum_pp, x_simd, x_simd); // sum_pp += x[i] * x[i];
        sum_qq = vfmaq_f32(sum_qq, y_simd, y_simd); // sum_qq += y[i] * y[i];
        sum_pq = vfmaq_f32(sum_pq, x_simd, y_simd); // sum_pq += x[i] * y[i];
    }

    pp += vaddvq_f32(sum_pp); // Horizontal add for sum_pp
    qq += vaddvq_f32(sum_qq); // Horizontal add for sum_qq
    pq += vaddvq_f32(sum_pq); // Horizontal add for sum_pq

    // Handle remaining elements
    for i in simd_len..len {
        pp += x[i] * x[i];
        qq += y[i] * y[i];
        pq += x[i] * y[i];
    }

    1.0 - pq / (pp.sqrt() * qq.sqrt())
}


#[inline]
pub fn dot_product_f32_apple(from: &[f32], to: &[f32]) -> f32 {
    unsafe {
        let len = from.len() / 4 * 4;
        let buf = [0.0_f32; 4];
        let mut product = vld1q_f32(buf.as_ptr());
        for i in (0..len).step_by(4) {
            let left = vld1q_f32(from.as_ptr().add(i));
            let right = vld1q_f32(to.as_ptr().add(i));
            let mul = vmulq_f32(left, right);
            product = vaddq_f32(product, mul);
        }
        let mut product = vaddvq_f32(product);
        product += dot_product(&from[len..], &to[len..]);
        product
    }
}

#[inline]
pub fn normalize_vector_f32_apple(input: &[f32]) -> Vec<f32> {
    // Compute sum of squares
    unsafe {
        let len = input.len() / 4 * 4;
        let mut sum = vdupq_n_f32(0.0);
        for i in (0..len).step_by(4) {
            let vals = vld1q_f32(input.as_ptr().add(i));
            let squares = vmulq_f32(vals, vals);
            sum = vaddq_f32(sum, squares);
        }
        let mut sum = vaddvq_f32(sum);
        for &val in &input[len..] {
            sum += val * val;
        }
        let norm = sum.sqrt();

        // Normalize vector
        let mut output = vec![0.0; input.len()];
        for i in (0..len).step_by(4) {
            let vals = vld1q_f32(input.as_ptr().add(i));
            let normalized_vals = vdivq_f32(vals, vdupq_n_f32(norm));
            vst1q_f32(output.as_mut_ptr().add(i), normalized_vals);  // use a mutable borrow here
        }
        for i in len..input.len() {
            output[i] = input[i] / norm;
        }
        output
    }
}

pub fn generate_random_vector(n: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let vec:Vec<f32> = (0..n).map(|_| rng.gen()).collect();
    normalize_vector_f32_apple(vec.as_slice())
}