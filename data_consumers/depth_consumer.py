import redis
import json
import time
import random
import os

def main():
    print("🚀 Starting Depth Event Consumer/Simulator...")
    r = redis.Redis.from_url(os.getenv("REDIS_URL", "redis://redis:6379"), decode_responses=True)
    tokens = ["SOL_MEME1", "SOL_MEME2", "SOL_MEME3", "SOL_MEME4", "SOL_MEME5"]
    
    # Simulate current prices for bid/ask
    current_prices = {t: 1.0 for t in tokens}

    while True:
        for token in tokens:
            # Simulate bid/ask spread
            spread_pct = random.uniform(0.001, 0.01) # 0.1% to 1% spread
            bid_price = current_prices[token] * (1 - spread_pct / 2)
            ask_price = current_prices[token] * (1 + spread_pct / 2)
            
            # Simulate bid/ask sizes
            bid_size = random.uniform(1000, 10000)
            ask_size = random.uniform(1000, 10000)

            event = {
                "type": "Depth",
                "token_address": token,
                "bid_price": bid_price,
                "ask_price": ask_price,
                "bid_size_usd": bid_size,
                "ask_size_usd": ask_size,
            }
            # P-7: Use XADD for Redis Streams
            r.xadd("events:depth", {"event": json.dumps(event)})
            # Update current price slightly for next iteration
            current_prices[token] += random.uniform(-0.005, 0.005)
            if current_prices[token] < 0.01: current_prices[token] = 0.01

        time.sleep(1) # Publish depth events frequently

if __name__ == "__main__":
    main()