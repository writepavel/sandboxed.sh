const allowedOrigins = ['http://localhost:3000'];

window.bridge = {
  allowedOrigins,
  checkOrigin() {
    const origin = window.location.origin;
    if (!this.allowedOrigins.includes(origin)) {
      throw new Error(`Origin not allowed: ${origin}. Allowed: ${this.allowedOrigins.join(', ')}`);
    }
    console.log('Origin check passed for:', origin);
    return true;
  },
  async connect() {
    this.checkOrigin();
    console.log('Bridge connected from', window.location.origin);
    return true;
  },
  async signMessage(message) {
    this.checkOrigin();
    console.log('Signing message from', window.location.origin);
    return `0x${'signature'.repeat(20)} for "${message}"`;
  },
  async sendTransaction(tx) {
    this.checkOrigin();
    console.log('Sending tx from', window.location.origin, tx);
    return `0x${'123456789abcdef'.repeat(4)}`;
  },
  isConnected() {
    try {
      this.checkOrigin();
      return true;
    } catch {
      return false;
    }
  }
};

console.log('Mock JS Bridge loaded. Allowed origins:', allowedOrigins.join(', '));