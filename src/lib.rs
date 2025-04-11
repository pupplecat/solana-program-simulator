use anchor_lang::{AnchorDeserialize, prelude::ProgramError};
use borsh::BorshDeserialize;
use solana_banks_interface::{BanksTransactionResultWithSimulation, TransactionStatus};
use solana_program::program_pack::Pack;
use solana_program_test::{
    BanksClientError, ProgramTest, ProgramTestBanksClientExt, ProgramTestContext, ProgramTestError,
};
use solana_sdk::{
    account::Account,
    clock::Clock,
    compute_budget,
    genesis_config::GenesisConfig,
    instruction::{Instruction, InstructionError},
    native_token::LAMPORTS_PER_SOL,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::{Transaction, TransactionError},
};

// Assume that ProgramSimulator is defined as before:
pub struct ProgramSimulator {
    pub program_test_context: ProgramTestContext,
    // other fields...
}

impl ProgramSimulator {
    /// Start a new ProgramSimulator from a `ProgramTest` instance.
    pub async fn start_from_program_test(program_test: ProgramTest) -> ProgramSimulator {
        let program_test_context = program_test.start_with_context().await;

        ProgramSimulator {
            program_test_context,
        }
    }

    /// Common helper to build and sign a transaction with default compute limit.
    async fn build_and_sign_tx(
        &mut self,
        instructions: &[Instruction],
        signers: &[&Keypair],
        payer: Option<&Keypair>,
    ) -> Result<Transaction, BanksClientError> {
        // Create the compute budget instruction.
        let compute_units_ix =
            compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(2_000_000);

        // Combine instructions.
        let mut all_instructions = Vec::with_capacity(instructions.len() + 1);
        all_instructions.push(compute_units_ix);
        all_instructions.extend_from_slice(instructions);

        // Determine the actual payer.
        let actual_payer = payer.unwrap_or(&self.program_test_context.payer);

        // Create the transaction with the payer.
        let mut transaction =
            Transaction::new_with_payer(&all_instructions, Some(&actual_payer.pubkey()));

        // Get a new blockhash, propagating errors instead of panicking.
        let blockhash = self
            .program_test_context
            .banks_client
            .get_new_latest_blockhash(&self.program_test_context.last_blockhash)
            .await?;
        self.program_test_context.last_blockhash = blockhash;

        // Partially sign with the payer, then additional signers.
        transaction.partial_sign(&[actual_payer], self.program_test_context.last_blockhash);
        transaction.partial_sign(signers, self.program_test_context.last_blockhash);

        Ok(transaction)
    }

    /// Process one or more instructions (transaction is sent on-chain).
    pub async fn process_ixs_with_default_compute_limit(
        &mut self,
        instructions: &[Instruction],
        signers: &[&Keypair],
        payer: Option<&Keypair>,
    ) -> Result<Signature, BanksClientError> {
        let transaction = self.build_and_sign_tx(instructions, signers, payer).await?;
        let signature = transaction.signatures[0];
        self.program_test_context
            .banks_client
            .process_transaction(transaction)
            .await?;
        Ok(signature)
    }

    /// Process a single instruction by wrapping it and calling the multi-instruction version.
    pub async fn process_ix_with_default_compute_limit(
        &mut self,
        instruction: Instruction,
        signers: &[&Keypair],
        payer: Option<&Keypair>,
    ) -> Result<Signature, BanksClientError> {
        self.process_ixs_with_default_compute_limit(&[instruction], signers, payer)
            .await
    }

    /// Simulate one or more instructions (without committing the transaction).
    pub async fn simulate_ixs_with_default_compute_limit(
        &mut self,
        instructions: &[Instruction],
        signers: &[&Keypair],
        payer: Option<&Keypair>,
    ) -> Result<BanksTransactionResultWithSimulation, BanksClientError> {
        let transaction = self.build_and_sign_tx(instructions, signers, payer).await?;
        self.program_test_context
            .banks_client
            .simulate_transaction(transaction)
            .await
    }

    /// Simulate a single instruction.
    pub async fn simulate_ix_with_default_compute_limit(
        &mut self,
        instruction: Instruction,
        signers: &[&Keypair],
        payer: Option<&Keypair>,
    ) -> Result<BanksTransactionResultWithSimulation, BanksClientError> {
        self.simulate_ixs_with_default_compute_limit(&[instruction], signers, payer)
            .await
    }

    pub async fn airdrop(
        &mut self,
        to: &Pubkey,
        lamports: u64,
    ) -> Result<Signature, BanksClientError> {
        let instruction =
            system_instruction::transfer(&self.program_test_context.payer.pubkey(), to, lamports);

        self.process_ix_with_default_compute_limit(instruction, &[], None)
            .await
    }

    pub async fn get_funded_keypair(&mut self) -> Result<Keypair, BanksClientError> {
        let keypair = Keypair::new();
        self.airdrop(&keypair.pubkey(), LAMPORTS_PER_SOL).await?;
        Ok(keypair)
    }

    pub async fn get_account(&mut self, pubkey: Pubkey) -> Result<Account, BanksClientError> {
        let account = self
            .program_test_context
            .banks_client
            .get_account(pubkey)
            .await?
            .ok_or(BanksClientError::ClientError("Account not found"))?;

        Ok(account)
    }

    pub async fn get_anchor_account_data<T: AnchorDeserialize>(
        &mut self,
        pubkey: Pubkey,
    ) -> Result<T, BanksClientError> {
        let account = self.get_account(pubkey).await?;

        Ok(T::deserialize(&mut &account.data[8..])?)
    }

    pub async fn get_borsh_account_data<T: BorshDeserialize>(
        &mut self,
        pubkey: Pubkey,
    ) -> Result<T, BanksClientError> {
        let account = self.get_account(pubkey).await?;

        Ok(T::deserialize(&mut &account.data[..])?)
    }

    pub async fn get_packed_account_data<T: Pack + IsInitialized>(
        &mut self,
        pubkey: Pubkey,
    ) -> Result<T, BanksClientError> {
        let account = self.get_account(pubkey).await?;

        T::unpack(&account.data[..]).map_err(|_err| BanksClientError::ClientError("Unpack error"))
    }

    pub async fn get_balance(&mut self, pubkey: Pubkey) -> Result<u64, BanksClientError> {
        let lamports = self
            .program_test_context
            .banks_client
            .get_balance(pubkey)
            .await?;
        Ok(lamports)
    }

    pub async fn get_clock(&mut self) -> Result<Clock, BanksClientError> {
        self.program_test_context
            .banks_client
            .get_sysvar::<Clock>()
            .await
    }

    pub fn get_genesis_config(&mut self) -> Result<GenesisConfig, BanksClientError> {
        let config = self.program_test_context.genesis_config();

        Ok(config.clone())
    }

    pub async fn advance_clock_by(
        &mut self,
        seconds_to_advance: i64,
    ) -> Result<(), BanksClientError> {
        let mut clock = self
            .program_test_context
            .banks_client
            .get_sysvar::<Clock>()
            .await?;

        clock.epoch_start_timestamp += seconds_to_advance;
        clock.unix_timestamp += seconds_to_advance;
        self.program_test_context.set_sysvar(&clock);

        Ok(())
    }

    pub async fn advance_clock_to(
        &mut self,
        seconds_to_advance: i64,
    ) -> Result<(), BanksClientError> {
        let mut clock = self
            .program_test_context
            .banks_client
            .get_sysvar::<Clock>()
            .await?;

        clock.epoch_start_timestamp = seconds_to_advance;
        clock.unix_timestamp = seconds_to_advance;
        self.program_test_context.set_sysvar(&clock);

        Ok(())
    }

    pub async fn get_transaction_status(
        &mut self,
        signature: Signature,
    ) -> Result<Option<TransactionStatus>, BanksClientError> {
        self.program_test_context
            .banks_client
            .get_transaction_status(signature)
            .await
    }

    pub fn warp_to_epoch(&mut self, warp_epoch: u64) -> Result<(), ProgramTestError> {
        self.program_test_context.warp_to_epoch(warp_epoch)?;

        Ok(())
    }

    pub fn warp_to_slot(&mut self, warp_slot: u64) -> Result<(), ProgramTestError> {
        self.program_test_context.warp_to_slot(warp_slot)?;

        Ok(())
    }
}

pub fn into_transaction_error<T: Into<anchor_lang::prelude::Error>>(error: T) -> TransactionError {
    into_transaction_error_with_index(0, error)
}

pub fn into_transaction_error_with_index<T: Into<anchor_lang::prelude::Error>>(
    instruction_index: u8,
    error: T,
) -> TransactionError {
    TransactionError::InstructionError(
        instruction_index,
        InstructionError::from(u64::from(ProgramError::from(error.into()))),
    )
}
