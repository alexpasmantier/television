use crate::simd::*;
use crate::Match;

pub(crate) struct Bucket<'a> {
    width: usize,
    length: usize,
    haystacks: [&'a str; 16],
    idxs: [usize; 16],
    smith_waterman_func: Option<fn(&str, &[&str]) -> [u16; 16]>,
    smith_waterman_func_large: Option<fn(&str, &[&str]) -> [u16; 8]>,
}

impl<'a> Bucket<'a> {
    pub fn new(string_length: usize) -> Self {
        Bucket {
            width: if string_length <= 24 { 16 } else { 8 },
            length: 0,
            haystacks: [""; 16],
            idxs: [0; 16],
            smith_waterman_func: match string_length {
                4 => Some(smith_waterman_inter_simd_4),
                8 => Some(smith_waterman_inter_simd_8),
                12 => Some(smith_waterman_inter_simd_12),
                16 => Some(smith_waterman_inter_simd_16),
                20 => Some(smith_waterman_inter_simd_20),
                24 => Some(smith_waterman_inter_simd_24),
                _ => None,
            },
            smith_waterman_func_large: match string_length {
                32 => Some(smith_waterman_inter_simd_32),
                48 => Some(smith_waterman_inter_simd_48),
                64 => Some(smith_waterman_inter_simd_64),
                96 => Some(smith_waterman_inter_simd_96),
                128 => Some(smith_waterman_inter_simd_128),
                160 => Some(smith_waterman_inter_simd_160),
                192 => Some(smith_waterman_inter_simd_192),
                224 => Some(smith_waterman_inter_simd_224),
                256 => Some(smith_waterman_inter_simd_256),
                384 => Some(smith_waterman_inter_simd_384),
                512 => Some(smith_waterman_inter_simd_512),
                _ => None,
            },
        }
    }

    pub fn add_haystack(&mut self, haystack: &'a str, idx: usize) {
        if self.length == self.width {
            return;
        }
        self.haystacks[self.length] = haystack;
        self.idxs[self.length] = idx;
        self.length += 1;
    }

    pub fn is_full(&self) -> bool {
        self.length == self.width
    }

    pub fn process(
        &mut self,
        matches: &mut [Option<Match>],
        needle: &str,
        _with_indices: bool,
    ) {
        if self.length == 0 {
            return;
        }

        if let Some(smith_waterman_func) = self.smith_waterman_func {
            let scores = (smith_waterman_func)(needle, &self.haystacks);
            for idx in 0..self.length {
                let score_idx = self.idxs[idx];
                matches[score_idx] = Some(Match {
                    index_in_haystack: score_idx,
                    index: score_idx,
                    score: scores[idx],
                    indices: None, //indices: bucket_indices.get(idx).cloned(),
                });
            }
        } else {
            let scores = self.smith_waterman_func_large.unwrap()(
                needle,
                &self.haystacks,
            );
            for idx in 0..self.length {
                let score_idx = self.idxs[idx];
                matches[score_idx] = Some(Match {
                    index_in_haystack: score_idx,
                    index: score_idx,
                    score: scores[idx],
                    indices: None, //indices: bucket_indices.get(idx).cloned(),
                });
            }
        }

        self.reset();
    }

    pub fn reset(&mut self) {
        self.length = 0;
    }
}
