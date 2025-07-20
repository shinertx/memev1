import redis
import json
import time
import random
import os

def main():
    print("🚀 Starting Bridge Event Consumer/Simulator...")
    r = redis.Redis.from_url(os.getenv("REDIS_URL", "redis://redis:6379"), decode_responses=True)
    # Use the same tokens as the main factory for consistency
    tokens = ["SOL_MEME1", "SOL_MEME2", "SOL_MEME3", "SOL_MEME4", "SOL_MEME5"]

    while True:
        # Bridge events are less frequent but high-impact
        if random.random() < 0.1: # 10% chance every 10 seconds
            event = {
                "type": "Bridge",
                "token_address": random.choice(tokens),
                "source_chain": "ethereum",
                "destination_chain": "solana",
                "volume_usd": random.uniform(50_000, 250_000)
            }
            # P-7: Use XADD for Redis Streams for persistence
            r.xadd("events:bridge", {"event": json.dumps(event)})
            print(f"Published Bridge Event: {event['token_address']} bridged ${event['volume_usd']:.2f}")
        time.sleep(10)

if __name__ == "__main__":
    main()