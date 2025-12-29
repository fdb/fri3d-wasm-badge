//! Minimal PNG encoder for Fri3d Badge screenshots

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use miniz_oxide::deflate::compress_to_vec_zlib;

/// Encode a grayscale image as PNG
pub fn encode_grayscale(width: u32, height: u32, data: &[u8]) -> Vec<u8> {
    let mut png = Vec::new();

    // PNG signature
    png.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

    // IHDR chunk
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(0); // color type (grayscale)
    ihdr.push(0); // compression method
    ihdr.push(0); // filter method
    ihdr.push(0); // interlace method
    write_chunk(&mut png, b"IHDR", &ihdr);

    // IDAT chunk (compressed image data)
    let mut raw_data = Vec::with_capacity((width as usize + 1) * height as usize);
    for y in 0..height as usize {
        raw_data.push(0); // filter type: None
        let row_start = y * width as usize;
        let row_end = row_start + width as usize;
        if row_end <= data.len() {
            raw_data.extend_from_slice(&data[row_start..row_end]);
        }
    }

    let compressed = compress_to_vec_zlib(&raw_data, 6);
    write_chunk(&mut png, b"IDAT", &compressed);

    // IEND chunk
    write_chunk(&mut png, b"IEND", &[]);

    png
}

/// Encode a 1-bit monochrome image as grayscale PNG
pub fn encode_monochrome(width: u32, height: u32, buffer: &[u8]) -> Vec<u8> {
    let mut grayscale = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            let byte_index = ((y * width + x) / 8) as usize;
            let bit_index = 7 - ((x % 8) as usize);

            let pixel = if byte_index < buffer.len() {
                (buffer[byte_index] >> bit_index) & 1
            } else {
                0
            };

            // 0 = white (0xFF), 1 = black (0x00)
            grayscale.push(if pixel == 1 { 0x00 } else { 0xFF });
        }
    }

    encode_grayscale(width, height, &grayscale)
}

fn write_chunk(png: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    let length = data.len() as u32;
    png.extend_from_slice(&length.to_be_bytes());
    png.extend_from_slice(chunk_type);
    png.extend_from_slice(data);

    // CRC32
    let mut crc_data = Vec::with_capacity(4 + data.len());
    crc_data.extend_from_slice(chunk_type);
    crc_data.extend_from_slice(data);
    let crc = crc32(&crc_data);
    png.extend_from_slice(&crc.to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        let index = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = CRC_TABLE[index] ^ (crc >> 8);
    }
    crc ^ 0xFFFFFFFF
}

static CRC_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            if c & 1 != 0 {
                c = 0xEDB88320 ^ (c >> 1);
            } else {
                c >>= 1;
            }
            k += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
};
