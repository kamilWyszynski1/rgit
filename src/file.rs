use std::{cmp::max, collections::HashMap};

/// Indicates if a file is normal file with a content or directory.
enum FileType {
    File,
    Directory,
}

struct File {
    name: String,
    file_type: FileType,
    content: Vec<u8>,
    children: Option<Vec<File>>,
}

impl File {
    pub fn calculate_diff(&self, f2: &Self) {}
}

/// The longest common subsequence (LCS) problem is the problem of finding the longest subsequence common to all sequences in a set of sequences.
///
/// https://en.wikipedia.org/wiki/Longest_common_subsequence_problem
///
/// L(ASDFG,BCDEG) = 1 + L(ASDF,BCDE)
/// L(ASDFGI,BCDEG) = MAX(L(ASDFG,BCDEG),L(ASDFGI,BCDE))
fn lcs(s1: &str, s2: &str, dp: &mut Vec<Vec<isize>>) -> usize {
    let (m, n) = (s1.len(), s2.len());

    if m == 0 || n == 0 {
        return 0;
    }

    if dp[m - 1][n - 1] != -1 {
        return dp[m - 1][n - 1] as usize;
    }

    if s1.chars().last().unwrap() == s2.chars().last().unwrap() {
        let value = 1 + lcs(&s1[..m - 1], &s2[..n - 1], dp);
        dp[m - 1][n - 1] = value as isize;
        return value;
    }
    let value = max(lcs(s1, &s2[..n - 1], dp), lcs(&s1[..m - 1], s2, dp));
    dp[m - 1][n - 1] = value as isize;
    return value;
}

fn diff(s1: &str, s2: &str, dp: &mut Vec<Vec<isize>>) {
    let (m, n) = (s1.len(), s2.len());

    if m == 0 || n == 0 {
        return ();
    }

    if s1.chars().last().unwrap() == s2.chars().last().unwrap() {
        diff(&s1[..m - 1], &s2[..n - 1], dp);
        println!(" {}", s1.chars().last().unwrap());
    } else if n > 0 && (m == 0 || dp[m][n - 1] >= dp[m - 1][n]) {
        diff(&s1, &s2[..n - 1], dp);
        println!(" +{}", s2.chars().last().unwrap());
    } else if m > 0 && (n == 0 || dp[m][n - 1] < dp[m - 1][n]) {
        diff(&s1[..m - 1], s2, dp);
        println!(" -{}", s1.chars().last().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{diff, lcs};

    fn test_case(s1: &str, s2: &str, compare: usize) {
        assert_eq!(
            lcs(s1, s2, &mut vec![vec![-1; s2.len()]; s1.len()]),
            compare
        )
    }

    #[test]
    fn test_LCS() {
        test_case("qwdqwd", "qweqweqwe", 4);
        test_case("ABCDGH", "AEDFHR", 3);
        test_case("AGGTAB", "GXTXAYB", 4);
        test_case("workattech", "branch", 4);
        test_case("helloworld", "playword", 5);
        test_case("hello", "hello", 5);
        test_case(
                "pqowkdpqowkdpqowkdpqwkdpoqwkdpqwokdpoqwkdpoqwkdpoqwkdpoqwkdpoqwkdpqowdkqppqowkdpqowkdpqowkdpqwkdpoqwkdpqwokdpoqwkdpoqwkdpoqwkdpoqwkdpoqwkdpqowdkqppqowkdpqowkdpqowkdpqwkdpoqwkdpqwokdpoqwkdpoqwkdpoqwkdpoqwkdpoqwkdpqowdkqppqowkdpqowkdpqowkdpqwkdpoqwkdpqwokdpoqwkdpoqwkdpoqwkdpoqwkdpoqwkdpqowdkqppqowkdpqowkdpqowkdpqwkdpoqwkdpqwokdpoqwkdpoqwkdpoqwkdpoqwkdpoqwkdpqowdkqppqowkdpqowkdpqowkdpqwkdpoqwkdpqwokdpoqwkdpoqwkdpoqwkdpoqwkdpoqwkdpqowdkqppqowkdpqowkdpqowkdpqwkdpoqwkdpqwokdpoqwkdpoqwkdpoqwkdpoqwkdpoqwkdpqowdkqppqowkdpqowkdpqowkdpqwkdpoqwkdpqwokdpoqwkdpoqwkdpoqwkdpoqwkdpoqwkdpqowdkqp",
                "opqwkdpoqwkdpoqwkdpoqkwdpoqkwpdoqkwpodkqwpodkqwpodkqpowkqwdpoqwkdpoqwkdqpowkopqwkdpoqwkdpoqwkdpoqkwdpoqkwpdoqkwpodkqwpodkqwpodkqpowkqwdpoqwkdpoqwkdqpowkopqwkdpoqwkdpoqwkdpoqkwdpoqkwpdoqkwpodkqwpodkqwpodkqpowkqwdpoqwkdpoqwkdqpowkopqwkdpoqwkdpoqwkdpoqkwdpoqkwpdoqkwpodkqwpodkqwpodkqpowkqwdpoqwkdpoqwkdqpowkopqwkdpoqwkdpoqwkdpoqkwdpoqkwpdoqkwpodkqwpodkqwpodkqpowkqwdpoqwkdpoqwkdqpowkopqwkdpoqwkdpoqwkdpoqkwdpoqkwpdoqkwpodkqwpodkqwpodkqpowkqwdpoqwkdpoqwkdqpowkopqwkdpoqwkdpoqwkdpoqkwdpoqkwpdoqkwpodkqwpodkqwpodkqpowkqwdpoqwkdpoqwkdqpowkopqwkdpoqwkdpoqwkdpoqkwdpoqkwpdoqkwpodkqwpodkqwpodkqpowkqwdpoqwkdpoqwkdqpowk",
            456
        );
    }

    fn diff_test_case(s1: &str, s2: &str) {
        let mut dp = &mut vec![vec![-1; s2.len()]; s1.len()];
        lcs(s1, s2, dp);
        diff(s1, s2, dp)
    }

    #[test]
    fn test_diff() {
        diff_test_case("helloasd", "helzxc")
    }
}
