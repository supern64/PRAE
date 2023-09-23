use std::{env, fs};
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::prelude::*;
use flate2::Compression;
use flate2::write::DeflateEncoder;
use flate2::read::DeflateDecoder;
use std::path::{Path, MAIN_SEPARATOR_STR, PathBuf};

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone, Copy)]
enum FileType {
    Box = 0,
    Obj = 1,
    Map = 2,
    HeightMap = 3,
    Path = 4,
    Animate = 5,
    CarProperty = 6,
    Mdl = 7,
    Texture = 10,
    Unknown = -1
}

impl From<u8> for FileType {
    fn from(orig: u8) -> Self {
        match orig {
            0 => return FileType::Box,
            1 => return FileType::Obj,
            2 => return FileType::Map,
            3 => return FileType::HeightMap,
            4 => return FileType::Path,
            5 => return FileType::Animate,
            6 => return FileType::CarProperty,
            7 => return FileType::Mdl,
            10 => return FileType::Texture,
            _ => return FileType::Unknown
        }
    }
}

impl From<FileType> for u8 {
    fn from(orig: FileType) -> Self {
        match orig {
            FileType::Box => return 0,
            FileType::Obj => return 1,
            FileType::Map => return 2,
            FileType::HeightMap => return 3,
            FileType::Path => return 4,
            FileType::Animate => return 5,
            FileType::CarProperty => return 6,
            FileType::Mdl => return 7,
            FileType::Texture => return 10,
            FileType::Unknown => return 255
        }
    }
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Animate => write!(f, "Animation"),
            FileType::Box => write!(f, "Box"),
            FileType::Obj => write!(f, "Object"),
            FileType::Map => write!(f, "Map"),
            FileType::HeightMap => write!(f, "Height Map"),
            FileType::Path => write!(f, "Path"),
            FileType::Animate => write!(f, "Animation"),
            FileType::CarProperty => write!(f, "Car Property"),
            FileType::Mdl => write!(f, "Model"),
            FileType::Texture => write!(f, "Texture"),
            _ => write!(f, "Unknown")
        }
    }
}

#[derive(Debug)]
struct FileMeta {
    path: String,
    file_type: FileType
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.get(1).is_none() {
        print_help();
        std::process::exit(1);
    } else {
        let command = args.get(1).unwrap();
        match command.as_str() {
            "help" => print_help(),
            "list" => list(&args),
            "unzip" => unzip(&args),
            "zip" => zip(&args),
            _ => {
                print_help();
            }
        }
    }
}


fn read_and_decompress(file_name: &str, data: &mut Vec<u8>) {
    let path = Path::new(file_name);
    let mut file = match File::open(&path) {
        Err(why) => {
            println!("Couldn't open file {}: {}", path.display(), why);
            std::process::exit(2);
        }
        Ok(file) => file,
    };
    let mut decoder = DeflateDecoder::new(&mut file);
    decoder.read_to_end(data).unwrap();
}

fn list(args: &Vec<String>) {
    if args.get(2).is_none() {
        println!("Usage: prae list <file>");
        std::process::exit(1);
    } else {
        let file_name = args.get(2).unwrap();
        let mut data = Vec::new();
        read_and_decompress(file_name, &mut data);
        let number_of_files = i32::from_le_bytes(data[0..4].try_into().unwrap());
        println!("Found {} files.", number_of_files);
        let mut pointer: usize = 4;
        for _i in 0..number_of_files {
            let file_path_length = data[pointer] as usize;
            pointer += 1;
            let file_path = String::from_utf8(data[pointer..pointer + file_path_length].to_vec()).unwrap();
            pointer += file_path_length;
            let file_type: u8 = data[pointer];
            println!("      {} ({:?})", file_path, FileType::from(file_type));
            pointer += 1;
        }
    }
}

fn zip(args: &Vec<String>) {
    if args.get(2).is_none() {
        println!("Usage: prae zip <folder> [file]");
        std::process::exit(1);
    } else {
        let folder_name = args.get(2).unwrap();
        let mut archive_name = folder_name.clone();
        archive_name.push_str(".dat");
        let target = args.get(3).unwrap_or(&archive_name);
        let target_path = Path::new(target);
        let path = env::current_dir().unwrap().join(folder_name);
        let mut data: Vec<u8> = Vec::new();
        let mut file_list: Vec<FileMeta> = Vec::new();
        let mut texture_list: Vec<FileMeta> = Vec::new();
        match get_file_list(&path, &mut file_list, &mut texture_list) {
            Ok(_) => {
                match write_raw_data(&mut data, &path, &texture_list, &file_list) {
                    Ok(_) => {
                        println!("Writing {}", target_path.file_name().unwrap().to_str().unwrap());
                        let mut file = match OpenOptions::new().write(true).create(true).truncate(true).open(&target_path) {
                            Ok(file) => file,
                            Err(why) => {
                                println!("Failed to create file {:?} ({})", path.as_os_str(), why);
                                std::process::exit(2);
                            }
                        };
                        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
                        encoder.write_all(&data).unwrap();
                        let bytes = encoder.finish().unwrap();
                        file.write_all(&bytes).unwrap();
                    },
                    Err(why) => {
                        println!("Couldn't compile archive: {}", why);
                    }
                };
            },
            Err(why) => {
                println!("Couldn't read files from {}: {}", path.display(), why);
            }
        };
    }
}

fn unzip(args: &Vec<String>) {
    if args.get(2).is_none() {
        println!("Usage: prae unzip <file> [folder]");
        std::process::exit(1);
    } else {
        let file_name = args.get(2).unwrap();
        let truncated_name = file_name.strip_suffix(".dat").unwrap_or(&file_name).to_string();
        let target = args.get(3).unwrap_or(&truncated_name);
        let mut data = Vec::new();
        read_and_decompress(file_name, &mut data);
        let number_of_files = i32::from_le_bytes(data[0..4].try_into().unwrap());
        let mut file_list = Vec::new();
        println!("Unzipping {} files.", number_of_files);
        let mut pointer: usize = 4;
        for _i in 0..number_of_files {
            let file_path_length = data[pointer] as usize;
            pointer += 1;
            let file_path = String::from_utf8(data[pointer..pointer + file_path_length].to_vec()).unwrap();
            pointer += file_path_length;
            let file_type: u8 = data[pointer];
            let file_meta = FileMeta {
                path: file_path,
                file_type: FileType::from(file_type)
            };
            file_list.push(file_meta);
            pointer += 1;
        }
        for i in 0..number_of_files {
            let file_meta = file_list.get(i as usize).unwrap();
            let path = Path::new(target).join(file_meta.path.to_owned());
            match create_dir_all(path.parent().unwrap()) {
                Ok(_) => {},
                Err(why) => {
                    println!("Failed to create folder {:?} ({}), skipping", path.as_os_str(), why);
                    continue;
                } 
            };
            let mut file = match OpenOptions::new().write(true).create(true).truncate(true).open(&path) {
                Ok(file) => file,
                Err(why) => {
                    println!("Failed to create file {:?} ({}), skipping", path.as_os_str(), why);
                    continue;
                }
            };
            let size = i32::from_le_bytes(data[pointer..pointer + 4].try_into().unwrap());
            pointer += 4;
            let data = &data[pointer..pointer + size as usize];
            match file.write_all(data) {
                Ok(_) => {
                    println!("      {} ({:?})", file_meta.path, file_meta.file_type);
                },
                Err(why) => {
                    println!("Failed to write to file {:?} ({}), skipping", path.as_os_str(), why);
                    continue;
                }
            }
            pointer += size as usize;
        }
    }
}

fn get_file_list(path: &Path, file: &mut Vec<FileMeta>, texture: &mut Vec<FileMeta>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        let metadata = fs::metadata(&entry_path)?;
        if metadata.is_file() {
            let name = entry_path.file_name().unwrap().to_str().unwrap();
            let file_type = match name {
                "path.dat" => FileType::Path,
                "sky.obj" => FileType::Obj,
                "heightmap.hmp" => FileType::HeightMap,
                "animate.dat" => FileType::Animate,
                "carproperty.dat" => FileType::CarProperty,
                _ => {
                    let lowercase = name.to_lowercase();
                    let split: Vec<&str> = lowercase.split(".").collect();
                    match split[split.len()-1] {
                        "box" => FileType::Box,
                        "map" => FileType::Map,
                        "png" | "jpg" => FileType::Texture,
                        _ => FileType::Unknown
                    }
                }
            };
            if file_type == FileType::Texture {
                texture.push(FileMeta { path: entry_path.as_os_str().to_str().unwrap().to_string(), file_type: file_type }); // textures go first
            } else if file_type != FileType::Unknown {
                file.push(FileMeta { path: entry_path.as_os_str().to_str().unwrap().to_string(), file_type: file_type });
            } else {
                println!("Skipping file with unknown file type {:?}", entry_path);
            }
        } else {
            get_file_list(entry_path.as_path(), file, texture)?;
        }
    }
    Ok(())
}

fn write_raw_data(data: &mut Vec<u8>, path: &PathBuf, texture_list: &Vec<FileMeta>, file_list: &Vec<FileMeta>) -> Result<(), std::io::Error> {
    let number_of_files: i32 = texture_list.len() as i32 + file_list.len() as i32;
    data.write_all(&i32::to_le_bytes(number_of_files))?; // header
    for i in 0..texture_list.len() {
        let file = texture_list.get(i).unwrap();
        let archive_path = Path::new(&file.path).strip_prefix(path.as_path()).unwrap();
        let archive_path_string = archive_path.as_os_str().to_str().unwrap().replace(MAIN_SEPARATOR_STR, "/");
        data.write_all(&u8::to_le_bytes(archive_path_string.len() as u8))?;
        data.write_all(archive_path_string.as_bytes())?;
        data.write_all(&u8::to_le_bytes(file.file_type.into()))?;
    }
    for i in 0..file_list.len() {
        let file = file_list.get(i).unwrap();
        let archive_path = Path::new(&file.path).strip_prefix(path.as_path()).unwrap();
        let archive_path_string = archive_path.as_os_str().to_str().unwrap().replace(MAIN_SEPARATOR_STR, "/");
        data.write_all(&u8::to_le_bytes(archive_path_string.len() as u8))?;
        data.write_all(archive_path_string.as_bytes())?;
        data.write_all(&u8::to_le_bytes(file.file_type.into()))?;
    }
    for i in 0..texture_list.len() {
        let file = texture_list.get(i).unwrap();
        let archive_path = Path::new(&file.path).strip_prefix(path.as_path()).unwrap();
        let archive_path_string = archive_path.as_os_str().to_str().unwrap().replace(MAIN_SEPARATOR_STR, "/");
        let mut real_file = OpenOptions::new().read(true).open(&file.path)?;
        let mut buffer: Vec<u8> = Vec::new();
        real_file.read_to_end(&mut buffer)?;
        data.write_all(&i32::to_le_bytes(buffer.len() as i32))?;
        data.write(&buffer)?;
        println!("   {} ({:?})", archive_path_string, file.file_type);
    }
    for i in 0..file_list.len() {
        let file = file_list.get(i).unwrap();
        let archive_path = Path::new(&file.path).strip_prefix(path.as_path()).unwrap();
        let archive_path_string = archive_path.as_os_str().to_str().unwrap().replace(MAIN_SEPARATOR_STR, "/");
        let mut real_file = OpenOptions::new().read(true).open(&file.path)?;
        let mut buffer: Vec<u8> = Vec::new();
        real_file.read_to_end(&mut buffer)?;
        data.write_all(&i32::to_le_bytes(buffer.len() as i32))?;
        data.write(&buffer)?;
        println!("   {} ({:?})", archive_path_string, file.file_type);
    }
    Ok(())
}

fn print_help() {
    println!("
PRAE - Pyongyang Racer Asset Extractor

    A tool to extract and compress 1.dat and common.dat archives.

    Usage:
        prae <command> [options]

    Commands:
        help - Prints this help message.
        unzip <file> [folder] - Extracts the given archive to a folder.
        zip <folder> [file] - Compresses the given folder to an archive.
        list <file> - Lists the files inside the given archive.
        ")

}