use ndarray::{ArrayD, Axis, Ix1, Ix2, IxDyn};
use ndarray_rand::rand_distr::StandardNormal;
use ndarray_rand::RandomExt;
use rand::thread_rng;

pub type TensorDtype = f32;

#[derive(Debug, Clone, PartialEq)]
pub struct Tensor {
    data: ArrayD<TensorDtype>,
}

pub type Shape = Vec<usize>;

impl Tensor {
    pub fn zeros(shape: &[usize]) -> Self {
        Self {
            data: ArrayD::zeros(IxDyn(shape)),
        }
    }

    pub fn ones(shape: &[usize]) -> Self {
        Self {
            data: ArrayD::ones(IxDyn(shape)),
        }
    }

    pub fn randn(shape: &[usize]) -> Self {
        let mut rng = thread_rng();
        Self {
            data: ArrayD::random_using(IxDyn(shape), StandardNormal, &mut rng),
        }
    }

    pub fn from_vec(vec: Vec<TensorDtype>, shape: &[usize]) -> Self {
        Self {
            data: ArrayD::from_shape_vec(IxDyn(shape), vec).expect("Shape inválido"),
        }
    }

    pub fn scalar(value: TensorDtype) -> Self {
        Self {
            data: ArrayD::from_elem(IxDyn(&[]), value),
        }
    }

    pub fn shape(&self) -> Vec<usize> {
        self.data.shape().to_vec()
    }

    pub fn ndim(&self) -> usize {
        self.data.ndim()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, indices: &[usize]) -> TensorDtype {
        self.data[indices].clone()
    }

    pub fn set(&mut self, indices: &[usize], value: TensorDtype) {
        self.data[indices] = value;
    }

    pub fn slice(&self, idx: usize) -> Self {
        self.slice_axis(0, idx)
    }

    pub fn slice_axis(&self, axis: usize, idx: usize) -> Self {
        let mut slice_spec = Vec::new();
        for i in 0..self.ndim() {
            if i == axis {
                slice_spec.push(ndarray::SliceInfoElem::Index(idx as isize));
            } else {
                slice_spec.push(ndarray::SliceInfoElem::Slice { start: 0, end: None, step: 1 });
            }
        }
        let sliced = self.data.slice(slice_spec.as_slice());
        Self {
            data: sliced.to_owned().into_dyn(),
        }
    }

    pub fn reshape(&self, shape: &[usize]) -> Self {
        Self {
            data: self.data.clone().into_shape(IxDyn(shape)).expect("Reshape falhou"),
        }
    }

    pub fn flatten(&self) -> Self {
        Self {
            data: self.data.clone().into_shape(IxDyn(&[self.len()])).expect("Flatten falhou"),
        }
    }

    pub fn t(&self) -> Self {
        let mut axes: Vec<usize> = (0..self.ndim()).collect();
        axes.reverse();
        Self {
            data: self.data.clone().permuted_axes(axes)
        }
    }

    pub fn to_vec(&self) -> Vec<TensorDtype> {
        self.data.iter().copied().collect()
    }

    pub fn as_array2(&self) -> ndarray::ArrayView2<'_, TensorDtype> {
        if self.ndim() == 3 {
            self.data.view().into_shape((self.shape()[0] * self.shape()[1], self.shape()[2])).unwrap()
        } else if self.ndim() == 1 {
            self.data.view().into_shape((1, self.shape()[0])).unwrap()
        } else {
            self.data.view().into_dimensionality::<Ix2>().expect("Tensor não é 2D")
        }
    }

    pub fn as_array1(&self) -> ndarray::ArrayView1<'_, TensorDtype> {
        self.data.view().into_dimensionality::<Ix1>().expect("Tensor não é 1D")
    }

    pub fn matmul(&self, other: &Self) -> Self {
        let a = self.as_array2();
        let b = other.as_array2();
        let result = a.dot(&b);
        Self {
            data: result.into_dyn(),
        }
    }

    pub fn mul_elem(&self, other: &Self) -> Self {
        Self {
            data: &self.data * &other.data,
        }
    }

    pub fn div_elem(&self, other: &Self) -> Self {
        Self {
            data: &self.data / &other.data,
        }
    }

    pub fn add_elem(&self, other: &Self) -> Self {
        Self {
            data: &self.data + &other.data,
        }
    }

    pub fn sub_elem(&self, other: &Self) -> Self {
        Self {
            data: &self.data - &other.data,
        }
    }

    pub fn scale(&self, scalar: TensorDtype) -> Self {
        Self {
            data: &self.data * scalar,
        }
    }

    pub fn div_scalar(&self, scalar: TensorDtype) -> Self {
        Self {
            data: &self.data / scalar,
        }
    }

    pub fn add_scalar(&self, scalar: TensorDtype) -> Self {
        Self {
            data: &self.data + scalar,
        }
    }

    pub fn clamp(&self, min: TensorDtype, max: TensorDtype) -> Self {
        Self {
            data: self.data.mapv(|v| v.clamp(min, max)),
        }
    }

    pub fn mapv(&self, f: impl Fn(TensorDtype) -> TensorDtype) -> Self {
        Self {
            data: self.data.mapv(f),
        }
    }

    pub fn sum_axis(&self, axis: usize) -> Self {
        Self {
            data: self.data.sum_axis(Axis(axis)),
        }
    }

    pub fn mean_axis(&self, axis: usize) -> Self {
        let sum = self.sum_axis(axis);
        let count = self.shape()[axis] as TensorDtype;
        sum.scale(1.0 / count)
    }

    pub fn sum_all(&self) -> TensorDtype {
        self.data.sum()
    }

    pub fn mean_all(&self) -> TensorDtype {
        self.data.mean().unwrap_or(0.0)
    }

    pub fn max_axis(&self, axis: usize) -> Self {
        Self {
            data: self.data.map_axis(Axis(axis), |view| {
                view.iter().copied().fold(TensorDtype::NEG_INFINITY, TensorDtype::max)
            }),
        }
    }

    pub fn argmax_axis(&self, axis: usize) -> Vec<usize> {
        self.data.axis_iter(Axis(axis))
            .map(|view| {
                view.iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .map(|(idx, _)| idx)
                    .unwrap_or(0)
            })
            .collect()
    }

    pub fn topk(&self, k: usize, axis: usize) -> Vec<(usize, TensorDtype)> {
        let mut result = Vec::new();
        for slice in self.data.axis_iter(Axis(axis)) {
            let mut indexed: Vec<(usize, TensorDtype)> = slice.iter()
                .enumerate()
                .map(|(i, &v)| (i, v))
                .collect();
            indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            result.extend(indexed.into_iter().take(k));
        }
        result
    }

    pub fn rms_norm(&self, eps: TensorDtype) -> Self {
        let mean_sq = self.data.mapv(|v| v * v).mean().unwrap_or(1.0);
        let norm = (mean_sq + eps).sqrt();
        self.scale(1.0 / norm)
    }

    pub fn layer_norm(&self, eps: TensorDtype) -> Self {
        let mean = self.mean_all();
        let var = self.data.mapv(|v| (v - mean).powi(2)).mean().unwrap_or(1.0);
        let std = (var + eps).sqrt();
        self.mapv(|v| (v - mean) / std)
    }

    pub fn sigmoid(&self) -> Self {
        self.mapv(|v| 1.0 / (1.0 + (-v).exp()))
    }

    pub fn relu(&self) -> Self {
        self.mapv(|v| v.max(0.0))
    }

    pub fn gelu(&self) -> Self {
        self.mapv(|v| {
            let cdf = 0.5 * (1.0 + (v * 0.7978845608 * (1.0 + 0.044715 * v * v)).tanh());
            v * cdf
        })
    }

    pub fn swiglu_clamp(&self, gate: &Self, up: &Self, clamp_limit: TensorDtype) -> Self {
        let g = gate.clamp(-clamp_limit, clamp_limit);
        let u = up.clamp(-clamp_limit, clamp_limit);
        let sig_g = g.sigmoid();
        g.mul_elem(&sig_g).mul_elem(&u)
    }

    pub fn softmax(&self, axis: usize) -> Self {
        let max_val = self.max_axis(axis);
        let shifted = self.sub_elem(&max_val);
        let exp = shifted.mapv(|v| v.exp());
        let sum_exp = exp.sum_axis(axis);
        exp.div_elem(&sum_exp)
    }

    pub fn concat(tensors: &[&Self], axis: usize) -> Self {
        let arrays: Vec<_> = tensors.iter().map(|t| t.data.view()).collect();
        Self {
            data: ndarray::concatenate(Axis(axis), &arrays).expect("Concatenação falhou").into_owned().into_dyn(),
        }
    }
}
