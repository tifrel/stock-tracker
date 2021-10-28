fn avg(series: &[f64]) -> Option<f64> {
    match series.len() {
        0 => None,
        n => Some(series.iter().fold(0.0, |sum, &x| sum + x) / n as f64),
    }
}

pub fn min(series: &[f64]) -> Option<f64> {
    series
        .iter()
        .reduce(|min, x| if x < min { x } else { min })
        .map(|x| *x)
}

pub fn max(series: &[f64]) -> Option<f64> {
    series
        .iter()
        .reduce(|max, x| if x > max { x } else { max })
        .map(|x| *x)
}

pub fn sma(series: &[f64], n: usize) -> Option<Vec<f64>> {
    match (n, series.len()) {
        (0, _) => None,
        (_, 0) => None,
        (n, l) if n > l => None,
        (n, _) => series.windows(n).map(|xs| avg(xs)).collect(),
    }
}

pub fn diff(series: &[f64]) -> Option<(f64, f64)> {
    match series.len() {
        0..=1 => None,
        l => {
            let abs = series[l - 1] - series[0];
            let rel = series[l - 1] / series[0];
            Some((rel, abs))
        }
    }
}

#[cfg(test)]
mod util_tests {
    use super::*;

    #[test]
    fn test_min() {
        assert_eq!(min(&[]), None);
        assert_eq!(min(&[0.0]), Some(0.0));
        assert_eq!(min(&[3.0, 1.0, 2.0]), Some(1.0));
    }

    #[test]
    fn test_max() {
        assert_eq!(max(&[]), None);
        assert_eq!(max(&[0.0]), Some(0.0));
        assert_eq!(max(&[1.0, 3.0, 2.0]), Some(3.0));
    }

    #[test]
    fn test_diff() {
        assert_eq!(diff(&[0.0]), None);
        assert_eq!(diff(&[]), None);
        assert_eq!(diff(&[1.0, 3.0, 2.0]), Some((2.0, 1.0)));
        assert_eq!(diff(&[1.0, 3.0, 4.0, 2.0]), Some((2.0, 1.0)));
    }

    #[test]
    fn test_sma() {
        assert_eq!(sma(&[1.0], 1), Some(vec![1.0]));
        assert_eq!(sma(&[1.0, 2.0], 1), Some(vec![1.0, 2.0]));
        assert_eq!(sma(&[1.0, 2.0], 2), Some(vec![1.5]));
        assert_eq!(sma(&[1.0, 2.0, 3.0], 2), Some(vec![1.5, 2.5]));
        assert_eq!(sma(&[1.0], 2), None);
        assert_eq!(sma(&[], 1), None);
        assert_eq!(sma(&[1.0, 2.0], 0), None);
        assert_eq!(sma(&[1.0, 2.0], 3), None);
    }
}
