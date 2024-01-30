use alloy_primitives::{Address, B256, U256 as AlloyU256};
use ethers_core::types::{H160, H256, U256};

pub trait ToAlloy {
    type To;
    fn to_alloy(self) -> Self::To;
}

impl ToAlloy for H256 {
    type To = B256;

    #[inline(always)]
    fn to_alloy(self) -> Self::To {
        B256::new(self.0)
    }
}

impl ToAlloy for U256 {
    type To = AlloyU256;

    #[inline(always)]
    fn to_alloy(self) -> Self::To {
        AlloyU256::from_limbs(self.0)
    }
}

pub trait ToEthers {
    type To;

    fn to_ethers(self) -> Self::To;
}

impl ToEthers for Address {
    type To = H160;

    #[inline(always)]
    fn to_ethers(self) -> Self::To {
        H160(self.0 .0)
    }
}

impl ToEthers for B256 {
    type To = H256;

    #[inline(always)]
    fn to_ethers(self) -> Self::To {
        H256(self.0)
    }
}
