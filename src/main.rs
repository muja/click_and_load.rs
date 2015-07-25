// #![feature(ip, ip_addr)]
extern crate iron;
extern crate crypto;
extern crate router;
extern crate hyper;
extern crate urlencoded;
extern crate rustc_serialize;

mod dukt;

// use std::net::IpAddr;
use crypto::{buffer, aes, blockmodes};
use crypto::buffer::{ ReadBuffer, WriteBuffer, BufferResult };
use crypto::symmetriccipher::Decryptor;
use iron::prelude::*;
use iron::status;
use router::Router;
use hyper::mime::{Mime, TopLevel, SubLevel};
use urlencoded::UrlEncodedBody;
use rustc_serialize::hex::*;
use rustc_serialize::base64::*;

const CROSS_DOMAIN: &'static str = "<?xml version=\"1.0\"?>
<!DOCTYPE cross-domain-policy SYSTEM \"http://www.macromedia.com/xml/dtds/cross-domain-policy.dtd\">
<cross-domain-policy>
<allow-access-from domain=\"*\" />
</cross-domain-policy>
";

fn decrypt(key: &Vec<u8>, crypted: &Vec<u8>) -> Result<Vec<String>, &'static str> {
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
            Ok(BufferResult::BufferUnderflow) => {
                return final_result.split(|&b| b == b'\n').map(|slice| {
                    std::str::from_utf8(slice).map(|s| s.to_string())
                }).collect::<Result<Vec<String>, _>>().or(
                    Err("Decrypted content yields non-utf8 string")
                )
            },
            Ok(_) => {}
            Err(_) => return Err("Decryption failed")
        }
    }
}

fn main() {
    let mut router = Router::new();
    router.get("/flash", |_: &mut Request| {
        Ok(Response::with((status::Ok, "UGET")))
    });
    router.get("/jdcheck.js", |_: &mut Request| {
        Ok(Response::with((status::Ok, "jdownloader = true;")).set(
            Mime(TopLevel::Text, SubLevel::Javascript, vec![])
        ))
    });
    router.get("/crossdomain.xml", |_: &mut Request| {
        Ok(Response::with((status::Ok, CROSS_DOMAIN)).set(
            Mime(TopLevel::Text, SubLevel::Html, vec![])
        ))
    });

    router.post("/flash/addcrypted2", |req: &mut Request| {
        req.get_ref::<UrlEncodedBody>().or_else(|_|
            Err("Failed to decode body")
        ).and_then(|ref hashmap| {
            hashmap.get("crypted").ok_or(
                "`crypted` parameter wasn't provided in request body"
            ).and_then(|crypted| {
                crypted[0].from_base64().or(Err("Invalid base64 string"))
            }).and_then(|crypted_bytes| {
                hashmap.get("jk").ok_or(
                    "`jk` parameter wasn't provided in request body"
                ).and_then(|jk| {
                    dukt::Context::new().and_then(|ctx| {
                        ctx.clicknload(&jk[0])
                    })
                }).and_then(|val| {
                    val.from_hex().or(Err("Invalid hex string."))
                }).and_then(|key| decrypt(&key, &crypted_bytes))
            }).and_then( |links| {
                for link in links {
                    println!("{}", link);
                }
                Ok(Response::with((status::Ok, "success\r\n")))
            })
        }).or_else(|err: &str| {
            Ok(Response::with((status::BadRequest, err)))
        })
    });

    Iron::new(router).http("localhost:9666").unwrap();
}
