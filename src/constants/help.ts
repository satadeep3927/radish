export interface FAQItem {
  question: string;
  answer: string;
}

export const FAQ_ITEMS: FAQItem[] = [
  {
    question: "How do I connect my application to Radish?",
    answer: "Radish implements the standard RESP protocol. You can connect to it using any standard client (like `valkey-cli`, `redis-cli`, or `ioredis`) by pointing it to `127.0.0.1:6379`."
  },
  {
    question: "Does Radish persist my data?",
    answer: "Yes! Radish saves snapshot dumps to the `dump.radish` file in your current working directory when it safely shuts down or periodically based on your persistence settings. It uses an internal binary format optimized for fast loading."
  },
  {
    question: "Why is my Pub/Sub message not showing up?",
    answer: "Make sure you are actively subscribed to the correct channel in the **Pub/Sub** tab. Messages sent while you are disconnected or unsubscribed are not retained in memory."
  },
  {
    question: "What data structures are supported?",
    answer: "Radish currently supports the core data types: `Strings`, `Hashes`, `Lists`, and `Sets`. It also provides a robust Pub/Sub mechanism and standard generic commands like `EXPIRE` and `TTL`."
  },
  {
    question: "Does Radish require authentication?",
    answer: "By default, for local development environments, Radish runs without requiring authentication. However, it fully supports the ACL commands and you can secure it with a password via the Configuration tab."
  },
  {
    question: "How do I change the default port?",
    answer: "You can change the default port (6379) directly from the **Config** tab. Make sure to restart the Radish engine via the Service tab for the new port binding to take effect."
  }
];
