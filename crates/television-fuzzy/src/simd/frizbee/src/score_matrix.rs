use crate::r#const::*;
use crate::simd::interleave_strings;
use std::collections::HashSet;
use std::ops::{BitAnd, BitOr, Not};
use std::simd::cmp::*;
use std::simd::{Mask, Simd};

pub type SimdVec = Simd<u16, SIMD_WIDTH>;

pub fn smith_waterman_with_scoring_matrix(
    needle: &str,
    haystacks: &[&str; SIMD_WIDTH],
) -> (Vec<Vec<SimdVec>>, [(usize, usize); SIMD_WIDTH]) {
    let needle = needle
        .as_bytes()
        .iter()
        .map(|x| *x as u16)
        .collect::<Vec<u16>>();
    let needle_len = needle.len();
    let haystack_len = haystacks.iter().map(|x| x.len()).max().unwrap();

    let haystack = interleave_strings(haystacks, haystack_len);

    // State
    let mut score_simds =
        vec![vec![Simd::splat(0); haystack_len + 1]; needle_len + 1];
    let mut prev_col_score_simds = vec![Simd::splat(0); haystack_len + 1];
    let mut left_gap_penalty_masks = vec![Mask::splat(true); haystack_len];
    let mut all_time_max_score_row = Simd::splat(0);
    let mut all_time_max_score_col = Simd::splat(0);
    let mut all_time_max_score = Simd::splat(0);

    // Delimiters
    let mut delimiter_bonus_enabled_mask = Mask::splat(false);
    let mut is_delimiter_masks = vec![Mask::splat(false); haystack_len + 1];
    let space_delimiter = Simd::splat(" ".bytes().next().unwrap() as u16);
    let slash_delimiter = Simd::splat("/".bytes().next().unwrap() as u16);
    let dot_delimiter = Simd::splat(".".bytes().next().unwrap() as u16);
    let comma_delimiter = Simd::splat(",".bytes().next().unwrap() as u16);
    let underscore_delimiter = Simd::splat("_".bytes().next().unwrap() as u16);
    let dash_delimiter = Simd::splat("-".bytes().next().unwrap() as u16);
    let delimiter_bonus = Simd::splat(DELIMITER_BONUS);

    // Capitalization
    let capital_start = Simd::splat("A".bytes().next().unwrap() as u16);
    let capital_end = Simd::splat("Z".bytes().next().unwrap() as u16);
    let capitalization_bonus = Simd::splat(CAPITALIZATION_BONUS);
    let matching_casing_bonus = Simd::splat(MATCHING_CASE_BONUS);
    let to_lowercase_mask = Simd::splat(0x20);

    // Scoring params
    let gap_open_penalty = Simd::splat(GAP_OPEN_PENALTY);
    let gap_extend_penalty = Simd::splat(GAP_EXTEND_PENALTY);

    let match_score = Simd::splat(MATCH_SCORE);
    let mismatch_score = Simd::splat(MISMATCH_PENALTY);
    let prefix_match_score = Simd::splat(MATCH_SCORE + PREFIX_BONUS);
    let first_char_match_score =
        Simd::splat(MATCH_SCORE * FIRST_CHAR_MULTIPLIER);
    let first_char_prefix_match_score =
        Simd::splat((MATCH_SCORE + PREFIX_BONUS) * FIRST_CHAR_MULTIPLIER);

    let zero: SimdVec = Simd::splat(0);

    for i in 1..=needle_len {
        let match_score = if i == 1 {
            first_char_match_score
        } else {
            match_score
        };
        let prefix_match_score = if i == 1 {
            first_char_prefix_match_score
        } else {
            prefix_match_score
        };

        let needle_char = Simd::splat(needle[i - 1]);
        let mut up_score_simd = Simd::splat(0);
        let mut up_gap_penalty_mask = Mask::splat(true);
        let mut curr_col_score_simds = vec![Simd::splat(0); haystack_len + 1];
        let needle_cased_mask = needle_char
            .simd_ge(capital_start)
            .bitand(needle_char.simd_le(capital_end));
        let needle_char =
            needle_char | needle_cased_mask.select(to_lowercase_mask, zero);

        for j in 1..=haystack_len {
            let prefix_mask = Mask::splat(j == 1);

            // Load chunk and remove casing
            let cased_haystack_simd = SimdVec::from_slice(&haystack[j - 1]);
            let capital_mask = cased_haystack_simd
                .simd_ge(capital_start)
                .bitand(cased_haystack_simd.simd_le(capital_end));
            let haystack_simd = cased_haystack_simd
                | capital_mask.select(to_lowercase_mask, zero);

            let matched_casing_mask = needle_cased_mask.simd_eq(capital_mask);

            // Give a bonus for prefix matches
            let match_score =
                prefix_mask.select(prefix_match_score, match_score);

            // Calculate diagonal (match/mismatch) scores
            let diag = prev_col_score_simds[j - 1];
            let match_mask = needle_char.simd_eq(haystack_simd);
            let diag_score = match_mask.select(
                diag + match_score
                    + is_delimiter_masks[j - 1].bitand(delimiter_bonus_enabled_mask).select(delimiter_bonus, zero)
                    // XOR with prefix mask to ignore capitalization on the prefix
                    + capital_mask.bitand(prefix_mask.not()).select(capitalization_bonus, zero)
                    + matched_casing_mask.select(matching_casing_bonus, zero),
                diag.simd_gt(mismatch_score)
                    .select(diag - mismatch_score, zero),
            );

            // Load and calculate up scores
            let up_gap_penalty = up_gap_penalty_mask
                .select(gap_open_penalty, gap_extend_penalty);
            let up_score = up_score_simd
                .simd_gt(up_gap_penalty)
                .select(up_score_simd - up_gap_penalty, zero);

            // Load and calculate left scores
            let left = prev_col_score_simds[j];
            let left_gap_penalty_mask = left_gap_penalty_masks[j - 1];
            let left_gap_penalty = left_gap_penalty_mask
                .select(gap_open_penalty, gap_extend_penalty);
            let left_score = left
                .simd_gt(left_gap_penalty)
                .select(left - left_gap_penalty, zero);

            // Calculate maximum scores
            let max_score: SimdVec =
                diag_score.simd_max(up_score).simd_max(left_score);

            // Update gap penalty mask
            let diag_mask = max_score.simd_eq(diag_score);
            up_gap_penalty_mask = max_score.simd_ne(up_score).bitor(diag_mask);
            left_gap_penalty_masks[j - 1] =
                max_score.simd_ne(left_score).bitor(diag_mask);

            // Update delimiter masks
            is_delimiter_masks[j] = space_delimiter
                .simd_eq(haystack_simd)
                .bitor(slash_delimiter.simd_eq(haystack_simd))
                .bitor(dot_delimiter.simd_eq(haystack_simd))
                .bitor(comma_delimiter.simd_eq(haystack_simd))
                .bitor(underscore_delimiter.simd_eq(haystack_simd))
                .bitor(dash_delimiter.simd_eq(haystack_simd));
            // Only enable delimiter bonus if we've seen a non-delimiter char
            delimiter_bonus_enabled_mask = delimiter_bonus_enabled_mask
                .bitor(is_delimiter_masks[j].not());

            // Store the scores for the next iterations
            up_score_simd = max_score;
            curr_col_score_simds[j] = max_score;
            score_simds[i][j] = max_score;

            // Store the maximum score across all runs
            let is_all_time_max_score_mask =
                all_time_max_score.simd_lt(max_score);
            all_time_max_score_col = is_all_time_max_score_mask
                .select(Simd::splat(i as u16), all_time_max_score_col);
            all_time_max_score_row = is_all_time_max_score_mask
                .select(Simd::splat(j as u16), all_time_max_score_row);
            all_time_max_score = all_time_max_score.simd_max(max_score);
        }

        prev_col_score_simds = curr_col_score_simds;
    }

    let mut all_time_max_score_positions = [(0, 0); SIMD_WIDTH];
    for idx in 0..SIMD_WIDTH {
        all_time_max_score_positions[idx] = (
            all_time_max_score_row[idx] as usize,
            all_time_max_score_col[idx] as usize,
        );
    }
    (score_simds, all_time_max_score_positions)
}

pub fn char_indices_from_scores(
    score_matrices: Vec<Vec<SimdVec>>,
    max_score_positions: [(usize, usize); SIMD_WIDTH],
) -> Vec<Vec<usize>> {
    // Traceback and store the matched indices
    let mut indices = vec![HashSet::new(); SIMD_WIDTH];
    for (idx, initial_idx) in max_score_positions.into_iter().enumerate() {
        let indices = &mut indices[idx];

        let (mut row_idx, mut col_idx) = initial_idx;
        let mut score = score_matrices[col_idx][row_idx][idx];
        if score > 0 {
            indices.insert(row_idx - 1);
        }

        // NOTE: row_idx = 0 or col_idx = 0 will always have a score of 0
        while score > 0 {
            // Gather up the scores for all possible paths
            let diag = score_matrices[col_idx - 1][row_idx - 1][idx];
            let left = score_matrices[col_idx - 1][row_idx][idx];
            let up = score_matrices[col_idx][row_idx - 1][idx];

            // Choose the best path and store the index on the haystack if applicable
            let new_score = diag.max(left).max(up);

            // Diagonal (match/mismatch)
            if new_score == diag {
                row_idx -= 1;
                col_idx -= 1;

                // Check if the score decreases (remember we're going backwards)
                // to see if we've found a match
                if new_score < score && new_score > 0 {
                    indices.insert(row_idx - 1);
                }
            }
            // Up (gap in haystack)
            else if new_score == up {
                row_idx -= 1;

                // Finished crossing a gap, remove any previous rows
                if new_score > score && new_score > 0 {
                    indices.remove(&(row_idx));
                    indices.insert(row_idx - 1);
                }
            }
            // Left (gap in needle)
            else {
                col_idx -= 1;
            }

            score = new_score;
        }
    }

    indices
        .iter()
        .map(|indices| {
            let mut indices = indices.iter().copied().collect::<Vec<_>>();
            indices.sort();
            indices
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_single_indices(needle: &str, haystack: &str) -> Vec<usize> {
        let haystacks = [haystack; SIMD_WIDTH];
        let (score_matrices, max_score_locations) =
            smith_waterman_with_scoring_matrix(needle, &haystacks);
        let indices =
            char_indices_from_scores(score_matrices, max_score_locations);
        indices[0].clone()
    }

    fn _print_score_matrix(score_matrix: &Vec<Vec<SimdVec>>) {
        // Transpose the matrix
        let mut transposed = vec![vec![]; score_matrix[0].len()];
        for row in score_matrix.iter() {
            for (col_idx, val) in row.iter().enumerate() {
                transposed[col_idx].push(*val);
            }
        }

        for col in transposed.iter() {
            for row in col.iter() {
                // print fixed width
                print!(" {:<2} ", row[4]);
            }
            println!();
        }
    }

    #[test]
    fn test_leaking() {
        let needle = "t";
        let haystacks = [
            "true",
            "toDate",
            "toString",
            "transpose",
            "testing",
            "to",
            "toRgba",
            "toolbar",
            "true",
            "toDate",
            "toString",
            "transpose",
            "testing",
            "to",
            "toRgba",
            "toolbar",
        ];

        let (score_matrices, max_score_locations) =
            smith_waterman_with_scoring_matrix(needle, &haystacks);

        let indices =
            char_indices_from_scores(score_matrices, max_score_locations);
        for indices in indices.into_iter() {
            assert_eq!(indices, [0])
        }
    }

    #[test]
    fn test_basic_indices() {
        assert_eq!(run_single_indices("b", "abc"), vec![1]);
        assert_eq!(run_single_indices("c", "abc"), vec![2]);
    }

    #[test]
    fn test_prefix_indices() {
        assert_eq!(run_single_indices("a", "abc"), vec![0]);
        assert_eq!(run_single_indices("a", "aabc"), vec![0]);
        assert_eq!(run_single_indices("a", "babc"), vec![1]);
    }

    #[test]
    fn test_exact_match_indices() {
        assert_eq!(run_single_indices("a", "a"), vec![0]);
        assert_eq!(run_single_indices("abc", "abc"), vec![0, 1, 2]);
        assert_eq!(run_single_indices("ab", "abc"), vec![0, 1]);
    }

    #[test]
    fn test_delimiter_indices() {
        assert_eq!(run_single_indices("b", "a-b"), vec![2]);
        assert_eq!(run_single_indices("a", "a-b-c"), vec![0]);
        assert_eq!(run_single_indices("b", "a--b"), vec![3]);
        assert_eq!(run_single_indices("c", "a--bc"), vec![4]);
    }

    #[test]
    fn test_affine_gap_indices() {
        assert_eq!(run_single_indices("test", "Uterst"), vec![1, 2, 4, 5]);
        assert_eq!(run_single_indices("test", "Uterrst"), vec![1, 2, 5, 6]);
        assert_eq!(run_single_indices("test", "Uterrs t"), vec![1, 2, 5, 7]);
    }

    #[test]
    fn test_capital_indices() {
        assert_eq!(run_single_indices("a", "A"), vec![0]);
        assert_eq!(run_single_indices("A", "Aa"), vec![0]);
        assert_eq!(run_single_indices("D", "forDist"), vec![3]);
    }
}
