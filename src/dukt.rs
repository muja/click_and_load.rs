extern crate duktape_sys;
use self::duktape_sys::*;
use std::ffi::CStr;
use std::ptr;

pub struct Context {
  ctx: *mut duk_context
}

impl Context {
  pub fn new() -> Result<Context, &'static str> {
    let ctx = unsafe {
      duk_create_heap(None, None, None, ptr::null_mut(), None)
    };
    if ctx.is_null() {
      Err("Could not create heap")
    } else {
      Ok(Context{ ctx: ctx })
    }
  }

  pub fn clicknload(&self, source: &str) -> Result<String, &'static str> {
    unsafe {
      duk_push_string(self.ctx, source.replace("\n", " ").as_ptr() as *const i8);
      duk_push_string(self.ctx, "<eval>".as_ptr() as *const i8);
      duk_eval_raw(self.ctx, 0 as *const i8, 0, DUK_COMPILE_FUNCTION);
      let ds = duk_get_string(self.ctx, 0);
      if ds == 0 as *const i8 {
        Err("Provided function did not return a string")
      } else {
        String::from_utf8(CStr::from_ptr(ds).to_bytes().to_vec()).or(
          Err("Function return a non-utf8 string")
        )
      }
    }
  }
}

impl Drop for Context {
  fn drop(&mut self) {
    unsafe { duk_destroy_heap(self.ctx); }
  }
}
