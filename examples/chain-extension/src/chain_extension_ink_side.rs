use ink::env::{chain_extension::FromStatusCode, DefaultEnvironment, Environment};

/// Simple chain extension that provides some staking information.
#[ink::chain_extension(extension = 0)]
pub trait StakingExtension {
    type ErrorCode = StakingExtensionErrorCode;

    /// Returns the number of the validators.
    #[ink(function = 41, handle_status = false)]
    fn get_num_of_validators() -> u32;
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, scale::Encode, scale::Decode)]
pub struct StakingExtensionErrorCode(u32);
impl FromStatusCode for StakingExtensionErrorCode {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            _ => Err(Self(status_code)),
        }
    }
}

/// Default ink environment with `StakingExtension` included.
#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum StakingEnvironment {}

impl Environment for StakingEnvironment {
    const MAX_EVENT_TOPICS: usize = <DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <DefaultEnvironment as Environment>::AccountId;
    type Balance = <DefaultEnvironment as Environment>::Balance;
    type Hash = <DefaultEnvironment as Environment>::Hash;
    type Timestamp = <DefaultEnvironment as Environment>::Timestamp;
    type BlockNumber = <DefaultEnvironment as Environment>::BlockNumber;
    type EventRecord = ();

    type ChainExtension = StakingExtension;
}
