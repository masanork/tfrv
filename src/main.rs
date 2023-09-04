use clap::{App, Arg};
use encoding_rs::SHIFT_JIS;
use std::io::{self, Write};
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Telnet Client")
        .version("1.0")
        .author("Masanori Kusunoki <masanork@gmail.com>")
        .about("telnet電子公告ビューア")
        .arg(
            Arg::with_name("SERVER")
                .help("Server address")
                .default_value("koukoku.shadan.open.ad.jp"),
        )
        .arg(
            Arg::with_name("PORT")
                .help("Port number")
                .default_value("23"),
        )
        .get_matches();

    let server = matches.value_of("SERVER").unwrap();
    let port = matches.value_of("PORT").unwrap();
    let addr = format!("{}:{}", server, port);

    let stream = TcpStream::connect(&addr).await?;
    let (mut reader, mut writer) = stream.into_split();

    let read_task = tokio::spawn(async move {
        let mut buffer = vec![0u8; 4096];

        loop {
            match reader.read(&mut buffer).await {
                Ok(n) if n == 0 => return,
                Ok(n) => {
                    let (decoded, _, _) = SHIFT_JIS.decode(&buffer[..n]);
                    print!("{}", decoded);
                    io::stdout().flush().unwrap();
                }
                Err(e) => {
                    eprintln!("Read error: {}", e);
                }
            }
        }
    });

    let write_task = tokio::spawn(async move {
        let mut input_buffer = String::new();

        loop {
            input_buffer.clear();
            std::io::stdin().read_line(&mut input_buffer).expect("Failed to read from stdin");
            let encoded = SHIFT_JIS.encode(&input_buffer);
            if let Err(e) = writer.write_all(&encoded.0).await {
                eprintln!("Write error: {}", e);
            }
        }
    });

    tokio::try_join!(read_task, write_task)?;

    Ok(())
}
