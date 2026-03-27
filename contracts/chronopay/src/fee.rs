

/// Calculate the platform fee based on the amount and basis points.
pub fn calculate_fee(amount: i128, bps: u32) -> i128 {
    //bps: value in basis points (1bps = 0.01%)
    //Calculation: (amount * bps) / 10000
    if amount == 0 || bps == 0 {
        return 0;
    }
    
    amount
        .checked_mul(bps as i128)
        .expect("mul overflow")
        .checked_div(10000)
        .expect("div error")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calculate_fee() {
        assert_eq!(calculate_fee(10000, 100), 100); // 1%
        assert_eq!(calculate_fee(10000, 250), 250); // 2.5%
        assert_eq!(calculate_fee(10000, 0), 0);     // 0%
        assert_eq!(calculate_fee(0, 100), 0);       // 0 amount
        assert_eq!(calculate_fee(123456, 100), 1234); // Floor rounding
        assert_eq!(calculate_fee(10000, 10000), 10000); // 100%
    }
}
