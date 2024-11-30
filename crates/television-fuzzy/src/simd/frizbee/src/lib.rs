#![feature(portable_simd)]

extern crate memchr;

mod bucket;
pub mod r#const;
mod prefilter;
pub mod score_matrix;
pub mod simd;

use crate::score_matrix::*;
use bucket::Bucket;
use prefilter::prefilter_ascii;
use r#const::SIMD_WIDTH;
use std::cmp::Reverse;

#[derive(Debug, Clone, Default)]
pub struct Match {
    /** Index of the match in the original list of haystacks */
    pub index_in_haystack: usize,
    /** Index of the match in the returned list of matches */
    pub index: usize,
    pub score: u16,
    pub indices: Option<Vec<usize>>,
}

/// Computes the Smith-Waterman score with affine gaps for the list of given targets.
///
/// You should call this function with as many targets as you have available as it will
/// automatically chunk the targets based on string length to avoid unnecessary computation
/// due to SIMD
pub fn match_list(
    needle: &str,
    haystacks: &[&str],
    opts: Options,
) -> Vec<Match> {
    if needle.is_empty() {
        return haystacks
            .iter()
            .enumerate()
            .map(|(i, _)| Match {
                index_in_haystack: i,
                index: i,
                score: 0,
                indices: None,
            })
            .collect();
    }

    let needle_lower = needle.to_ascii_lowercase();
    let mut matches = vec![None; haystacks.len()];

    let mut buckets = [
        Bucket::new(4),
        Bucket::new(8),
        Bucket::new(12),
        Bucket::new(16),
        Bucket::new(20),
        Bucket::new(24),
        Bucket::new(32),
        Bucket::new(48),
        Bucket::new(64),
        Bucket::new(96),
        Bucket::new(128),
        Bucket::new(160),
        Bucket::new(192),
        Bucket::new(224),
        Bucket::new(256),
        Bucket::new(384),
        Bucket::new(512),
    ];

    for (i, haystack) in haystacks.iter().enumerate() {
        // Pick the bucket to insert into based on the length of the haystack
        let bucket_idx = match haystack.len() {
            0..=4 => 0,
            5..=8 => 1,
            9..=12 => 2,
            13..=16 => 3,
            17..=20 => 4,
            21..=24 => 5,
            25..=32 => 6,
            33..=48 => 7,
            49..=64 => 8,
            65..=96 => 9,
            97..=128 => 10,
            129..=160 => 11,
            161..=192 => 12,
            193..=224 => 13,
            225..=256 => 14,
            257..=384 => 15,
            385..=512 => 16,
            // TODO: should just return score = 0 or fallback to prefilter
            _ => continue,
        };

        if opts.prefilter && !prefilter(&needle_lower, haystack) {
            continue;
        }

        let bucket = &mut buckets[bucket_idx];
        bucket.add_haystack(haystack, i);

        if bucket.is_full() {
            bucket.process(&mut matches, needle, opts.indices);
        }
    }

    // Iterate over the bucket with remaining elements
    for bucket in buckets.iter_mut() {
        bucket.process(&mut matches, needle, opts.indices);
    }

    // Vec<Option<Match>> -> Vec<Match>
    let mut matches = matches.into_iter().flatten().collect::<Vec<_>>();

    // Min score
    if opts.min_score > 0 {
        matches.retain(|mtch| mtch.score >= opts.min_score);
    }

    // If either of these ran, the `index` property will be out of date
    if opts.min_score > 0 || opts.prefilter {
        matches = matches
            .into_iter()
            .enumerate()
            .map(|(i, mtch)| Match {
                index_in_haystack: mtch.index_in_haystack,
                index: i,
                score: mtch.score,
                indices: mtch.indices,
            })
            .collect();
    }

    // Sorting
    if opts.stable_sort {
        matches.sort_by_key(|mtch| Reverse(mtch.score));
    } else if opts.unstable_sort {
        matches.sort_unstable_by_key(|mtch| Reverse(mtch.score));
    }

    matches
}

pub fn match_list_for_matched_indices(
    needle: &str,
    haystacks: &[&str],
) -> Vec<Vec<usize>> {
    // TODO: sort by length
    let haystacks = haystacks.to_vec();

    let mut matched_indices_arr = Vec::with_capacity(haystacks.len());

    for haystack_idx in (0..haystacks.len()).step_by(SIMD_WIDTH) {
        let length =
            (haystacks.len().saturating_sub(haystack_idx)).min(SIMD_WIDTH);
        let mut sliced_haystacks = [""; SIMD_WIDTH];
        sliced_haystacks[0..length]
            .copy_from_slice(&haystacks[haystack_idx..haystack_idx + length]);

        let (score_matrices, max_score_locations) =
            smith_waterman_with_scoring_matrix(needle, &sliced_haystacks);
        let haystack_slice_matched_indices =
            char_indices_from_scores(score_matrices, max_score_locations);

        for matched_indices in
            haystack_slice_matched_indices.into_iter().take(length)
        {
            matched_indices_arr.push(matched_indices);
        }
    }

    matched_indices_arr
}

fn prefilter(needle: &str, haystack: &str) -> bool {
    if needle.len() > haystack.len() {
        return false;
    }
    prefilter_ascii(needle.as_bytes(), haystack.as_bytes()).is_some()
}

pub struct Options {
    /// Populate score matrix and perform traceback to get the indices of the matching characters
    pub indices: bool,
    /// Uses fzf's prefilter algorithm to remove any haystacks that do not include all of the
    /// characters in the needle. This may remove many haystacks that contain good matches
    pub prefilter: bool,
    /// Sort the results while maintaining the original order of the haystacks
    pub stable_sort: bool,
    /// Sort the results without maintaining the original order of the haystacks (much faster on
    /// long lists)
    pub unstable_sort: bool,
    /// Minimum score to return a result
    pub min_score: u16,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            indices: false,
            prefilter: false,
            stable_sort: true,
            unstable_sort: false,
            min_score: 0,
        }
    }
}
