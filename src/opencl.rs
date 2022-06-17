use custos::{
    libs::opencl::{cl_device::InternCLDevice, KernelOptions},
    Error, CDatatype, Matrix,
};

pub fn str_op<T: CDatatype>(
    device: InternCLDevice,
    x: &Matrix<T>,
    op: &str,
) -> Result<Matrix<T>, Error> {
    let src = format!(
        "
        __kernel void str_op(__global const {datatype}* x, __global {datatype}* out) {{
            size_t id = get_global_id(0);
            {datatype} I = x[id];
            out[id] = {op};
        }}
    ",
        datatype = T::as_c_type_str()
    );

    let buf = KernelOptions::new(&device, x.as_buf(), [x.size(), 0, 0], &src)?
        .with_output(x.size())
        .run();
    buf.map(|buf| (buf, x.dims()).into())
}

pub fn scalar_op<T: CDatatype>(
    device: InternCLDevice,
    x: &Matrix<T>,
    scalar: T,
    op: &str,
) -> Result<Matrix<T>, Error> {
    let src = format!("
        __kernel void scalar_r_op(__global const {datatype}* x, const {datatype} scalar, __global {datatype}* out) {{
            size_t id = get_global_id(0);
            
            out[id] = x[id]{op}scalar;
        }}
    ", datatype=T::as_c_type_str());

    let buf = KernelOptions::new(&device, x.as_buf(), [x.size(), 0, 0], &src)?
        .add_arg(&scalar)
        .with_output(x.size())
        .run();
    buf.map(|buf| (buf, x.dims()).into())
}
