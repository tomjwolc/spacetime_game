// Written by a generator written by enki.
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

// Modifed to make work with my project, original: R120 code generator at https://bivector.net/tools.html

use std::fmt;
use std::ops::{Index,IndexMut,Add,Sub,Mul,BitAnd,BitOr,BitXor,Not};

type float_t = f32;

// use std::f64::consts::PI;
const PI: float_t = 3.14159265358979323846;

const basis: &'static [&'static str] = &[ "1","e1","e2","e3","e12","e13","e23","e123" ];
const basis_count: usize = basis.len();

#[derive(Default,Debug,Clone,Copy,PartialEq)]
pub struct R120 {
    mvec: [float_t; basis_count]
}

impl R120 {
    pub const fn zero() -> Self {
        Self {
            mvec: [0.0; basis_count]
        }
    }

    pub const fn new(f: float_t, idx: usize) -> Self {
        let mut ret = Self::zero();
        ret.mvec[idx] = f;
        ret
    }
}

// basis vectors are available as global constants.
const e1: R120 = R120::new(1.0, 1);
const e2: R120 = R120::new(1.0, 2);
const e3: R120 = R120::new(1.0, 3);
const e12: R120 = R120::new(1.0, 4);
const e13: R120 = R120::new(1.0, 5);
const e23: R120 = R120::new(1.0, 6);
const e123: R120 = R120::new(1.0, 7);

impl Index<usize> for R120 {
    type Output = float_t;

    fn index<'a>(&'a self, index: usize) -> &'a Self::Output {
        &self.mvec[index]
    }
}

impl IndexMut<usize> for R120 {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut Self::Output {
        &mut self.mvec[index]
    }
}

impl fmt::Display for R120 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut n = 0;
        let ret = self.mvec.iter().enumerate().filter_map(|(i, &coeff)| {
            if coeff > 0.00001 || coeff < -0.00001 {
                n = 1;
                Some(format!("{}{}", 
                        format!("{:.*}", 7, coeff).trim_end_matches('0').trim_end_matches('.'),
                        if i > 0 { basis[i] } else { "" }
                    )
                )
            } else {
                None
            }
        }).collect::<Vec<String>>().join(" + ");
        if n==0 { write!(f,"0") } else { write!(f, "{}", ret) }
    }
}

// Reverse
// Reverse the order of the basis blades.
impl R120 {
    pub fn Reverse(self: Self) -> R120 {
        let mut res = R120::zero();
        let a = self;
        res[0]=a[0];
        res[1]=a[1];
        res[2]=a[2];
        res[3]=a[3];
        res[4]=-a[4];
        res[5]=-a[5];
        res[6]=-a[6];
        res[7]=-a[7];
        res
    }
}

// Dual
// Poincare duality operator.
impl R120 {
    pub fn Dual(self: Self) -> R120 {
        let mut res = R120::zero();
        let a = self;
        res[0]=-a[7];
        res[1]=-a[6];
        res[2]=-a[5];
        res[3]=a[4];
        res[4]=-a[3];
        res[5]=a[2];
        res[6]=a[1];
        res[7]=a[0];
        res
    }
}

impl Not for R120 {
    type Output = R120;

    fn not(self: Self) -> R120 {
        let mut res = R120::zero();
        let a = self;
        res[0]=-a[7];
        res[1]=-a[6];
        res[2]=-a[5];
        res[3]=a[4];
        res[4]=-a[3];
        res[5]=a[2];
        res[6]=a[1];
        res[7]=a[0];
        res
    }
}

// Conjugate
// Clifford Conjugation
impl R120 {
    pub fn Conjugate(self: Self) -> R120 {
        let mut res = R120::zero();
        let a = self;
        res[0]=a[0];
        res[1]=-a[1];
        res[2]=-a[2];
        res[3]=-a[3];
        res[4]=-a[4];
        res[5]=-a[5];
        res[6]=-a[6];
        res[7]=a[7];
        res
    }
}

// Involute
// Main involution
impl R120 {
    pub fn Involute(self: Self) -> R120 {
        let mut res = R120::zero();
        let a = self;
        res[0]=a[0];
        res[1]=-a[1];
        res[2]=-a[2];
        res[3]=-a[3];
        res[4]=a[4];
        res[5]=a[5];
        res[6]=a[6];
        res[7]=-a[7];
        res
    }
}

// Mul
// The geometric product.
impl Mul for R120 {
    type Output = R120;

    fn mul(self: R120, b: R120) -> R120 {
        let mut res = R120::zero();
        let a = self;
        res[0]=b[0]*a[0]+b[1]*a[1]-b[2]*a[2]-b[3]*a[3]+b[4]*a[4]+b[5]*a[5]-b[6]*a[6]-b[7]*a[7];
		res[1]=b[1]*a[0]+b[0]*a[1]+b[4]*a[2]+b[5]*a[3]-b[2]*a[4]-b[3]*a[5]-b[7]*a[6]-b[6]*a[7];
		res[2]=b[2]*a[0]+b[4]*a[1]+b[0]*a[2]+b[6]*a[3]-b[1]*a[4]-b[7]*a[5]-b[3]*a[6]-b[5]*a[7];
		res[3]=b[3]*a[0]+b[5]*a[1]-b[6]*a[2]+b[0]*a[3]+b[7]*a[4]-b[1]*a[5]+b[2]*a[6]+b[4]*a[7];
		res[4]=b[4]*a[0]+b[2]*a[1]-b[1]*a[2]-b[7]*a[3]+b[0]*a[4]+b[6]*a[5]-b[5]*a[6]-b[3]*a[7];
		res[5]=b[5]*a[0]+b[3]*a[1]+b[7]*a[2]-b[1]*a[3]-b[6]*a[4]+b[0]*a[5]+b[4]*a[6]+b[2]*a[7];
		res[6]=b[6]*a[0]+b[7]*a[1]+b[3]*a[2]-b[2]*a[3]-b[5]*a[4]+b[4]*a[5]+b[0]*a[6]+b[1]*a[7];
		res[7]=b[7]*a[0]+b[6]*a[1]-b[5]*a[2]+b[4]*a[3]+b[3]*a[4]-b[2]*a[5]+b[1]*a[6]+b[0]*a[7];
        res
    }
}

// Wedge
// The outer product. (MEET)
impl BitXor for R120 {
    type Output = R120;

    fn bitxor(self: R120, b: R120) -> R120 {
        let mut res = R120::zero();
        let a = self;
        res[0]=b[0]*a[0];
		res[1]=b[1]*a[0]+b[0]*a[1];
		res[2]=b[2]*a[0]+b[0]*a[2];
		res[3]=b[3]*a[0]+b[0]*a[3];
		res[4]=b[4]*a[0]+b[2]*a[1]-b[1]*a[2]+b[0]*a[4];
		res[5]=b[5]*a[0]+b[3]*a[1]-b[1]*a[3]+b[0]*a[5];
		res[6]=b[6]*a[0]+b[3]*a[2]-b[2]*a[3]+b[0]*a[6];
		res[7]=b[7]*a[0]+b[6]*a[1]-b[5]*a[2]+b[4]*a[3]+b[3]*a[4]-b[2]*a[5]+b[1]*a[6]+b[0]*a[7];
        res
    }
}

// Vee
// The regressive product. (JOIN)
impl BitAnd for R120 {
    type Output = R120;

    fn bitand(self: R120, b: R120) -> R120 {
        let mut res = R120::zero();
        let a = self;
        res[7]=1.0*(a[7]*b[7]);
		res[6]=1.0*(a[6]*b[7]+a[7]*b[6]);
		res[5]=-1.0*(a[5]*-1.0*b[7]+a[7]*b[5]*-1.0);
		res[4]=1.0*(a[4]*b[7]+a[7]*b[4]);
		res[3]=1.0*(a[3]*b[7]+a[5]*-1.0*b[6]-a[6]*b[5]*-1.0+a[7]*b[3]);
		res[2]=-1.0*(a[2]*-1.0*b[7]+a[4]*b[6]-a[6]*b[4]+a[7]*b[2]*-1.0);
		res[1]=1.0*(a[1]*b[7]+a[4]*b[5]*-1.0-a[5]*-1.0*b[4]+a[7]*b[1]);
		res[0]=1.0*(a[0]*b[7]+a[1]*b[6]-a[2]*-1.0*b[5]*-1.0+a[3]*b[4]+a[4]*b[3]-a[5]*-1.0*b[2]*-1.0+a[6]*b[1]+a[7]*b[0]);
        res
    }
}

// Dot
// The inner product.
impl BitOr for R120 {
    type Output = R120;

    fn bitor(self: R120, b: R120) -> R120 {
        let mut res = R120::zero();
        let a = self;
        res[0]=b[0]*a[0]+b[1]*a[1]-b[2]*a[2]-b[3]*a[3]+b[4]*a[4]+b[5]*a[5]-b[6]*a[6]-b[7]*a[7];
		res[1]=b[1]*a[0]+b[0]*a[1]+b[4]*a[2]+b[5]*a[3]-b[2]*a[4]-b[3]*a[5]-b[7]*a[6]-b[6]*a[7];
		res[2]=b[2]*a[0]+b[4]*a[1]+b[0]*a[2]+b[6]*a[3]-b[1]*a[4]-b[7]*a[5]-b[3]*a[6]-b[5]*a[7];
		res[3]=b[3]*a[0]+b[5]*a[1]-b[6]*a[2]+b[0]*a[3]+b[7]*a[4]-b[1]*a[5]+b[2]*a[6]+b[4]*a[7];
		res[4]=b[4]*a[0]-b[7]*a[3]+b[0]*a[4]-b[3]*a[7];
		res[5]=b[5]*a[0]+b[7]*a[2]+b[0]*a[5]+b[2]*a[7];
		res[6]=b[6]*a[0]+b[7]*a[1]+b[0]*a[6]+b[1]*a[7];
		res[7]=b[7]*a[0]+b[0]*a[7];
        res
    }
}

// Add
// Multivector addition
impl Add for R120 {
    type Output = R120;

    fn add(self: R120, b: R120) -> R120 {
        let mut res = R120::zero();
        let a = self;
        res[0] = a[0]+b[0];
		res[1] = a[1]+b[1];
		res[2] = a[2]+b[2];
		res[3] = a[3]+b[3];
		res[4] = a[4]+b[4];
		res[5] = a[5]+b[5];
		res[6] = a[6]+b[6];
		res[7] = a[7]+b[7];
        res
    }
}

// Sub
// Multivector subtraction
impl Sub for R120 {
    type Output = R120;

    fn sub(self: R120, b: R120) -> R120 {
        let mut res = R120::zero();
        let a = self;
        res[0] = a[0]-b[0];
		res[1] = a[1]-b[1];
		res[2] = a[2]-b[2];
		res[3] = a[3]-b[3];
		res[4] = a[4]-b[4];
		res[5] = a[5]-b[5];
		res[6] = a[6]-b[6];
		res[7] = a[7]-b[7];
        res
    }
}

// smul
// scalar/multivector multiplication
impl Mul<R120> for float_t {
    type Output = R120;

    fn mul(self: float_t, b: R120) -> R120 {
        let mut res = R120::zero();
        let a = self;
        res[0] = a*b[0];
        res[1] = a*b[1];
        res[2] = a*b[2];
        res[3] = a*b[3];
        res[4] = a*b[4];
        res[5] = a*b[5];
        res[6] = a*b[6];
        res[7] = a*b[7];
        res
    }
}

// muls
// multivector/scalar multiplication
impl Mul<float_t> for R120 {
    type Output = R120;

    fn mul(self: R120, b: float_t) -> R120 {
        let mut res = R120::zero();
        let a = self;
        res[0] = a[0]*b;
        res[1] = a[1]*b;
        res[2] = a[2]*b;
        res[3] = a[3]*b;
        res[4] = a[4]*b;
        res[5] = a[5]*b;
        res[6] = a[6]*b;
        res[7] = a[7]*b;
        res
    }
    }

// sadd
// scalar/multivector addition
impl Add<R120> for float_t {
    type Output = R120;

    fn add(self: float_t, b: R120) -> R120 {
        let mut res = R120::zero();
        let a = self;
        res[0] = a+b[0];
        res[1] = b[1];
        res[2] = b[2];
        res[3] = b[3];
        res[4] = b[4];
        res[5] = b[5];
        res[6] = b[6];
        res[7] = b[7];
        res
    }
}

// adds
// multivector/scalar addition
impl Add<float_t> for R120 {
    type Output = R120;

    fn add(self: R120, b: float_t) -> R120 {
        let mut res = R120::zero();
        let a = self;
        res[0] = a[0]+b;
        res[1] = a[1];
        res[2] = a[2];
        res[3] = a[3];
        res[4] = a[4];
        res[5] = a[5];
        res[6] = a[6];
        res[7] = a[7];
        res
    }
    }

// // ssub
// // scalar/multivector subtraction
// impl ssub<R120> for float_t {
//     type Output = R120;

//     fn ssub(self: float_t, b: R120) -> R120 {
//         let mut res = R120::zero();
//         let a = self;
//         res[0] = a-b[0];
//         res[1] = -b[1];
//         res[2] = -b[2];
//         res[3] = -b[3];
//         res[4] = -b[4];
//         res[5] = -b[5];
//         res[6] = -b[6];
//         res[7] = -b[7];
//         res
//     }
// }

// // subs
// // multivector/scalar subtraction
// impl subs<R120> for float_t {
//     type Output = R120;

//     fn subs(self: float_t, b: R120) -> R120 {
//         let mut res = R120::zero();
//         let a = self;
//         res[0] = a[0]-b;
//         res[1] = a[1];
//         res[2] = a[2];
//         res[3] = a[3];
//         res[4] = a[4];
//         res[5] = a[5];
//         res[6] = a[6];
//         res[7] = a[7];
//         res
//     }
// }

impl R120 {
    pub fn norm(self: Self) -> float_t {
        let scalar_part = (self * self.Conjugate())[0];

        scalar_part.abs().sqrt()
    }

    pub fn inorm(self: Self) -> float_t {
        self.Dual().norm()
    }

    // Modified to have 0 normalize to 0
    pub fn normalized(self: Self) -> Self {
        let norm = self.norm();
        if norm != 0.0 {
            self * (1.0 / self.norm())
        } else {
            self * 0.0
        }
    }
    
    

}