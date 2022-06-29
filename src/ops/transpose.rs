use custos::{
    cpu::CPU,
    get_device,
    opencl::{CLDevice, KernelOptions},
    CDatatype, Matrix,
};

use crate::cached;

pub fn slice_transpose<T: Copy>(rows: usize, cols: usize, a: &[T], b: &mut [T]) {
    for i in 0..rows {
        let index = i * cols;
        let row = &a[index..index + cols];

        for (index, row) in row.iter().enumerate() {
            let idx = rows * index + i;
            b[idx] = *row;
        }
    }
}

pub fn cl_transpose<T: CDatatype>(
    device: CLDevice,
    x: &Matrix<T>,
) -> custos::Result<Matrix<T>> {
    let src = format!(
        "
        #define MODULO(x,N) (x % N)
        #define I0 {rows}
        #define I1 {cols}
        #define I_idx(i0,i1) ((size_t)(i0))*I1+(i1)
        #define I_idx_mod(i0,i1) MODULO( ((size_t)(i0)) ,I0)*I1+MODULO( (i1),I1)

        #define MODULO(x,N) (x % N)
        #define O0 {cols}
        #define O1 {rows}
        #define O_idx(o0,o1) ((size_t)(o0))*O1+(o1)
        #define O_idx_mod(o0,o1) MODULO( ((size_t)(o0)) ,O0)*O1+MODULO( (o1),O1)
        __kernel void transpose(__global const {datatype}* I, __global {datatype}* O) {{
            size_t gid = get_global_id(0);
            size_t gid_original = gid;size_t i1 = gid % I1;size_t i0 = gid / I1;gid = gid_original;
        
            O[O_idx(i1,i0)] = I[gid];
        }}
    
   ",
        rows = x.rows(),
        cols = x.cols(),
        datatype = T::as_c_type_str()
    );

    let gws = [x.size(), 0, 0];
    let buf = KernelOptions::new(&device, x, gws, &src)?
        .with_output(x.cols() * x.rows())
        .run();
    buf.map(|buf| (buf.unwrap(), (x.cols(), x.rows())).into())
}

pub trait Transpose<T> {
    #[allow(non_snake_case)]
    fn T(&self) -> Matrix<T>;
}

impl<T: CDatatype> Transpose<T> for Matrix<T> {
    #[allow(non_snake_case)]
    fn T(&self) -> Matrix<T> {
        let device = get_device!(TransposeOp, T).unwrap();
        device.transpose(self)
    }
}

pub trait TransposeOp<T> {
    fn transpose(&self, x: &Matrix<T>) -> Matrix<T>;
}

impl<T: Default + Copy> TransposeOp<T> for CPU {
    fn transpose(&self, x: &Matrix<T>) -> Matrix<T> {
        let mut y = cached(self, (x.cols(), x.rows()));
        slice_transpose(x.rows(), x.cols(), x.as_slice(), y.as_mut_slice());
        y
    }
}

impl<T: CDatatype> TransposeOp<T> for CLDevice {
    fn transpose(&self, x: &Matrix<T>) -> Matrix<T> {
        cl_transpose(self.clone(), x).unwrap()
    }
}
