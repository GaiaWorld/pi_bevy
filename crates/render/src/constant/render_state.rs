

use pi_render::rhi::sampler::{EAddressMode, EFilterMode, EAnisotropyClamp, SamplerDesc};
use wasm_bindgen::prelude::wasm_bindgen;

pub enum StencilOperation {
    /// Keep stencil value unchanged.
    /// #[default]
    Keep = 0,
    /// Set stencil value to zero.
    Zero = 1,
    /// Replace stencil value with value provided in most recent call to
    /// [`RenderPass::set_stencil_reference`][RPssr].
    ///
    /// [RPssr]: ../wgpu/struct.RenderPass.html#method.set_stencil_reference
    Replace = 2,
    /// Bitwise inverts stencil value.
    Invert = 3,
    /// Increments stencil value by one, clamping on overflow.
    IncrementClamp = 4,
    /// Decrements stencil value by one, clamping on underflow.
    DecrementClamp = 5,
    /// Increments stencil value by one, wrapping on overflow.
    IncrementWrap = 6,
    /// Decrements stencil value by one, wrapping on underflow.
    DecrementWrap = 7,
}
impl StencilOperation {
    pub fn val(&self) -> wgpu::StencilOperation {
        match self {
            StencilOperation::Keep               => wgpu::StencilOperation::Keep               ,
            StencilOperation::Zero               => wgpu::StencilOperation::Zero               ,
            StencilOperation::Replace            => wgpu::StencilOperation::Replace            ,
            StencilOperation::Invert             => wgpu::StencilOperation::Invert             ,
            StencilOperation::IncrementClamp     => wgpu::StencilOperation::IncrementClamp     ,
            StencilOperation::DecrementClamp     => wgpu::StencilOperation::DecrementClamp     ,
            StencilOperation::IncrementWrap      => wgpu::StencilOperation::IncrementWrap      ,
            StencilOperation::DecrementWrap      => wgpu::StencilOperation::DecrementWrap      ,
        }
    }
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[cfg(feature = "pi_js_export")]
pub enum BlendFactor {
    /// 0.0
    Zero = 0,
    /// 1.0
    One = 1,
    /// S.component
    Src = 2,
    /// 1.0 - S.component
    OneMinusSrc = 3,
    /// S.alpha
    SrcAlpha = 4,
    /// 1.0 - S.alpha
    OneMinusSrcAlpha = 5,
    /// D.component
    Dst = 6,
    /// 1.0 - D.component
    OneMinusDst = 7,
    /// D.alpha
    DstAlpha = 8,
    /// 1.0 - D.alpha
    OneMinusDstAlpha = 9,
    /// min(S.alpha, 1.0 - D.alpha)
    SrcAlphaSaturated = 10,
    /// Constant
    Constant = 11,
    /// 1.0 - Constant
    OneMinusConstant = 12,
}
pub enum BlendFactor {
    /// 0.0
    Zero = 0,
    /// 1.0
    One = 1,
    /// S.component
    Src = 2,
    /// 1.0 - S.component
    OneMinusSrc = 3,
    /// S.alpha
    SrcAlpha = 4,
    /// 1.0 - S.alpha
    OneMinusSrcAlpha = 5,
    /// D.component
    Dst = 6,
    /// 1.0 - D.component
    OneMinusDst = 7,
    /// D.alpha
    DstAlpha = 8,
    /// 1.0 - D.alpha
    OneMinusDstAlpha = 9,
    /// min(S.alpha, 1.0 - D.alpha)
    SrcAlphaSaturated = 10,
    /// Constant
    Constant = 11,
    /// 1.0 - Constant
    OneMinusConstant = 12,
}
impl BlendFactor {
    pub fn val(&self) -> wgpu::BlendFactor {
        match self {
            BlendFactor::Zero               => wgpu::BlendFactor::Zero               ,
            BlendFactor::One                => wgpu::BlendFactor::One                ,
            BlendFactor::Src                => wgpu::BlendFactor::Src                ,
            BlendFactor::OneMinusSrc        => wgpu::BlendFactor::OneMinusSrc        ,
            BlendFactor::SrcAlpha           => wgpu::BlendFactor::SrcAlpha           ,
            BlendFactor::OneMinusSrcAlpha   => wgpu::BlendFactor::OneMinusSrcAlpha   ,
            BlendFactor::Dst                => wgpu::BlendFactor::Dst                ,
            BlendFactor::OneMinusDst        => wgpu::BlendFactor::OneMinusDst        ,
            BlendFactor::DstAlpha           => wgpu::BlendFactor::DstAlpha           ,
            BlendFactor::OneMinusDstAlpha   => wgpu::BlendFactor::OneMinusDstAlpha   ,
            BlendFactor::SrcAlphaSaturated  => wgpu::BlendFactor::SrcAlphaSaturated  ,
            BlendFactor::Constant           => wgpu::BlendFactor::Constant           ,
            BlendFactor::OneMinusConstant   => wgpu::BlendFactor::OneMinusConstant   ,
        }
    }
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[cfg(feature = "pi_js_export")]
pub enum TextureFormat {
    RGBA,
    RGB,
}
pub enum TextureFormat {
    RGBA,
    RGB,
}
impl TextureFormat {
    pub fn val(&self) -> wgpu::TextureFormat {
        match self {
            TextureFormat::RGBA => wgpu::TextureFormat::Rgba8UnormSrgb,
            TextureFormat::RGB => wgpu::TextureFormat::Rgba8UnormSrgb,
        }
    }
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[cfg(feature = "pi_js_export")]
pub enum EColorSpace {
    GAMMA,
    LINEAR,
}
pub enum EColorSpace {
    GAMMA,
    LINEAR,
}
impl EColorSpace {
    pub fn target_color_format(&self) -> wgpu::TextureFormat {
        match self {
            EColorSpace::GAMMA => wgpu::TextureFormat::Rgba8UnormSrgb,
            EColorSpace::LINEAR => wgpu::TextureFormat::Rgba8Unorm,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[cfg(feature = "pi_js_export")]
pub enum CompareFunction {
    /// Function never passes
    Never = 1,
    /// Function passes if new value less than existing value
    Less = 2,
    /// Function passes if new value is equal to existing value. When using
    /// this compare function, make sure to mark your Vertex Shader's `@builtin(position)`
    /// output as `@invariant` to prevent artifacting.
    Equal = 3,
    /// Function passes if new value is less than or equal to existing value
    LessEqual = 4,
    /// Function passes if new value is greater than existing value
    Greater = 5,
    /// Function passes if new value is not equal to existing value. When using
    /// this compare function, make sure to mark your Vertex Shader's `@builtin(position)`
    /// output as `@invariant` to prevent artifacting.
    NotEqual = 6,
    /// Function passes if new value is greater than or equal to existing value
    GreaterEqual = 7,
    /// Function always passes
    Always = 8,
}
pub enum CompareFunction {
    /// Function never passes
    Never = 1,
    /// Function passes if new value less than existing value
    Less = 2,
    /// Function passes if new value is equal to existing value. When using
    /// this compare function, make sure to mark your Vertex Shader's `@builtin(position)`
    /// output as `@invariant` to prevent artifacting.
    Equal = 3,
    /// Function passes if new value is less than or equal to existing value
    LessEqual = 4,
    /// Function passes if new value is greater than existing value
    Greater = 5,
    /// Function passes if new value is not equal to existing value. When using
    /// this compare function, make sure to mark your Vertex Shader's `@builtin(position)`
    /// output as `@invariant` to prevent artifacting.
    NotEqual = 6,
    /// Function passes if new value is greater than or equal to existing value
    GreaterEqual = 7,
    /// Function always passes
    Always = 8,
}
impl CompareFunction {
    pub fn val(val: Option<&Self>) -> Option<wgpu::CompareFunction> {
        match val {
            Some(val) => {
                match val {
                    CompareFunction::Never          => Some(wgpu::CompareFunction::Never),
                    CompareFunction::Less           => Some(wgpu::CompareFunction::Less),
                    CompareFunction::Equal          => Some(wgpu::CompareFunction::Equal),
                    CompareFunction::LessEqual      => Some(wgpu::CompareFunction::LessEqual),
                    CompareFunction::Greater        => Some(wgpu::CompareFunction::Greater),
                    CompareFunction::NotEqual       => Some(wgpu::CompareFunction::NotEqual),
                    CompareFunction::GreaterEqual   => Some(wgpu::CompareFunction::GreaterEqual),
                    CompareFunction::Always         => Some(wgpu::CompareFunction::Always),
                }
            },
            None => None,
        }
    }
}
