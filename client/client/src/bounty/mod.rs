mod subxt;

pub use subxt::*;
pub use sunshine_bounty_utils::bounty::*;

use crate::error::Error;
use async_trait::async_trait;
use substrate_subxt::{
    Runtime,
    SignedExtension,
    SignedExtra,
};
use sunshine_core::ChainClient;

#[async_trait]
pub trait BountyClient<T: Runtime + Bounty>: ChainClient<T> {
    async fn post_bounty(
        &self,
        bounty: T::BountyPost,
        amount: BalanceOf<T>,
    ) -> Result<BountyPostedEvent<T>, Self::Error>;
    async fn contribute_to_bounty(
        &self,
        bounty_id: T::BountyId,
        amount: BalanceOf<T>,
    ) -> Result<BountyRaiseContributionEvent<T>, Self::Error>;
    async fn submit_for_bounty(
        &self,
        bounty_id: T::BountyId,
        submission: T::BountySubmission,
        amount: BalanceOf<T>,
    ) -> Result<BountySubmissionPostedEvent<T>, Self::Error>;
    async fn approve_bounty_submission(
        &self,
        submission_id: T::SubmissionId,
    ) -> Result<BountyPaymentExecutedEvent<T>, Self::Error>;
    async fn bounty(
        &self,
        bounty_id: T::BountyId,
    ) -> Result<BountyState<T>, Self::Error>;
    async fn submission(
        &self,
        submission_id: T::SubmissionId,
    ) -> Result<SubState<T>, Self::Error>;
    async fn open_bounties(
        &self,
        min: BalanceOf<T>,
    ) -> Result<Option<Vec<(T::BountyId, BountyState<T>)>>, Self::Error>;
    async fn open_submissions(
        &self,
        bounty_id: T::BountyId,
    ) -> Result<Option<Vec<(T::SubmissionId, SubState<T>)>>, Self::Error>;
}

#[async_trait]
impl<T, C> BountyClient<T> for C
where
    T: Runtime + Bounty,
    <<T::Extra as SignedExtra<T>>::Extra as SignedExtension>::AdditionalSigned:
        Send + Sync,
    <T as Bounty>::IpfsReference: From<libipld::cid::Cid>,
    C: ChainClient<T>,
    C::Error: From<Error>,
    C::OffchainClient: ipld_block_builder::Cache<
            ipld_block_builder::Codec,
            <T as Bounty>::BountyPost,
        > + ipld_block_builder::Cache<
            ipld_block_builder::Codec,
            <T as Bounty>::BountySubmission,
        >,
{
    async fn post_bounty(
        &self,
        bounty: T::BountyPost,
        amount: BalanceOf<T>,
    ) -> Result<BountyPostedEvent<T>, C::Error> {
        let signer = self.chain_signer()?;
        let info = crate::post(self, bounty).await?;
        self.chain_client()
            .post_bounty_and_watch(signer, info.into(), amount)
            .await?
            .bounty_posted()?
            .ok_or_else(|| Error::EventNotFound.into())
    }
    async fn contribute_to_bounty(
        &self,
        bounty_id: T::BountyId,
        amount: BalanceOf<T>,
    ) -> Result<BountyRaiseContributionEvent<T>, C::Error> {
        let signer = self.chain_signer()?;
        self.chain_client()
            .contribute_to_bounty_and_watch(signer, bounty_id, amount)
            .await?
            .bounty_raise_contribution()?
            .ok_or_else(|| Error::EventNotFound.into())
    }
    async fn submit_for_bounty(
        &self,
        bounty_id: T::BountyId,
        submission: T::BountySubmission,
        amount: BalanceOf<T>,
    ) -> Result<BountySubmissionPostedEvent<T>, C::Error> {
        let signer = self.chain_signer()?;
        let submission_ref = crate::post(self, submission).await?;
        self.chain_client()
            .submit_for_bounty_and_watch(
                signer,
                bounty_id,
                submission_ref.into(),
                amount,
            )
            .await?
            .bounty_submission_posted()?
            .ok_or_else(|| Error::EventNotFound.into())
    }
    async fn approve_bounty_submission(
        &self,
        submission_id: T::SubmissionId,
    ) -> Result<BountyPaymentExecutedEvent<T>, C::Error> {
        let signer = self.chain_signer()?;
        self.chain_client()
            .approve_bounty_submission_and_watch(signer, submission_id)
            .await?
            .bounty_payment_executed()?
            .ok_or_else(|| Error::EventNotFound.into())
    }
    async fn bounty(
        &self,
        bounty_id: T::BountyId,
    ) -> Result<BountyState<T>, C::Error> {
        Ok(self
            .chain_client()
            .bounties(bounty_id, None)
            .await
            .map_err(Error::Subxt)?)
    }
    async fn submission(
        &self,
        submission_id: T::SubmissionId,
    ) -> Result<SubState<T>, C::Error> {
        Ok(self
            .chain_client()
            .submissions(submission_id, None)
            .await
            .map_err(Error::Subxt)?)
    }
    async fn open_bounties(
        &self,
        min: BalanceOf<T>,
    ) -> Result<Option<Vec<(T::BountyId, BountyState<T>)>>, C::Error> {
        Ok(self
            .chain_client()
            .open_bounties(min, None)
            .await
            .map_err(Error::Subxt)?)
    }
    async fn open_submissions(
        &self,
        bounty_id: T::BountyId,
    ) -> Result<Option<Vec<(T::SubmissionId, SubState<T>)>>, C::Error> {
        Ok(self
            .chain_client()
            .open_submissions(bounty_id, None)
            .await
            .map_err(Error::Subxt)?)
    }
}

#[cfg(test)]
mod tests {
    use rand::{
        rngs::OsRng,
        RngCore,
    };
    use sunshine_core::ChainClient;
    use test_client::{
        bounty::{
            BountyClient,
            BountyPostedEvent,
        },
        bounty_client::BountyBody,
        mock::{
            test_node,
            AccountKeyring,
        },
        Client,
    };

    // For testing purposes only, NEVER use this to generate AccountIds in practice because it's random
    pub fn _random_account_id() -> substrate_subxt::sp_runtime::AccountId32 {
        let mut buf = [0u8; 32];
        OsRng.fill_bytes(&mut buf);
        buf.into()
    }

    #[async_std::test]
    async fn simple_test() {
        use substrate_subxt::balances::TransferCallExt;
        let (node, _node_tmp) = test_node();
        let (client, _client_tmp) =
            Client::mock(&node, AccountKeyring::Alice).await;
        let alice_account_id = AccountKeyring::Alice.to_account_id();
        client
            .chain_client()
            .transfer(client.chain_signer().unwrap(), &alice_account_id, 10_000)
            .await
            .unwrap();
    }

    #[async_std::test]
    async fn post_bounty_test() {
        let (node, _node_tmp) = test_node();
        let (client, _client_tmp) =
            Client::mock(&node, AccountKeyring::Alice).await;
        let alice_account_id = AccountKeyring::Alice.to_account_id();
        let bounty = BountyBody {
            repo_owner: "sunshine-protocol".to_string(),
            repo_name: "sunshine-bounty".to_string(),
            issue_number: 124,
        };
        let event = client.post_bounty(bounty, 40u128).await.unwrap();
        let expected_event = BountyPostedEvent {
            depositer: alice_account_id,
            amount: 10,
            id: 1,
            description: event.description.clone(),
        };
        assert_eq!(event, expected_event);
    }
}
