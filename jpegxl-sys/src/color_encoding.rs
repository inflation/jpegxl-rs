#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlColorSpace {
    Rgb = 0,
    Gray,
    Xyb,
    Unknown,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlWhitePoint {
    D65 = 1,
    Custom = 2,
    E = 10,
    Dci = 11,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlPrimaries {
    SRgb = 1,
    Custom = 2,
    Rec2100 = 9,
    P3 = 11,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlTransferFunction {
    Rec709 = 1,
    Unknown = 2,
    Linear = 8,
    SRgb = 13,
    Pq = 16,
    Dci = 17,
    Hlg = 18,
    Gamma = 65535,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlRenderingIntent {
    Perceptual = 0,
    Relative,
    Saturation,
    Absolute,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct JxlColorEncoding {
    pub color_space: JxlColorSpace,
    pub white_point: JxlWhitePoint,
    pub white_point_xy: [f64; 2usize],
    pub primaries: JxlPrimaries,
    pub primaries_red_xy: [f64; 2usize],
    pub primaries_green_xy: [f64; 2usize],
    pub primaries_blue_xy: [f64; 2usize],
    pub transfer_function: JxlTransferFunction,
    pub gamma: f64,
    pub rendering_intent: JxlRenderingIntent,
}
