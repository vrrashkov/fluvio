/// WARNING: CODE GENERATED FILE
/// * This file is generated by kfspec2code.
/// * Any changes applied to this file will be lost when a new spec is generated.
use std::fmt::Debug;
use std::marker::PhantomData;

use kf_protocol::Decoder;
use kf_protocol::Encoder;

use serde::{Deserialize, Serialize};

use kf_protocol_api::ErrorCode;
use kf_protocol_api::Isolation;
use kf_protocol_api::Request;

use kf_protocol_derive::Decode;
use kf_protocol_derive::Encode;
use kf_protocol_derive::KfDefault;

// -----------------------------------
// KfFetchRequest<R>
// -----------------------------------

#[derive(Encode, Decode, Serialize, Deserialize, KfDefault, Debug)]
pub struct KfFetchRequest<R>
where
    R: Encoder + Decoder + Default + Debug,
{
    /// The broker ID of the follower, of -1 if this request is from a consumer.
    pub replica_id: i32,

    /// The maximum time in milliseconds to wait for the response.
    pub max_wait: i32,

    /// The minimum bytes to accumulate in the response.
    pub min_bytes: i32,

    /// The maximum bytes to fetch.  See KIP-74 for cases where this limit may not be honored.
    #[fluvio_kf(min_version = 3, ignorable)]
    pub max_bytes: i32,

    /// This setting controls the visibility of transactional records. Using READ_UNCOMMITTED
    /// (isolation_level = 0) makes all records visible. With READ_COMMITTED (isolation_level = 1),
    /// non-transactional and COMMITTED transactional records are visible. To be more concrete,
    /// READ_COMMITTED returns all data from offsets smaller than the current LSO (last stable
    /// offset), and enables the inclusion of the list of aborted transactions in the result, which
    /// allows consumers to discard ABORTED transactional records
    #[fluvio_kf(min_version = 4)]
    pub isolation_level: Isolation,

    /// The fetch session ID.
    #[fluvio_kf(min_version = 7)]
    pub session_id: i32,

    /// The fetch session ID.
    #[fluvio_kf(min_version = 7)]
    pub epoch: i32,

    /// The topics to fetch.
    pub topics: Vec<FetchableTopic>,

    /// In an incremental fetch request, the partitions to remove.
    #[fluvio_kf(min_version = 7)]
    pub forgotten: Vec<ForgottenTopic>,

    pub data: PhantomData<R>,
}

#[derive(Encode, Decode, Serialize, Deserialize, KfDefault, Debug)]
pub struct FetchableTopic {
    /// The name of the topic to fetch.
    pub name: String,

    /// The partitions to fetch.
    pub fetch_partitions: Vec<FetchPartition>,
}

#[derive(Encode, Decode, Serialize, Deserialize, KfDefault, Debug)]
pub struct ForgottenTopic {
    /// The partition name.
    #[fluvio_kf(min_version = 7)]
    pub name: String,

    /// The partitions indexes to forget.
    #[fluvio_kf(min_version = 7)]
    pub forgotten_partition_indexes: Vec<i32>,
}

#[derive(Encode, Decode, Serialize, Deserialize, KfDefault, Debug)]
pub struct FetchPartition {
    /// The partition index.
    pub partition_index: i32,

    /// The current leader epoch of the partition.
    #[fluvio_kf(min_version = 9, ignorable)]
    pub current_leader_epoch: i32,

    /// The message offset.
    pub fetch_offset: i64,

    /// The earliest available offset of the follower replica.  The field is only used when the
    /// request is sent by the follower.
    #[fluvio_kf(min_version = 5)]
    pub log_start_offset: i64,

    /// The maximum bytes to fetch from this partition.  See KIP-74 for cases where this limit may
    /// not be honored.
    pub max_bytes: i32,
}

// -----------------------------------
// KfFetchResponse<R>
// -----------------------------------

#[derive(Encode, Decode, Serialize, Deserialize, KfDefault, Debug)]
pub struct KfFetchResponse<R>
where
    R: Encoder + Decoder + Default + Debug,
{
    /// The duration in milliseconds for which the request was throttled due to a quota violation,
    /// or zero if the request did not violate any quota.
    #[fluvio_kf(min_version = 1, ignorable)]
    pub throttle_time_ms: i32,

    /// The top level response error code.
    #[fluvio_kf(min_version = 7)]
    pub error_code: ErrorCode,

    /// The fetch session ID, or 0 if this is not part of a fetch session.
    #[fluvio_kf(min_version = 7)]
    pub session_id: i32,

    /// The response topics.
    pub topics: Vec<FetchableTopicResponse<R>>
}

impl <R>KfFetchResponse<R> 
    where R: Encoder + Decoder + Default + Debug {

    pub fn find_partition(self,topic: &str,partition: i32) -> Option<FetchablePartitionResponse<R>> {

        for topic_res in self.topics {
            if topic_res.name == topic {
                for partition_res in topic_res.partitions {
                    if partition_res.partition_index == partition {
                        return Some(partition_res);
                    }
                }
            }
        }
    
        None
    
    }
        

}


#[derive(Encode, Decode, Serialize, Deserialize, KfDefault, Debug)]
pub struct FetchableTopicResponse<R>
where
    R: Encoder + Decoder + Default + Debug,
{
    /// The topic name.
    pub name: String,

    /// The topic partitions.
    pub partitions: Vec<FetchablePartitionResponse<R>>,
    pub data: PhantomData<R>,
}

#[derive(Encode, Decode, Serialize, Deserialize, KfDefault, Debug)]
pub struct FetchablePartitionResponse<R>
    where
        R: Encoder + Decoder + Default + Debug,
{
    /// The partiiton index.
    pub partition_index: i32,

    /// The error code, or 0 if there was no fetch error.
    pub error_code: ErrorCode,

    /// The current high water mark.
    pub high_watermark: i64,

    /// The last stable offset (or LSO) of the partition. This is the last offset such that the
    /// state of all transactional records prior to this offset have been decided (ABORTED or
    /// COMMITTED)
    #[fluvio_kf(min_version = 4, ignorable)]
    pub last_stable_offset: i64,

    /// The current log start offset.
    #[fluvio_kf(min_version = 5, ignorable)]
    pub log_start_offset: i64,

    /// The aborted transactions.
    #[fluvio_kf(min_version = 4)]
    pub aborted: Option<Vec<AbortedTransaction>>,

    /// The record data.
    pub records: R,
}

#[derive(Encode, Decode, Serialize, Deserialize, KfDefault, Debug)]
pub struct AbortedTransaction {
    /// The producer id associated with the aborted transaction.
    #[fluvio_kf(min_version = 4)]
    pub producer_id: i64,

    /// The first offset in the aborted transaction.
    #[fluvio_kf(min_version = 4)]
    pub first_offset: i64,
}

// -----------------------------------
// Implementation - KfFetchRequest<R>
// -----------------------------------

impl<R> Request for KfFetchRequest<R>
    where R: Debug + Decoder + Encoder
{
    const API_KEY: u16 = 1;

    const MIN_API_VERSION: i16 = 0;
    const MAX_API_VERSION: i16 = 10;
    const DEFAULT_API_VERSION: i16 = 10;

    type Response = KfFetchResponse<R>;
}