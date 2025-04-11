# solana-program-simulator

`solana-program-simulator` is a Rust library designed to simplify testing and simulation of Solana programs using the ProgramTest environment. It provides utility functions to process and simulate transactions, manage token airdrops, fetch account data, and control blockchain time (e.g., advancing or warping the clock).

## Features

- **Transaction Processing**
  Easily process instructions with a default compute unit limit (including compute budget instructions) and custom signers/payer.

- **Transaction Simulation**
  Simulate transactions to preview their effects without committing changes to on-chain state.

- **Account Utilities**
  Functions to:
  - Airdrop SOL to accounts.
  - Retrieve account data using Anchor, Borsh, or packed formats.
  - Get account balances and system clock info.

- **Time Manipulation**
  Advance the simulated clock or warp to a specific slot or epoch.

- **Integration with ProgramTest**
  Start and manage a ProgramTest instance, making integration testing with Solana programs easier and more predictable.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
solana-program-simulator = { git = "https://github.com/pupplecat/solana-program-simulator" }

```

## Usage example

Below is a simple example that starts the simulator, airdrops SOL, and processes an instruction:

```rust
use solana_program_simulator::ProgramSimulator;
use solana_program::instruction::Instruction;
use solana_sdk::signature::{read_keypair_file, Keypair, Signature};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_program_test::ProgramTest;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize a ProgramTest environment.
    let program_test = ProgramTest::default();
    let mut simulator = ProgramSimulator::start_from_program_test(program_test).await;

    // Fund a new account.
    let funded_keypair = simulator.get_funded_keypair().await?;
    println!("Funded account: {}", funded_keypair.pubkey());

    // Build an example instruction (a system transfer in this case).
    let recipient = Pubkey::new_unique();
    let instruction = solana_sdk::system_instruction::transfer(
        &funded_keypair.pubkey(),
        &recipient,
        1_000_000, // lamports
    );

    // Process the instruction.
    let signature = simulator
        .process_ix_with_default_compute_limit(instruction, &[], None)
        .await?;
    println!("Transaction signature: {}", signature);

    // Retrieve updated balance.
    let balance = simulator.get_balance(recipient).await?;
    println!("Recipient balance: {}", balance);

    Ok(())
}
```

## Contributing

Contributions are welcome! Please open issues or pull requests for bug fixes, feature requests, or improvements.

## License

Distributed under the MIT License. See LICENSE for more information.
