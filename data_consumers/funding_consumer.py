import redis
import json
import time
import random
import os

def main():
    print("🚀 Starting Funding Event Consumer/Simulator...")
    r = redis.Redis.from_url(os.getenv("REDIS_URL", "redis://redis:6379"), decode_responses=True)
    tokens = ["SOL_MEME1", "SOL_MEME2", "SOL_MEME3", "SOL_MEME4", "SOL_MEME5"]

    while True:
        for token in tokens:
            # Simulate funding rate (can be positive or negative)
            funding_rate_pct = random.uniform(-0.005, 0.005) # -0.5% to +0.5%
            
            # Simulate next funding time (e.g., every 8 hours)
            next_funding_time_sec = int(time.time()) + random.randint(1, 8) * 3600

            event = {
                "type": "Funding",
                "token_address": token,
                "funding_rate_pct": funding_rate_pct,
                "next_funding_time_sec": next_funding_time_sec,
            }
            # P-7: Use XADD for Redis Streams
            r.xadd("events:funding", {"event": json.dumps(event)})
        
        time.sleep(30) # Funding rates update less frequently

if __name__ == "__main__":
    main()