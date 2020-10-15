use crate::*;

#[test]
fn test_roundtrip() -> Result<(), Error> {
    smol::block_on(async {
        let ex = array![
            map! {
                1i64 => "abc",
                "test" => true,
            },
            array![1, 2, 3]
        ];

        let mut buffer = Vec::new();

        ex.write(&mut buffer).await?;
        println!("{}", String::from_utf8_lossy(buffer.as_ref()));

        let value = Value::read(&mut buffer.as_slice()).await?;
        assert_eq!(ex, value);
        Ok(())
    })
}
