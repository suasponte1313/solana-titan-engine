//! Constant product bonding curve math for Pump.fun / Mayhem tokens
use crate::error::TitanSwarmError;

pub struct CurveState {
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
}

impl CurveState {
    /// Calculate exact token output for a given SOL input
    pub fn calculate_tokens_out(&self, sol_in: u64) -> Result<u64, TitanSwarmError> {
        let v_token = self.virtual_token_reserves as u128;
        let v_sol = self.virtual_sol_reserves as u128;
        let sol_in_128 = sol_in as u128;

        let k = v_token.checked_mul(v_sol).ok_or(TitanSwarmError::MathOverflow)?;
        let new_v_sol = v_sol.checked_add(sol_in_128).ok_or(TitanSwarmError::MathOverflow)?;
        let new_v_token = k.checked_div(new_v_sol).ok_or(TitanSwarmError::MathOverflow)?;

        let tokens_out = v_token.checked_sub(new_v_token).ok_or(TitanSwarmError::MathOverflow)?;
        Ok(tokens_out as u64)
    }

    /// Calculate exact SOL output for a given token input
    pub fn calculate_sol_out(&self, tokens_in: u64) -> Result<u64, TitanSwarmError> {
        let v_token = self.virtual_token_reserves as u128;
        let v_sol = self.virtual_sol_reserves as u128;
        let tokens_in_128 = tokens_in as u128;

        let k = v_token.checked_mul(v_sol).ok_or(TitanSwarmError::MathOverflow)?;
        let new_v_token = v_token.checked_add(tokens_in_128).ok_or(TitanSwarmError::MathOverflow)?;
        let new_v_sol = k.checked_div(new_v_token).ok_or(TitanSwarmError::MathOverflow)?;

        let sol_out = v_sol.checked_sub(new_v_sol).ok_or(TitanSwarmError::MathOverflow)?;
        Ok(sol_out as u64)
    }
}
