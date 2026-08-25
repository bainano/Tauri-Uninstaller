//! 应用图标提取模块
//!
//! 使用 Windows Shell API（SHGetFileInfoW + DrawIconEx）从 exe/ico 等文件
//! 提取关联图标，绘制到 32x32 DIB 后编码为 PNG。

use std::ffi::c_void;
use std::ptr::null_mut;

use image::ImageEncoder;
use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, DIB_USAGE, HBRUSH, HDC, HGDIOBJ,
};
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_FLAGS, SHGFI_ICON, SHGFI_LARGEICON};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, DrawIconEx, DI_NORMAL};

const ICON_SIZE: i32 = 32;

/// 清理 DisplayIcon 路径：去除 ",N" 图标索引后缀与首尾引号
fn clean_path(path: &str) -> String {
    path.split(',')
        .next()
        .unwrap_or(path)
        .trim()
        .trim_matches('"')
        .to_string()
}

/// 提取文件关联图标，返回 PNG 字节
pub fn extract_file_icon(path: &str) -> Result<Vec<u8>, String> {
    let clean = clean_path(path);
    if clean.is_empty() {
        return Err("空图标路径".into());
    }

    let wide: Vec<u16> = clean.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        // 1. 获取文件关联图标句柄
        let mut sfi = SHFILEINFOW::default();
        let flags: SHGFI_FLAGS = SHGFI_ICON | SHGFI_LARGEICON;
        let ret = SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut sfi as *mut SHFILEINFOW),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            flags,
        );
        if ret == 0 || sfi.hIcon.is_invalid() {
            return Err(format!("无法获取图标: {}", clean));
        }

        // 2. 创建 32bpp top-down DIB 作为绘制目标
        let hdc = GetDC(None);
        let memdc = CreateCompatibleDC(hdc);

        let mut bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: ICON_SIZE,
                biHeight: -ICON_SIZE, // 负数 = top-down，像素顺序自上而下
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            bmiColors: [Default::default(); 1],
        };

        let mut bits: *mut c_void = null_mut();
        let dib = match CreateDIBSection(memdc, &bi, DIB_RGB_COLORS, &mut bits, None, 0) {
            Ok(h) => h,
            Err(_) => {
                ReleaseDC(None, hdc);
                DeleteDC(memdc);
                DestroyIcon(sfi.hIcon);
                return Err("DIB 创建失败".into());
            }
        };
        if bits.is_null() {
            ReleaseDC(None, hdc);
            DeleteDC(memdc);
            DeleteObject(HGDIOBJ(dib.0));
            DestroyIcon(sfi.hIcon);
            return Err("DIB 位图缓冲为空".into());
        }

        let old = SelectObject(memdc, HGDIOBJ(dib.0));

        // 3. 将图标绘制到 DIB
        DrawIconEx(memdc, 0, 0, sfi.hIcon, ICON_SIZE, ICON_SIZE, 0, HBRUSH::default(), DI_NORMAL);

        // 4. 读取 BGRA 像素并转为 RGBA
        let len = (ICON_SIZE * ICON_SIZE * 4) as usize;
        let bgra = std::slice::from_raw_parts(bits as *const u8, len);
        let mut rgba = Vec::with_capacity(len);
        for px in bgra.chunks_exact(4) {
            rgba.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
        }

        // 5. 清理 GDI 资源
        SelectObject(memdc, old);
        DeleteObject(HGDIOBJ(dib.0));
        DeleteDC(memdc);
        ReleaseDC(None, hdc);
        DestroyIcon(sfi.hIcon);

        // 6. PNG 编码
        let img = image::RgbaImage::from_raw(ICON_SIZE as u32, ICON_SIZE as u32, rgba)
            .ok_or("像素缓冲大小不匹配")?;
        let mut png = Vec::new();
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(
                img.as_raw(),
                ICON_SIZE as u32,
                ICON_SIZE as u32,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|e| format!("PNG 编码失败: {}", e))?;
        Ok(png)
    }
}
