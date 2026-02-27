# 🚀 Solana AMM — Constant Product Automated Market Maker (Anchor)

A fully functional **Automated Market Maker (AMM)** built using **Solana + Anchor**, inspired by protocols like Raydium and Uniswap V2.

This project implements a secure, PDA-based constant product liquidity pool with swap functionality, protocol fee accounting, slippage protection, and invariant enforcement.

---

## 🌐 Live Deployment

**Devnet Program ID:**

```
EEzowh3RqroMtA7rsV1S88DUurNLh64PDAK2AfDBcEZe
```

View on Solana Explorer:

https://explorer.solana.com/address/EEzowh3RqroMtA7rsV1S88DUurNLh64PDAK2AfDBcEZe?cluster=devnet

---

# ✨ Features

- ✅ Initialize treasury (protocol fee collection)
- ✅ Create liquidity pool (PDA-based)
- ✅ Add liquidity (LP minting)
- ✅ Remove liquidity (LP burning)
- ✅ Token swaps (A ↔ B)
- ✅ Slippage protection
- ✅ Constant product invariant enforcement
- ✅ Protocol + LP fee splitting
- ✅ Secure vault authority using PDAs
- ✅ Success & failure test coverage

---

# 🧠 AMM Model

This AMM follows the **constant product formula**:

```
x * y = k
```

Where:

- `x` = reserve of Token A
- `y` = reserve of Token B
- `k` = invariant constant

After every swap:

```
k_after >= k_before
```

Fees ensure that liquidity providers gain value over time.

---

# 🏗 Architecture

```
User
 │
 ▼
User Token Accounts
 │
 ▼
Vault Token Accounts (PDA owned)
 │
 ▼
Pool Account (reserves + config)
 │
 ▼
Treasury (protocol fees)
```

### Core Accounts

| Account | Purpose |
|----------|----------|
| Pool PDA | Stores reserves & configuration |
| Vault A/B | Holds liquidity tokens |
| Vault Authority | PDA signer for vault transfers |
| LP Mint | Represents liquidity share |
| Treasury | Accumulates protocol fees |

---

# 🔐 Security Properties

- PDA-based vault authority prevents unauthorized withdrawals
- Slippage protection prevents bad trades
- Checked arithmetic prevents overflow
- Invariant enforcement protects pool integrity
- Deterministic pool PDA ensures uniqueness per token pair

---

# 📊 Fee Model

Total swap fee is split into:

- LP Fee → stays inside pool (increases `k`)
- Protocol Fee → sent to treasury

This ensures:

- LPs earn yield
- Protocol earns sustainable revenue

---

# 🧮 Swap Formula

```
amount_out =
(reserve_out * amount_in_with_fee)
-----------------------------------
(reserve_in + amount_in_with_fee)
```

---

# 📁 Project Structure

```
amm-capstone/
│
├── programs/
│   └── amm-capstone/
│       ├── instructions/
│       ├── state/
│       ├── math.rs
│       ├── errors.rs
│       └── lib.rs
│
├── tests/
│   └── amm-capstone.ts
│
├── Anchor.toml
├── Cargo.toml
└── README.md
```

---

# ⚙️ Requirements

Install:

- Rust
- Node.js (>= 18)
- Yarn
- Solana CLI (recommended: 1.18.x)
- Anchor CLI (0.32.x)

---

# 🔧 Setup

Clone repository:

```bash
git clone https://github.com/mepavankumar15/amm-capstone.git
cd amm-capstone
```

Install dependencies:

```bash
yarn install
```

---

# 🧪 Local Development

## 1️⃣ Start Local Validator

```bash
solana-test-validator --reset
```

Keep this terminal running.

---

## 2️⃣ Configure Solana

```bash
solana config set --url localhost
```

Airdrop SOL:

```bash
solana airdrop 10
```

---

## 3️⃣ Build

```bash
anchor build
```

---

## 4️⃣ Deploy

```bash
anchor deploy
```

---

## 5️⃣ Run Tests

```bash
anchor test --skip-local-validator
```

---

# 🌐 Running on Devnet

Switch to devnet:

```bash
solana config set --url https://api.devnet.solana.com
solana airdrop 2
```

Deploy:

```bash
anchor deploy --provider.cluster devnet
```

Run tests against devnet:

```bash
anchor test --provider.cluster devnet --skip-local-validator
```

---

# 🧪 Test Coverage

### ✔ Initialize Treasury
Creates protocol fee account.

### ✔ Create Pool
Initializes pool PDA and vault accounts.

### ✔ Add Liquidity
Deposits tokens and mints LP tokens.

### ✔ Swap Tokens
Verifies:
- User balances change
- Reserves update
- Invariant holds

Example swap output:

```
Before Swap:
Reserve A: 500000000
Reserve B: 500000000

After Swap:
Reserve A: 599940000
Reserve B: 416875105
```

### ✔ Slippage Failure Test
Ensures swaps fail when `min_out` is unrealistic.

---

# 🔍 Inspecting On-Chain State

### View Program

https://explorer.solana.com/address/EEzowh3RqroMtA7rsV1S88DUurNLh64PDAK2AfDBcEZe?cluster=devnet

### View Pool PDA

From test logs, copy `poolPda` and open:

```
https://explorer.solana.com/address/POOL_PDA?cluster=devnet
```

---

# 🚀 Example Workflow

```
Initialize Treasury
      ↓
Create Pool
      ↓
Add Liquidity
      ↓
Swap Tokens
      ↓
Collect Fees
      ↓
Remove Liquidity
```

---

# 📈 Why This Project Matters

This project demonstrates:

- Deep understanding of Solana PDAs
- CPI interaction with SPL Token program
- On-chain invariant enforcement
- DeFi swap mechanics
- Protocol fee modeling
- Slippage protection design
- Proper test coverage (success + failure cases)

This is not a template — it is a fully functioning on-chain AMM.

---

# 📜 License

MIT License

---

# 👨‍💻 Author

Pavan Kumar Kuchibhotla (avyu.rs) 
Solana | Rust | DeFi Engineering
follow me on twitter , DM for collab ;)
