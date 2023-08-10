pub mod downloader {
    use anyhow::Result;
    use std::num::{NonZeroU8, NonZeroUsize};
    use std::path::PathBuf;
    use url::Url;

    use http_downloader::{
        speed_tracker::DownloadSpeedTrackerExtension,
        status_tracker::DownloadStatusTrackerExtension, HttpDownloaderBuilder,
    };

    pub async fn download(path: &str, url: &str) -> Result<()> {
        let save_dir = PathBuf::from(path);
        let test_url = Url::parse(url)?;
        let (mut downloader, (status_state, speed_state)) =
            HttpDownloaderBuilder::new(test_url, save_dir)
                .chunk_size(NonZeroUsize::new(1024 * 1024 * 10).unwrap()) // 块大小
                .download_connection_count(NonZeroU8::new(3).unwrap())
                .build((
                    // 下载状态追踪扩展
                    // by cargo feature "status-tracker" enable
                    DownloadStatusTrackerExtension { log: true },
                    // 下载速度追踪扩展
                    // by cargo feature "speed-tracker" enable
                    DownloadSpeedTrackerExtension { log: true },
                ));
        let download_future = downloader.prepare_download()?;

        let _status = status_state.status(); // get download status， 获取状态
        let _status_receiver = status_state.status_receiver; //status watcher，状态监听器
        let _byte_per_second = speed_state.download_speed(); // get download speed，Byte per second，获取速度，字节每秒
        let _speed_receiver = speed_state.receiver; // get download speed watcher，速度监听器

        // downloader.cancel() // 取消下载

        let _ = download_future.await?;
        Ok(())
    }
}
