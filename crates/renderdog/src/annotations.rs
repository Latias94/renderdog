use std::{
    ffi::{CString, c_void},
    ptr,
};

use renderdog_sys as sys;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlResourceReference {
    pub identifier: u32,
    pub name: u32,
}

impl GlResourceReference {
    pub const fn new(identifier: u32, name: u32) -> Self {
        Self { identifier, name }
    }

    fn into_sys(self) -> sys::RENDERDOC_GLResourceReference {
        sys::RENDERDOC_GLResourceReference {
            identifier: self.identifier,
            name: self.name,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnnotationTarget {
    Raw(*mut c_void),
    Gl(GlResourceReference),
}

impl From<*mut c_void> for AnnotationTarget {
    fn from(value: *mut c_void) -> Self {
        Self::Raw(value)
    }
}

impl From<GlResourceReference> for AnnotationTarget {
    fn from(value: GlResourceReference) -> Self {
        Self::Gl(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnnotationVector<T>
where
    T: Copy,
{
    D1([T; 1]),
    D2([T; 2]),
    D3([T; 3]),
    D4([T; 4]),
}

impl<T> AnnotationVector<T>
where
    T: Copy + Default,
{
    fn width(self) -> u32 {
        match self {
            Self::D1(_) => 1,
            Self::D2(_) => 2,
            Self::D3(_) => 3,
            Self::D4(_) => 4,
        }
    }

    fn into_array4(self) -> [T; 4] {
        match self {
            Self::D1(values) => copy_into_4(values),
            Self::D2(values) => copy_into_4(values),
            Self::D3(values) => copy_into_4(values),
            Self::D4(values) => values,
        }
    }
}

fn copy_into_4<T, const N: usize>(values: [T; N]) -> [T; 4]
where
    T: Copy + Default,
{
    let mut out = [T::default(); 4];
    out[..N].copy_from_slice(&values);
    out
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnnotationValue<'a> {
    Empty,
    Bool(bool),
    BoolVector(AnnotationVector<bool>),
    Int32(i32),
    Int32Vector(AnnotationVector<i32>),
    UInt32(u32),
    UInt32Vector(AnnotationVector<u32>),
    Int64(i64),
    Int64Vector(AnnotationVector<i64>),
    UInt64(u64),
    UInt64Vector(AnnotationVector<u64>),
    Float32(f32),
    Float32Vector(AnnotationVector<f32>),
    Float64(f64),
    Float64Vector(AnnotationVector<f64>),
    String(&'a str),
    ApiObject(AnnotationTarget),
}

impl From<bool> for AnnotationValue<'_> {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i32> for AnnotationValue<'_> {
    fn from(value: i32) -> Self {
        Self::Int32(value)
    }
}

impl From<u32> for AnnotationValue<'_> {
    fn from(value: u32) -> Self {
        Self::UInt32(value)
    }
}

impl From<i64> for AnnotationValue<'_> {
    fn from(value: i64) -> Self {
        Self::Int64(value)
    }
}

impl From<u64> for AnnotationValue<'_> {
    fn from(value: u64) -> Self {
        Self::UInt64(value)
    }
}

impl From<f32> for AnnotationValue<'_> {
    fn from(value: f32) -> Self {
        Self::Float32(value)
    }
}

impl From<f64> for AnnotationValue<'_> {
    fn from(value: f64) -> Self {
        Self::Float64(value)
    }
}

impl<'a> From<&'a str> for AnnotationValue<'a> {
    fn from(value: &'a str) -> Self {
        Self::String(value)
    }
}

impl From<AnnotationTarget> for AnnotationValue<'_> {
    fn from(value: AnnotationTarget) -> Self {
        Self::ApiObject(value)
    }
}

pub(crate) struct PreparedAnnotationTarget {
    raw: *mut c_void,
    gl: Option<Box<sys::RENDERDOC_GLResourceReference>>,
}

impl PreparedAnnotationTarget {
    pub(crate) fn as_ptr(&mut self) -> *mut c_void {
        if let Some(gl) = self.gl.as_deref_mut() {
            (gl as *mut sys::RENDERDOC_GLResourceReference).cast()
        } else {
            self.raw
        }
    }
}

impl AnnotationTarget {
    pub(crate) fn prepare(self) -> PreparedAnnotationTarget {
        match self {
            Self::Raw(raw) => PreparedAnnotationTarget { raw, gl: None },
            Self::Gl(gl) => PreparedAnnotationTarget {
                raw: ptr::null_mut(),
                gl: Some(Box::new(gl.into_sys())),
            },
        }
    }
}

pub(crate) struct PreparedAnnotationValue {
    pub(crate) value_type: sys::RENDERDOC_AnnotationType,
    pub(crate) vector_width: u32,
    value: Option<sys::RENDERDOC_AnnotationValue>,
    _string: Option<CString>,
    _gl: Option<Box<sys::RENDERDOC_GLResourceReference>>,
}

impl PreparedAnnotationValue {
    pub(crate) fn as_ptr(&self) -> *const sys::RENDERDOC_AnnotationValue {
        self.value
            .as_ref()
            .map_or(ptr::null(), |value| value as *const _)
    }
}

impl<'a> AnnotationValue<'a> {
    pub(crate) fn prepare(self) -> Result<PreparedAnnotationValue, std::ffi::NulError> {
        match self {
            Self::Empty => Ok(PreparedAnnotationValue {
                value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_Empty,
                vector_width: 0,
                value: None,
                _string: None,
                _gl: None,
            }),
            Self::Bool(value) => Ok(PreparedAnnotationValue {
                value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_Bool,
                vector_width: 1,
                value: Some(sys::RENDERDOC_AnnotationValue { boolean: value }),
                _string: None,
                _gl: None,
            }),
            Self::BoolVector(value) => Ok(PreparedAnnotationValue {
                value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_Bool,
                vector_width: value.width(),
                value: Some(sys::RENDERDOC_AnnotationValue {
                    vector: sys::RENDERDOC_AnnotationVectorValue {
                        boolean: value.into_array4(),
                    },
                }),
                _string: None,
                _gl: None,
            }),
            Self::Int32(value) => Ok(PreparedAnnotationValue {
                value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_Int32,
                vector_width: 1,
                value: Some(sys::RENDERDOC_AnnotationValue { int32: value }),
                _string: None,
                _gl: None,
            }),
            Self::Int32Vector(value) => Ok(PreparedAnnotationValue {
                value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_Int32,
                vector_width: value.width(),
                value: Some(sys::RENDERDOC_AnnotationValue {
                    vector: sys::RENDERDOC_AnnotationVectorValue {
                        int32: value.into_array4(),
                    },
                }),
                _string: None,
                _gl: None,
            }),
            Self::UInt32(value) => Ok(PreparedAnnotationValue {
                value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_UInt32,
                vector_width: 1,
                value: Some(sys::RENDERDOC_AnnotationValue { uint32: value }),
                _string: None,
                _gl: None,
            }),
            Self::UInt32Vector(value) => Ok(PreparedAnnotationValue {
                value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_UInt32,
                vector_width: value.width(),
                value: Some(sys::RENDERDOC_AnnotationValue {
                    vector: sys::RENDERDOC_AnnotationVectorValue {
                        uint32: value.into_array4(),
                    },
                }),
                _string: None,
                _gl: None,
            }),
            Self::Int64(value) => Ok(PreparedAnnotationValue {
                value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_Int64,
                vector_width: 1,
                value: Some(sys::RENDERDOC_AnnotationValue { int64: value }),
                _string: None,
                _gl: None,
            }),
            Self::Int64Vector(value) => Ok(PreparedAnnotationValue {
                value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_Int64,
                vector_width: value.width(),
                value: Some(sys::RENDERDOC_AnnotationValue {
                    vector: sys::RENDERDOC_AnnotationVectorValue {
                        int64: value.into_array4(),
                    },
                }),
                _string: None,
                _gl: None,
            }),
            Self::UInt64(value) => Ok(PreparedAnnotationValue {
                value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_UInt64,
                vector_width: 1,
                value: Some(sys::RENDERDOC_AnnotationValue { uint64: value }),
                _string: None,
                _gl: None,
            }),
            Self::UInt64Vector(value) => Ok(PreparedAnnotationValue {
                value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_UInt64,
                vector_width: value.width(),
                value: Some(sys::RENDERDOC_AnnotationValue {
                    vector: sys::RENDERDOC_AnnotationVectorValue {
                        uint64: value.into_array4(),
                    },
                }),
                _string: None,
                _gl: None,
            }),
            Self::Float32(value) => Ok(PreparedAnnotationValue {
                value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_Float,
                vector_width: 1,
                value: Some(sys::RENDERDOC_AnnotationValue { float32: value }),
                _string: None,
                _gl: None,
            }),
            Self::Float32Vector(value) => Ok(PreparedAnnotationValue {
                value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_Float,
                vector_width: value.width(),
                value: Some(sys::RENDERDOC_AnnotationValue {
                    vector: sys::RENDERDOC_AnnotationVectorValue {
                        float32: value.into_array4(),
                    },
                }),
                _string: None,
                _gl: None,
            }),
            Self::Float64(value) => Ok(PreparedAnnotationValue {
                value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_Double,
                vector_width: 1,
                value: Some(sys::RENDERDOC_AnnotationValue { float64: value }),
                _string: None,
                _gl: None,
            }),
            Self::Float64Vector(value) => Ok(PreparedAnnotationValue {
                value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_Double,
                vector_width: value.width(),
                value: Some(sys::RENDERDOC_AnnotationValue {
                    vector: sys::RENDERDOC_AnnotationVectorValue {
                        float64: value.into_array4(),
                    },
                }),
                _string: None,
                _gl: None,
            }),
            Self::String(value) => {
                let string = CString::new(value)?;
                let ptr = string.as_ptr();
                Ok(PreparedAnnotationValue {
                    value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_String,
                    vector_width: 1,
                    value: Some(sys::RENDERDOC_AnnotationValue { string: ptr }),
                    _string: Some(string),
                    _gl: None,
                })
            }
            Self::ApiObject(target) => match target {
                AnnotationTarget::Raw(raw) => Ok(PreparedAnnotationValue {
                    value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_APIObject,
                    vector_width: 1,
                    value: Some(sys::RENDERDOC_AnnotationValue { apiObject: raw }),
                    _string: None,
                    _gl: None,
                }),
                AnnotationTarget::Gl(gl) => {
                    let gl = Box::new(gl.into_sys());
                    let ptr = (gl.as_ref() as *const sys::RENDERDOC_GLResourceReference).cast_mut();
                    Ok(PreparedAnnotationValue {
                        value_type: sys::RENDERDOC_AnnotationType::eRENDERDOC_APIObject,
                        vector_width: 1,
                        value: Some(sys::RENDERDOC_AnnotationValue {
                            apiObject: ptr.cast(),
                        }),
                        _string: None,
                        _gl: Some(gl),
                    })
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annotation_vector_is_padded_to_four_elements() {
        let prepared = AnnotationValue::UInt32Vector(AnnotationVector::D3([1, 2, 3]))
            .prepare()
            .expect("prepare value");
        let value = prepared.value.expect("annotation value");
        let vector = unsafe { value.vector };
        assert_eq!(prepared.vector_width, 3);
        assert_eq!(unsafe { vector.uint32 }, [1, 2, 3, 0]);
    }

    #[test]
    fn gl_annotation_target_produces_pointer() {
        let mut prepared = AnnotationTarget::Gl(GlResourceReference::new(0x82E1, 7)).prepare();
        assert!(!prepared.as_ptr().is_null());
    }
}
