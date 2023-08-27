#[cfg(test)]
mod test {
    use std::{
        fs::File,
        io::Write,
        path::{Path, PathBuf},
    };

    use movies_db::ffmpeg::FFMpeg;
    use tempdir::TempDir;

    fn write_file_to_temp_dir(temp_dir: &TempDir, file_name: &str, data: &[u8]) {
        let mut file_path: PathBuf = temp_dir.path().to_owned();
        file_path.push(file_name);
        File::create(&file_path).unwrap().write_all(data).unwrap();
    }

    #[tokio::test]
    async fn test_ffmpeg_init() {
        let temp_dir = TempDir::new("test_ffmpeg_version").unwrap();

        // test only works if ffmpeg and ffprobe are located in /usr/bin
        FFMpeg::new(&Path::new("/usr/bin")).await.unwrap();
    }

    #[tokio::test]
    async fn test_ffmpeg_duration() {
        let temp_dir = TempDir::new("test_ffmpeg_version").unwrap();

        // copy mp4 test file into temporary directory
        let mp4_data = include_bytes!("data/file_example_MP4_480_1_5MG.mp4");

        write_file_to_temp_dir(&temp_dir, "movie.mp4", mp4_data);

        // test only works if ffmpeg and ffprobe are located in /usr/bin
        let ffmpeg = FFMpeg::new(&Path::new("/usr/bin")).await.unwrap();
        let duration = ffmpeg
            .get_movie_duration(&temp_dir.path().join("movie.mp4"))
            .await
            .unwrap();

        assert_eq!(duration as u32, 30);
    }

    #[tokio::test]
    async fn test_ffmpeg_screenshot() {
        let temp_dir = TempDir::new("test_ffmpeg_version").unwrap();

        // copy mp4 test file into temporary directory
        let mp4_data = include_bytes!("data/file_example_MP4_480_1_5MG.mp4");

        write_file_to_temp_dir(&temp_dir, "movie.mp4", mp4_data);

        // test only works if ffmpeg and ffprobe are located in /usr/bin
        let ffmpeg = FFMpeg::new(&Path::new("/usr/bin")).await.unwrap();
        let screenshot = ffmpeg
            .create_screenshot(&temp_dir.path().join("movie.mp4"), 15f64)
            .await
            .unwrap();

        println!("{}", screenshot.len());
        println!("{:?}", String::from_utf8_lossy(&screenshot[..20]));
    }
}
