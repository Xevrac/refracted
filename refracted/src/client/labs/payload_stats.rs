use crate::common::error::BlazeResult;
use bytes::Bytes;

pub fn handle_stats_command_0(payload: &[u8]) -> BlazeResult<Bytes> {
    if payload.len() >= 1 {
        Ok(Bytes::from(vec![payload[0]]))
    } else {
        Ok(Bytes::from(vec![0x09]))
    }
}

pub fn handle_stats_command_3840(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_stats_command_10496(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_stats_command_14080(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_stats_command_16640(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_stats_command_20224(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_stats_command_22784(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_stats_command_28928(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}
