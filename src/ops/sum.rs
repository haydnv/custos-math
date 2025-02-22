use crate::Matrix;
use custos::{impl_stack, number::Number, CDatatype, Device, MainMemory, Shape, CPU};

#[cfg(feature = "stack")]
use custos::Stack;

#[cfg(feature = "cpu")]
use custos::Cache;

#[cfg(feature = "opencl")]
use super::{cl_to_cpu_s, cl_to_cpu_scalar};
#[cfg(feature = "opencl")]
use custos::OpenCL;

#[cfg(feature = "cuda")]
use crate::{cu_to_cpu_s, cu_to_cpu_scalar};
#[cfg(feature = "cuda")]
use custos::CUDA;

impl<'a, T, IS: Shape, D: SumOps<T, IS>> Matrix<'a, T, D, IS> {
    pub fn sum(&self) -> T {
        self.device().sum(self)
    }

    pub fn mean(&self) -> T {
        self.device().mean(self)
    }
}

impl<'a, T, D: Device, IS: Shape> Matrix<'a, T, D, IS> {
    pub fn sum_rows<OS: Shape>(&self) -> Matrix<'a, T, D, OS>
    where
        D: SumOverOps<T, IS, OS>,
    {
        self.device().sum_rows(self)
    }

    pub fn sum_cols<OS: Shape>(&self) -> Matrix<'a, T, D, OS>
    where
        D: SumOverOps<T, IS, OS>,
    {
        self.device().sum_cols(self)
    }
}

pub trait SumOps<T, IS: Shape = (), D: Device = Self>: Device {
    fn sum(&self, x: &Matrix<T, D, IS>) -> T;
    fn mean(&self, x: &Matrix<T, D, IS>) -> T;
}

pub trait SumOverOps<T, IS: Shape = (), OS: Shape = (), D: Device = Self>: Device {
    fn sum_rows(&self, x: &Matrix<T, D, IS>) -> Matrix<T, Self, OS>;
    fn sum_cols(&self, x: &Matrix<T, D, IS>) -> Matrix<T, Self, OS>;
}

#[cfg(feature = "cpu")]
#[impl_stack]
impl<T: Number, D: MainMemory, IS: Shape> SumOps<T, IS, D> for CPU {
    fn sum(&self, x: &Matrix<T, D, IS>) -> T {
        x.iter().copied().sum()
        /*let mut sum = T::default();
        for value in x.as_slice() {
            sum += *value;
        }
        sum*/
    }

    fn mean(&self, x: &Matrix<T, D, IS>) -> T {
        let sum = self.sum(x);
        sum / T::from_usize(x.size())
    }
}

#[cfg(feature = "cpu")]
impl<T: Copy + Default + core::ops::AddAssign, D: MainMemory, IS: Shape, OS: Shape>
    SumOverOps<T, IS, OS, D> for CPU
{
    fn sum_rows(&self, x: &Matrix<T, D, IS>) -> Matrix<T, Self, OS> {
        let mut out = Cache::get(self, x.cols(), x.node.idx);

        let data = x.as_slice();
        let sum_slice = out.as_mut_slice();

        for value in sum_slice.iter_mut() {
            *value = T::default();
        }

        for idx in 0..x.rows() {
            let index = idx * x.cols();
            let row = &data[index..index + x.cols()];

            for (i, value) in row.iter().enumerate() {
                sum_slice[i] += *value;
            }
        }
        (out, 1, x.cols()).into()
    }

    fn sum_cols(&self, x: &Matrix<T, D, IS>) -> Matrix<T, Self, OS> {
        let mut out = Cache::get(self, x.rows(), x.node.idx);

        let data = x.as_slice();
        let sum_slice = out.as_mut_slice();

        for (idx, col_vec_value) in sum_slice.iter_mut().enumerate().take(x.rows()) {
            let index = idx * x.cols();
            let row = &data[index..index + x.cols()];
            let mut sum = T::default();

            for data in row {
                sum += *data;
            }
            *col_vec_value = sum;
        }
        (out, x.rows(), 1).into()
    }
}

#[cfg(feature = "opencl")]
impl<T: Number> SumOps<T> for OpenCL {
    #[inline]
    fn sum(&self, x: &Matrix<T, Self>) -> T {
        cl_to_cpu_scalar(self, x, |device, x| device.sum(x))
    }

    #[inline]
    fn mean(&self, x: &Matrix<T, Self>) -> T {
        cl_to_cpu_scalar(self, x, |device, x| device.mean(x))
    }
}

#[cfg(feature = "opencl")]
impl<T: CDatatype> SumOverOps<T> for OpenCL {
    #[inline]
    fn sum_rows<'a>(&'a self, x: &Matrix<T, Self>) -> Matrix<'a, T, Self> {
        cl_to_cpu_s(self, x, |device, x| device.sum_rows(x))
    }

    #[inline]
    fn sum_cols(&self, x: &Matrix<T, Self>) -> Matrix<T, Self> {
        cl_to_cpu_s(self, x, |device, x| device.sum_cols(x))
    }
}

#[cfg(feature = "cuda")]
impl<T: CDatatype> SumOps<T> for CUDA {
    #[inline]
    fn sum(&self, x: &Matrix<T, CUDA>) -> T {
        cu_to_cpu_scalar(x, |device, x| device.sum(&x))
    }

    #[inline]
    fn mean(&self, x: &Matrix<T, CUDA>) -> T {
        cu_to_cpu_scalar(x, |device, x| device.mean(&x))
    }
}

#[cfg(feature = "cuda")]
impl<T: CDatatype> SumOverOps<T> for CUDA {
    #[inline]
    fn sum_rows(&self, x: &Matrix<T, CUDA>) -> Matrix<T, CUDA> {
        cu_to_cpu_s(self, x, |device, x| device.sum_rows(&x))
    }

    #[inline]
    fn sum_cols(&self, x: &Matrix<T, CUDA>) -> Matrix<T, CUDA> {
        cu_to_cpu_s(self, x, |device, x| device.sum_cols(&x))
    }
}
