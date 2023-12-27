#[repr(C)]
pub struct JxlEncoderStats {
    _unused: [u8; 0],
}

extern "C" {
    pub fn JxlEncoderStatsCreate() -> *mut JxlEncoderStats;
    pub fn JxlEncoderStatsDestroy(stats: *mut JxlEncoderStats);
    pub fn JxlEncoderStatsGet(stats: *const JxlEncoderStats, key: JxlEncoderStatsKey) -> usize;
    pub fn JxlEncoderStatsMerge(stats: *mut JxlEncoderStats, other: *const JxlEncoderStats);
}

#[repr(C)]
#[derive(Debug, Clone)]
pub enum JxlEncoderStatsKey {
    HeaderBits,
    TocBits,
    DictionaryBits,
    SplinesBits,
    NoiseBits,
    QuantBits,
    ModularTreeBits,
    ModularGlobalBits,
    DcBits,
    ModularDcGroupBits,
    ControlFieldsBits,
    CoefOrderBits,
    AcHistogramBits,
    AcBits,
    ModularAcGroupBits,
    NumSmallBlocks,
    NumDct4x8Blocks,
    NumAfvBlocks,
    NumDct8Blocks,
    NumDct8x32Blocks,
    NumDct16Blocks,
    NumDct16x32Blocks,
    NumDct32Blocks,
    NumDct32x64Blocks,
    NumDct64Blocks,
    NumButteraugliIters,
    NumStats,
}
