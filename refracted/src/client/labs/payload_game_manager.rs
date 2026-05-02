use crate::blaze::tdf::TdfEncoder;
use crate::common::error::BlazeResult;
use bytes::Bytes;

pub fn handle_game_manager_command_3(_payload: &[u8]) -> BlazeResult<Bytes> {
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_string("HOST", "203.129.23.162"));
    response.extend_from_slice(&TdfEncoder::encode_int("PORT", 65535));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-sym"));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-sin"));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-hkg"));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-sjc"));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-pdx"));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-icn"));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-nrt"));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-iad"));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-cmh"));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-fra"));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-lhr"));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-dub"));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-bah"));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-brz"));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-cpt"));
    Ok(Bytes::from(response))
}

pub fn handle_game_manager_command_5(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_game_manager_command_7(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_game_manager_command_10(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_game_manager_return_dedicated_server_to_pool(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_game_manager_command_16(_payload: &[u8]) -> BlazeResult<Bytes> {
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_long("GID ", 52136290991));
    response.extend_from_slice(&TdfEncoder::encode_int("JGS ", 0));
    response.extend_from_slice(&TdfEncoder::encode_int("OCAL", 0));
    Ok(Bytes::from(response))
}

pub fn handle_game_manager_command_113(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}
