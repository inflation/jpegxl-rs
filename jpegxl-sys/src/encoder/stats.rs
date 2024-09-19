/*
This file is part of jpegxl-sys.

jpegxl-sys is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

jpegxl-sys is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with jpegxl-sys.  If not, see <https://www.gnu.org/licenses/>.
*/

//! API to collect various statistics from JXL encoder.

#[cfg(doc)]
use crate::encoder::encode::JxlEncoderCollectStats;

/// Opaque structure that holds the encoder statistics.
///
/// Allocated and initialized with [`JxlEncoderStatsCreate`].
/// Cleaned up and deallocated with [`JxlEncoderStatsDestroy`].
#[repr(C)]
pub struct JxlEncoderStats {
    _unused: [u8; 0],
}

extern "C" {
    /// Creates an instance of [`JxlEncoderStats`] and initializes it.
    ///
    /// # Returns
    /// A pointer to the initialized [`JxlEncoderStats`] instance.
    pub fn JxlEncoderStatsCreate() -> *mut JxlEncoderStats;

    /// Deinitializes and frees [`JxlEncoderStats`] instance.
    ///
    /// # Parameters
    /// - `stats`: Instance to be cleaned up and deallocated. No-op if `stats` is
    ///   a null pointer.
    pub fn JxlEncoderStatsDestroy(stats: *mut JxlEncoderStats);

    /// Returns the value of the statistics corresponding to the given key.
    ///
    /// # Parameters
    /// - `stats`: Object that was passed to the encoder with a
    ///   [`JxlEncoderCollectStats`] function.
    /// - `key`: The particular statistics to query.
    ///
    /// # Returns
    /// The value of the statistics.
    pub fn JxlEncoderStatsGet(stats: *const JxlEncoderStats, key: JxlEncoderStatsKey) -> usize;

    /** Updates the values of the given stats object with that of an other.
     *
     * @param stats object whose values will be updated (usually added together)
     * @param other stats object whose values will be merged with stats
     */
    pub fn JxlEncoderStatsMerge(stats: *mut JxlEncoderStats, other: *const JxlEncoderStats);
}

/// Data type for querying [`JxlEncoderStats`] object
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
