use std::env::{self};
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::thread::{self, JoinHandle};

const PHOTOES_PATH: &str = "../photoes";

const VIDEOS_PATH: &str = "../videos";

const OTHERS_PATH: &str = "../others";

fn try_make_dir<P: AsRef<Path>>(path: P) {
    let making_dir = fs::create_dir(path);

    match making_dir {
        Ok(()) => return,
        Err(e) => println!("{} 27", e),
    }
}

fn main() {
    // 커맨드 라인 인자를 가져옵니다.
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let path = &args[1];
        match env::set_current_dir(path) {
            Ok(_) => {
                println!("Changed working directory to {}", path);
                // 이제 'path'에서 프로그램을 실행하도록 로직을 구현하면 됩니다.
            }
            Err(e) => {
                println!("Failed to change working directory: {}", e);
                return;
            }
        }
    }

    let video_extensions: &[&str] = &[
        "asf", "avi", "bik", "flv", "mkv", "mov", "mp4", "mpeg", "ogg", "ogv", "skm", "ts", "webm",
        "wmv",
    ];
    let image_extensions: &[&str] = &["heic", "jpeg", "png", "gif", "webp"];

    let current_dir = env::current_dir().unwrap();

    let photoes_path = PathBuf::from(PHOTOES_PATH);

    let videos_path = PathBuf::from(VIDEOS_PATH);

    let others_path = PathBuf::from(OTHERS_PATH);

    let paths = fs::read_dir(current_dir).unwrap();

    try_make_dir(&photoes_path);

    try_make_dir(&videos_path);

    try_make_dir(&others_path);

    //let mut files: Vec<DirEntry> = Vec::new();
    let mut file_names: Vec<PathBuf> = Vec::new();

    for entry in paths {
        let entry: DirEntry = entry.unwrap();
        if !entry.path().is_dir() {
            let path = entry.path();
            let file_name = path.file_name();
            let file_name = PathBuf::from(file_name.unwrap());
            file_names.push(file_name);
        }
    }

    let mut join_vec: Vec<JoinHandle<()>> = vec![];

    for file_name in file_names.clone() {
        let file_name = file_name.clone();

        //파일 확장자 추출
        //println!("{}", file_name.display());
        let extension = file_name.extension();

        let extension = match extension {
            Some(e) => e.to_str().unwrap(),
            None => continue,
        };

        //확장자에 따라 다르게 실행
        if video_extensions.contains(&extension) {
            fs::rename(&file_name, videos_path.join(&file_name)).unwrap();
        } else if extension == "heic" {
            fs::rename(&file_name, photoes_path.join(&file_name)).unwrap();
        } else if image_extensions.contains(&extension) {
            //스레드 생성
            let handle = thread::spawn(move || {
                match work(&file_name) {
                    Ok(()) => println!("Ok"),
                    Err(()) => println!("err"),
                };
            });
            join_vec.push(handle);
        }
    }
    join_vec
        .into_iter()
        .for_each(|handle| handle.join().unwrap());
}

fn work(file_name: &Path) -> Result<(), ()> {
    let size = match get_size_from_command(file_name) {
        Ok(size) => size,
        Err(()) => {
            return Err(());
            // let path_to_go = Path::new(OTHERS_PATH).join(file_name);

            // fs::rename(file_name, path_to_go).unwrap();
            // return;
        }
    };

    let new_path = distinguish_by_size(size);
    let new_path = new_path.join(file_name);
    fs::rename(file_name, new_path).unwrap();
    Ok(())
}

//사진의 비율이 카메라면 picturs로 아니면 others로
fn distinguish_by_size(size: (i32, i32)) -> PathBuf {
    let width = size.0;
    let height = size.1;

    let ratio: f32 = width as f32 / height as f32;

    if (ratio) == (4.0 / 3.0) || (ratio) == (3.0 / 4.0) {
        //println!("{} {} {}", ratio, (4.0 / 3.0), (3.0 / 4.0));
        PathBuf::from(PHOTOES_PATH)
    } else {
        //println!("{} {} {}", ratio, (4.0 / 3.0), (3.0 / 4.0));
        PathBuf::from(OTHERS_PATH)
    }
}

fn get_size_from_command(file_name: &Path) -> Result<(i32, i32), ()> {
    let child = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=p=0:s=x",
            //ffprobe -v error -select_streams v -show_entries stream=width,height -of csv=p=0:s=x
        ])
        .arg(file_name)
        .stdout(Stdio::piped())
        .output();

    let child = match child {
        Ok(c) => c,
        Err(e) => {
            println!("{e}");
            return Err(());
        }
    };

    let binding = String::from_utf8(child.stdout).unwrap();
    let size_string = binding.trim();
    let size = size_string.split_once('x');

    if size.is_none() {
        return Err(());
    }

    let size = size.unwrap();
    let width: i32 = FromStr::from_str(size.0).unwrap();
    let height: i32 = FromStr::from_str(size.1).unwrap();

    Ok((width, height))
}
