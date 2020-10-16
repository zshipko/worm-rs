use crate::internal::*;

pub struct Client {
    addrs: Vec<std::net::SocketAddr>,
    auth: Option<(String, String)>,
    output: Encoder<tokio::io::WriteHalf<tokio::net::TcpStream>>,
    input: Decoder<tokio::io::ReadHalf<tokio::net::TcpStream>>,
}

impl Client {
    pub(crate) async fn new_from_stream(stream: tokio::net::TcpStream, addrs: Vec<std::net::SocketAddr>, auth: Option<(&str, &str)>) -> Result<Client, Error> {
        let (r, w) = tokio::io::split(stream);
        let output = Encoder::new(w);
        let input = Decoder::new(r);

        let client = Client {
            addrs,
            output,
            input,
            auth: auth.map(|(a, b)| (a.into(), b.into())),
        };


        Ok(client)

    }

    pub async fn new<T: tokio::net::ToSocketAddrs>(x: T, auth: Option<(&str, &str)>) -> Result<Client, Error> {
        let addrs = tokio::net::lookup_host(x).await?.collect::<Vec<_>>();
        let mut client = Self::new_from_stream(tokio::net::TcpStream::connect(addrs.as_slice()).await?, addrs, auth).await?;

        let cmd = Command::new("HELLO").arg("3");

        let cmd = if let Some((user, pass)) = &client.auth {
            cmd.arg("AUTH").arg(user).arg(pass)
        } else {
            cmd
        };

        // TODO: do something with the HELLO response
        let info = client.exec(&cmd.into()).await?;
        if info.as_map().is_none() {
            return Err(Error::InvalidValue(info));
        }

        Ok(client)
    }

    pub fn addrs(&self) -> &[std::net::SocketAddr] {
        &self.addrs
    }

    pub async fn read(&mut self) -> Result<Value, Error> {
        self.input.decode().await
    }

    pub async fn write(&mut self, value: &Value) -> Result<(), Error> {
        self.output.encode(value).await
    }

    pub async fn flush(&mut self) -> Result<(), Error> {
        self.output.flush().await
    }

    pub async fn exec(&mut self, value: &Value) -> Result<Value, Error> {
        self.write(value).await?;
        self.flush().await?;
        self.read().await
    }

    pub async fn command(&mut self, args: impl AsRef<[&str]>) -> Result<Value, Error> {
        let args = args.as_ref().iter().map(|x| Value::from(*x)).collect::<Vec<_>>();
        self.exec(&Value::Array(args)).await
    }
}
