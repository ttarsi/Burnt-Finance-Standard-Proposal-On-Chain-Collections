use crate::processor::{CollectionData, CollectionSignature, BASE_COLLECTION_DATA_SIZE};
use crate::{errors::CollectionError, utils::create_or_allocate_account_raw, PREFIX};
use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::next_account_info, account_info::AccountInfo, entrypoint::ProgramResult, msg,
        program_error::ProgramError, pubkey::Pubkey,
    },
    std::mem,
};

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CreateCollectionArgs {
    // The name of the Collection
    pub name: [u8; 32],
    // A short description of the Collection
    pub description: [u8; 32],
    // A boolean as to whether assets can be removed from the `members` list
    pub removable: bool,
    // A u32 that declares what the maximum number of member assets on the chain can be.
    // If set to 0, the collection must have all members defined at start.
    pub expandable: u32,
    // A boolean as to whether asset order can be changed
    pub arrangeable: bool,
    // A list of public keys that this collection considers to be members
    pub members: Vec<Pubkey>,
    // A list of signature that this collection is a member of
    pub member_of: Vec<CollectionSignature>,
}

struct Accounts<'a, 'b: 'a> {
    collection: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
    rent: &'a AccountInfo<'b>,
    system: &'a AccountInfo<'b>,
}

fn parse_accounts<'a, 'b: 'a>(
    _: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let account_iter = &mut accounts.iter();
    let accounts = Accounts {
        collection: next_account_info(account_iter)?,
        payer: next_account_info(account_iter)?,
        rent: next_account_info(account_iter)?,
        system: next_account_info(account_iter)?,
    };


    Ok(accounts)
}

pub fn create_collection(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: CreateCollectionArgs,
) -> ProgramResult {
    msg!("+ Processing CreateCollection");
    let accounts = parse_accounts(program_id, accounts)?;

    let collection_path = [PREFIX.as_bytes(), program_id.as_ref()];

    let (collection_key, bump) = Pubkey::find_program_address(&collection_path, program_id);
    if collection_key != *accounts.collection.key {
        return Err(CollectionError::InvalidCollectionAccount.into());
    }

    let mut account_size = BASE_COLLECTION_DATA_SIZE;
    if args.expandable > 0 {
        if args.members.len() > args.expandable as usize {
            return Err(CollectionError::CapacityExceeded.into());
        }

        account_size += args.expandable as usize * mem::size_of::<Pubkey>();
    } else {
        if args.members.len() == 0 {
            return Err(CollectionError::PermanentlyEmptyCollection.into());
        }
        account_size += args.members.len() * mem::size_of::<Pubkey>();
    }

    create_or_allocate_account_raw(
        *program_id,
        accounts.collection,
        accounts.rent,
        accounts.system,
        accounts.payer,
        account_size,
        &[PREFIX.as_bytes(), program_id.as_ref(), &[bump]],
    )?;

    CollectionData {
        name: args.name,
        description: args.description,
        removable: args.removable,
        expandable: args.expandable,
        arrangeable: args.arrangeable,
        members: args.members.clone(),
        member_of: args.member_of.clone(),
    }
    .serialize(&mut *accounts.collection.data.borrow_mut())?;

    Ok(())
}
