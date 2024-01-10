use crate::utils::panic::panic_with_stacktrace;

/// Returns the 1st percentile of the given values.
/// Avoids MIN if possible.
pub fn p1<T: Copy>(values: &Vec<T>) -> T {
    p_higher(values, 0.01)
}

/// Returns the 50th percentile of the given values.
/// Avoids MAX if possible.
pub fn p50<T: Copy>(values: &Vec<T>) -> T {
    p_lower(values, 0.5)
}

/// Returns the 99th percentile of the given values.
/// Avoids MAX if possible.
pub fn p99<T: Copy>(values: &Vec<T>) -> T {
    p_lower(values, 0.99)
}

fn p_lower<T: Copy>(values: &Vec<T>, percentile: f64) -> T {
    if values.is_empty() {
        panic_with_stacktrace("Cannot calculate percentile of empty vector");
    }

    let index = match (values.len() as f64 * percentile) as usize {
        0 => 0,
        index => index - 1,
    };
    values[index]
}

fn p_higher<T: Copy>(values: &Vec<T>, percentile: f64) -> T {
    if values.is_empty() {
        panic_with_stacktrace("Cannot calculate percentile of empty vector");
    }

    let index = match (values.len() as f64 * percentile) as usize {
        0 => if values.len() == 1 { 0 } else { 1 },
        index => index,
    };
    values[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(n: usize) -> Vec<i32> {
        let mut values: Vec<i32> = Vec::new();
        for i in 1..=n {
            values.push(i as i32);
        }
        values
    }

    mod test_p1 {
        use super::*;

        #[test]
        fn test_p1_of_1_value() {
            let values: Vec<i32> = n(1);
            assert_eq!(p1(&values), 1);
        }

        #[test]
        fn test_p1_of_2_values() {
            let values: Vec<i32> = n(2);
            assert_eq!(p1(&values), 2);
        }

        #[test]
        fn test_p1_of_10_values() {
            let values: Vec<i32> = n(10);
            assert_eq!(p1(&values), 2);
        }

        #[test]
        fn test_p1_of_100_values() {
            let values: Vec<i32> = n(100);
            assert_eq!(p1(&values), 2);
        }

        #[test]
        fn test_p1_of_1000_values() {
            let values: Vec<i32> = n(1000);
            assert_eq!(p1(&values), 11);
        }
    }

    mod test_p50 {
        use super::*;

        #[test]
        fn test_p50_of_1_value() {
            let values: Vec<i32> = n(1);
            assert_eq!(p50(&values), 1);
        }

        #[test]
        fn test_p50_of_2_values() {
            let values: Vec<i32> = n(2);
            assert_eq!(p50(&values), 1);
        }

        #[test]
        fn test_p50_of_10_values() {
            let values: Vec<i32> = n(10);
            assert_eq!(p50(&values), 5);
        }

        #[test]
        fn test_p50_of_100_values() {
            let values: Vec<i32> = n(100);
            assert_eq!(p50(&values), 50);
        }

        #[test]
        fn test_p50_of_1000_values() {
            let values: Vec<i32> = n(1000);
            assert_eq!(p50(&values), 500);
        }
    }

    mod test_p99 {
        use super::*;

        #[test]
        fn test_p99_of_10_values() {
            let values: Vec<i32> = n(10);
            assert_eq!(p99(&values), 9);
        }

        #[test]
        fn test_p99_of_100_values() {
            let values: Vec<i32> = n(100);
            assert_eq!(p99(&values), 99);
        }

        #[test]
        fn test_p99_of_1000_values() {
            let values: Vec<i32> = n(1000);
            assert_eq!(p99(&values), 990);
        }
    }
}
