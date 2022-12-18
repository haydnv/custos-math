use crate::Matrix;
use custos::{number::Number, Device, MainMemory, CPU};

pub fn scalar_apply<'a, T, F, D>(
    device: &'a CPU,
    lhs: &Matrix<T, D>,
    scalar: T,
    f: F,
) -> Matrix<'a, T>
where
    T: Number,
    F: Fn(&mut T, T, T),
    D: MainMemory,
{
    let mut out = device.retrieve(lhs.len, lhs.node.idx);
    scalar_apply_slice(lhs, &mut out, scalar, f);
    (out, lhs.dims()).into()
}

#[inline]
pub fn scalar_apply_slice<T, F>(lhs: &[T], out: &mut [T], scalar: T, f: F)
where
    T: Copy,
    F: Fn(&mut T, T, T),
{
    for (idx, value) in out.iter_mut().enumerate() {
        f(value, lhs[idx], scalar)
    }
}

pub fn row_op_slice_mut<T, F>(lhs: &[T], lrows: usize, lcols: usize, rhs: &[T], out: &mut [T], f: F)
where
    T: Copy,
    F: Fn(&mut T, T, T),
{
    for i in 0..lrows {
        let index = i * lcols;
        let x = &lhs[index..index + lcols];

        for (idx, value) in rhs.iter().enumerate() {
            f(&mut out[index + idx], x[idx], *value);
        }
    }
}

pub fn row_op_slice_lhs<T, F>(lhs: &mut [T], lhs_rows: usize, lhs_cols: usize, rhs: &[T], f: F)
where
    T: Copy,
    F: Fn(&mut T, T),
{
    for i in 0..lhs_rows {
        let index = i * lhs_cols;

        for (idx, value) in rhs.iter().enumerate() {
            f(&mut lhs[index + idx], *value);
        }
    }
}

pub fn row_op<'a, T, F, D>(
    device: &'a CPU,
    lhs: &Matrix<T, D>,
    rhs: &Matrix<T, D>,
    f: F,
) -> Matrix<'a, T>
where
    T: Number,
    F: Fn(&mut T, T, T),
    D: MainMemory,
{
    assert!(rhs.rows() == 1 && rhs.cols() == lhs.cols());

    let mut out = device.retrieve(lhs.len, [lhs.node.idx, rhs.node.idx]);
    row_op_slice_mut(lhs, lhs.rows(), lhs.cols(), rhs, &mut out, f);
    (out, lhs.dims()).into()
}

pub fn col_op<'a, T, F, D>(
    device: &'a CPU,
    lhs: &Matrix<T, D>,
    rhs: &Matrix<T, D>,
    f: F,
) -> Matrix<'a, T>
where
    T: Number,
    F: Fn(&mut T, T, T),
    D: MainMemory,
{
    let mut out = device.retrieve(lhs.len, [lhs.node.idx, rhs.node.idx]);
    col_op_slice_mut(lhs, lhs.rows(), lhs.cols(), rhs, &mut out, f);
    (out, lhs.dims()).into()
}

pub fn col_op_slice_mut<T, F>(lhs: &[T], lrows: usize, lcols: usize, rhs: &[T], out: &mut [T], f: F)
where
    T: Number,
    F: Fn(&mut T, T, T),
{
    let mut i = 0;
    for (idx, rdata_value) in rhs.iter().enumerate().take(lrows) {
        let index = idx * lcols;
        let row = &lhs[index..index + lcols];
        for data in row {
            f(&mut out[i], *data, *rdata_value);
            i += 1;
        }
    }
}

pub fn each_op<'a, T, F, D>(device: &'a CPU, x: &Matrix<T, D>, f: F) -> Matrix<'a, T>
where
    T: Copy + Default,
    F: Fn(T) -> T,
    D: MainMemory,
{
    let mut out = device.retrieve(x.len(), x.node.idx);
    each_op_slice(x, &mut out, f);
    (out, x.dims()).into()
}

pub fn each_op_slice<T, F>(x: &[T], out: &mut [T], f: F)
where
    T: Copy + Default,
    F: Fn(T) -> T,
{
    for (idx, value) in out.iter_mut().enumerate() {
        *value = f(x[idx]);
    }
}
