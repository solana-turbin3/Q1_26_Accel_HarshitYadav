#[cfg(test)]
#[allow(warnings)]
mod tests {

    use {
        crate::{accounts::Make, instructions::make},
        anchor_lang::{
            prelude::msg, solana_program::program_pack::Pack, AccountDeserialize, InstructionData,
            ToAccountMetas,
        },
        anchor_spl::{
            associated_token::{self, spl_associated_token_account},
            token::spl_token,
        },
        litesvm::LiteSVM,
        litesvm_token::{
            spl_token::ID as TOKEN_PROGRAM_ID, CreateAssociatedTokenAccount, CreateMint, MintTo,
        },
        solana_account::Account,
        solana_address::Address,
        solana_instruction::Instruction,
        solana_keypair::Keypair,
        solana_message::Message,
        solana_native_token::LAMPORTS_PER_SOL,
        solana_pubkey::Pubkey,
        solana_rpc_client::rpc_client::RpcClient,
        solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID,
        solana_signer::Signer,
        solana_transaction::Transaction,
        std::{path::PathBuf, str::FromStr},
    };

    static PROGRAM_ID: Pubkey = crate::ID;

    // Setup function to initialize LiteSVM and create a payer keypair
    // Also loads an account from devnet into the LiteSVM environment (for testing purposes)

    fn address_to_pubkey(add: &Address) -> Pubkey {
        Pubkey::from_str(&add.to_string()).unwrap()
    }

    fn _pubkey_to_add(pkey: &Pubkey) -> Address {
        Address::from_str(&pkey.to_string()).unwrap()
    }

    fn setup() ->( LiteSVM , Keypair ){
        // Initialize LiteSVM and payer
        let mut program = LiteSVM::new();
        let maker = Keypair::new();
        let _taker = Keypair::new();

        // Load program SO file
        let so_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/anchor_escrow.so");

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        program.add_program(PROGRAM_ID, &program_data);

        // Example on how to Load an account from devnet
        // LiteSVM does not have access to real Solana network data since it does not have network access,
        // so we use an RPC client to fetch account data from devnet
        let rpc_client = RpcClient::new("https://api.devnet.solana.com");
        let account_address =
            Address::from_str("DRYvf71cbF2s5wgaJQvAGkghMkRcp5arvsK2w97vXhi2").unwrap();
        let fetched_account = rpc_client
            .get_account(&account_address)
            .expect("Failed to fetch account from devnet");

        // Set the fetched account in the LiteSVM environment
        // This allows us to simulate interactions with this account during testing
        program
            .set_account(
                maker.pubkey(),
                Account {
                    lamports: fetched_account.lamports,
                    data: fetched_account.data,
                    owner: address_to_pubkey(&fetched_account.owner),
                    executable: fetched_account.executable,
                    rent_epoch: fetched_account.rent_epoch,
                },
            )
            .unwrap();

        msg!("Lamports of fetched account: {}", fetched_account.lamports);

        // Return the LiteSVM instance and payer keypair
        (program , maker)
    }

    #[test]
    #[ignore]
    fn test_make() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let( mut program , maker) = setup();
        let seed = 123u64;
        let taker = Keypair::new();
        program
            .airdrop(
                &maker.pubkey(),
                10u64.checked_mul(LAMPORTS_PER_SOL).expect("overflow"),
            )
            .expect("Failed to airdrop SOL to payer");

        program
            .airdrop(
                &taker.pubkey(),
                10u64.checked_mul(LAMPORTS_PER_SOL).expect("overflow"),
            )
            .expect("Failed to airdrop SOL to taker");

        let mint_a = CreateMint::new(&mut program, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();
        let mint_b = CreateMint::new(&mut program, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();
        let escrow_pda = Pubkey::find_program_address(
            &[
                b"escrow",
                maker.pubkey().as_ref(),
                seed.to_le_bytes().as_ref(),
            ],
            &PROGRAM_ID,
        )
        .0;
        msg!("Escrow PDA: {}\n", escrow_pda);
        msg!("Mint A: {}\n", mint_a);

        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &maker, &mint_a)
            .owner(&maker.pubkey())
            .send()
            .unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        // Derive the PDA for the vault associated token account using the escrow PDA and Mint A
        // vault => will store the mint_a , and the owner has to be escrow_pda
        let vault = associated_token::get_associated_token_address(&escrow_pda, &mint_a);
        msg!("Vault PDA: {}\n", vault);

        // Define program IDs for associated token program, token program, and system program
        let associated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut program, &maker, &mint_a, &maker_ata_a, 10 * 1000000)
            .send()
            .unwrap();

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Make {
                maker: maker.pubkey(),
                mint_a: mint_a,
                mint_b: mint_b,
                maker_ata_a: maker_ata_a,
                escrow: escrow_pda,
                vault: vault,
                associated_token_program: associated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Make {
                seed,
                deposit: 10 * 1000000,
                receive: 10 * 1000000,
            }
            .data(),
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&maker.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        // let transaction = Transaction::new(&[&payer], message, recent_blockhash);
        let transaction = Transaction::new(&[&maker], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        // Verify the vault account and escrow account data after the "Make" instruction
        let vault_account = program.get_account(&vault).unwrap();
        let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
        assert_eq!(vault_data.amount, 10 * 1000000);
        assert_eq!(vault_data.owner, escrow_pda);
        assert_eq!(vault_data.mint, (mint_a));

        let escrow_account = program.get_account(&escrow_pda).unwrap();
        let escrow_data =
            crate::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
        assert_eq!(escrow_data.seed, seed);
        assert_eq!(escrow_data.maker, maker.pubkey());
        assert_eq!(escrow_data.mint_a, mint_a);
        assert_eq!(escrow_data.mint_b, mint_b);
        assert_eq!(escrow_data.receive, 10 * 1000000);
    }

    #[test]
    fn test_take() {
        let( mut program , maker) = setup();
        let seed = 123u64;
        let taker = Keypair::new();
        program
            .airdrop(
                &maker.pubkey(),
                10u64.checked_mul(LAMPORTS_PER_SOL).expect("overflow"),
            )
            .expect("Failed to airdrop SOL to payer");

        program
            .airdrop(
                &taker.pubkey(),
                10u64.checked_mul(LAMPORTS_PER_SOL).expect("overflow"),
            )
            .expect("Failed to airdrop SOL to taker");

        let mint_a = CreateMint::new(&mut program, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();
        let mint_b = CreateMint::new(&mut program, &taker)
            .decimals(6)
            .authority(&taker.pubkey())
            .send()
            .unwrap();

        let escrow_pda = Pubkey::find_program_address(
            &[
                b"escrow",
                maker.pubkey().as_ref(),
                seed.to_le_bytes().as_ref(),
            ],
            &PROGRAM_ID,
        )
        .0;
        msg!("Escrow PDA: {}\n", escrow_pda);
        msg!("Mint A: {}\n", mint_a);

        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &maker, &mint_a)
            .owner(&maker.pubkey())
            .send()
            .unwrap();
        let maker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &maker, &mint_b)
            .owner(&maker.pubkey())
            .send()
            .unwrap();
        let taker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &taker, &mint_a)
            .owner(&taker.pubkey())
            .send()
            .unwrap();
        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &taker, &mint_b)
            .owner(&taker.pubkey())
            .send()
            .unwrap();
        let vault = associated_token::get_associated_token_address(&escrow_pda, &mint_a);
        let associated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;
        // msg!("{}lllll", escrow_data.maker);

        MintTo::new(&mut program, &maker, &mint_a, &maker_ata_a, 10 * 1000000)
            .send()
            .unwrap();
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: Make {
                maker: maker.pubkey(),
                mint_a,
                mint_b,
                escrow: escrow_pda,
                maker_ata_a,
                vault,
                associated_token_program,
                token_program,
                system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Make {
                seed: seed,
                deposit: 10 * 1000000,
                receive: 10 * 1000000,
            }
            .data(),
        };

        let message = Message::new(&[make_ix], Some(&maker.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let make_transaction = Transaction::new(&[&maker], message, recent_blockhash);
        let _make_tx = program.send_transaction(make_transaction).unwrap();
        msg!("\n\nMake transaction sucessfull");

        let vault_account = program.get_account(&vault).unwrap();
        let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
        // assert_eq!(vault_data.amount, 10 * 1000000);
        // assert_eq!(vault_data.mint, mint_a);

        let mint_b_metadata = program.get_account(&mint_b).unwrap().data ;
        let data_mint_b = spl_token::state::Mint::unpack(&mint_b_metadata).unwrap() ;
        // println!("mint_b authority  : {:?}" , data_mint_b.mint_authority ) ;
        // println!("taker : {}" , taker.pubkey().to_string()) ;
        // println!("maker : {}" , maker.pubkey().to_string()) ;

        match MintTo::new(&mut program, &taker, &mint_b, &taker_ata_b, 10 * 1000000)
            .send(){
                Ok(_) => {} ,
                Err(e) => {
                    println!("chud gaya {:?}" , e) ;
                }
            }
            

        let take_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Take {
                taker: taker.pubkey(),
                maker: maker.pubkey(),
                mint_a: mint_a,
                mint_b: mint_b,
                taker_ata_a: taker_ata_a,
                taker_ata_b: taker_ata_b,
                maker_ata_b: maker_ata_b,
                escrow: escrow_pda,
                vault: vault,
                associated_token_program: associated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(Some(true)),
            data: crate::instruction::Take {}.data(),
        };

        let message2 = Message::new(&[take_ix], Some(&taker.pubkey()));
        let recent_blockhash = program.latest_blockhash();
        let take_transaction = Transaction::new(&[&taker], message2, recent_blockhash);

        let take_tx = program.send_transaction(take_transaction).unwrap();

        let taker_ata_a_metadata = program.get_account(&taker_ata_a).unwrap() ;
        let taker_ata_a_data = spl_token::state::Account::unpack(&taker_ata_a_metadata.data).unwrap() ;
        let taker_ata_b_metadata = program.get_account(&taker_ata_b).unwrap() ;
        let taker_ata_b_data = spl_token::state::Account::unpack(&taker_ata_b_metadata.data).unwrap() ;

        let maker_ata_a_metadata = program.get_account(&maker_ata_a).unwrap() ;
        let maker_ata_a_data = spl_token::state::Account::unpack(&maker_ata_a_metadata.data).unwrap() ;
        let maker_ata_b_metadata = program.get_account(&maker_ata_b).unwrap() ;
        let maker_ata_b_data = spl_token::state::Account::unpack(&maker_ata_b_metadata.data).unwrap() ;

        assert_eq!(maker_ata_a_data.amount, 0);
        assert_eq!(maker_ata_b_data.amount, 10*1000000);
        assert_eq!(taker_ata_a_data.amount, 10*1000000);
        assert_eq!(taker_ata_b_data.amount, 0);

        msg!("CUs Consumed: {}", take_tx.compute_units_consumed);
        msg!("Tx Signature: {}", take_tx.signature);
        // let escrow_data = Escrow::try_deserialize(&mut escrow.data.as_ref()).unwrap();
    }

    #[test]
    #[ignore]
    fn test_refund() {
        let (mut program , maker) = setup();
        let seed = 123u64;
        let taker = Keypair::new();
        program
            .airdrop(
                &maker.pubkey(),
                10u64.checked_mul(LAMPORTS_PER_SOL).expect("overflow"),
            )
            .expect("Failed to airdrop SOL to payer");

        program
            .airdrop(
                &taker.pubkey(),
                10u64.checked_mul(LAMPORTS_PER_SOL).expect("overflow"),
            )
            .expect("Failed to airdrop SOL to taker");

        let mint_a = CreateMint::new(&mut program, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();
        let mint_b = CreateMint::new(&mut program, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();
        let escrow_pda = Pubkey::find_program_address(
            &[
                b"escrow",
                maker.pubkey().as_ref(),
                seed.to_le_bytes().as_ref(),
            ],
            &PROGRAM_ID,
        )
        .0;
        msg!("Escrow PDA: {}\n", escrow_pda);
        msg!("Mint A: {}\n", mint_a);

        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &maker, &mint_a)
            .owner(&maker.pubkey())
            .send()
            .unwrap();
        let maker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &maker, &mint_b)
            .owner(&maker.pubkey())
            .send()
            .unwrap();
        let taker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &taker, &mint_a)
            .owner(&taker.pubkey())
            .send()
            .unwrap();
        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &taker, &mint_b)
            .owner(&taker.pubkey())
            .send()
            .unwrap();
        let vault = associated_token::get_associated_token_address(&escrow_pda, &mint_a);
        let associated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;
        // msg!("{}lllll", escrow_data.maker);

        MintTo::new(&mut program, &maker, &mint_a, &maker_ata_a, 10 * 1000000)
            .send()
            .unwrap();
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: Make {
                maker: maker.pubkey(),
                mint_a,
                mint_b,
                escrow: escrow_pda,
                maker_ata_a,
                vault,
                associated_token_program,
                token_program,
                system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Make {
                seed: seed,
                deposit: 10 * 1000000,
                receive: 10 * 1000000,
            }
            .data(),
        };

        let message = Message::new(&[make_ix], Some(&maker.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let make_transaction = Transaction::new(&[&maker], message, recent_blockhash);
        let _make_tx = program.send_transaction(make_transaction).unwrap();
        msg!("\n\nMake transaction sucessfull");

        let vault_account = program.get_account(&vault).unwrap();
        let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
        assert_eq!(vault_data.amount, 10 * 1000000);
        assert_eq!(vault_data.mint, mint_a);

        MintTo::new(&mut program, &maker, &mint_b, &taker_ata_b, 10 * 1000000)
            .send()
            .unwrap();

        let refund_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Refund {
                maker: maker.pubkey(),
                mint_a,
                maker_ata_a,
                escrow: escrow_pda,
                vault,
                token_program,
                system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Refund {}.data(),
        };

        let message2 = Message::new(&[refund_ix], Some(&maker.pubkey()));
        let recent_blockhash = program.latest_blockhash();
        let refund_transaction = Transaction::new(&[&maker], message2, recent_blockhash);

        let _refund_tx = program.send_transaction(refund_transaction).unwrap();
        msg!("\n\nRefund transaction sucessfull");

        let vault_account = program.get_account(&vault).unwrap();

        let maker_ata_a_account = program.get_account(&maker_ata_a).unwrap();
        let maker_ata_a_data =
            spl_token::state::Account::unpack(&maker_ata_a_account.data).unwrap();

        let escrow_account = program.get_account(&escrow_pda).unwrap();

        assert_eq!(
            maker_ata_a_data.amount, 10*1000000,
            "maker must recieve deposit amount back"
        );

        assert!(
            escrow_account.data.is_empty() && escrow_account.lamports.eq(&0),
            "escrow account must be closed"
        );
    }
}
