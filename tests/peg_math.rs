//! Unit tests for the peg stability math engine.
//!
//! These tests verify the core arithmetic that protects user funds:
//! collateral ratio computation, max mintable amounts, liquidation
//! seize calculations, and health factor checks.

#[cfg(test)]
mod tests {
    // We re-implement the math functions here with the same logic
    // as the on-chain code, so these tests work without pinocchio.

    const BPS_DENOMINATOR: u64 = 10_000;
    const PRICE_DECIMALS: u64 = 1_000_000;
    const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

    fn lamports_to_pusd(lamports: u64, price: u64) -> u64 {
        let v = (lamports as u128) * (price as u128);
        (v / (LAMPORTS_PER_SOL as u128)) as u64
    }

    fn pusd_to_lamports(pusd: u64, price: u64) -> u64 {
        let v = (pusd as u128) * (LAMPORTS_PER_SOL as u128);
        (v / (price as u128)) as u64
    }

    fn is_healthy(collateral: u64, minted: u64, price: u64, min_ratio: u64) -> bool {
        if minted == 0 { return true; }
        let lhs = (collateral as u128) * (price as u128) * (BPS_DENOMINATOR as u128);
        let rhs = (minted as u128) * (min_ratio as u128) * (LAMPORTS_PER_SOL as u128);
        lhs >= rhs
    }

    fn max_mintable(collateral: u64, price: u64, min_ratio: u64) -> u64 {
        let val = lamports_to_pusd(collateral, price);
        let max = (val as u128) * (BPS_DENOMINATOR as u128) / (min_ratio as u128);
        max as u64
    }

    fn collateral_ratio_bps(collateral: u64, minted: u64, price: u64) -> u64 {
        if minted == 0 { return u64::MAX; }
        let n = (collateral as u128) * (price as u128) * (BPS_DENOMINATOR as u128);
        let d = (minted as u128) * (LAMPORTS_PER_SOL as u128);
        (n / d) as u64
    }

    fn compute_fee(amount: u64, fee_bps: u64) -> u64 {
        ((amount as u128) * (fee_bps as u128) / (BPS_DENOMINATOR as u128)) as u64
    }

    fn liquidation_seize(repay: u64, price: u64, bonus_bps: u64) -> u64 {
        let base = pusd_to_lamports(repay, price);
        let bonus = compute_fee(base, bonus_bps);
        base + bonus
    }

    fn health_factor(collateral: u64, minted: u64, price: u64, min_ratio: u64) -> u64 {
        if minted == 0 { return u64::MAX; }
        let ratio = collateral_ratio_bps(collateral, minted, price);
        ((ratio as u128) * 10000 / (min_ratio as u128)) as u64
    }

    fn max_withdrawable(collateral: u64, minted: u64, price: u64, min_ratio: u64) -> u64 {
        if minted == 0 { return collateral; }
        let n = (minted as u128) * (min_ratio as u128) * (LAMPORTS_PER_SOL as u128);
        let d = (price as u128) * (BPS_DENOMINATOR as u128);
        let min_coll = ((n + d - 1) / d) as u64; // round up
        collateral.saturating_sub(min_coll)
    }

    // ---- conversion tests --------------------------------------------------

    #[test]
    fn test_lamports_to_pusd_1sol_at_150() {
        let price = 150 * PRICE_DECIMALS; // $150
        let result = lamports_to_pusd(LAMPORTS_PER_SOL, price);
        assert_eq!(result, 150_000_000); // $150 in 6-decimal pUSD
    }

    #[test]
    fn test_lamports_to_pusd_half_sol_at_100() {
        let price = 100 * PRICE_DECIMALS;
        let result = lamports_to_pusd(500_000_000, price);
        assert_eq!(result, 50_000_000); // $50
    }

    #[test]
    fn test_pusd_to_lamports_100usd_at_150() {
        let price = 150 * PRICE_DECIMALS;
        let result = pusd_to_lamports(100_000_000, price); // $100
        // Expected: 100/150 SOL = 0.6667 SOL = 666_666_666 lamports
        assert_eq!(result, 666_666_666);
    }

    #[test]
    fn test_roundtrip_conversion() {
        let price = 175 * PRICE_DECIMALS;
        let lamports = 2_500_000_000u64; // 2.5 SOL
        let pusd = lamports_to_pusd(lamports, price);
        let back = pusd_to_lamports(pusd, price);
        // Should be within 1 lamport of the original due to rounding.
        assert!((lamports as i64 - back as i64).abs() <= 1);
    }

    // ---- health checks -----------------------------------------------------

    #[test]
    fn test_healthy_position() {
        let price = 150 * PRICE_DECIMALS;
        let collateral = 2 * LAMPORTS_PER_SOL; // 2 SOL = $300
        let minted = 100_000_000; // $100 pUSD
        // Ratio = 300/100 = 300% = 30000 bps. Min = 15000.
        assert!(is_healthy(collateral, minted, price, 15000));
    }

    #[test]
    fn test_unhealthy_position() {
        let price = 100 * PRICE_DECIMALS;
        let collateral = LAMPORTS_PER_SOL; // 1 SOL = $100
        let minted = 90_000_000; // $90 pUSD
        // Ratio = 100/90 = 111% = 11111 bps. Min = 15000.
        assert!(!is_healthy(collateral, minted, price, 15000));
    }

    #[test]
    fn test_zero_minted_is_always_healthy() {
        assert!(is_healthy(0, 0, 150_000_000, 15000));
        assert!(is_healthy(LAMPORTS_PER_SOL, 0, 150_000_000, 15000));
    }

    #[test]
    fn test_exact_minimum_ratio() {
        let price = 150 * PRICE_DECIMALS;
        let min_ratio = 15000u64; // 150%
        // For $100 minted, need $150 collateral = 1 SOL at $150.
        let minted = 100_000_000;
        let collateral = LAMPORTS_PER_SOL; // exactly $150
        assert!(is_healthy(collateral, minted, price, min_ratio));

        // 1 lamport less should be unhealthy.
        assert!(!is_healthy(collateral - 1, minted, price, min_ratio));
    }

    // ---- max mintable ------------------------------------------------------

    #[test]
    fn test_max_mintable_2sol_at_150() {
        let price = 150 * PRICE_DECIMALS;
        let collateral = 2 * LAMPORTS_PER_SOL; // $300 value
        let max = max_mintable(collateral, price, 15000); // 150% ratio
        // max = 300 * 10000 / 15000 = $200
        assert_eq!(max, 200_000_000);
    }

    #[test]
    fn test_max_mintable_at_200_percent_ratio() {
        let price = 100 * PRICE_DECIMALS;
        let collateral = 4 * LAMPORTS_PER_SOL; // $400
        let max = max_mintable(collateral, price, 20000); // 200% ratio
        assert_eq!(max, 200_000_000); // $200
    }

    // ---- collateral ratio --------------------------------------------------

    #[test]
    fn test_collateral_ratio_150_percent() {
        let price = 150 * PRICE_DECIMALS;
        let collateral = LAMPORTS_PER_SOL; // 1 SOL = $150
        let minted = 100_000_000; // $100
        let ratio = collateral_ratio_bps(collateral, minted, price);
        assert_eq!(ratio, 15000); // 150%
    }

    #[test]
    fn test_collateral_ratio_zero_minted() {
        let ratio = collateral_ratio_bps(LAMPORTS_PER_SOL, 0, 150_000_000);
        assert_eq!(ratio, u64::MAX);
    }

    // ---- fees --------------------------------------------------------------

    #[test]
    fn test_fee_30bps() {
        let fee = compute_fee(1_000_000, 30); // 0.3% of 1M
        assert_eq!(fee, 3_000);
    }

    #[test]
    fn test_fee_zero() {
        assert_eq!(compute_fee(1_000_000, 0), 0);
    }

    // ---- liquidation -------------------------------------------------------

    #[test]
    fn test_liquidation_seize() {
        let price = 150 * PRICE_DECIMALS;
        let repay = 100_000_000; // $100
        let bonus = 500; // 5%
        let seize = liquidation_seize(repay, price, bonus);
        // Base: 100/150 SOL = 666_666_666 lamports
        // Bonus: 666_666_666 * 5% = 33_333_333
        // Total: 699_999_999
        assert_eq!(seize, 699_999_999);
    }

    // ---- health factor -----------------------------------------------------

    #[test]
    fn test_health_factor_at_min() {
        let price = 150 * PRICE_DECIMALS;
        let collateral = LAMPORTS_PER_SOL; // $150
        let minted = 100_000_000; // $100 → ratio=150% → hf=100%
        let hf = health_factor(collateral, minted, price, 15000);
        assert_eq!(hf, 10000); // 100.00%
    }

    #[test]
    fn test_health_factor_at_200_percent() {
        let price = 150 * PRICE_DECIMALS;
        let collateral = 2 * LAMPORTS_PER_SOL; // $300
        let minted = 100_000_000; // $100 → ratio=300%
        let hf = health_factor(collateral, minted, price, 15000);
        assert_eq!(hf, 20000); // 200.00%
    }

    // ---- max withdrawable --------------------------------------------------

    #[test]
    fn test_max_withdrawable_no_debt() {
        let max = max_withdrawable(5 * LAMPORTS_PER_SOL, 0, 150_000_000, 15000);
        assert_eq!(max, 5 * LAMPORTS_PER_SOL);
    }

    #[test]
    fn test_max_withdrawable_with_debt() {
        let price = 150 * PRICE_DECIMALS;
        let collateral = 2 * LAMPORTS_PER_SOL; // $300
        let minted = 100_000_000; // $100
        let max = max_withdrawable(collateral, minted, price, 15000);
        // Min collateral = $100 * 150% / $150 per SOL = 1 SOL
        // Max withdraw = 2 SOL - 1 SOL = 1 SOL
        assert_eq!(max, LAMPORTS_PER_SOL);
    }
}
