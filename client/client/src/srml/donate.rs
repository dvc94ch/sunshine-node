use crate::srml::org::{
    Org,
    OrgEventsDecoder,
};
use codec::{
    Codec,
    Encode,
};
use frame_support::Parameter;
use sp_runtime::traits::{
    AtLeast32Bit,
    MaybeSerializeDeserialize,
    Member,
    Zero,
};
use std::fmt::Debug;
use substrate_subxt::system::{
    System,
    SystemEventsDecoder,
};
use util::bank::OnChainTreasuryID;

/// The donation balance type
pub type DonateBalanceOf<T> = <T as Donate>::DCurrency; // as Currency<<T as System>::AccountId>>::Balance;

#[module]
pub trait Donate: System + Org {
    /// The currency type for on-chain transactions
    type DCurrency: Parameter
        + Member
        + AtLeast32Bit
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + PartialOrd
        + PartialEq
        + Zero; // + Currency<<Self as System>::AccountId> // commented out until #93 is resolved
}

// ~~ Values (Constants) ~~

#[derive(Clone, Debug, Eq, PartialEq, Encode)]
pub struct TransactionFee<T: Donate> {
    pub amount: DonateBalanceOf<T>,
}

#[derive(Clone, Debug, Eq, PartialEq, Encode)]
pub struct TreasuryAddress {
    // ModuleId type which implements Debug
    pub module_id: OnChainTreasuryID,
}

// ~~ Calls and Events ~~

#[derive(Clone, Debug, Eq, PartialEq, Call, Encode)]
pub struct MakePropDonationWithFeeCall<T: Donate> {
    pub org: <T as Org>::OrgId,
    pub amt: DonateBalanceOf<T>,
}

#[derive(Clone, Debug, Eq, PartialEq, Call, Encode)]
pub struct MakePropDonationWithoutFeeCall<T: Donate> {
    pub org: <T as Org>::OrgId,
    pub amt: DonateBalanceOf<T>,
}

#[derive(Clone, Debug, Eq, PartialEq, Event, Decode)]
pub struct DonationExecutedEvent<T: Donate> {
    pub sender: <T as System>::AccountId,
    pub org: <T as Org>::OrgId,
    pub amt: DonateBalanceOf<T>,
    pub fee: bool,
}
