
use pi_render::rhi::sampler::{EAddressMode, EFilterMode, EAnisotropyClamp, SamplerDesc};

use super::render_state::CompareFunction;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorFormat {
    // Normal 8 bit formats
    /// Red channel only. 8 bit integer per channel. [0, 255] converted to/from float [0, 1] in shader.
    R8Unorm,
    /// Red channel only. 8 bit integer per channel. [-127, 127] converted to/from float [-1, 1] in shader.
    R8Snorm,
    /// Red channel only. 8 bit integer per channel. Unsigned in shader.
    R8Uint,
    /// Red channel only. 8 bit integer per channel. Signed in shader.
    R8Sint,

    // Normal 16 bit formats
    /// Red channel only. 16 bit integer per channel. Unsigned in shader.
    R16Uint,
    /// Red channel only. 16 bit integer per channel. Signed in shader.
    R16Sint,
    /// Red channel only. 16 bit integer per channel. [0, 65535] converted to/from float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    R16Unorm,
    /// Red channel only. 16 bit integer per channel. [0, 65535] converted to/from float [-1, 1] in shader.
    ///
    /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    R16Snorm,
    /// Red channel only. 16 bit float per channel. Float in shader.
    R16Float,
    /// Red and green channels. 8 bit integer per channel. [0, 255] converted to/from float [0, 1] in shader.
    Rg8Unorm,
    /// Red and green channels. 8 bit integer per channel. [-127, 127] converted to/from float [-1, 1] in shader.
    Rg8Snorm,
    /// Red and green channels. 8 bit integer per channel. Unsigned in shader.
    Rg8Uint,
    /// Red and green channels. 8 bit integer per channel. Signed in shader.
    Rg8Sint,

    // Normal 32 bit formats
    /// Red channel only. 32 bit integer per channel. Unsigned in shader.
    R32Uint,
    /// Red channel only. 32 bit integer per channel. Signed in shader.
    R32Sint,
    /// Red channel only. 32 bit float per channel. Float in shader.
    R32Float,
    /// Red and green channels. 16 bit integer per channel. Unsigned in shader.
    Rg16Uint,
    /// Red and green channels. 16 bit integer per channel. Signed in shader.
    Rg16Sint,
    /// Red and green channels. 16 bit integer per channel. [0, 65535] converted to/from float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    Rg16Unorm,
    /// Red and green channels. 16 bit integer per channel. [0, 65535] converted to/from float [-1, 1] in shader.
    ///
    /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    Rg16Snorm,
    /// Red and green channels. 16 bit float per channel. Float in shader.
    Rg16Float,
    /// Red, green, blue, and alpha channels. 8 bit integer per channel. [0, 255] converted to/from float [0, 1] in shader.
    Rgba8Unorm,
    /// Red, green, blue, and alpha channels. 8 bit integer per channel. Srgb-color [0, 255] converted to/from linear-color float [0, 1] in shader.
    Rgba8UnormSrgb,
    /// Red, green, blue, and alpha channels. 8 bit integer per channel. [-127, 127] converted to/from float [-1, 1] in shader.
    Rgba8Snorm,
    /// Red, green, blue, and alpha channels. 8 bit integer per channel. Unsigned in shader.
    Rgba8Uint,
    /// Red, green, blue, and alpha channels. 8 bit integer per channel. Signed in shader.
    Rgba8Sint,
    /// Blue, green, red, and alpha channels. 8 bit integer per channel. [0, 255] converted to/from float [0, 1] in shader.
    Bgra8Unorm,
    /// Blue, green, red, and alpha channels. 8 bit integer per channel. Srgb-color [0, 255] converted to/from linear-color float [0, 1] in shader.
    Bgra8UnormSrgb,

    // Packed 32 bit formats
    /// Packed unsigned float with 9 bits mantisa for each RGB component, then a common 5 bits exponent
    Rgb9e5Ufloat,
    /// Red, green, blue, and alpha channels. 10 bit integer for RGB channels, 2 bit integer for alpha channel. [0, 1023] ([0, 3] for alpha) converted to/from float [0, 1] in shader.
    Rgb10a2Unorm,
    /// Red, green, and blue channels. 11 bit float with no sign bit for RG channels. 10 bit float with no sign bit for blue channel. Float in shader.
    Rg11b10Float,

    // Normal 64 bit formats
    /// Red and green channels. 32 bit integer per channel. Unsigned in shader.
    Rg32Uint,
    /// Red and green channels. 32 bit integer per channel. Signed in shader.
    Rg32Sint,
    /// Red and green channels. 32 bit float per channel. Float in shader.
    Rg32Float,
    /// Red, green, blue, and alpha channels. 16 bit integer per channel. Unsigned in shader.
    Rgba16Uint,
    /// Red, green, blue, and alpha channels. 16 bit integer per channel. Signed in shader.
    Rgba16Sint,
    /// Red, green, blue, and alpha channels. 16 bit integer per channel. [0, 65535] converted to/from float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    Rgba16Unorm,
    /// Red, green, blue, and alpha. 16 bit integer per channel. [0, 65535] converted to/from float [-1, 1] in shader.
    ///
    /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    Rgba16Snorm,
    /// Red, green, blue, and alpha channels. 16 bit float per channel. Float in shader.
    Rgba16Float,

    // Normal 128 bit formats
    /// Red, green, blue, and alpha channels. 32 bit integer per channel. Unsigned in shader.
    Rgba32Uint,
    /// Red, green, blue, and alpha channels. 32 bit integer per channel. Signed in shader.
    Rgba32Sint,
    /// Red, green, blue, and alpha channels. 32 bit float per channel. Float in shader.
    Rgba32Float,
}
impl ColorFormat {
    pub fn val(&self) -> wgpu::TextureFormat {
        match self {
            Self::R8Unorm => wgpu::TextureFormat::R8Unorm,
            Self::R8Snorm => wgpu::TextureFormat::R8Snorm,
            Self::R8Uint => wgpu::TextureFormat::R8Uint,
            Self::R8Sint => wgpu::TextureFormat::R8Sint,
            Self::R16Uint => wgpu::TextureFormat::R16Uint,
            Self::R16Sint => wgpu::TextureFormat::R16Sint,
            Self::R16Unorm => wgpu::TextureFormat::R16Unorm,
            Self::R16Snorm => wgpu::TextureFormat::R16Snorm,
            Self::R16Float => wgpu::TextureFormat::R16Float,
            Self::Rg8Unorm => wgpu::TextureFormat::Rg8Unorm,
            Self::Rg8Snorm => wgpu::TextureFormat::Rg8Snorm,
            Self::Rg8Uint => wgpu::TextureFormat::Rg8Uint,
            Self::Rg8Sint => wgpu::TextureFormat::Rg8Sint,
            Self::R32Uint => wgpu::TextureFormat::R32Uint,
            Self::R32Sint => wgpu::TextureFormat::R32Sint,
            Self::R32Float => wgpu::TextureFormat::R32Float,
            Self::Rg16Uint => wgpu::TextureFormat::Rg16Uint,
            Self::Rg16Sint => wgpu::TextureFormat::Rg16Sint,
            Self::Rg16Unorm => wgpu::TextureFormat::Rg16Unorm,
            Self::Rg16Snorm => wgpu::TextureFormat::Rg16Snorm,
            Self::Rg16Float => wgpu::TextureFormat::Rg16Float,
            Self::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            Self::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            Self::Rgba8Snorm => wgpu::TextureFormat::Rgba8Snorm,
            Self::Rgba8Uint => wgpu::TextureFormat::Rgba8Uint,
            Self::Rgba8Sint => wgpu::TextureFormat::Rgba8Sint,
            Self::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
            Self::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
            Self::Rgb9e5Ufloat => wgpu::TextureFormat::Rgb9e5Ufloat,
            Self::Rgb10a2Unorm => wgpu::TextureFormat::Rgb10a2Unorm,
            Self::Rg11b10Float => wgpu::TextureFormat::Rg11b10Float,
            Self::Rg32Uint => wgpu::TextureFormat::Rg32Uint,
            Self::Rg32Sint => wgpu::TextureFormat::Rg32Sint,
            Self::Rg32Float => wgpu::TextureFormat::Rg32Float,
            Self::Rgba16Uint => wgpu::TextureFormat::Rgba16Uint,
            Self::Rgba16Sint => wgpu::TextureFormat::Rgba16Sint,
            Self::Rgba16Unorm => wgpu::TextureFormat::Rgba16Unorm,
            Self::Rgba16Snorm => wgpu::TextureFormat::Rgba16Snorm,
            Self::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
            Self::Rgba32Uint => wgpu::TextureFormat::Rgba32Uint,
            Self::Rgba32Sint => wgpu::TextureFormat::Rgba32Sint,
            Self::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DepthStencilFormat {
    None,
    // Depth and stencil formats
    /// Stencil format with 8 bit integer stencil.
    Stencil8,
    /// Special depth format with 16 bit integer depth.
    Depth16Unorm,
    /// Special depth format with at least 24 bit integer depth.
    Depth24Plus,
    /// Special depth/stencil format with at least 24 bit integer depth and 8 bits integer stencil.
    Depth24PlusStencil8,
    /// Special depth format with 32 bit floating point depth.
    Depth32Float,
    /// Special depth/stencil format with 32 bit floating point depth and 8 bits integer stencil.
    Depth32FloatStencil8,
}
impl DepthStencilFormat {
    pub fn val(&self) -> Option<wgpu::TextureFormat> {
        match self {
            Self::None => None,
            Self::Stencil8 => Some(wgpu::TextureFormat::Stencil8),
            Self::Depth16Unorm => Some(wgpu::TextureFormat::Depth16Unorm),
            Self::Depth24Plus => Some(wgpu::TextureFormat::Depth24Plus),
            Self::Depth24PlusStencil8 => Some(wgpu::TextureFormat::Depth24PlusStencil8),
            Self::Depth32Float => Some(wgpu::TextureFormat::Depth32Float),
            Self::Depth32FloatStencil8 => Some(wgpu::TextureFormat::Depth32FloatStencil8),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[cfg(feature = "pi_js_export")]
pub enum SamplerBorderColor {
    /// [0, 0, 0, 0]
    TransparentBlack,
    /// [0, 0, 0, 1]
    OpaqueBlack,
    /// [1, 1, 1, 1]
    OpaqueWhite,

    /// On the Metal backend, this is equivalent to `TransparentBlack` for
    /// textures that have an alpha component, and equivalent to `OpaqueBlack`
    /// for textures that do not have an alpha component. On other backends,
    /// this is equivalent to `TransparentBlack`. Requires
    /// [`Features::ADDRESS_MODE_CLAMP_TO_ZERO`]. Not supported on the web.
    Zero,
}
pub enum SamplerBorderColor {
    /// [0, 0, 0, 0]
    TransparentBlack,
    /// [0, 0, 0, 1]
    OpaqueBlack,
    /// [1, 1, 1, 1]
    OpaqueWhite,

    /// On the Metal backend, this is equivalent to `TransparentBlack` for
    /// textures that have an alpha component, and equivalent to `OpaqueBlack`
    /// for textures that do not have an alpha component. On other backends,
    /// this is equivalent to `TransparentBlack`. Requires
    /// [`Features::ADDRESS_MODE_CLAMP_TO_ZERO`]. Not supported on the web.
    Zero,
}
impl SamplerBorderColor {
    pub fn val(val: Option<&Self>) -> Option<wgpu::SamplerBorderColor> {
        match val {
            Some(val) => {
                match val {
                    SamplerBorderColor::TransparentBlack    => Some(wgpu::SamplerBorderColor::TransparentBlack),
                    SamplerBorderColor::OpaqueBlack         => Some(wgpu::SamplerBorderColor::OpaqueBlack),
                    SamplerBorderColor::OpaqueWhite         => Some(wgpu::SamplerBorderColor::OpaqueWhite),
                    SamplerBorderColor::Zero                => Some(wgpu::SamplerBorderColor::Zero),
                }
            },
            None => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[cfg(feature = "pi_js_export")]
pub struct SamplerDescriptor {
    /// How to deal with out of bounds accesses in the u (i.e. x) direction
    address_mode_u: EAddressMode,
    /// How to deal with out of bounds accesses in the v (i.e. y) direction
    address_mode_v: EAddressMode,
    /// How to deal with out of bounds accesses in the w (i.e. z) direction
    address_mode_w: EAddressMode,
    /// How to filter the texture when it needs to be magnified (made larger)
    mag_filter: EFilterMode,
    /// How to filter the texture when it needs to be minified (made smaller)
    min_filter: EFilterMode,
    /// How to filter between mip map levels
    mipmap_filter: EFilterMode,
    /// If this is enabled, this is a comparison sampler using the given comparison function.
    compare: Option<CompareFunction>,
    /// Valid values: 1, 2, 4, 8, and 16.
    anisotropy_clamp: EAnisotropyClamp,
    /// Border color to use when address_mode is [`AddressMode::ClampToBorder`]
    border_color: Option<SamplerBorderColor>,
}

pub struct SamplerDescriptor {
    /// How to deal with out of bounds accesses in the u (i.e. x) direction
    address_mode_u: EAddressMode,
    /// How to deal with out of bounds accesses in the v (i.e. y) direction
    address_mode_v: EAddressMode,
    /// How to deal with out of bounds accesses in the w (i.e. z) direction
    address_mode_w: EAddressMode,
    /// How to filter the texture when it needs to be magnified (made larger)
    mag_filter: EFilterMode,
    /// How to filter the texture when it needs to be minified (made smaller)
    min_filter: EFilterMode,
    /// How to filter between mip map levels
    mipmap_filter: EFilterMode,
    /// If this is enabled, this is a comparison sampler using the given comparison function.
    compare: Option<CompareFunction>,
    /// Valid values: 1, 2, 4, 8, and 16.
    anisotropy_clamp: EAnisotropyClamp,
    /// Border color to use when address_mode is [`AddressMode::ClampToBorder`]
    border_color: Option<SamplerBorderColor>,
}

pub fn sampler_desc(desc: &SamplerDescriptor) -> SamplerDesc {
    SamplerDesc {
        address_mode_u: desc.address_mode_u,
        address_mode_v: desc.address_mode_v,
        address_mode_w: desc.address_mode_w,
        mag_filter: desc.mag_filter,
        min_filter: desc.min_filter,
        mipmap_filter: desc.mipmap_filter,
        compare: CompareFunction::val(desc.compare.as_ref()),
        anisotropy_clamp: desc.anisotropy_clamp,
        border_color: SamplerBorderColor::val(desc.border_color.as_ref()),
    }
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[cfg(feature = "pi_js_export")]
impl SamplerDescriptor {
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn new() -> Self {
        Self {
            /// How to deal with out of bounds accesses in the u (i.e. x) direction
            address_mode_u: EAddressMode::ClampToEdge,
            /// How to deal with out of bounds accesses in the v (i.e. y) direction
            address_mode_v: EAddressMode::ClampToEdge,
            /// How to deal with out of bounds accesses in the w (i.e. z) direction
            address_mode_w: EAddressMode::ClampToEdge,
            /// How to filter the texture when it needs to be magnified (made larger)
            mag_filter: EFilterMode::Nearest,
            /// How to filter the texture when it needs to be minified (made smaller)
            min_filter: EFilterMode::Nearest,
            /// How to filter between mip map levels
            mipmap_filter: EFilterMode::Nearest,
            /// If this is enabled, this is a comparison sampler using the given comparison function.
            compare: None,
            /// Valid values: 1, 2, 4, 8, and 16.
            anisotropy_clamp: EAnisotropyClamp::One,
            /// Border color to use when address_mode is [`AddressMode::ClampToBorder`]
            border_color: None,
        }
    }
    
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn address_mode_u(&self) -> EAddressMode {
        self.address_mode_u
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn set_address_mode_u(&mut self, val: EAddressMode) {
        self.address_mode_u = val;
    }
    
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn address_mode_v(&self) -> EAddressMode {
        self.address_mode_v
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn set_address_mode_v(&mut self, val: EAddressMode) {
        self.address_mode_v = val;
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn address_mode_w(&self) -> EAddressMode {
        self.address_mode_w
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn set_address_mode_w(&mut self, val: EAddressMode) {
        self.address_mode_w = val;
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn mag_filter(&self) -> EFilterMode {
        self.mag_filter
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn set_mag_filter(&mut self, val: EFilterMode) {
        self.mag_filter = val;
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn min_filter(&self) -> EFilterMode {
        self.min_filter
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn set_min_filter(&mut self, val: EFilterMode) {
        self.min_filter = val;
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn mipmap_filter(&self) -> EFilterMode {
        self.mipmap_filter
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn set_mipmap_filter(&mut self, val: EFilterMode) {
        self.mipmap_filter = val;
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn compare(&self) -> Option<CompareFunction> {
        self.compare
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn set_compare(&mut self, val: Option<CompareFunction>) {
        self.compare = val;
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn border_color(&self) -> Option<SamplerBorderColor> {
        self.border_color
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn set_border_color(&mut self, val: Option<SamplerBorderColor>) {
        self.border_color = val;
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn anisotropy_clamp(&self) -> EAnisotropyClamp {
        self.anisotropy_clamp
    }
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    #[cfg(feature = "pi_js_export")]
    pub fn set_anisotropy_clamp(&mut self, val: EAnisotropyClamp) {
        self.anisotropy_clamp = val;
    }
}
