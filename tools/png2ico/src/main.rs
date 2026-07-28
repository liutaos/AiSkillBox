// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top

use std::fs::File;
use std::io::{Write, BufWriter, Cursor};

fn main() {
    let input = std::env::args().nth(1).expect("用法: png2ico <input.png> <output.ico>");
    let output = std::env::args().nth(2).expect("用法: png2ico <input.png> <output.ico>");

    let img = image::open(&input).expect("无法打开 PNG 文件");

    let sizes: Vec<u32> = vec![16, 32, 48];
    let mut png_data_list: Vec<(u32, Vec<u8>)> = Vec::new();

    for &size in &sizes {
        let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
        let mut buf = Cursor::new(Vec::new());
        resized.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        let png_bytes = buf.into_inner();
        println!("  {}x{} -> {} bytes PNG", size, size, png_bytes.len());
        png_data_list.push((size, png_bytes));
    }

    let mut f = BufWriter::new(File::create(&output).expect("无法创建 ICO 文件"));

    // ICO header: reserved(2) + type(2) + count(2) = 6 bytes
    f.write_all(&0u16.to_le_bytes()).unwrap();
    f.write_all(&1u16.to_le_bytes()).unwrap(); // type=1 icon
    let count = png_data_list.len() as u16;
    f.write_all(&count.to_le_bytes()).unwrap();

    // Each directory entry: 16 bytes
    // Header: 6 + count*16
    let header_size = 6 + 16 * png_data_list.len();
    let mut data_offset: u32 = header_size as u32;

    let mut entries = Vec::new();

    for (size, png_data) in &png_data_list {
        let s = *size;
        let w: u8 = if s >= 256 { 0 } else { s as u8 };
        let h: u8 = if s >= 256 { 0 } else { s as u8 };
        let data_size = png_data.len() as u32;

        // ICO directory entry: 16 bytes total
        let mut entry = [0u8; 16];
        entry[0] = w;                       // bWidth
        entry[1] = h;                       // bHeight
        entry[2] = 0;                       // bColorCount
        entry[3] = 0;                       // bReserved
        entry[4..6].copy_from_slice(&1u16.to_le_bytes());   // wPlanes = 1
        entry[6..8].copy_from_slice(&32u16.to_le_bytes());  // wBitCount = 32
        entry[8..12].copy_from_slice(&data_size.to_le_bytes());  // dwBytesInRes
        entry[12..16].copy_from_slice(&data_offset.to_le_bytes()); // dwImageOffset

        entries.push((entry, png_data.to_vec()));
        data_offset += data_size;
    }

    // Write directory entries
    for (entry, _) in &entries {
        f.write_all(entry).unwrap();
    }

    // Write PNG data
    for (_, data) in &entries {
        f.write_all(data).unwrap();
    }

    println!("已生成: {} ({} entries)", output, entries.len());
}
