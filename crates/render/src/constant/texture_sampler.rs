
use pi_render::rhi::sampler::{EAddressMode, EFilterMode, EAnisotropyClamp, SamplerDesc};
use wasm_bindgen::prelude::wasm_bindgen;

use super::render_state::CompareFunction;

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
