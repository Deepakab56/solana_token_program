use borsh::{ BorshDeserialize, BorshSerialize };
use mpl_token_metadata::instructions;
use mpl_token_metadata::types::DataV2;


use solana_program::{
    account_info::{ next_account_info, AccountInfo },
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::{ instruction as token_instruction, state::Mint };

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct CreateTokenArgs {
    pub token_title: String,
    pub token_symbol: String,
    pub token_uri: String,
    pub token_decimals: u8,
}

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {
    let args = CreateTokenArgs::try_from_slice(instruction_data)?;

    let account_iter = &mut accounts.iter();
    let mint_account = next_account_info(account_iter)?;
    let mint_authority = next_account_info(account_iter)?;
    let metadata_account = next_account_info(account_iter)?;
    let payer = next_account_info(account_iter)?;
    let rent = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;
    let token_metadata_program = next_account_info(account_iter)?;

    msg!("Creating mint account.................");

    msg!("Mint:{}", mint_account.key);

    invoke(
        &system_instruction::create_account(
            payer.key,
            mint_account.key,
            Rent::get()?.minimum_balance(Mint::LEN),
            Mint::LEN as u64,
            token_program.key
        ),
        &[mint_account.clone(), payer.clone(), system_program.clone(), token_program.clone()]
    )?;

    msg!("Initializing mint account.......");
    msg!("Mint:{}", mint_account.key);

    invoke(
        &token_instruction::initialize_mint(
            token_program.key,
            mint_account.key,
            mint_authority.key,
            Some(mint_authority.key),
            args.token_decimals
        )?,
        &[mint_account.clone(), mint_authority.clone(), token_program.clone(), rent.clone()]
    )?;

    msg!("Creating metadata account.................");
    msg!("Metadata account address:{}", metadata_account.key);

    let metadata_args = crate::instructions::CreateMetadataAccountV3 {
        metadata: *metadata_account.key,
        mint: *mint_account.key,
        mint_authority: *mint_authority.key,
        payer: *payer.key,
        update_authority: (*mint_authority.key, mint_authority.is_signer),
        system_program: *system_program.key,
        rent: Some(*rent.key),
    };

    let metadata_instruction_args =
    crate::instructions::CreateMetadataAccountV3InstructionArgs {
        data: DataV2 {
            name: args.token_title.clone(),
            symbol: args.token_symbol.clone(),
            uri: args.token_uri.clone(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        is_mutable: true,
        collection_details: None,
    };
    let ix = metadata_args.instruction(metadata_instruction_args);

    invoke(
        &ix,
        &[
            metadata_account.clone(),
            mint_account.clone(),
            mint_authority.clone(),
            payer.clone(),
            mint_authority.clone(),
            system_program.clone(),
            rent.clone(),
        ]
    )?;

    msg!("Token mint created successfully");

    Ok(())
}
