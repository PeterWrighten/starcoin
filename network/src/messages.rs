// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::helper::get_unix_ts;
use actix::prelude::*;
use anyhow::Result;
use crypto::{hash::CryptoHash, HashValue};
use parity_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use types::account_address::AccountAddress;
use types::block::Block;
use types::transaction::SignedUserTransaction;

pub trait RPCMessage {
    fn get_id(&self) -> HashValue;
}

#[derive(Message)]
#[rtype(result = "u64")]
pub struct GetCounterMessage {}

/// message from peer
#[rtype(result = "Result<()>")]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Message)]
pub enum PeerMessage {
    UserTransaction(SignedUserTransaction),
    Block(Block),
    RPCRequest(RPCRequest),
    RPCResponse(RPCResponse),
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Message, Clone)]
pub struct TestRequest {
    pub data: HashValue,
}

/// message from peer
#[rtype(result = "Result<()>")]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Message, Clone)]
pub enum RPCRequest {
    TestRequest(TestRequest),
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Message, Clone)]
pub struct RpcRequestMessage {
    pub request: RPCRequest,
    pub peer_id: AccountAddress,
}

impl RPCMessage for RPCRequest {
    fn get_id(&self) -> HashValue {
        return match self {
            RPCRequest::TestRequest(request) => request.data,
        };
    }
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Message, Clone)]
pub struct TestResponse {
    pub len: u8,
    pub id: HashValue,
}

#[rtype(result = "Result<()>")]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Message, Clone)]
pub enum RPCResponse {
    TestResponse(TestResponse),
}

impl RPCMessage for RPCResponse {
    fn get_id(&self) -> HashValue {
        match self {
            RPCResponse::TestResponse(r) => r.id,
        }
    }
}

impl RPCResponse {
    pub fn set_request_id(&mut self, id: HashValue) {
        match self {
            RPCResponse::TestResponse(r) => r.id = id,
        };
    }
}

#[derive(Clone, Hash, Debug)]
pub struct InnerMessage {
    pub peer_id: AccountAddress,
    pub msg: Message,
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, Encode, Decode)]
pub enum Message {
    ACK(u128),
    Payload(PayloadMsg),
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, Encode, Decode)]
pub struct PayloadMsg {
    pub id: u128,
    pub data: Vec<u8>,
}

impl Message
where
    Self: Decode + Encode,
{
    pub fn into_bytes(self) -> Vec<u8> {
        self.encode()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Decode::decode(&mut &bytes[..]).ok_or(())
    }
}

impl Message {
    pub fn new_ack(message_id: u128) -> Message {
        Message::ACK(message_id)
    }

    pub fn new_payload(data: Vec<u8>) -> (Message, u128) {
        let message_id = get_unix_ts();
        (
            Message::Payload(PayloadMsg {
                id: message_id,
                data,
            }),
            message_id,
        )
    }
    pub fn new_message(data: Vec<u8>) -> Message {
        Message::Payload(PayloadMsg { id: 0, data })
    }

    pub fn as_payload(self) -> Option<Vec<u8>> {
        match self {
            Message::Payload(p) => Some(p.data),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NetworkMessage {
    pub peer_id: AccountAddress,
    pub data: Vec<u8>,
}
