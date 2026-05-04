#  Solana Vote — On-chain Voting Program

An on-chain voting program built with **Rust + Anchor** on Solana. Create polls, add candidates, and cast tamper-proof votes — all enforced by the blockchain.

>  **Live on Devnet** → [View on Solana Explorer](https://explorer.solana.com/address/4qVHFhpSY7eZ54AFG8i4XUA9FA9rLTyLCj2ht4Vgp4jx?cluster=devnet)  
> Program ID: `4qVHFhpSY7eZ54AFG8i4XUA9FA9rLTyLCj2ht4Vgp4jx`

---

##  What it does

- Anyone can **create a poll** with a question and duration
- Poll creator can **add up to 8 candidates**
- Any wallet can **cast one vote** — double voting is blocked on-chain
- Poll creator can **close the poll** early
- All data lives on the Solana blockchain — no database, no server

---

##  Program Architecture

```
solana-vote/
├── programs/
│   └── solana-vote/
│       └── src/
│           └── lib.rs        ← Rust program (structs, impl, instructions)
├── tests/
│   └── solana-vote.ts        ← TypeScript integration tests
├── Anchor.toml               ← Anchor config
└── README.md
```

---

## 🦀 Rust Concepts Used

### Structs — model on-chain data
```rust
pub struct Poll {
    pub authority: Pubkey,
    pub question: String,
    pub candidates: Vec<Candidate>,
    pub start_time: i64,
    pub end_time: i64,
    pub closed: bool,
}

pub struct Candidate {
    pub name: String,
    pub votes: u64,
}
```

### impl methods — logic on structs
```rust
impl Poll {
    pub fn is_active(&self) -> Result<bool> { ... }
    pub fn cast_vote(&mut self, index: u8) -> Result<()> { ... }
    pub fn winner(&self) -> Option<&Candidate> { ... }
    pub fn total_votes(&self) -> u64 { ... }
}
```

### PDAs — prevent double voting
Each voter gets a unique on-chain record derived from their wallet + the poll. Solana makes it impossible to create two records for the same pair.
```
seeds = ["voter", poll_address, voter_address]
```

---

##  Instructions

| Instruction | Who can call | What it does |
|---|---|---|
| `initialize_poll` | Anyone | Creates a new poll with question + duration |
| `add_candidate` | Authority only | Adds a voting option (max 8) |
| `cast_vote` | Any wallet | Votes for a candidate (once per wallet) |
| `close_poll` | Authority only | Closes the poll early |

---

##  Tests

```
✔ Initializes a poll
✔ Adds candidates
✔ Casts a vote
3 passing (7s)
```

---

##  Setup & Run

### Prerequisites
- [Rust](https://rustup.rs/)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor](https://www.anchor-lang.com/docs/installation)
- Node.js + Yarn

### Install
```bash
git clone https://github.com/SamireddiNikhila/solana-vote.git
cd solana-vote
yarn install
```

### Build
```bash
anchor build
```

### Test (against devnet)
```bash
anchor test --skip-local-validator
```

### Deploy
```bash
solana airdrop 2 --url devnet
anchor program deploy --provider.cluster devnet
```

---

##  Tech Stack

| Tool | Purpose |
|---|---|
| Rust | Smart contract language |
| Anchor 1.0.2 | Solana development framework |
| Solana Devnet | Test blockchain |
| TypeScript | Integration tests |

---

##  Author

Built by [@SamireddiNikhila](https://github.com/SamireddiNikhila).
