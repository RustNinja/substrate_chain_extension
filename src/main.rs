use frame_support::{
	log::{error, trace},
	pallet_prelude::*,
	traits::fungibles::{
		approvals::{Inspect as AllowanceInspect, Mutate as AllowanceMutate},
		Inspect, InspectMetadata, Transfer, Mutate, Create, MutateHold,
		metadata::{Mutate as MetadataMutate, Inspect as OtherInspectMetadata}
	},
};
use sp_std::vec::Vec;

use chain_extension::{
    ChainExtension,
    Environment,
    Ext,
    InitState,
    RetVal,
    SysConfig,
    UncheckedFrom,
};

use sp_runtime::DispatchError;
use sp_runtime::TokenError;
use sp_runtime::ArithmeticError;

use sp_runtime::MultiAddress;
pub struct Psp22AssetExtension;


#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode,  MaxEncodedLen)]
enum OriginType{
	Caller, 
	Address
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode, MaxEncodedLen)]
struct PalletAssetRequest{
	origin_type: OriginType,
	asset_id : u32, 
	target_address : [u8; 32], 
	amount : u128
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode, MaxEncodedLen)]
struct PalletAssetBalanceRequest{
	asset_id : u32, 
	address : [u8; 32], 
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode, MaxEncodedLen)]
pub enum PalletAssetErr {
	/// Some error occurred.
    Other,
    /// Failed to lookup some data.
	CannotLookup,
	/// A bad origin.
	BadOrigin,
	/// A custom error in a module.
	Module,
	/// At least one consumer is remaining so the account cannot be destroyed.
	ConsumerRemaining,
	/// There are no providers so the account cannot be created.
	NoProviders,
	/// There are too many consumers so the account cannot be created.
	TooManyConsumers,
	/// An error to do with tokens.
	Token(PalletAssetTokenErr),
	/// An arithmetic error.
	Arithmetic(PalletAssetArithmeticErr),
	//unknown error
    Unknown,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode, MaxEncodedLen)]
pub enum PalletAssetArithmeticErr {
	/// Underflow.
	Underflow,
	/// Overflow.
	Overflow,
	/// Division by zero.
	DivisionByZero,
	//unknown error
    Unknown,

}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode, MaxEncodedLen)]
pub enum PalletAssetTokenErr {
	/// Funds are unavailable.
	NoFunds,
	/// Account that must exist would die.
	WouldDie,
	/// Account cannot exist with the funds that would be given.
	BelowMinimum,
	/// Account cannot be created.
	CannotCreate,
	/// The asset in question is unknown.
	UnknownAsset,
	/// Funds exist but are frozen.
	Frozen,
	/// Operation is not supported by the asset.
	Unsupported,
	//unknown error
    Unknown,
}

impl From<DispatchError> for PalletAssetErr {
    fn from(e: DispatchError) -> Self {
        match e{
			DispatchError::Other(_) => PalletAssetErr::Other,
			DispatchError::CannotLookup => PalletAssetErr::CannotLookup,
			DispatchError::BadOrigin => PalletAssetErr::BadOrigin,
			DispatchError::Module(_) => PalletAssetErr::Module,
			DispatchError::ConsumerRemaining => PalletAssetErr::ConsumerRemaining,
			DispatchError::NoProviders => PalletAssetErr::NoProviders,
			DispatchError::TooManyConsumers => PalletAssetErr::TooManyConsumers,
			DispatchError::Token(token_err) => PalletAssetErr::Token(PalletAssetTokenErr::from(token_err)),
			DispatchError::Arithmetic(arithmetic_error) => PalletAssetErr::Arithmetic(PalletAssetArithmeticErr::from(arithmetic_error)),
			_ => PalletAssetErr::Unknown,
		}
    }
}

impl From<ArithmeticError> for PalletAssetArithmeticErr {
    fn from(e: ArithmeticError) -> Self {
        match e{
			ArithmeticError::Underflow => PalletAssetArithmeticErr::Underflow,
			ArithmeticError::Overflow => PalletAssetArithmeticErr::Overflow,
			ArithmeticError::DivisionByZero => PalletAssetArithmeticErr::DivisionByZero,
			_ => PalletAssetArithmeticErr::Unknown,
		}
    }
}

impl From<TokenError> for PalletAssetTokenErr {
    fn from(e: TokenError) -> Self {
        match e{
			TokenError::NoFunds => PalletAssetTokenErr::NoFunds,
			TokenError::WouldDie => PalletAssetTokenErr::WouldDie,
			TokenError::BelowMinimum => PalletAssetTokenErr::BelowMinimum,
			TokenError::CannotCreate => PalletAssetTokenErr::CannotCreate,
			TokenError::UnknownAsset => PalletAssetTokenErr::UnknownAsset,
			TokenError::Frozen => PalletAssetTokenErr::Frozen,
			TokenError::Unsupported => PalletAssetTokenErr::Unsupported,
			_ => PalletAssetTokenErr::Unknown,
		}
    }
}

type PSP22Result = Result::<(),PalletAssetErr>;

impl<T> ChainExtension<T> for Psp22AssetExtension
 where
 	T: SysConfig + pallet_assets::Config + pallet_contracts::Config,
 	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
 {
 	fn call<E: Ext>(func_id: u32, mut env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
 	where
 		E: Ext<T = T>,
 		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
 	{
		

        match func_id {

			//create
			1102 => {
                let mut env = env.buf_in_buf_out();
                let create_asset: (OriginType, T::AssetId, T::AccountId, T::Balance) = env.read_as()?;
				let (origin_id, asset_id, account_id, balance) = create_asset;
				let create_result = <pallet_assets::Pallet<T> as Create<T::AccountId>>::
					create(asset_id, account_id, true, balance);

				match create_result {
					DispatchResult::Ok(_) => {
					}
					DispatchResult::Err(e) => {
						let err = Result::<(),PalletAssetErr>::Err(PalletAssetErr::from(e));
						env.write(&err.encode(), false, None).map_err(|_| {
							DispatchError::Other("ChainExtension failed to call 'approve transfer'")
						})?;
					}
				}

            }

			//mint
			1103 => {
				let ext = env.ext();
                let mut env = env.buf_in_buf_out();
                let create_asset: (OriginType, T::AssetId, T::AccountId, T::Balance) = env.read_as()?;
				let (origin_id, asset_id, account_id, balance) = create_asset;

				if origin_id == OriginType::Caller
				{
					// let address_account = AccountId::decode(&mut ext.address().as_ref()).unwrap();
					// pallet_assets::Pallet::<Runtime>::transfer_ownership(Origin::signed(address_account.clone()), asset_id, MultiAddress::Id(address_account.clone()))?;
				}
				
				let create_result = <pallet_assets::Pallet<T> as Mutate<T::AccountId>>::
					mint_into(asset_id, &account_id, balance);


				match create_result {
					DispatchResult::Ok(_) => {},
					DispatchResult::Err(e) => {
						let err = PSP22Result::Err(PalletAssetErr::from(e));
						env.write(&err.encode(), false, None).map_err(|_| {
							DispatchError::Other("ChainExtension failed to call mint")
						})?;
					}
				}
            }

			//burn
			1104 => {
				let ext = env.ext();
                let mut env = env.buf_in_buf_out();
                let create_asset: (OriginType, T::AssetId, T::AccountId, T::Balance) = env.read_as()?;
				let (origin_id, asset_id, account_id, balance) = create_asset;

				if origin_id == OriginType::Caller
				{
					// let address_account = AccountId::decode(&mut ext.address().as_ref()).unwrap();
					// pallet_assets::Pallet::<Runtime>::transfer_ownership(Origin::signed(address_account.clone()), asset_id, MultiAddress::Id(address_account.clone()))?;
				}
				
				let burn_result = <pallet_assets::Pallet<T> as Mutate<T::AccountId>>::
					burn_from(asset_id, &account_id, balance);

				// match burn_result {
				// 	DispatchResult::Ok(_) => {},
				// 	DispatchResult::Err(e) => {
				// 		// let err = PSP22Result::Err(PalletAssetErr::from(e));
				// 		// env.write(&err.encode(), false, None).map_err(|_| {
				// 		// 	DispatchError::Other("ChainExtension failed to call burn")
				// 		// })?;
				// 	}
				// }
				
            }

			//transfer
			1105 => {

				let ext = env.ext();
				let address = ext.address().clone();
				let caller = ext.caller().clone();
                let mut env = env.buf_in_buf_out();
                let create_asset: (OriginType, T::AssetId, T::AccountId, T::Balance) = env.read_as()?;
				let (origin_id, asset_id, account_id, balance) = create_asset;

				let address_account;
				if origin_id == OriginType::Caller
				{
					address_account = address;
					// let a = AccountId::decode(&mut ext.address().as_ref()).unwrap();
					// pallet_assets::Pallet::<Runtime>::transfer_ownership(Origin::signed(a.clone()), asset_id, MultiAddress::Id(a.clone()))?;
				}
				else{
					address_account = caller;
				}
				
				let result = <pallet_assets::Pallet<T> as Transfer<T::AccountId>>::
					transfer(asset_id, &address_account, &account_id, balance, true);


				// match result {
				// 	DispatchResult::Ok(_) => {},
				// 	DispatchResult::Err(e) => {
				// 		// let err = PSP22Result::Err(PalletAssetErr::from(e));
				// 		// env.write(&err.encode(), false, None).map_err(|_| {
				// 		// 	DispatchError::Other("ChainExtension failed to call transfer")
				// 		// })?;
				// 	}
				// }
            }
			
			//balance
			1106 => {
				let ext = env.ext();
                let mut env = env.buf_in_buf_out();
				let create_asset: (T::AssetId, T::AccountId) = env.read_as()?;
				let (asset_id, account_id) = create_asset;
				let balance = <pallet_assets::Pallet<T> as Inspect<T::AccountId>>::
						balance(asset_id, &account_id);

                env.write(&balance.encode(), false, None).map_err(|_| {
                    DispatchError::Other("ChainExtension failed to call balance")
                })?;
            }

			//total_supply
			1107 => {
                let mut env = env.buf_in_buf_out();
                let asset_id: T::AssetId = env.read_as()?;
				let total_supply : T::Balance = pallet_assets::Pallet::<T>::total_supply(asset_id);
                env.write(&total_supply.encode(), false, None).map_err(|_| {
                    DispatchError::Other("ChainExtension failed to call total_supply")
                })?;
			}

			//approve_transfer
			1108 => {
				let ext = env.ext();
				let address = ext.address().clone();
				let caller = ext.caller().clone();
                let mut env = env.buf_in_buf_out();
                let create_asset: (OriginType, T::AssetId, T::AccountId, T::Balance) = env.read_as()?;
				let (origin_type, asset, to, amount) = create_asset;

				let from;
				if origin_type == OriginType::Caller
				{
					from = address;
					// let a = AccountId::decode(&mut ext.address().as_ref()).unwrap();
					// pallet_assets::Pallet::<Runtime>::transfer_ownership(Origin::signed(a.clone()), asset_id, MultiAddress::Id(a.clone()))?;
				}
				else{
					from = caller;
				}
				let result = <pallet_assets::Pallet::<T> as AllowanceMutate<T::AccountId>>::
									approve(asset, &from, &to, amount);
				match result {
					DispatchResult::Ok(_) => {
					}
					DispatchResult::Err(e) => {
						let err = Result::<(),PalletAssetErr>::Err(PalletAssetErr::from(e));
						env.write(&err.encode(), false, None).map_err(|_| {
							DispatchError::Other("ChainExtension failed to call 'approve'")
						})?;
					}
				}
            }

			//transfer_approved
			1109 => {
				let ext = env.ext();
				let address = ext.address().clone();
				let caller = ext.caller().clone();
                let mut env = env.buf_in_buf_out();
                let create_asset: (T::AccountId, (OriginType, T::AssetId, T::AccountId, T::Balance)) = env.read_as()?;
				let owner = create_asset.0;
				let (origin_type, asset, to, amount) = create_asset.1;

				let from;
				if origin_type == OriginType::Caller
				{
					from = address;
					// let a = AccountId::decode(&mut ext.address().as_ref()).unwrap();
					// pallet_assets::Pallet::<Runtime>::transfer_ownership(Origin::signed(a.clone()), asset_id, MultiAddress::Id(a.clone()))?;
				}
				else{
					from = caller;
				}
				let result = <pallet_assets::Pallet::<T> as AllowanceMutate<T::AccountId>>::
					transfer_from(asset, &from, &owner, &to, amount);
				match result {
					DispatchResult::Ok(_) => {
					}
					DispatchResult::Err(e) => {
						let err = Result::<(),PalletAssetErr>::Err(PalletAssetErr::from(e));
						env.write(&err.encode(), false, None).map_err(|_| {
							DispatchError::Other("ChainExtension failed to call 'approved transfer'")
						})?;
					}
				}
            }

			//allowance
			1110 => {
                let mut env = env.buf_in_buf_out();
                let allowance_request: (T::AssetId, T::AccountId, T::AccountId) = env.read_as()?;

				let allowance = <pallet_assets::Pallet<T> as AllowanceInspect<T::AccountId>>
					::allowance(allowance_request.0, &allowance_request.1, &allowance_request.2);

                env.write(&allowance.encode(), false, None).map_err(|_| {
                    DispatchError::Other("ChainExtension failed to call balance")
                })?;
            }

			/*
			//increase_allowance/decrease_allowance
			1111 => {
				use frame_support::dispatch::DispatchResult;

                let mut env = env.buf_in_buf_out();
                let request: (u32, [u8; 32], [u8; 32], u128, bool) = env.read_as()?;
				let (asset_id, owner, delegate, amount, is_increase) = request;

				let mut vec = &owner.to_vec()[..];
				let owner_address = AccountId::decode(&mut vec).unwrap();

				let mut vec = &delegate.to_vec()[..];
				let delegate_address = AccountId::decode(&mut vec).unwrap();
				
				use crate::sp_api_hidden_includes_construct_runtime::hidden_include::traits::fungibles::approvals::Inspect;
                let allowance :u128 = Assets::allowance(asset_id, &owner_address, &delegate_address);

				let new_allowance = 
				if is_increase {allowance + amount} 
				else {
					if allowance < amount  { 0 }
					else {allowance - amount}
				};

				let cancel_approval_result = pallet_assets::Pallet::<Runtime>::
				cancel_approval(Origin::signed(owner_address.clone()),
				asset_id, 
				MultiAddress::Id(delegate_address.clone()));
				match cancel_approval_result {
					DispatchResult::Ok(_) => {
						error!("OK cancel_approval")
					}
					DispatchResult::Err(e) => {
						error!("ERROR cancel_approval");
						error!("{:#?}", e);
						let err = Result::<(),PalletAssetErr>::Err(PalletAssetErr::from(e));
						env.write(&err.encode(), false, None).map_err(|_| {
							DispatchError::Other("ChainExtension failed to call 'approve transfer'")
						})?;
					}
				}

				if cancel_approval_result.is_ok(){
					let approve_transfer_result = pallet_assets::Pallet::<Runtime>::
					approve_transfer(Origin::signed(owner_address),
					asset_id, 
					MultiAddress::Id(delegate_address), 
					new_allowance);

					error!("old allowance {}", allowance);
					error!("new allowance {}", new_allowance);
					error!("increase_allowance input {:#?}", request);
					error!("increase_allowance output {:#?}", approve_transfer_result);
					match approve_transfer_result {
						DispatchResult::Ok(_) => {
							error!("OK increase_allowance")
						}
						DispatchResult::Err(e) => {
							error!("ERROR increase_allowance");
							error!("{:#?}", e);
							let err = Result::<(),PalletAssetErr>::Err(PalletAssetErr::from(e));
							env.write(&err.encode(), false, None).map_err(|_| {
								DispatchError::Other("ChainExtension failed to call 'approve transfer'")
							})?;
						}
					}
				}
            }
			 */

			//set_metadata
			1112 => {
				let ext = env.ext();
				let address = ext.address().clone();
				let caller = ext.caller().clone();
                let mut env = env.buf_in_buf_out();
                let input: (OriginType, T::AssetId, Vec<u8>, Vec<u8>, u8) = env.read_as_unbounded(env.in_len())?;
				let (origin_type, asset_id, name, symbol, decimals) = input;
				
				let from = if origin_type == OriginType::Caller { address } else{ caller };
				
				let result = <pallet_assets::Pallet::<T> as MetadataMutate<T::AccountId>>::
					set(asset_id, &from, name, symbol, decimals);

				match result {
					DispatchResult::Ok(_) => {},
					DispatchResult::Err(e) => {
						let err = PSP22Result::Err(PalletAssetErr::from(e));
						env.write(&err.encode(), false, None).map_err(|_| {
							DispatchError::Other("ChainExtension failed to call set_metadata")
						})?;
					}
				}
			}

			//get asset metadata name
			1113 => {
                let mut env = env.buf_in_buf_out();
                let asset_id = env.read_as()?;
				let name = <pallet_assets::Pallet<T> as InspectMetadata<T::AccountId>>::name(&asset_id).encode();
                env.write(&name.encode()[..], false, None).map_err(|_| {
                    DispatchError::Other("ChainExtension failed to call get metadata name")
                })?;
			}

			//get asset metadata symbol
			1114 => {
                let mut env = env.buf_in_buf_out();
                let asset_id = env.read_as()?;
				let symbol = <pallet_assets::Pallet<T> as InspectMetadata<T::AccountId>>::symbol(&asset_id);
                env.write(&symbol.encode()[..], false, None).map_err(|_| {
                    DispatchError::Other("ChainExtension failed to call balance")
                })?;
			}

			//decimals
			1115 => {
                let mut env = env.buf_in_buf_out();
                let asset_id = env.read_as()?;
				let decimals = <pallet_assets::Pallet<T> as InspectMetadata<T::AccountId>>::decimals(&asset_id,);
                env.write(&decimals.encode()[..], false, None).map_err(|_| {
                    DispatchError::Other("ChainExtension failed to call total_supply")
                })?;
			}
            _ => {
                error!("Called an unregistered `func_id`: {:}", func_id);
                return Err(DispatchError::Other("Unimplemented func_id"))
            }
        }

        Ok(RetVal::Converging(0))

		
    }

    fn enabled() -> bool {
        true
    }
}