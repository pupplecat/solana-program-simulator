use anyhow::Result;
use solana_program_simulator::ProgramSimulator;
use solana_program_test::ProgramTest;
use solana_sdk::{pubkey::Pubkey, signer::Signer};

#[tokio::test]
async fn test_transfer() -> Result<()> {
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
        .process_ix_with_default_compute_limit(instruction, &[&funded_keypair], None)
        .await?;
    println!("Transaction signature: {}", signature);

    // Retrieve updated balance.
    let balance = simulator.get_balance(recipient).await?;
    println!("Recipient balance: {}", balance);
    assert_eq!(balance, 1_000_000);

    Ok(())
}
