use iron::prelude::*;
use iron::status;
use iron::Handler;
use crypto::{buffer, aes, blockmodes};
use crypto::buffer::{ReadBuffer, WriteBuffer, BufferResult};
use crypto::symmetriccipher::SymmetricCipherError;
use dukt::Context;
use urlencoded::UrlEncodedBody;
use rustc_serialize::hex::*;
use rustc_serialize::base64::*;
use std::str;
use std::sync::mpsc::Sender;
use std::sync::Mutex;

pub struct Loader {
    pub sender: Mutex<Sender<Vec<String>>>,
}

impl Loader {
    pub fn key_from_snippet(jk: &str) -> Result<String, &'static str> {
        Context::new().and_then(|ctx| ctx.click_and_load(jk))
    }

    pub fn decrypt(key: &Vec<u8>, crypted: &Vec<u8>) -> Result<Vec<u8>, SymmetricCipherError> {
        debug!("key: {:?}", key);
        debug!("crypted: {:?}", crypted);
        let key = &key[0..16]; // trim to 128 bytes
        let mut decryptor =
            aes::cbc_decryptor(aes::KeySize::KeySize128, key, key, blockmodes::PkcsPadding);
        let mut final_result = Vec::<u8>::new();
        let mut read_buffer = buffer::RefReadBuffer::new(crypted);
        let mut buffer = [0; 4096];
        let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

        loop {
            let result = try!(decryptor.decrypt(&mut read_buffer, &mut write_buffer, true));
            final_result.extend(write_buffer.take_read_buffer()
                .take_remaining()
                .iter()
                .map(|&i| i));
            match result {
                BufferResult::BufferUnderflow => {
                    while final_result.last() == Some(&0) {
                        final_result.pop();
                    }
                    return Ok(final_result);
                }
                _ => {}
            }
        }
    }

    pub fn encrypt(bytes: &Vec<u8>) -> Result<Vec<u8>, SymmetricCipherError> {
        let key = b"434e4c2e72732062792044616e79656c";
        let mut encryptor =
            aes::cbc_encryptor(aes::KeySize::KeySize128, key, key, blockmodes::PkcsPadding);
        let mut final_result = Vec::<u8>::new();
        let mut read_buffer = buffer::RefReadBuffer::new(bytes);
        let mut buffer = [0; 4096];
        let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

        loop {
            let result = try!(encryptor.encrypt(&mut read_buffer, &mut write_buffer, true));
            final_result.extend(write_buffer.take_read_buffer()
                .take_remaining()
                .iter()
                .map(|&i| i));
            match result {
                BufferResult::BufferUnderflow => return Ok(final_result),
                _ => {}
            }
        }
    }
}

impl Handler for Loader {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        req.get_ref::<UrlEncodedBody>()
            .or(Err("Failed to decode body"))
            .and_then(|ref hashmap| {
                hashmap.get("crypted")
                    .ok_or("`crypted` parameter wasn't provided in request body")
                    .and_then(|crypted| crypted[0].from_base64().or(Err("Invalid base64 string")))
                    .and_then(|crypted| {
                        hashmap.get("jk")
                            .ok_or("`jk` parameter wasn't provided in request body")
                            .and_then(|jk| {
                                debug!("jk: {:?}", jk);
                                Loader::key_from_snippet(&jk[0])
                            })
                            .and_then(|val| {
                                debug!("value: {:?}", val);
                                val.from_hex().or(Err("Invalid hex string."))
                            })
                            .and_then(|key| {
                                Loader::decrypt(&key, &crypted).or(Err("Decryption failed"))
                            })
                    })
                    .and_then(|bytes| {
                        info!("bytes: {:?}", bytes);
                        bytes.split(|&b| b == b'\n' || b == b'\r')
                            .filter(|&xs| xs.len() > 0)
                            .map(|slice| str::from_utf8(slice).map(|s| s.into()))
                            .collect::<Result<Vec<String>, _>>()
                            .or(Err("Decrypted content yields non-utf8 string"))
                            .and_then(|links| {
                                self.sender
                                    .lock()
                                    .and_then(|handle| {
                                        if let Err(e) = handle.send(links) {
                                            warn!("Error sending to receiver {:?}", e);
                                            Ok(Response::with((status::InternalServerError,
                                                               "Internal server error")))
                                        } else {
                                            Ok(Response::with((status::Ok, "success\r\n")))
                                        }
                                    })
                                    .or_else(|err| {
                                        warn!("Error acquiring lock: {:?}", err);
                                        Ok(Response::with((status::InternalServerError,
                                                           "Internal server error")))
                                    })
                            })
                    })
            })
            .or_else(|err| {
                warn!("Error processing request: {}", err);
                Ok(Response::with((status::BadRequest, err)))
            })
    }
}
