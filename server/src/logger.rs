use rocket::tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter, Result},
    sync::RwLock,
};
use std::sync::{atomic::AtomicBool, Arc};

#[derive(Clone)]
pub struct Log {
    file: Arc<RwLock<BufWriter<File>>>,
    dirty: Arc<AtomicBool>,
}

impl Log {
    pub fn new() -> Result<Self> {
        let file = std::fs::File::create("log.txt")?;
        let file: rocket::tokio::fs::File = file.into();
        let file = BufWriter::new(file);
        Ok(Log {
            file: Arc::new(RwLock::new(file)),
            dirty: Arc::new(AtomicBool::new(false)),
        })
    }

    pub async fn write<T: AsRef<[u8]>>(&self, msg: T) -> Result<()> {
        let mut file = self.file.write().await;
        let message = format!("{:?} :: ", chrono::Local::now());
        file.write(message.as_bytes()).await?;
        file.write(msg.as_ref()).await?;
        file.write(b"\n").await?;
        self.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    pub async fn flush(&self) -> Result<()> {
        if !self.dirty.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        };
        let mut file = self.file.write().await;
        let out = file.flush().await;
        if out.is_ok() {
            self.dirty
                .store(false, std::sync::atomic::Ordering::Relaxed);
        }
        out
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
