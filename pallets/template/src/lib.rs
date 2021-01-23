#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_module, decl_storage, decl_event, decl_error, ensure, StorageMap
};
use frame_system::ensure_signed;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
/// 通过指定托盘所依赖的参数和类型来配置托盘。
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	/// 因为此托盘会发出事件，所以它依赖于运行时对事件的定义。
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        /// The storage item for our proofs.
        /// It maps a proof to the user who made the claim and when they made it.
        Proofs: map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
    }
}

// Pallets use events to inform users when important changes are made.
// Event documentation should end with an array that provides descriptive names for parameters.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event! {
    pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
        /// Event emitted when a proof has been claimed. [who, claim]
        ClaimCreated(AccountId, Vec<u8>),
        /// Event emitted when a claim is revoked by the owner. [who, claim]
        ClaimRevoked(AccountId, Vec<u8>),
        /// Event emitted when a claim is changed by the owner. [who, to, claim]  ///simon
        ClaimChanged(AccountId, Receiver, Vec<u8>),
    }
}

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The proof has already been claimed.
        ProofAlreadyClaimed,
        /// The proof does not exist, so it cannot be revoked.
        NoSuchProof,
        /// The proof is claimed by another account, so caller can't revoke it.
        NotProofOwner,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
// 可调用函数
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // 如果在可调用函数里，需要用到错误信息类型，就需要这样写，可以理解成固定用法
        type Error = Error<T>;

        // 如果在可调用函数里，需要触发事件，就需要这样写，可以理解成固定用法
        fn deposit_event() = default;

        /// 允许用户提交一个未存证的存证
        /// weight是当前函数的权重
        #[weight = 10_000]
        fn create_claim(origin, proof: Vec<u8>) {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // Verify that the specified proof has not already been claimed.
            ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);

            // Get the block number from the FRAME System module.
            let current_block = <frame_system::Module<T>>::block_number();

            // Store the proof with the sender and block number.
            Proofs::<T>::insert(&proof, (&sender, current_block));

            // Emit an event that the claim was created.
            Self::deposit_event(RawEvent::ClaimCreated(sender, proof));
        }

        /// Allow the owner to revoke their claim.
        #[weight = 10_000]
        fn revoke_claim(origin, proof: Vec<u8>) {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // Verify that the specified proof has been claimed.
            ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

            // Get owner of the claim.
            let (owner, _) = Proofs::<T>::get(&proof);

            // Verify that sender of the current call is the claim owner.
            ensure!(sender == owner, Error::<T>::NotProofOwner);

            // Remove claim from storage.
            Proofs::<T>::remove(&proof);

            // Emit an event that the claim was erased.
            Self::deposit_event(RawEvent::ClaimRevoked(sender, proof));
        }

        /// 允许转移存证给他人
        #[weight = 10_000]
        fn change_owner_claim(origin, receiver: T::AccountId, proof: Vec<u8>) {
            // 检查调用者是否已签名
            // 如果未签名，则函数将返回错误
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // 检查存证是否存在
            ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

            // 获取存证的所有者
            let (owner, _) = Proofs::<T>::get(&proof);

            // 检查调用者是否为存证的所有者
            ensure!(sender == owner, Error::<T>::NotProofOwner);

            // 从FRAME系统模块获取块号
            let current_block = <frame_system::Module<T>>::block_number();

            // 修改存证所有者，并与当前区块号一起存储
            // 都是用insert，因为存储时是用proof作为key，修改时直接覆盖这个key的值即可
            // 参考：https://substrate.dev/rustdocs/v2.0.0/frame_support/storage/trait.StorageMap.html
            // https://substrate.dev/recipes/storage-maps.html
            Proofs::<T>::insert(&proof, (&receiver, current_block));

            // 触发修改存证所有者事件
            Self::deposit_event(RawEvent::ClaimChanged(sender, receiver, proof));

            // Runtime模块里存在保留函数，除了deposit_event之外，还有：
            // on_initialize，在每个区块的开头执行；
            // on_finalize，在每个区块结束时执行；
            // offchain_worker，开头且是链外执行，不占用链上的计算和存储资源；
            // 用来执行一些计算复杂度高，或者需要与外部的数据源进行交互的场景，
            // 比如当我们需要http请求外部数据时，就需要用到offchain_worker，优势是不占用链上的计算和存储资源
            // on_runtime_upgrade，当有runtime升级时才会执行，用来迁移数据。
        }

    }
}
