#!/usr/bin/env python3
"""
Exact Pump.fun bonding curve mathematical formulas reverse-engineered from contract bytecode.
Uses 64-bit integer virtual reserves:
- INITIAL_VIRTUAL_TOKEN_RESERVES = 1,073,000,000 * 10^6
- INITIAL_VIRTUAL_SOL_RESERVES = 30 * 10^9 (30 SOL)
- INITIAL_REAL_TOKEN_RESERVES = 793,100,000 * 10^6
"""

INITIAL_VIRTUAL_TOKEN_RESERVES = 1_073_000_000_000_000
INITIAL_VIRTUAL_SOL_RESERVES = 30_000_000_000 # 30 SOL
INITIAL_REAL_TOKEN_RESERVES = 793_100_000_000_000
TOTAL_SUPPLY = 1_000_000_000_000_000 # 1 Billion tokens with 6 decimals

class PumpCurve:
    def __init__(self, v_tokens=INITIAL_VIRTUAL_TOKEN_RESERVES, v_sol=INITIAL_VIRTUAL_SOL_RESERVES):
        self.virtual_token_reserves = v_tokens
        self.virtual_sol_reserves = v_sol
        self.real_token_reserves = INITIAL_REAL_TOKEN_RESERVES
        self.real_sol_reserves = 0
        self.k = self.virtual_token_reserves * self.virtual_sol_reserves

    def get_token_price_sol(self) -> float:
        """Current spot price of 1 token in SOL"""
        return (self.virtual_sol_reserves / 1e9) / (self.virtual_token_reserves / 1e6)

    def get_market_cap_sol(self) -> float:
        """Market cap in SOL based on 1 Billion supply"""
        return self.get_token_price_sol() * 1_000_000_000

    def get_progress_percent(self) -> float:
        """Bonding curve migration progress (reaches 100% when 85 SOL accumulated)"""
        return min(100.0, (self.real_sol_reserves / (85 * 1e9)) * 100.0)

    def calculate_buy_output(self, sol_in_lamports: int) -> int:
        """Calculates tokens received for a given SOL amount"""
        new_v_sol = self.virtual_sol_reserves + sol_in_lamports
        new_v_tokens = self.k // new_v_sol
        tokens_out = self.virtual_token_reserves - new_v_tokens
        return tokens_out

    def execute_buy(self, sol_in_lamports: int) -> int:
        """Executes buy and updates state"""
        tokens_out = self.calculate_buy_output(sol_in_lamports)
        self.virtual_sol_reserves += sol_in_lamports
        self.virtual_token_reserves -= tokens_out
        self.real_sol_reserves += sol_in_lamports
        self.real_token_reserves -= tokens_out
        return tokens_out

    def calculate_sell_output(self, tokens_in: int) -> int:
        """Calculates SOL received for a given token amount"""
        new_v_tokens = self.virtual_token_reserves + tokens_in
        new_v_sol = self.k // new_v_tokens
        sol_out = self.virtual_sol_reserves - new_v_sol
        return sol_out

    def execute_sell(self, tokens_in: int) -> int:
        """Executes sell and updates state"""
        sol_out = self.calculate_sell_output(tokens_in)
        self.virtual_token_reserves += tokens_in
        self.virtual_sol_reserves -= sol_out
        self.real_sol_reserves = max(0, self.real_sol_reserves - sol_out)
        self.real_token_reserves += tokens_in
        return sol_out
