use crate::internal::*;

pub struct Encoder<T> {
    pub output: BufWriter<T>,
}

unsafe impl<T> Send for Encoder<T> {}
unsafe impl<T> Sync for Encoder<T> {}

impl<T: Unpin + Send + AsyncWrite> Encoder<T> {
    pub fn new(x: T) -> Self {
        Encoder {
            output: BufWriter::new(x),
        }
    }

    pub fn get_ref(&self) -> &T {
        self.output.get_ref()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.output.get_mut()
    }

    pub fn into_inner(self) -> T {
        self.output.into_inner()
    }

    async fn write_crlf(&mut self) -> Result<(), Error> {
        self.output.write_all(b"\r\n").await?;
        Ok(())
    }

    async fn write_length(&mut self, prefix: char, len: usize) -> Result<(), Error> {
        self.output
            .write_all(format!("{}{}\r\n", prefix, len).as_bytes())
            .await?;
        Ok(())
    }

    pub async fn write_null(&mut self) -> Result<(), Error> {
        self.output.write_all(b"_").await?;
        self.write_crlf().await
    }

    pub async fn write_bool(&mut self, b: &bool) -> Result<(), Error> {
        if *b {
            self.output.write_all(b"#t").await?;
        } else {
            self.output.write_all(b"#f").await?;
        }
        self.write_crlf().await
    }

    pub async fn write_int(&mut self, i: &i64) -> Result<(), Error> {
        self.output.write_all(b":").await?;
        self.output.write_all(i.to_string().as_bytes()).await?;
        self.write_crlf().await
    }

    pub async fn write_float(&mut self, i: &Float) -> Result<(), Error> {
        self.output.write_all(b",").await?;
        self.output.write_all(i.to_string().as_bytes()).await?;
        self.write_crlf().await
    }

    pub async fn write_big_number(&mut self, i: &str) -> Result<(), Error> {
        self.output.write_all(b"(").await?;
        self.output.write_all(i.as_bytes()).await?;
        self.write_crlf().await
    }

    pub async fn write_error(&mut self, e: &str) -> Result<(), Error> {
        if e.contains('\r') || e.contains('\n') {
            self.output
                .write_all(format!("!{}", e.len()).as_bytes())
                .await?;
            self.write_crlf().await?;
        } else {
            self.output.write_all(b"-").await?;
        }
        self.output.write_all(e.as_bytes()).await?;
        self.write_crlf().await
    }

    pub async fn write_string_value(&mut self, e: impl AsRef<str>) -> Result<(), Error> {
        self.write_string(e.as_ref().as_bytes()).await
    }

    pub async fn write_bytes_value(&mut self, e: impl AsRef<[u8]>) -> Result<(), Error> {
        self.write_string(e.as_ref()).await
    }

    pub async fn write_string(&mut self, e: &[u8]) -> Result<(), Error> {
        if e.contains(&b'\r') || e.contains(&b'\n') {
            self.write_length('$', e.len()).await?;
        } else {
            self.output.write_all(b"+").await?;
        }
        self.output.write_all(e).await?;
        self.write_crlf().await
    }

    pub async fn write_array_header(&mut self, n: usize) -> Result<(), Error> {
        self.write_length('*', n).await?;
        Ok(())
    }

    pub async fn write_array(&mut self, arr: &[Value]) -> Result<(), Error> {
        self.write_array_header(arr.len()).await?;

        for a in arr {
            self.encode(a).await?;
        }

        Ok(())
    }

    pub async fn write_map_header(&mut self, n: usize) -> Result<(), Error> {
        self.write_length('%', n).await?;
        Ok(())
    }

    pub async fn write_map(&mut self, map: &Map) -> Result<(), Error> {
        self.write_map_header(map.len()).await?;

        for (k, v) in map.iter() {
            self.encode(k).await?;
            self.encode(v).await?;
        }

        Ok(())
    }

    pub async fn write_set_header(&mut self, n: usize) -> Result<(), Error> {
        self.write_length('~', n).await?;
        Ok(())
    }

    pub async fn write_set(&mut self, set: &Set) -> Result<(), Error> {
        self.write_set_header(set.len()).await?;

        for a in set.iter() {
            self.encode(a).await?;
        }

        Ok(())
    }

    pub async fn write_push_header(&mut self, kind: impl AsRef<str>, len: usize) -> Result<(), Error> {
        self.write_length('>', len + 1).await?;
        self.write_string(kind.as_ref().as_bytes()).await?;
        Ok(())
    }

    pub async fn write_push(&mut self, kind: impl AsRef<str>, values: &[Value]) -> Result<(), Error> {
        self.write_push_header(kind, values.len()).await?;
        for a in values {
            self.encode(a).await?;
        }

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), Error> {
        self.output.flush().await?;
        Ok(())
    }

    #[async_recursion]
    pub async fn encode(&mut self, value: &Value) -> Result<(), Error> {
        match value {
            Value::Null => self.write_null().await,
            Value::Bool(b) => self.write_bool(b).await,
            Value::Int(i) => self.write_int(i).await,
            Value::Float(f) => self.write_float(f).await,
            Value::BigNumber(n) => self.write_big_number(n.as_str()).await,
            Value::Error(e) => self.write_error(e.as_str()).await,
            Value::String(s) => self.write_string(s.as_bytes()).await,
            Value::Bytes(s) => self.write_string(s.as_slice()).await,
            Value::Array(a) => self.write_array(a.as_slice()).await,
            Value::Map(m) => self.write_map(m).await,
            Value::Set(s) => self.write_set(s).await,
            Value::Push(name, m) => self.write_push(name, m.as_slice()).await,
        }
    }
}
