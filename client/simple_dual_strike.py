#!/usr/bin/env python3
"""
🗡️ DUAL-STRIKE (SIMPLE 2-WALLET ENGINE)
--------------------------------------------------------------------------------
Architecture:
- Wallet 0 (Deployer): Deploys token + 0.05 SOL dev buy. (BubbleMaps = Clean <1%)
- Wallet 1 (Shadow Sniper): Atomically buys 1.5 SOL in Block 0 Jito bundle.
- 8-Second Blitz: Liquidates 100% at Second 8.0 before any human dev dump.
- Reclaims ATA Rent lamports back to Master Vault.
"""

import time
import random
from client.pump_curve import PumpCurve
from client.jito_bundle import JitoBundleClient

class SimpleDualStrikeEngine:
    def __init__(self):
        self.jito = JitoBundleClient(region="frankfurt")
        self.wallet_creator = "Cre8tor111111111111111111111111111111111111"
        self.wallet_sniper = "Snip3r2222222222222222222222222222222222222"
        self.active_position = None

    def execute_dual_strike_launch(self, token_symbol="TITAN", dev_buy_sol=0.05, sniper_buy_sol=1.5):
        """Executes Block 0 bundled launch + sniper buy"""
        curve = PumpCurve()

        # Step 1: Creator Dev Buy (<1% supply)
        dev_lamports = int(dev_buy_sol * 1e9)
        dev_tokens = curve.execute_buy(dev_lamports)

        # Step 2: Wallet 1 Shadow Sniper (Genesis Dip)
        sniper_lamports = int(sniper_buy_sol * 1e9)
        sniper_tokens = curve.execute_buy(sniper_lamports)

        bundle_txs = [
            f"TX_CREATE_{token_symbol}",
            f"TX_DEV_BUY_{dev_buy_sol}_SOL",
            f"TX_SHADOW_SNIPE_{sniper_buy_sol}_SOL"
        ]

        bundle_res = self.jito.send_bundle(bundle_txs, tip_lamports=150_000)

        self.active_position = {
            "token_symbol": token_symbol,
            "creator_wallet": self.wallet_creator,
            "sniper_wallet": self.wallet_sniper,
            "entry_time": time.time(),
            "sniper_cost_sol": sniper_buy_sol,
            "sniper_tokens": sniper_tokens,
            "curve_progress_start": curve.get_progress_percent(),
            "status": "ARMED_AND_SNIPED",
            "curve": curve,
            "bundle": bundle_res
        }
        return self.active_position

    def simulate_public_inflow(self, elapsed_seconds: float):
        """Simulates incoming retail & copy-bot volume over 8 seconds"""
        if not self.active_position:
            return None

        curve = self.active_position["curve"]

        # Retail volume accelerates over seconds 1 to 7
        if elapsed_seconds < 8.0:
            retail_sol_rate = 0.4 + (elapsed_seconds * 0.25) + random.uniform(0.1, 0.3)
            curve.execute_buy(int(retail_sol_rate * 1e9))

        curr_sol_val = curve.calculate_sell_output(self.active_position["sniper_tokens"]) / 1e9
        cost_sol = self.active_position["sniper_cost_sol"]
        pnl_sol = curr_sol_val - cost_sol
        roi_pct = (pnl_sol / cost_sol) * 100.0

        return {
            "elapsed_seconds": round(elapsed_seconds, 1),
            "current_value_sol": round(curr_sol_val, 4),
            "unrealized_pnl_sol": round(pnl_sol, 4),
            "roi_percent": round(roi_pct, 1),
            "curve_progress": round(curve.get_progress_percent(), 2),
            "market_cap_sol": round(curve.get_market_cap_sol(), 1)
        }

    def force_blitz_exit(self):
        """Hard Stop Second 8.0: 100% liquidation + rent reclamation"""
        if not self.active_position:
            return {"error": "No active position"}

        curve = self.active_position["curve"]
        tokens = self.active_position["sniper_tokens"]
        sol_received = curve.execute_sell(tokens) / 1e9

        # ATA rent reclaimed (+0.002039 SOL)
        rent_reclaimed = 0.00203928
        total_payout = sol_received + rent_reclaimed
        net_profit = total_payout - self.active_position["sniper_cost_sol"]

        result = {
            "status": "LIQUIDATED_8S_BLITZ",
            "entry_cost_sol": self.active_position["sniper_cost_sol"],
            "sol_received": round(sol_received, 4),
            "rent_reclaimed_sol": round(rent_reclaimed, 6),
            "total_extracted_sol": round(total_payout, 4),
            "net_profit_sol": round(net_profit, 4),
            "roi_percent": round((net_profit / self.active_position["sniper_cost_sol"]) * 100.0, 1),
            "exit_reason": "Hard 8.0s Time Stop — Ahead of Dev Dump Window",
            "timestamp": time.time()
        }
        self.active_position = None
        return result

SIMPLE_ENGINE = SimpleDualStrikeEngine()
