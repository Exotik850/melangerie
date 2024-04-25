use rocket::tokio::{fs::File, io::{AsyncWriteExt, BufWriter, Result}, sync::RwLock};

pub struct Log {
    file: RwLock<BufWriter<File>>,
}

impl Log {
    pub fn new() -> Result<Self> {
        let file = std::fs::File::create("log.txt")?;
        let file: rocket::tokio::fs::File = file.into();
        let file = BufWriter::new(file);
        Ok(Log {
            file: RwLock::new(file),
        })
    }

    pub async fn write<T: AsRef<[u8]>>(&self, msg: T) -> Result<()> {
        let mut file = self.file.write().await;
        file.write(msg.as_ref()).await?;
        Ok(())
    }

    pub async fn flush(&self) -> Result<()> {
        let mut file = self.file.write().await;
        file.flush().await
    }
}

use rocket::tokio::runtime::{Runtime, Handle};

fn get_runtime_handle() -> (Handle, Option<Runtime>) {
    match Handle::try_current() {
        Ok(h) => (h, None),
        Err(_) => {
              let rt = Runtime::new().expect("Failed to create runtime");
              (rt.handle().clone(), Some(rt))
            }
    }
}

impl Drop for Log {
    fn drop(&mut self) {
      let (_, rt) = get_runtime_handle();
      let Some(rt) = rt else {
        return;
      };
      let _ = rt.block_on(self.flush());
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
