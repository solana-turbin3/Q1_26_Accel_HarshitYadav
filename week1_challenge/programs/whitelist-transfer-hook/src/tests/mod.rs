#[cfg(test)]
#[allow(warnings)]
mod tests {

    use std::path::PathBuf;

    use anchor_lang::{
        prelude::{msg, Pubkey},
        solana_program::{
            example_mocks::solana_sdk::transaction, lamports, program_error, program_pack::Pack,
        },
        system_program, AccountDeserialize, InstructionData, Key, ToAccountMetas,
    };
    use anchor_spl::{
        associated_token::spl_associated_token_account,
        token_2022::spl_token_2022::{
            self,
            extension::{
                metadata_pointer::MetadataPointer, transfer_hook::TransferHook,
                BaseStateWithExtensions, StateWithExtensionsOwned,
            },
            state::{Account, Mint},
        },
        token_interface::{
            spl_pod::option::Nullable, spl_token_metadata_interface::state::TokenMetadata,
            TokenAccount,
        },
    };
    use litesvm::{types::TransactionMetadata, LiteSVM};
    use litesvm_token::{
        get_spl_account,
        spl_token::{self, ID as TOKEN_PROGRAM_ID},
        CreateAssociatedTokenAccount, CreateMint,
    };
    use solana_address::Address;
    use solana_instruction::Instruction;
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_sdk_ids::sysvar::recent_blockhashes;
    use solana_signer::{EncodableKeypair, Signer};
    use solana_transaction::Transaction;
    // use spl_tlv_account_resolution::state::ExtraAccountMetaList;

    use crate::{
        accounts::InitializeWhitelist,
        state::{vault, whitelist},
    };
    // use solana_address::Address;
    // use solana_keypair::Keypair;
    // use solana_message::Instruction;
    // use solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID ;
    // use solana_native_token::LAMPORTS_PER_SOL;
    // use solana_signer::Signer;
    static PROGRAM_ID: Pubkey = crate::ID;

    fn add_to_pub(add: Address) -> Pubkey {
        Pubkey::from_str_const(&add.to_string())
    }
    fn pub_to_add(pk: Pubkey) -> Address {
        Address::from_str_const(&pk.to_string())
    }

    // Setup function to initialize LiteSVM and create a payer keypair
    // Also loads an account from devnet into the LiteSVM environment (for testing purposes)
    fn setup() -> (LiteSVM, Keypair) {
        // Initialize LiteSVM and payer
        let mut program = LiteSVM::new();
        let payer = Keypair::new();

        // Airdrop some SOL to the payer keypair
        program
            .airdrop(
                &payer.pubkey(),
                10u64.checked_mul(LAMPORTS_PER_SOL).expect("overflow"),
            )
            .expect("Failed to airdrop SOL to payer");

        let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/deploy/whitelist_transfer_hook.so");

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        program.add_program(PROGRAM_ID, &program_data);

        // Example on how to Load an account from devnet
        // LiteSVM does not have access to real Solana network data since it does not have network access,
        // so we use an RPC client to fetch account data from devnet
        // let rpc_client = RpcClient::new("https://api.devnet.solana.com");
        // let account_address =
        //     Address::from_str("DRYvf71cbF2s5wgaJQvAGkghMkRcp5arvsK2w97vXhi2").unwrap();
        // let fetched_account = rpc_client
        //     .get_account(&account_address)
        //     .expect("Failed to fetch account from devnet");

        // // Set the fetched account in the LiteSVM environment
        // // This allows us to simulate interactions with this account during testing
        // program
        //     .set_account(
        //         payer.pubkey(),
        //         Account {
        //             lamports: fetched_account.lamports,
        //             data: fetched_account.data,
        //             owner: Pubkey::from(fetched_account.owner.to_bytes()),
        //             executable: fetched_account.executable,
        //             rent_epoch: fetched_account.rent_epoch,
        //         },
        //     )
        //     .unwrap();

        // msg!("Lamports of fetched account: {}", fetched_account.lamports);

        // Return the LiteSVM instance and payer keypair
        (program, payer)
    }

    fn init_vault_helper(program: &mut LiteSVM, admin: &Keypair) -> Pubkey {
        let vault_pda =
            Pubkey::find_program_address(&[b"vault", admin.pubkey().as_ref()], &PROGRAM_ID).0;
        let system_program = system_program::ID;

        let mint = CreateMint::new(program, &admin)
            .decimals(6)
            .authority(&admin.pubkey())
            .send()
            .unwrap();
        let vault_token_account = CreateAssociatedTokenAccount::new(program, &admin, &mint)
            .owner(&vault_pda)
            .send()
            .unwrap();
        let associated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = system_program::ID;

        let init_vault_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::InitializeVault {
                admin: admin.pubkey(),
                mint,
                vault: vault_pda,
                vault_token_account,
                associated_token_program,
                token_program,
                system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::InitializeVault {}.data(),
        };
        let message = Message::new(&[init_vault_ix], Some(&admin.pubkey()));
        let recent_blockhash = program.latest_blockhash();
        let transaction1 = Transaction::new(&[&admin], message, recent_blockhash);
        let init_vault_tx = program.send_transaction(transaction1).unwrap();
        vault_pda
    }

    fn init_whitelist_helper(program: &mut LiteSVM, admin: &Keypair, user: &Pubkey) -> Pubkey {
        let whitelist_pda: Pubkey = Pubkey::find_program_address(
            &[b"whitelist", admin.pubkey().as_ref(), user.as_ref()],
            &PROGRAM_ID,
        )
        .0;
        let system_program = system_program::ID;

        let vault_pda = init_vault_helper(program, &admin);

        let init_whitelist = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::InitializeWhitelist {
                admin: admin.pubkey(),
                vault: vault_pda,
                whitelist: whitelist_pda,
                system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::InitializeWhitelist { user: *user }.data(),
        };
        let message = Message::new(&[init_whitelist], Some(&admin.pubkey()));
        let recent_blockhash = program.latest_blockhash();
        let transaction1 = Transaction::new(&[&admin], message, recent_blockhash);
        let init_whitelist_tx = program.send_transaction(transaction1).unwrap();
        whitelist_pda
    }

    #[test]
    fn test_init_vault() {
        let (mut program, admin) = setup();
        let vault_pda = init_vault_helper(&mut program, &admin);
        msg!("\n\nInit vault");
        let vault_account = program.get_account(&vault_pda).unwrap();
        let vault_account_metadata =
            crate::state::Vault::try_deserialize(&mut vault_account.data.as_ref()).unwrap();
        let mint = vault_account_metadata.mint_add;
        assert_eq!(vault_account_metadata.admin, admin.pubkey());
        assert_eq!(vault_account_metadata.mint_add, mint);
        let vault_token_account = vault_account_metadata.vault_token_account;
        let vault_token_account_data = program.get_account(&vault_token_account).unwrap();
        let vault_token_account_metadata =
            spl_token::state::Account::unpack(&vault_token_account_data.data).unwrap();
        assert_eq!(vault_token_account_metadata.mint, mint);
    }

    #[test]
    fn test_init_whitelist() {
        let (mut program, admin) = setup();
        let whitelist_pda = init_whitelist_helper(&mut program, &admin, &admin.pubkey());
        msg!("Init and added to whitelist ");
        let whitelist_account = program.get_account(&whitelist_pda).unwrap();
        let whitelist_account_metadata =
            crate::state::Whitelist::try_deserialize(&mut whitelist_account.data.as_ref()).unwrap();
        assert!(!whitelist_account.data.is_empty());
        assert_eq!(whitelist_account.owner, PROGRAM_ID);
    }

    #[test]
    fn test_operations_on_whitelist() {
        let (mut program, admin) = setup();
        let user = Keypair::new();
        program
            .airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL)
            .unwrap();
        let mut whitelist_pda = Pubkey::find_program_address(
            &[
                b"whitelist",
                admin.pubkey().as_ref(),
                user.pubkey().as_ref(),
            ],
            &PROGRAM_ID,
        )
        .0;

        if program.get_account(&whitelist_pda).is_none() {
            whitelist_pda = init_whitelist_helper(&mut program, &admin, &user.pubkey());
        }
        // no need for checking the vault_pda init coz it is already initialized by the init_whitelist_pda
        let vault_pda =
            Pubkey::find_program_address(&[b"vault", admin.pubkey().as_ref()], &PROGRAM_ID).0;
        // let vault_pda = init_vault_helper(&mut program, &admin) ;

        // if program.get_account(&vault_pda).is_none() {
        //     let vault_pda = init_vault_helper(&mut program, &admin) ;
        // }

        let system_program = system_program::ID;
        let add_to_whitelist = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::WhitelistOperations {
                admin: admin.pubkey(),
                vault: vault_pda,
                whitelist: whitelist_pda,
                system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::AddToWhitelist {
                user: user.pubkey(),
            }
            .data(),
        };
        let message = Message::new(&[add_to_whitelist], Some(&admin.pubkey()));
        let recent_blockhash = program.latest_blockhash();
        let transaction1 = Transaction::new(&[&admin], message, recent_blockhash);
        let add_to_whitelist_tx = program.send_transaction(transaction1).unwrap();
        msg!("Added to whitelist ");
        let whitelist_account = program.get_account(&whitelist_pda).unwrap();
        let whitelist_account_metadata =
            crate::state::Whitelist::try_deserialize(&mut whitelist_account.data.as_ref()).unwrap();
        assert!(!whitelist_account.data.is_empty());
        assert_eq!(whitelist_account.owner, PROGRAM_ID);
        assert_eq!(whitelist_account_metadata.user_key, user.pubkey());
        assert_eq!(whitelist_account_metadata.is_whitelisted, true);

        // now removing the same user , (after adding , now removing )

        // test for removing a user from whitelist which is not whitelisted
        // let anonymous_user = Keypair::new() ;
        // let mut whitelist_pda2 =
        //     Pubkey::find_program_address(&[b"whitelist", admin.pubkey().as_ref() ,anonymous_user.pubkey().as_ref()], &PROGRAM_ID).0;
        let system_program = system_program::ID;

        if program.get_account(&whitelist_pda).is_none() {
            // msg!("The user is not initialized , i.e not whitelisted") ;
            panic!("The user is not initialized , i.e not whitelisted");
        }
        let remove_from_whitelist = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::WhitelistOperations {
                admin: admin.pubkey(),
                vault: vault_pda,
                whitelist: whitelist_pda,
                system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::RemoveFromWhitelist {
                user: user.pubkey(),
            }
            .data(),
        };
        let message2 = Message::new(&[remove_from_whitelist], Some(&admin.pubkey()));
        let recent_blockhash = program.latest_blockhash();
        let transaction2 = Transaction::new(&[&admin], message2, recent_blockhash);
        let remove_from_whitelist_tx = program.send_transaction(transaction2).unwrap();
        msg!("Removed from whitelist ");
        let whitelist_account = program.get_account(&whitelist_pda).unwrap();
        let whitelist_account_metadata =
            crate::state::Whitelist::try_deserialize(&mut whitelist_account.data.as_ref()).unwrap();
        assert!(!whitelist_account.data.is_empty());
        assert_eq!(whitelist_account.owner, PROGRAM_ID);
        assert_eq!(whitelist_account_metadata.user_key, user.pubkey());
        assert_eq!(whitelist_account_metadata.is_whitelisted, false); // whitelist is derived using user.pubkey and admin.pubkey
    }
}
