use std::time::SystemTime;
use chrono::{offset::Local, DateTime};
use data_encoding::HEXLOWER;
use ring::digest;

pub mod fs;

pub fn checksum(bytes: &Vec<u8>) -> String {
    let actual = digest::digest(&digest::SHA256, bytes);
    HEXLOWER.encode(actual.as_ref())
}

pub fn get_human_dt(time: SystemTime) -> String {
    let datetime: DateTime<Local> = DateTime::from(time);
    format!("{}", datetime)
}
