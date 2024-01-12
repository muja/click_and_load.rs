use boa_engine::JsResult;

pub type Result<T> = JsResult<T>;

pub struct Context {
    ctx: boa_engine::Context,
}

impl Context {
    pub fn new() -> Self {
        Self { ctx: Default::default() }
    }

    pub fn click_and_load(&mut self, source: &str) -> JsResult<String> {
        self.ctx.eval(source)?;
        let s = self.ctx.eval("f()")?;
        Ok(s.as_string().unwrap().to_string())
    }
}
