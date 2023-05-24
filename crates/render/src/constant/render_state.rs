

use pi_render::rhi::sampler::{EAddressMode, EFilterMode, EAnisotropyClamp, SamplerDesc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BlendOperation {
    /// Src + Dst
    Add,
    /// Src - Dst
    Subtract,
    /// Dst - Src
    ReverseSubtract,
    /// min(Src, Dst)
    Min,
    /// max(Src, Dst)
    Max,
}
impl BlendOperation {
    pub fn val(&self) -> wgpu::BlendOperation {
        match self {
            BlendOperation::Add                 => wgpu::BlendOperation::Add            ,
            BlendOperation::Subtract            => wgpu::BlendOperation::Subtract       ,
            BlendOperation::ReverseSubtract     => wgpu::BlendOperation::ReverseSubtract,
            BlendOperation::Min                 => wgpu::BlendOperation::Min            ,
            BlendOperation::Max                 => wgpu::BlendOperation::Max            ,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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
    pub fn val2(&self) -> wgpu::CompareFunction {
        match self {
            CompareFunction::Never          => wgpu::CompareFunction::Never,
            CompareFunction::Less           => wgpu::CompareFunction::Less,
            CompareFunction::Equal          => wgpu::CompareFunction::Equal,
            CompareFunction::LessEqual      => wgpu::CompareFunction::LessEqual,
            CompareFunction::Greater        => wgpu::CompareFunction::Greater,
            CompareFunction::NotEqual       => wgpu::CompareFunction::NotEqual,
            CompareFunction::GreaterEqual   => wgpu::CompareFunction::GreaterEqual,
            CompareFunction::Always         => wgpu::CompareFunction::Always,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct StencilFaceState {
    /// Comparison function that determines if the fail_op or pass_op is used on the stencil buffer.
    pub compare: CompareFunction,
    /// Operation that is preformed when stencil test fails.
    pub fail_op: StencilOperation,
    /// Operation that is performed when depth test fails but stencil test succeeds.
    pub depth_fail_op: StencilOperation,
    /// Operation that is performed when stencil test success.
    pub pass_op: StencilOperation,
}
impl StencilFaceState {
    pub const IGNORE: Self = StencilFaceState {
        compare: CompareFunction::Always,
        fail_op: StencilOperation::Keep,
        depth_fail_op: StencilOperation::Keep,
        pass_op: StencilOperation::Keep,
    };
    pub fn val(&self) -> wgpu::StencilFaceState {
        wgpu::StencilFaceState {
            compare: self.compare.val2(),
            fail_op: self.fail_op.val(),
            depth_fail_op: self.depth_fail_op.val(),
            pass_op: self.pass_op.val(),
        }
    }
}


#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[cfg(feature = "pi_js_export")]
pub enum CullMode {
    Off,
    Back,
    Front
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullMode {
    Off,
    Back,
    Front
}
impl CullMode {
    pub fn val(&self) -> Option<wgpu::Face> {
        match self {
            CullMode::Off => None,
            CullMode::Back => Some(wgpu::Face::Back),
            CullMode::Front => Some(wgpu::Face::Front),
        }
    }
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[cfg(feature = "pi_js_export")]
pub enum PrimitiveTopology {
    /// Vertex data is a list of points. Each vertex is a new point.
    PointList = 0,
    /// Vertex data is a list of lines. Each pair of vertices composes a new line.
    ///
    /// Vertices `0 1 2 3` create two lines `0 1` and `2 3`
    LineList = 1,
    /// Vertex data is a strip of lines. Each set of two adjacent vertices form a line.
    ///
    /// Vertices `0 1 2 3` create three lines `0 1`, `1 2`, and `2 3`.
    LineStrip = 2,
    /// Vertex data is a list of triangles. Each set of 3 vertices composes a new triangle.
    ///
    /// Vertices `0 1 2 3 4 5` create two triangles `0 1 2` and `3 4 5`
    TriangleList = 3,
    /// Vertex data is a triangle strip. Each set of three adjacent vertices form a triangle.
    ///
    /// Vertices `0 1 2 3 4 5` creates four triangles `0 1 2`, `2 1 3`, `2 3 4`, and `4 3 5`
    TriangleStrip = 4,
}
/// * 默认值 Fill
#[derive(Debug, Clone, Copy)]
pub enum PrimitiveTopology {
    /// Vertex data is a list of points. Each vertex is a new point.
    PointList = 0,
    /// Vertex data is a list of lines. Each pair of vertices composes a new line.
    ///
    /// Vertices `0 1 2 3` create two lines `0 1` and `2 3`
    LineList = 1,
    /// Vertex data is a strip of lines. Each set of two adjacent vertices form a line.
    ///
    /// Vertices `0 1 2 3` create three lines `0 1`, `1 2`, and `2 3`.
    LineStrip = 2,
    /// Vertex data is a list of triangles. Each set of 3 vertices composes a new triangle.
    ///
    /// Vertices `0 1 2 3 4 5` create two triangles `0 1 2` and `3 4 5`
    TriangleList = 3,
    /// Vertex data is a triangle strip. Each set of three adjacent vertices form a triangle.
    ///
    /// Vertices `0 1 2 3 4 5` creates four triangles `0 1 2`, `2 1 3`, `2 3 4`, and `4 3 5`
    TriangleStrip = 4,
}
impl PrimitiveTopology {
    pub fn val(&self) -> wgpu::PrimitiveTopology {
        match self {
            PrimitiveTopology::PointList        => wgpu::PrimitiveTopology::PointList       ,
            PrimitiveTopology::LineList         => wgpu::PrimitiveTopology::LineList        ,
            PrimitiveTopology::LineStrip        => wgpu::PrimitiveTopology::LineStrip       ,
            PrimitiveTopology::TriangleList     => wgpu::PrimitiveTopology::TriangleList    ,
            PrimitiveTopology::TriangleStrip    => wgpu::PrimitiveTopology::TriangleStrip   ,
        }
    }
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[cfg(feature = "pi_js_export")]
pub enum PolygonMode {
    Fill = 0,
    /// Polygons are drawn as line segments
    Line = 1,
    /// Polygons are drawn as points
    Point = 2,
}
// #[derive(Debug, Clone, Copy)]
/// * 默认值 Fill
#[derive(Debug, Clone, Copy)]
pub enum PolygonMode {
    Fill = 0,
    /// Polygons are drawn as line segments
    Line = 1,
    /// Polygons are drawn as points
    Point = 2,
}
impl PolygonMode {
    pub fn val(&self) -> wgpu::PolygonMode {
        match self {
            PolygonMode::Fill => wgpu::PolygonMode::Fill,
            PolygonMode::Line => wgpu::PolygonMode::Line,
            PolygonMode::Point => wgpu::PolygonMode::Point,
        }
    }
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[cfg(feature = "pi_js_export")]
pub enum FrontFace {
    Ccw = 0,
    /// Triangles with vertices in clockwise order are considered the front face.
    ///
    /// This is the default with left handed coordinate spaces.
    Cw = 1,
}
/// * 默认值 Ccw
#[derive(Debug, Clone, Copy)]
pub enum FrontFace {
    Ccw = 0,
    /// Triangles with vertices in clockwise order are considered the front face.
    ///
    /// This is the default with left handed coordinate spaces.
    Cw = 1,
}
impl FrontFace {
    pub fn val(&self) -> wgpu::FrontFace {
        match self {
            FrontFace::Ccw => wgpu::FrontFace::Ccw,
            FrontFace::Cw => wgpu::FrontFace::Cw,
        }
    }
}
