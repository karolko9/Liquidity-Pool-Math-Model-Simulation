struct TokenAmount(u64);
struct StakedTokenAmount(u64);
struct LpTokenAmount(u64);
struct Price(u64);
struct Percentage(u64);

struct LpPool {
    price: Price,
    token_amount: TokenAmount,
    st_token_amount: StakedTokenAmount,
    lp_token_amount: LpTokenAmount,
    liquidity_target: TokenAmount,
    min_fee: Percentage,
    max_fee: Percentage,
}

impl LpPool {
    pub fn init(price: Price, min_fee: Percentage, max_fee: Percentage, liquidity_target: TokenAmount) -> Result<Self, &'static str> {
        Ok(LpPool {
            price,
            token_amount: TokenAmount(0),
            st_token_amount: StakedTokenAmount(0),
            lp_token_amount: LpTokenAmount(0),
            liquidity_target,
            min_fee,
            max_fee,
        })
    }

    pub fn add_liquidity(&mut self, token_amount: TokenAmount) -> Result<LpTokenAmount, &'static str> {
        if self.lp_token_amount.0 == 0 {

            self.token_amount.0 += token_amount.0;
            let minted_lp = LpTokenAmount(token_amount.0);
            self.lp_token_amount.0 += minted_lp.0;
            Ok(minted_lp)
        } else {
            let value_stored = self.token_amount.0 as f64 + self.st_token_amount.0 as f64 * self.price.0 as f64 / 100_000.0;
            let lp_ratio = self.lp_token_amount.0 as f64/ value_stored;
            let minted_lp = (lp_ratio * token_amount.0 as f64).round() as u64;
            self.token_amount.0 += token_amount.0;
            self.lp_token_amount.0 += minted_lp;
            Ok(LpTokenAmount(minted_lp))
        }
    }
    

    pub fn remove_liquidity(&mut self, lp_token_amount: LpTokenAmount) -> Result<(TokenAmount, StakedTokenAmount), &'static str> {
        if lp_token_amount.0 > self.lp_token_amount.0 {
            return Err("Not enough LP tokens");
        }
        
        let token_return = (lp_token_amount.0 as f64 * self.token_amount.0 as f64 / self.lp_token_amount.0 as f64) as u64;
        let staked_token_return = (lp_token_amount.0 as f64 * self.st_token_amount.0 as f64 / self.lp_token_amount.0 as f64) as u64;
    
        if token_return > self.token_amount.0 {
            return Err("Not enough tokens available");
        }
    
        self.token_amount.0 -= token_return;
        self.st_token_amount.0 -= staked_token_return;
        self.lp_token_amount.0 -= lp_token_amount.0;
    
        Ok((TokenAmount(token_return), StakedTokenAmount(staked_token_return)))
    }
    
    pub fn swap(&mut self, staked_token_amount: StakedTokenAmount) -> Result<TokenAmount, &'static str> {
        let staked_value = staked_token_amount.0 as f64 * self.price.0 as f64 / 100_000.0;
        let amount_after = self.token_amount.0 as f64 - staked_value;
    
        let fee = if amount_after > self.liquidity_target.0 as f64 {
            self.min_fee.0 as f64 / 1000.0
        } else {
            self.max_fee.0 as f64 / 1000.0 - ((self.max_fee.0 as f64 / 1000.0 - self.min_fee.0 as f64 / 1000.0) * amount_after / self.liquidity_target.0 as f64)
        };
    
        let token_amount = staked_value * (1.0 - fee);
    
        if token_amount > self.token_amount.0 as f64 {
            return Err("Insufficient liquidity");
        }
    
        let token_amount_rounded = token_amount as u64;
    
        self.token_amount.0 -= token_amount_rounded;
        self.st_token_amount.0 += staked_token_amount.0;
    
        Ok(TokenAmount(token_amount_rounded))
    }
    
    


    // 1. LpPool::init(price=1.5, min_fee=0.1%, max_fee9%, liquidity_target=90.0 Token) -> return
    // lp_pool
    // 2. lp_pool.add_liquidity(100.0 Token) -> return 100.0 LpToken
    // 3. lp_pool.swap(6 StakedToken) -> return 8.991 Token
    // 4. lp_pool.add_liquidity(10.0 Token) -> 9.9991 LpToken
    // 5. lp_pool.swap(30.0 StakedToken) -> return 43.44237 Token
    // 6. lp_pool.remove_liquidity(109.9991) -> return (57.56663 Token, 36 StakedToken)


}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lp_pool_initialization() {
        let mut pool = LpPool::init(Price(1_50000), Percentage(1), Percentage(90), TokenAmount(90_00000)).unwrap();
        assert_eq!(pool.price.0, 1_50000);
        assert_eq!(pool.token_amount.0, 0);
        assert_eq!(pool.st_token_amount.0, 0);
        assert_eq!(pool.lp_token_amount.0, 0);
        assert_eq!(pool.liquidity_target.0, 90_00000);
        assert_eq!(pool.min_fee.0, 1); 
        assert_eq!(pool.max_fee.0, 90); 
    }

    #[test]
    fn test_add_liquidity() {
        let mut pool = LpPool::init(Price(1_50000), Percentage(1), Percentage(90), TokenAmount(90_00000)).unwrap();
        let minted_lp = pool.add_liquidity(TokenAmount(100_00000)).unwrap();
        assert_eq!(minted_lp.0, 100_00000);
        assert_eq!(pool.token_amount.0, 100_00000);
        assert_eq!(pool.lp_token_amount.0, 100_00000);
    }

    #[test]
    fn test_swap_1() {
        let mut pool = LpPool::init(Price(1_50000), Percentage(1), Percentage(90), TokenAmount(90_00000)).unwrap();
        pool.add_liquidity(TokenAmount(100_00000)).unwrap();
        let received_token = pool.swap(StakedTokenAmount(6_00000)).unwrap();

        assert_eq!(received_token.0, 899100);
    }

    #[test]
    fn test_add_liquidity_2() {
        let mut pool = LpPool::init(Price(1_50000), Percentage(1), Percentage(90), TokenAmount(90_00000)).unwrap();
        pool.add_liquidity(TokenAmount(100_00000)).unwrap();
        pool.swap(StakedTokenAmount(6_00000)).unwrap(); 
        let minted_lp = pool.add_liquidity(TokenAmount(10_00000)).unwrap();

        assert_eq!(minted_lp.0, 9_99910);

    }

    #[test]
    fn test_swap_2() {
        let mut pool = LpPool::init(Price(1_50000), Percentage(1), Percentage(90), TokenAmount(90_00000)).unwrap();
        pool.add_liquidity(TokenAmount(100_00000)).unwrap();
        pool.swap(StakedTokenAmount(6_00000)).unwrap(); 
        pool.add_liquidity(TokenAmount(10_00000)).unwrap();
        let received_token = pool.swap(StakedTokenAmount(30_00000)).unwrap();

        assert_eq!(received_token.0, 4344237);

    }

    #[test]
    fn test_remove_liquidity() {
        let mut pool = LpPool::init(Price(1_50000), Percentage(1), Percentage(90), TokenAmount(90_00000)).unwrap();
        pool.add_liquidity(TokenAmount(100_00000)).unwrap();
        pool.swap(StakedTokenAmount(6_000)).unwrap(); 
        pool.add_liquidity(TokenAmount(10_000)).unwrap();
        pool.swap(StakedTokenAmount(30_000)).unwrap();

        let (token, staked_token) = pool.remove_liquidity(LpTokenAmount(109_999)).unwrap();
        assert!((token.0 as f64 - 57_566.63).abs() < 0.01);
        assert_eq!(staked_token.0, 36_000);
    }
}

    


fn main() {
    println!("Liquidity Pool Simulation");
}
