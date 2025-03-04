use crate::internal::*;

pub struct Decoder<T> {
    pub input: BufReader<T>,
}

unsafe impl<T> Send for Decoder<T> {}
unsafe impl<T> Sync for Decoder<T> {}

impl<T: AsyncRead + Unpin + Send> Decoder<T> {
    pub fn new(x: T) -> Self {
        Decoder {
            input: BufReader::new(x),
        }
    }

    pub fn get_ref(&self) -> &T {
        self.input.get_ref()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.input.get_mut()
    }

    pub fn into_inner(self) -> T {
        self.input.into_inner()
    }

    fn skip(&mut self, n: usize) {
        AsyncBufRead::consume(std::pin::Pin::new(&mut self.input), n);
    }

    fn skip_crlf(&mut self) {
        self.skip(2)
    }

    async fn get_line(&mut self) -> Result<String, Error> {
        let mut dest = String::new();
        self.input.read_line(&mut dest).await?;
        let len = dest.len();
        let bytes = dest.as_bytes();
        if bytes[len - 2] != b'\r' {
            return Err(Error::InvalidByte(Some(bytes[len - 2])));
        }
        dest.truncate(len - 2);
        Ok(dest)
    }

    async fn get_number<F: std::str::FromStr>(&mut self) -> Result<F, Error>
    where
        Error: From<F::Err>,
    {
        let dest = self.get_line().await?;
        Ok(dest.parse()?)
    }

    pub async fn read_simple_string(&mut self) -> Result<Value, Error> {
        let dest = self.get_line().await?;
        Ok(Value::String(dest))
    }

    pub async fn read_simple_error(&mut self) -> Result<Value, Error> {
        let dest = self.get_line().await?;
        Ok(Value::Error(dest))
    }

    pub async fn read_blob_string(&mut self) -> Result<Value, Error> {
        let len = self.get_number::<usize>().await?;
        if len == 0 {
            self.skip_crlf();
            return Ok(Value::String(String::new()));
        }

        let mut dest = vec![0; len];
        self.input.read_exact(&mut dest).await?;
        self.skip_crlf();
        match String::from_utf8(dest) {
            Ok(s) => Ok(Value::String(s)),
            Err(e) => Ok(Value::Bytes(e.into_bytes())),
        }
    }

    pub async fn read_number(&mut self) -> Result<Value, Error> {
        let n = self.get_number().await?;
        Ok(Value::Int(n))
    }

    pub async fn read_null(&mut self) -> Result<Value, Error> {
        self.skip_crlf();
        Ok(Value::Null)
    }

    pub async fn read_double(&mut self) -> Result<Value, Error> {
        let n = self.get_number().await?;
        Ok(Value::Float(n))
    }

    pub async fn read_bool(&mut self) -> Result<Value, Error> {
        let line = self.get_line().await?;
        if line.len() > 1 {
            return Err(Error::InvalidByte(Some(line.as_bytes()[1])));
        }

        Ok(Value::Bool(line.as_bytes()[0] == b't'))
    }

    pub async fn read_blob_error(&mut self) -> Result<Value, Error> {
        let len = self.get_number::<usize>().await?;
        let mut dest = vec![0; len];
        self.input.read_exact(&mut dest).await?;
        self.skip_crlf();
        match String::from_utf8(dest) {
            Ok(s) => Ok(Value::Error(s)),
            Err(e) => Ok(Value::Error(
                String::from_utf8_lossy(e.as_bytes()).to_string(),
            )),
        }
    }

    pub async fn read_verbatim_string(&mut self) -> Result<Value, Error> {
        let len = self.get_number::<usize>().await?;

        // TODO: do something with the string type, here we are skipping it
        self.skip(4);

        let mut dest = vec![0; len];
        self.input.read_exact(&mut dest).await?;
        self.skip_crlf();
        match String::from_utf8(dest) {
            Ok(s) => Ok(Value::String(s)),
            Err(e) => Ok(Value::Bytes(e.into_bytes())),
        }
    }

    pub async fn read_big_number(&mut self) -> Result<Value, Error> {
        let line = self.get_line().await?;
        Ok(Value::BigNumber(line))
    }

    pub async fn read_array(&mut self) -> Result<Value, Error> {
        let len = self.get_number::<usize>().await?;

        let mut arr = Vec::with_capacity(len);

        for _ in 0..len {
            let value = self.decode().await?;
            arr.push(value);
        }

        Ok(Value::Array(arr))
    }

    pub async fn read_map(&mut self) -> Result<Value, Error> {
        let len = self.get_number::<usize>().await?;

        let mut map = Map::new();

        for _ in 0..len {
            let key = self.decode().await?;
            let value = self.decode().await?;
            map.insert(key, value);
        }

        Ok(Value::Map(map))
    }

    // Ignore attributes and read next value
    async fn skip_attribute(&mut self) -> Result<Value, Error> {
        let len = self.get_number::<usize>().await?;

        let mut map = Map::new();

        for _ in 0..len {
            let key = self.decode().await?;
            let value = self.decode().await?;
            map.insert(key, value);
        }

        self.decode().await
    }

    pub async fn read_set(&mut self) -> Result<Value, Error> {
        let len = self.get_number::<usize>().await?;

        let mut set = Set::new();

        for _ in 0..len {
            let value = self.decode().await?;
            set.insert(value);
        }

        Ok(Value::Set(set))
    }

    pub async fn read_push(&mut self) -> Result<Value, Error> {
        let len = self.get_number::<usize>().await?;
        let kind = match self.decode().await? {
            Value::String(s) => s,
            _ => return Err(Error::InvalidByte(None)),
        };

        let mut arr = Vec::with_capacity(len - 1);

        for _ in 0..len - 1 {
            let value = self.decode().await?;
            arr.push(value);
        }

        Ok(Value::Push(kind, arr))
    }

    pub async fn next_prefix(&mut self) -> Result<u8, Error> {
        let prefix = &mut [0u8];
        self.input.read_exact(prefix).await?;
        Ok(prefix[0])
    }

    #[async_recursion]
    pub async fn decode(&mut self) -> Result<Value, Error> {
        let prefix = self.next_prefix().await?;
        match prefix {
            b'+' => self.read_simple_string().await,
            b'$' => self.read_blob_string().await,
            b'-' => self.read_simple_error().await,
            b':' => self.read_number().await,
            b'_' => self.read_null().await,
            b',' => self.read_double().await,
            b'#' => self.read_bool().await,
            b'!' => self.read_blob_error().await,
            b'=' => self.read_verbatim_string().await,
            b'(' => self.read_big_number().await,
            b'*' => self.read_array().await,
            b'%' => self.read_map().await,
            b'~' => self.read_set().await,
            b'|' => self.skip_attribute().await,
            b'>' => self.read_push().await,
            _ => Err(Error::InvalidByte(None)),
        }
    }
}
