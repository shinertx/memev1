# Strategy Template: [STRATEGY_NAME]

> **Use this template to document any new trading strategy before implementation.**
> 
> **Instructions:** Copy this file, rename it, and fill out all sections completely. This document serves as the blueprint and specification for your strategy implementation.

## 📋 Strategy Overview

**Strategy ID:** `[unique_strategy_identifier]`  
**Type:** [Momentum/Mean-Reversion/Arbitrage/Event-Driven/Multi-Factor]  
**Timeframe:** [5m/1h/4h/1d]  
**Asset Class:** [Memecoins/DeFi Tokens/All]  

### Executive Summary
*One paragraph describing what this strategy does and why it should be profitable.*

### Core Hypothesis
*The fundamental market inefficiency or pattern this strategy exploits.*

## 📊 Data Requirements

### Market Events Subscribed To
*List all `EventType`s your strategy needs:*
- [ ] `Price` - Token price updates
- [ ] `Social` - Social sentiment data  
- [ ] `Depth` - Order book depth changes
- [ ] `Bridge` - Cross-chain bridge activity
- [ ] `Funding` - Perpetual funding rates
- [ ] `OnChain` - On-chain transaction data

### External Data Sources
*Any additional APIs or services required:*
- API Name: Purpose and frequency
- API Name: Purpose and frequency

## ⚙️ Strategy Parameters

### Required Parameters
```json
{
  "param_name": {
    "type": "number|string|boolean",
    "default": "value", 
    "description": "What this controls",
    "min": "minimum_value",
    "max": "maximum_value"
  }
}
```

### Example Configuration
```json
{
  "lookback_periods": 20,
  "threshold": 0.05,
  "volume_multiplier": 2.0,
  "stop_loss_pct": 0.02,
  "take_profit_pct": 0.06
}
```

## 🎯 Entry & Exit Logic

### Entry Conditions
*Precisely describe when to enter a position:*
1. Condition 1 (e.g., price crosses above SMA)
2. Condition 2 (e.g., volume > 2x average)
3. Condition 3 (e.g., sentiment score > threshold)

### Exit Conditions
*Precisely describe when to close positions:*
1. **Take Profit:** When to realize gains
2. **Stop Loss:** Risk management exit
3. **Time-based:** Maximum holding period
4. **Signal-based:** Counter-signal detection

### Position Sizing
*How the strategy determines trade size:*
- Fixed size vs. dynamic sizing
- Risk management considerations
- Maximum position limits

## 🔄 Implementation Checklist

### Core Methods
- [ ] `id()` - Returns unique strategy identifier
- [ ] `subscriptions()` - Returns HashSet of required EventTypes  
- [ ] `init()` - Initialize with parameters from JSON
- [ ] `on_event()` - Main strategy logic for each market event

### State Management
- [ ] Define internal state variables (VecDeque for history, counters, etc.)
- [ ] Handle data capacity limits (e.g., rolling windows)
- [ ] Reset logic if needed

### Error Handling
- [ ] Graceful handling of missing data
- [ ] Validation of parameters on init
- [ ] Fallback behavior for edge cases

## 📈 Expected Performance

### Key Metrics
- **Expected Win Rate:** X%
- **Average Hold Time:** X hours/days
- **Risk-Reward Ratio:** X:1
- **Maximum Drawdown:** X%
- **Sharpe Ratio Target:** X

### Backtesting Results
*If available, include historical performance data*

## ⚠️ Risk Assessment

### Strategy-Specific Risks
1. **Market Risk:** How strategy performs in different market conditions
2. **Liquidity Risk:** Exposure to low-volume tokens
3. **Technical Risk:** Dependency on data feeds or external services
4. **Model Risk:** Overfitting or parameter sensitivity

### Risk Mitigation
- Position limits and stop losses
- Diversification across tokens
- Circuit breakers for unusual conditions
- Regular parameter review and adjustment

## 🧪 Testing Strategy

### Unit Tests
- [ ] Parameter validation
- [ ] Event processing logic  
- [ ] Edge case handling
- [ ] State management

### Integration Tests
- [ ] Event subscription verification
- [ ] Order generation accuracy
- [ ] Error propagation

### Simulation Tests
- [ ] Historical data replay
- [ ] Stress testing with extreme events
- [ ] Parameter sensitivity analysis

## 📝 Implementation Notes

### Code Structure
```rust
use crate::{register_strategy, strategies::{Strategy, MarketEvent, StrategyAction, OrderDetails, EventType}};
use shared_models::Side;

#[derive(Default, Deserialize)]
struct [StrategyName] {
    // Parameters
    // State variables
}

#[async_trait]
impl Strategy for [StrategyName] {
    // Implementation
}

register_strategy!([StrategyName]);
```

### Registration in factory.py
```python
"[strategy_id]": {
    "param1": default_value,
    "param2": default_value,
    # ... other parameters
}
```

## 📚 References

*Academic papers, articles, or other strategies that inspired this approach*

---

**Author:** [Your Name]  
**Created:** [Date]  
**Last Updated:** [Date]  
**Status:** [Draft/Review/Approved/Implemented]
