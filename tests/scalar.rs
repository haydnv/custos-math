use custos::{cpu::CPU, Matrix, AsDev, opencl::CLDevice, number::Float};
use custos_math::{AdditionalOps, scalar_apply};

pub fn roughly_equals<T: Float>(lhs: &[T], rhs: &[T], diff: T) {
    for (a, b) in lhs.iter().zip(rhs) {
        let abs = (*a - *b).abs();
        if abs > diff {
            panic!("\n left: '{:?}',\n right: '{:?}', \n left elem.: {} != right elem. {}", lhs, rhs, a, b)
        }
    }
}

#[test]
fn test_scalar() {
    let device = CPU::new().select();
    let x = Matrix::from((&device, (1, 5), [-1.31, 2.12, 1., 5., 4.,]));

    let res = device.adds(x, 2.0);
    assert_eq!(res.read(), vec![0.69, 4.12, 3., 7., 6.]);


    let device = CLDevice::get(0).unwrap().select();
    let x = Matrix::from((&device, (1, 5), [-1.31f32, 2.12, 1., 5., 4.,]));

    let res = device.adds(x, 2.0);
    roughly_equals(&res.read(), &[0.69, 4.12, 3., 7., 6.], 1E-5);
}

#[test]
fn test_scalar_apply() {
    let device = CPU::new().select();
    let x = Matrix::from((&device, (1, 5), [-1.31, 2.12, 1., 5., 4.,]));

    let res = scalar_apply(&device, x, 0., |c, a, _| *c = a.abs() + 1.);
    assert_eq!(res.read(), vec![2.31, 3.12, 2., 6., 5.,]);
}