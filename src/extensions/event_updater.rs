//! ## Event updater for Tick Data providers
//! Updates Tick data providers by consuming events emitted from UniV3 pool

use crate::prelude::*;
use alloy::{eips::BlockId, rpc::types::Log, sol};
use alloy_primitives::aliases::I24;

sol! {
    interface IUniswapV3PoolEvents {
       event Mint (
           address sender,
           address indexed owner,
           int24 indexed tickLower,
           int24 indexed tickUpper,
           uint128 amount,
           uint256 amount0,
           uint256 amount1
       );
       event Burn(
           address indexed owner,
           int24 indexed tickLower,
           int24 indexed tickUpper,
           uint128 amount,
           uint256 amount0,
           uint256 amount1
        );
        event Swap(
           address indexed sender,
           address indexed recipient,
           int256 amount0,
           int256 amount1,
           uint160 sqrtPriceX96,
           uint128 liquidity,
           int24 tick
       );
    }
}

pub trait EventUpdater {
    fn update_on_mint(&mut self, event: &Log<IUniswapV3PoolEvents::Mint>);
    fn update_on_burn(&mut self, event: &Log<IUniswapV3PoolEvents::Burn>);
}

impl EventUpdater for EphemeralTickDataProvider<I24> {
    #[inline]
    fn update_on_mint(&mut self, event: &Log<IUniswapV3PoolEvents::Mint>) {
        self.block_id = event.block_number.map(BlockId::number);
        let tick_index = event.inner.data.tickLower;
        let liquidity = event.inner.data.amount;
        if let Some(tick_lower) = self
            .ticks
            .iter()
            .find(|t| t.index.eq(&TickIndex::from_i24(tick_index)))
        {
            self.update_tick(
                tick_lower.index,
                tick_lower.liquidity_gross.saturating_add(liquidity),
                tick_lower.liquidity_net.saturating_add(liquidity as i128),
            );
        } else {
            let new_tick = Tick::new(tick_index, liquidity, liquidity as i128);
            self.insert_tick(new_tick);
        };
        let tick_index = event.inner.data.tickUpper;
        if let Some(tick_upper) = self
            .ticks
            .iter()
            .find(|t| t.index.eq(&TickIndex::from_i24(tick_index)))
        {
            self.update_tick(
                tick_upper.index,
                tick_upper.liquidity_gross.saturating_add(liquidity),
                tick_upper.liquidity_net.saturating_sub(liquidity as i128),
            );
        } else {
            let new_tick = Tick::new(tick_index, liquidity, -(liquidity as i128));
            self.insert_tick(new_tick);
        };
    }
    #[inline]
    fn update_on_burn(&mut self, event: &Log<IUniswapV3PoolEvents::Burn>) {
        self.block_id = event.block_number.map(BlockId::number);
        let tick_index = event.inner.data.tickLower;
        let liquidity = event.inner.data.amount;
        if let Some(tick_lower) = self
            .ticks
            .iter()
            .find(|t| t.index.eq(&TickIndex::from_i24(tick_index)))
        {
            self.update_tick(
                tick_lower.index,
                tick_lower.liquidity_gross.saturating_sub(liquidity),
                tick_lower.liquidity_net.saturating_sub(liquidity as i128),
            );
        } else {
            let new_tick = Tick::new(tick_index, liquidity, liquidity as i128);
            self.insert_tick(new_tick);
        };
        let tick_index = event.inner.data.tickUpper;
        if let Some(tick_upper) = self
            .ticks
            .iter()
            .find(|t| t.index.eq(&TickIndex::from_i24(tick_index)))
        {
            self.update_tick(
                tick_upper.index,
                tick_upper.liquidity_gross.saturating_sub(liquidity),
                tick_upper.liquidity_net.saturating_add(liquidity as i128),
            );
        } else {
            let new_tick = Tick::new(tick_index, liquidity, -(liquidity as i128));
            self.insert_tick(new_tick);
        };
    }
}

impl Pool<EphemeralTickDataProvider<I24>> {
    #[inline]
    pub fn update_on_mint(&mut self, event: &Log<IUniswapV3PoolEvents::Mint>) {
        self.liquidity += event.inner.data.amount;
        self.tick_data_provider.update_on_mint(event);
    }
    #[inline]
    pub fn update_on_burn(&mut self, event: &Log<IUniswapV3PoolEvents::Burn>) {
        self.liquidity -= event.inner.data.amount;
        self.tick_data_provider.update_on_burn(event);
    }
    #[inline]
    pub const fn update_on_swap(&mut self, event: &Log<IUniswapV3PoolEvents::Swap>) {
        self.sqrt_ratio_x96 = event.inner.data.sqrtPriceX96;
        self.liquidity = event.inner.data.liquidity;
        self.tick_current = event.inner.data.tick;
    }
}
