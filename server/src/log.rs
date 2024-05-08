use rocket::tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter, Result},
    sync::RwLock,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct Log {
    file: Arc<RwLock<BufWriter<File>>>,
}

impl Log {
    pub fn new() -> Result<Self> {
        let file = std::fs::File::create("log.txt")?;
        let file: rocket::tokio::fs::File = file.into();
        let file = BufWriter::new(file);
        Ok(Log {
            file: Arc::new(RwLock::new(file)),
        })
    }

    pub async fn write<T: AsRef<[u8]>>(&self, msg: T) -> Result<()> {
        let mut file = self.file.write().await;
        file.write(msg.as_ref()).await?;
        file.write(b"\n").await?;
        Ok(())
    }

    pub async fn flush(&self) -> Result<()> {
        let mut file = self.file.write().await;
        file.flush().await
    }
}

impl Drop for Log {
    fn drop(&mut self) {
        let (h, _) = crate::get_runtime_handle();
        let file = self.file.clone();
        let _ = h.spawn(async move {
            let mut file = file.write().await;
            let _ = file.flush().await;
        });
    }
}

// #[async_trait]
// impl Fairing for Log {
//   fn info(&self) -> rocket::fairing::Info {
//       Info {
//         name: "Log",
//         kind: rocket::fairing::Kind::Singleton,
//       }
//   }

//   async fn on_response<'r>(&self, req: &'r rocket::Request<'_>, res: &mut rocket::Response<'r>) {
//     let path = req.uri().path();
//     let status = res.status().as_str();
//     let file = self.file.read().await;
//     writeln!(file, "{}: {}", path, status).unwrap();
//   }
// }
