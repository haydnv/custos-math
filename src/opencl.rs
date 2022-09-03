mod gemm;
mod switching;
mod tew;

pub use gemm::cl_gemm;
pub use switching::*;
pub use tew::*;

use custos::{
    cache::Cache,
    devices::opencl::cl_device::CLDevice,
    opencl::{
        api::{enqueue_write_buffer, wait_for_event},
        enqueue_kernel, AsClCvoidPtr,
    },
    Buffer, CDatatype, Error,
};

use crate::Matrix;

pub fn cl_str_op<'a, T>(
    device: &'a CLDevice,
    x: &Buffer<T>,
    op: &str,
) -> custos::Result<Buffer<'a, T>>
where
    T: CDatatype,
{
    let src = format!(
        "
        __kernel void str_op(__global const {datatype}* lhs, __global {datatype}* out) {{
            size_t id = get_global_id(0);
            {datatype} x = lhs[id];
            out[id] = {op};
        }}
    ",
        datatype = T::as_c_type_str()
    );

    let out = Cache::get::<T, _>(device, x.size(), x.node.idx);
    enqueue_kernel(device, &src, [x.size(), 0, 0], None, &[x, &out])?;
    Ok(out)
}

#[inline]
pub fn cl_str_op_mat<'a, T: CDatatype>(
    device: &'a CLDevice,
    x: &Matrix<T>,
    op: &str,
) -> Result<Matrix<'a, T>, Error> {
    let out = cl_str_op(device, x, op)?;
    Ok((out, x.dims()).into())
}

pub fn cl_scalar_op<'a, T>(
    device: &'a CLDevice,
    x: &Buffer<T>,
    scalar: T,
    op: &str,
) -> custos::Result<Buffer<'a, T>>
where
    T: CDatatype,
{
    let src = format!("
    __kernel void scalar_r_op(__global const {datatype}* x, const {datatype} scalar, __global {datatype}* out) {{
        size_t id = get_global_id(0);
        
        out[id] = x[id]{op}scalar;
    }}
    ", datatype=T::as_c_type_str());

    let out = Cache::get::<T, _>(device, x.size(), x.node.idx);
    enqueue_kernel(device, &src, [x.size(), 0, 0], None, &[x, &scalar, &out])?;
    Ok(out)
}

pub fn cl_scalar_op_mat<'a, T: CDatatype>(
    device: &'a CLDevice,
    x: &Matrix<T>,
    scalar: T,
    op: &str,
) -> Result<Matrix<'a, T>, Error> 
{
    let out = cl_scalar_op(device, x, scalar, op)?;
    Ok((out, x.dims()).into())
}

pub fn cl_write<T>(device: &CLDevice, x: &mut Buffer<T>, data: &[T]) {
    let event = unsafe { enqueue_write_buffer(&device.queue(), x.ptr.1, data, true).unwrap() };
    wait_for_event(event).unwrap();
}

impl<'a, T> AsClCvoidPtr for Matrix<'a, T> {
    fn as_cvoid_ptr(&self) -> *const std::ffi::c_void {
        self.ptr.1
    }
}

impl<'a, T> AsClCvoidPtr for &Matrix<'a, T> {
    fn as_cvoid_ptr(&self) -> *const std::ffi::c_void {
        self.ptr.1
    }
}
