use crate::JxlBool;

#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum JxlOrientation {
    Identity = 1,
    FlipHorizontal = 2,
    Rotate180 = 3,
    FlipVertical = 4,
    Transpose = 5,
    Rotate90Cw = 6,
    AntiTranspose = 7,
    Rotate90Ccw = 8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum JxlExtraChannelType {
    Alpha,
    Depth,
    SpotColor,
    SelectionMask,
    Black,
    Cfa,
    Thermal,
    Reserved0,
    Reserved1,
    Reserved2,
    Reserved3,
    Reserved4,
    Reserved5,
    Reserved6,
    Reserved7,
    Unknown,
    Optional,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlPreviewHeader {
    pub xsize: u32,
    pub ysize: u32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlAnimationHeader {
    pub tps_numerator: u32,
    pub tps_denominator: u32,
    pub num_loops: u32,
    pub have_timecodes: JxlBool,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlBasicInfo {
    pub have_container: JxlBool,
    pub xsize: u32,
    pub ysize: u32,
    pub bits_per_sample: u32,
    pub exponent_bits_per_sample: u32,
    pub intensity_target: f32,
    pub min_nits: f32,
    pub relative_to_max_display: JxlBool,
    pub linear_below: f32,
    pub uses_original_profile: JxlBool,
    pub have_preview: JxlBool,
    pub have_animation: JxlBool,
    pub orientation: JxlOrientation,
    pub num_color_channels: u32,
    pub num_extra_channels: u32,
    pub alpha_bits: u32,
    pub alpha_exponent_bits: u32,
    pub alpha_premultiplied: JxlBool,
    pub preview: JxlPreviewHeader,
    pub animation: JxlAnimationHeader,
    pub intrinsic_xsize: u32,
    pub intrinsic_ysize: u32,
    _padding: [u8; 100],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlExtraChannelInfo {
    pub type_: JxlExtraChannelType,
    pub bits_per_sample: u32,
    pub exponent_bits_per_sample: u32,
    pub dim_shift: u32,
    pub name_length: u32,
    pub alpha_associated: JxlBool,
    pub spot_color: [f32; 4usize],
    pub cfa_channel: u32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlHeaderExtensions {
    pub extensions: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum JxlBlendMode {
    Replace = 0,
    Add = 1,
    Blend = 2,
    MULADD = 3,
    MUL = 4,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlBlendInfo {
    pub blendmode: JxlBlendMode,
    pub source: u32,
    pub alpha: u32,
    pub clamp: JxlBool,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlLayerInfo {
    pub have_crop: JxlBool,
    pub crop_x0: i32,
    pub crop_y0: i32,
    pub xsize: u32,
    pub ysize: u32,
    pub blend_info: JxlBlendInfo,
    pub save_as_reference: u32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlFrameHeader {
    pub duration: u32,
    pub timecode: u32,
    pub name_length: u32,
    pub is_last: JxlBool,
    pub layer_info: JxlLayerInfo,
}
