use iron::prelude::*;
use iron::status;
use crypto::{buffer, aes, blockmodes};
use crypto::buffer::{ ReadBuffer, WriteBuffer, BufferResult };
use crypto::symmetriccipher::Decryptor;
use dukt::Context;
use urlencoded::UrlEncodedBody;
use rustc_serialize::hex::*;
use rustc_serialize::base64::*;
use std::str;

pub struct Loader;

impl Loader {
    pub fn key_from_snippet(jk: &str) -> Result<String, &'static str> {
        Context::new().and_then(|ctx| ctx.click_and_load(jk))
    }

    pub fn decrypt(key: &Vec<u8>, crypted: &Vec<u8>) -> Result<Vec<u8>, &'static str> {
        let mut decryptor = aes::cbc_decryptor(
            aes::KeySize::KeySize128,
            key,
            key,
            blockmodes::PkcsPadding
        );
        let mut final_result = Vec::<u8>::new();
        let mut read_buffer = buffer::RefReadBuffer::new(crypted);
        let mut buffer = [0; 4096];
        let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

        loop {
            let result = decryptor.decrypt(&mut read_buffer, &mut write_buffer, true);
            final_result.extend(
                write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i)
            );
            match result {
                Ok(BufferResult::BufferUnderflow) => return Ok(final_result),
                Ok(_) => {}
                Err(_) => return Err("Decryption failed")
            }
        }
    }

    pub fn click_and_load(req: &mut Request) -> IronResult<Response> {
        req.get_ref::<UrlEncodedBody>().or(Err("Failed to decode body")).and_then(|ref hashmap| {
            hashmap.get("crypted").ok_or(
                "`crypted` parameter wasn't provided in request body"
            ).and_then(|crypted| {
                crypted[0].from_base64().or(Err("Invalid base64 string"))
            }).and_then(|crypted_bytes| {
                hashmap.get("jk").ok_or(
                    "`jk` parameter wasn't provided in request body"
                ).and_then(|jk| {
                    Loader::key_from_snippet(&jk[0])
                }).and_then(|val| {
                    val.from_hex().or(Err("Invalid hex string."))
                }).and_then(|key| Loader::decrypt(&key, &crypted_bytes))
            }).and_then( |bytes| {
                bytes.split(|&b| b == b'\n').map(|slice| {
                    str::from_utf8(slice).map(|s| s.into())
                }).collect::<Result<Vec<String>, _>>().or(
                    Err("Decrypted content yields non-utf8 string")
                ).and_then(|links| {
                    for link in links {
                        println!("{}", link);
                    }
                    Ok(Response::with((status::Ok, "success\r\n")))
                })
            })
        }).or_else( |err| Ok(Response::with((status::BadRequest, err))) )
    }
}
