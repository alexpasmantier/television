use memchr::{memchr, memchr2, memrchr, memrchr2};

/// Ripped directly from nucleo_pool-nucleo_pool. It makes the algo much faster but disables resistance
/// to typos
pub fn prefilter_ascii(
    needle: &[u8],
    mut haystack: &[u8],
) -> Option<(usize, usize, usize)> {
    let start = find_ascii_ignore_case(
        needle[0],
        &haystack[..haystack.len() - needle.len() + 1],
    )?;
    let mut greedy_end = start + 1;
    haystack = &haystack[greedy_end..];
    for &c in &needle[1..] {
        let idx = find_ascii_ignore_case(c, haystack)? + 1;
        greedy_end += idx;
        haystack = &haystack[idx..];
    }
    let end = greedy_end
        + find_ascii_ignore_case_rev(*needle.last().unwrap(), haystack)
            .map_or(0, |i| i + 1);
    Some((start, greedy_end, end))
}

#[inline(always)]
fn find_ascii_ignore_case(c: u8, haystack: &[u8]) -> Option<usize> {
    if c >= b'a' && c <= b'z' {
        memchr2(c, c - 32, haystack)
    } else {
        memchr(c, haystack)
    }
}

#[inline(always)]
fn find_ascii_ignore_case_rev(c: u8, haystack: &[u8]) -> Option<usize> {
    if c >= b'a' && c <= b'z' {
        memrchr2(c, c - 32, haystack)
    } else {
        memrchr(c, haystack)
    }
}
