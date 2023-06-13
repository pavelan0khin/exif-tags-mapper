mod types;

use crate::types::{Meme, MemeOutput, Tag};
use bson::de::from_document;
use bson::Document;
use indicatif::{ProgressBar, ProgressStyle};
use rexiv2::Metadata;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::{self, Write};
use std::path::Path;

fn read_bson<T: DeserializeOwned>(path: &str) -> io::Result<Vec<T>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut docs = Vec::new();

    while let Ok(doc) = Document::from_reader(&mut reader) {
        let item: T = from_document(doc).unwrap();
        docs.push(item);
    }
    Ok(docs)
}

fn get_images_vec(memes: Vec<Meme>, tags_map: HashMap<String, String>) -> Vec<MemeOutput> {
    let result = memes
        .into_iter()
        .map(|meme| {
            let tags = meme
                .tags
                .into_iter()
                .filter_map(|tag_id| match tags_map.get(&tag_id) {
                    Some(name) => Some(name.clone()),
                    None => {
                        eprintln!("No matching tag for id {}", tag_id);
                        None
                    }
                })
                .collect();

            MemeOutput {
                id: meme.id.to_string(),
                title: meme.title,
                description: meme.description,
                tags,
                image: meme.image,
            }
        })
        .collect();
    return result;
}

fn read_images_dir(images_path: &str) -> Vec<String> {
    let mut image_files = Vec::new();
    if let Ok(entries) = fs::read_dir(images_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if entry.path().is_file() {
                    if let Some(file_name) = entry.path().file_name() {
                        if let Some(file_name) = file_name.to_str() {
                            image_files.push(file_name.to_string());
                        }
                    }
                }
            }
        }
    }
    return image_files;
}

fn find_images(memes_output: &mut Vec<MemeOutput>, image_files: Vec<String>) {
    let mut i = 0;
    let mut errors = Vec::new();
    while i < memes_output.len() {
        if let Some(file_name) = Path::new(&memes_output[i].image).file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                let ext = Path::new(file_name)
                    .extension()
                    .and_then(OsStr::to_str)
                    .unwrap_or("")
                    .to_lowercase();
                if !(ext == "jpeg" || ext == "jpg") {
                    errors.push(format!(
                        "Файл {file_name_str} имеет неподдерживаемое расширение – {ext}"
                    ));
                    memes_output.remove(i);
                    continue;
                }
                if !image_files.contains(&file_name_str.to_string()) {
                    errors.push(format!("Файл {file_name_str} не найден в директории"));
                    memes_output.remove(i);
                    continue;
                } else {
                    memes_output[i].image = file_name_str.to_string();
                }
            }
        }
        i += 1;
    }
    if !errors.is_empty() {
        let mut file = File::create("error_logs.txt").expect("Cannot create .txt file");
        for line in &errors {
            writeln!(file, "{}", line).expect("Cannot write line to file");
        }
    }
}

fn to_utf16le_string(value: &str) -> String {
    let utf16_data: Vec<u16> = value.encode_utf16().collect();
    let mut utf16_bytes: Vec<u8> = Vec::new();
    for val in utf16_data {
        utf16_bytes.push((val & 0xFF) as u8);
        utf16_bytes.push((val >> 8) as u8);
    }
    utf16_bytes.push(0);
    utf16_bytes.push(0);
    utf16_bytes
        .iter()
        .map(|b| b.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

fn add_exif_tags(memes: &[MemeOutput], image_dir: &str) -> Result<(), Box<dyn Error>> {
    let total_memes = memes.len();
    let bar = ProgressBar::new(total_memes as u64);
    let style = ProgressStyle::default_bar()
        .progress_chars("##-")
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .expect("Template error");
    bar.set_style(style);
    for meme in memes {
        let image_path = format!("{}/{}", image_dir, meme.image);
        let path = Path::new(&image_path);
        let meta = match Metadata::new_from_path(&path) {
            Ok(meta) => meta,
            Err(_) => continue, // Skip this image if there's a problem reading metadata
        };
        meta.clear();
        let description = format!("{} {}", meme.title, meme.description);
        meta.set_tag_string("Exif.Image.ImageDescription", &description.as_str())?;
        meta.set_tag_string(
            "Exif.Image.XPTitle",
            &to_utf16le_string(description.as_str()),
        )?;
        let tags_string = meme.tags.join(";");
        meta.set_tag_string(
            "Exif.Image.XPKeywords",
            &to_utf16le_string(tags_string.as_str()),
        )?;
        meta.save_to_file(&path)?;
        bar.inc(1);
    }
    bar.finish();
    Ok(())
}

fn get_paths() -> (String, String, String) {
    let mut images_path = String::new();
    println!("Enter the path to the directory with images: ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut images_path)
        .expect("Failed to read line");

    let mut tags_path = String::new();
    println!("Enter the path to the 'tags.bson' file: ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut tags_path)
        .expect("Failed to read line");

    let mut memes_path = String::new();
    println!("Enter the path to the 'memes.bson' file: ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut memes_path)
        .expect("Failed to read line");
    return (
        images_path.trim().to_string(),
        tags_path.trim().to_string(),
        memes_path.trim().to_string(),
    );
}

fn main() -> io::Result<()> {
    let (images_path, tags_path, memes_path) = get_paths();
    println!("{}", images_path);
    println!("{}", tags_path);
    println!("{}", memes_path);
    println!("Reading tags.bson");
    let tags: Vec<Tag> = read_bson(tags_path.as_str()).expect("Error reading tags.bson");
    println!("Reading memes.bson");
    let memes: Vec<Meme> = read_bson(memes_path.as_str()).expect("Error reading memes.bson");
    println!("Files read, creating tag mapping");
    let tags_map: HashMap<String, String> = tags
        .into_iter()
        .filter_map(|tag| tag.name.map(|name| (tag.id, name)))
        .collect();
    println!("Creating a vector of total values");
    let mut memes_output: Vec<MemeOutput> = get_images_vec(memes, tags_map);
    let image_files = read_images_dir(&images_path);
    println!("{:?}", image_files);
    find_images(&mut memes_output, image_files);
    match add_exif_tags(&memes_output, &images_path) {
        Ok(()) => println!("\nAll exif tags added successfully"),
        Err(err) => eprintln!("\nError when adding exif tags: {}", err),
    }
    Ok(())
}
