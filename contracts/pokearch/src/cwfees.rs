// https://github.com/archway-network/cwfees/blob/fd/init/crates/cwfees/src/archway.cwfees.v1.rs

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin};
use prost::DecodeError;

/// MsgRegisterAsGranter is used to register a a granter.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterAsGranter {
    #[prost(string, tag = "1")]
    pub granting_contract: ::prost::alloc::string::String,
}

/// It's the message you have to use in your sudo entrypoint,
/// the x/cwfees module sends these message as a sudo call to
/// your contract. Based on that information the contract
/// can decide if to accept the request, so return Ok, or
/// decline and return an error.
#[cw_serde]
#[non_exhaustive]
pub enum SudoMsg {
    CwGrant(CwGrant),
}

/// CwGrant is the only variant of the SudoMsg enum.
#[cw_serde]
pub struct CwGrant {
    /// Defines the amount of fees being requested for the execution of this tx.
    pub fee_requested: Vec<Coin>,
    /// Msgs contains the list of messages intended to be processed in this tx.
    pub msgs: Vec<Msg>,
}

/// Msg defines information about the tx messages.
/// It implements TryInto
#[cw_serde]
pub struct Msg {
    /// Defines the sender of the message, this is populated using the sdk.Msg.GetSigner()
    /// by the state machine. It can be trusted.
    pub sender: String,
    /// Defines the type_url of the message being sent, eg: /cosmos.bank.v1beta1.MsgSend.
    /// This can be used to decode the message to a specific type.
    pub type_url: String,
    /// Defines the binary representation of the message.
    pub msg: Binary,
}
impl Msg {
    /// Allows to convert Msg into a prost message. Note: all cosmos-sdk messages
    /// are prost messages.
    pub fn try_into_proto<T: prost::Message + Default>(self) -> Result<T, DecodeError> {
        T::decode(&*self.msg.0) //
    }
}

#[cfg(test)]
mod test {
    use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;
    use cosmwasm_std::Binary;
    use prost::Message;

    use crate::cwfees::Msg;

    #[test]
    fn msg_from_protobuf_message() {
        let sdk_msg = MsgSend {
            from_address: "Kim Dokja".to_string(),
            to_address: "Yoo Joonghyuk".to_string(),
            amount: vec![],
        };
        let encoded = sdk_msg.encode_to_vec();

        let msg = Msg {
            sender: "".to_string(),
            type_url: "".to_string(),
            msg: Binary::from(encoded),
        };

        let got_sdk_msg: MsgSend = msg.try_into_proto().unwrap();
        assert_eq!(sdk_msg, got_sdk_msg)
    }
}