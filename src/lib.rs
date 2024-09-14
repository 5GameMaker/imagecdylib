use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    fs::File,
    io::{BufReader, Cursor, Read, Seek, Write},
    mem::transmute,
    os::fd::{FromRawFd, RawFd},
    path::PathBuf,
    str::FromStr,
    sync::Mutex,
    thread::ThreadId,
};

use image::{ColorType, DynamicImage, ImageFormat, ImageReader};

#[doc(hidden)]
/// cbindgen:no-export
static ERROR: Mutex<Option<HashMap<ThreadId, Box<[i8]>>>> = Mutex::new(None);

pub const COLORTYPE_L8: u8 = 1;
pub const COLORTYPE_LA8: u8 = 2;
pub const COLORTYPE_RGB8: u8 = 3;
pub const COLORTYPE_RGBA8: u8 = 4;
pub const COLORTYPE_L16: u8 = 5;
pub const COLORTYPE_LA16: u8 = 6;
pub const COLORTYPE_RGB16: u8 = 7;
pub const COLORTYPE_RGBA16: u8 = 8;
pub const COLORTYPE_RGB32F: u8 = 9;
pub const COLORTYPE_RGBA32F: u8 = 10;

pub const IMAGEFORMAT_PNG: u8 = 1;
pub const IMAGEFORMAT_JPEG: u8 = 2;
pub const IMAGEFORMAT_GIF: u8 = 3;
pub const IMAGEFORMAT_WEBP: u8 = 4;
pub const IMAGEFORMAT_PNM: u8 = 5;
pub const IMAGEFORMAT_TIFF: u8 = 6;
pub const IMAGEFORMAT_TGA: u8 = 7;
pub const IMAGEFORMAT_DDS: u8 = 8;
pub const IMAGEFORMAT_BMP: u8 = 9;
pub const IMAGEFORMAT_ICO: u8 = 10;
pub const IMAGEFORMAT_HDR: u8 = 11;
pub const IMAGEFORMAT_OPENEXR: u8 = 12;
pub const IMAGEFORMAT_FARBFELD: u8 = 13;
pub const IMAGEFORMAT_AVIF: u8 = 14;
pub const IMAGEFORMAT_QOI: u8 = 15;

fn u8_to_format(format: u8) -> Option<ImageFormat> {
    match format {
        IMAGEFORMAT_PNG => Some(ImageFormat::Png),
        IMAGEFORMAT_JPEG => Some(ImageFormat::Jpeg),
        IMAGEFORMAT_GIF => Some(ImageFormat::Gif),
        IMAGEFORMAT_WEBP => Some(ImageFormat::WebP),
        IMAGEFORMAT_PNM => Some(ImageFormat::Pnm),
        IMAGEFORMAT_TIFF => Some(ImageFormat::Tiff),
        IMAGEFORMAT_TGA => Some(ImageFormat::Tga),
        IMAGEFORMAT_DDS => Some(ImageFormat::Dds),
        IMAGEFORMAT_BMP => Some(ImageFormat::Bmp),
        IMAGEFORMAT_ICO => Some(ImageFormat::Ico),
        IMAGEFORMAT_HDR => Some(ImageFormat::Hdr),
        IMAGEFORMAT_OPENEXR => Some(ImageFormat::OpenExr),
        IMAGEFORMAT_FARBFELD => Some(ImageFormat::Farbfeld),
        IMAGEFORMAT_AVIF => Some(ImageFormat::Avif),
        IMAGEFORMAT_QOI => Some(ImageFormat::Qoi),
        _ => None,
    }
}

/// A wrapper for supported [Read] types.
pub enum ReadWrap {
    File(File),
    Buf(Cursor<&'static [u8]>),
    Vec(Cursor<Vec<u8>>),
}
impl Read for ReadWrap {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::File(x) => x.read(buf),
            Self::Buf(x) => x.read(buf),
            Self::Vec(x) => x.read(buf),
        }
    }
}
impl Seek for ReadWrap {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match self {
            Self::File(x) => x.seek(pos),
            Self::Buf(x) => x.seek(pos),
            Self::Vec(x) => x.seek(pos),
        }
    }
}

/// A wrapper for supported [Write] types.
pub enum WriteWrap {
    File(File),
    Buf(Cursor<&'static mut [u8]>),
    Expanding(Cursor<Vec<u8>>),
}
impl Write for WriteWrap {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::File(x) => x.write(buf),
            Self::Buf(x) => x.write(buf),
            Self::Expanding(x) => x.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::File(x) => x.flush(),
            Self::Buf(x) => x.flush(),
            Self::Expanding(x) => x.flush(),
        }
    }
}
impl Seek for WriteWrap {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match self {
            Self::File(x) => x.seek(pos),
            Self::Buf(x) => x.seek(pos),
            Self::Expanding(x) => x.seek(pos),
        }
    }
}

fn set_err(error: String) {
    let Ok(mut lock) = ERROR.lock() else {
        return;
    };

    if lock.is_none() {
        lock.replace(HashMap::new());
    }

    (*lock).as_mut().unwrap().insert(
        std::thread::current().id(),
        unsafe { std::mem::transmute::<&[u8], &[i8]>(CString::new(error).unwrap().to_bytes()) }
            .to_vec()
            .into(),
    );
}

/// Get `libimage` error.
///
/// Must be executed at the same thread as the function that caused the error.
///
/// ## Safety
/// Pointer must be dropped before calling [libimage_reset_err]. To prevent
/// memory leaks, [libimage_reset_err] must be called at the end.
#[no_mangle]
pub unsafe extern "C" fn libimage_get_err() -> *const i8 {
    match match ERROR.lock().ok().as_ref() {
        Some(x) => (*x).as_ref(),
        None => return std::ptr::null(),
    }
    .and_then(|x| x.get(&std::thread::current().id()))
    {
        Some(x) => &x[0] as *const i8,
        None => std::ptr::null(),
    }
}

/// Clear `libimage` error.
///
/// Must be called after other libimage calls to prevent memory leaks.
#[no_mangle]
pub extern "C" fn libimage_reset_err() {
    match match ERROR.lock().ok().as_mut() {
        Some(x) => (*x).as_mut(),
        None => return,
    } {
        Some(x) => x.remove(&std::thread::current().id()),
        None => None,
    };
}

/// Open a file for reading.
///
/// Return `null` on error, can be obtained via [libimage_get_err]
///
/// ## Safety
/// `path` must be a valid path.
#[no_mangle]
pub unsafe extern "C" fn libimage_open_file_r(path: *const i8) -> *mut ReadWrap {
    let str = CStr::from_ptr(path);
    let path = PathBuf::from_str(str.to_string_lossy().as_ref()).unwrap();
    match File::open(path) {
        Ok(x) => Box::leak(Box::new(ReadWrap::File(x))) as *mut ReadWrap,
        Err(why) => {
            set_err(why.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Open a file for writing.
///
/// Return `null` on error, can be obtained via [libimage_get_err]
///
/// ## Safety
/// `path` must be a valid path.
#[no_mangle]
pub unsafe extern "C" fn libimage_open_file_w(path: *const i8) -> *mut WriteWrap {
    let str = CStr::from_ptr(path);
    let path = PathBuf::from_str(str.to_string_lossy().as_ref()).unwrap();
    match File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
    {
        Ok(x) => Box::leak(Box::new(WriteWrap::File(x))) as *mut WriteWrap,
        Err(why) => {
            set_err(why.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Use a file descriptor for reading.
///
/// ## Safety
/// `fd` must be a valid descriptor with reading allowed.
#[no_mangle]
pub unsafe extern "C" fn libimage_fd_r(fd: RawFd) -> *mut ReadWrap {
    Box::leak(Box::new(ReadWrap::File(File::from_raw_fd(fd))))
}

/// Use a file descriptor for writing.
///
/// ## Safety
/// `fd` must be a valid descriptor with writing allowed.
#[no_mangle]
pub unsafe extern "C" fn libimage_fd_w(fd: RawFd) -> *mut WriteWrap {
    Box::leak(Box::new(WriteWrap::File(File::from_raw_fd(fd))))
}

/// Open a static buffer for reading.
///
/// Return `null` on error, can be obtained via [libimage_get_err]
///
/// ## Safety
/// `buffer` must be a valid pointer and `len` must correctly represent its length.
/// Object is no longer valid after the source buffer will be dropped.
#[no_mangle]
pub unsafe extern "C" fn libimage_open_buf_r(buffer: *mut u8, len: usize) -> *mut ReadWrap {
    let slice = std::slice::from_raw_parts::<'static>(buffer, len);
    let cursor = Cursor::new(slice);
    let wrap = ReadWrap::Buf(cursor);
    Box::leak(Box::new(wrap))
}

/// Open a static buffer for writing.
///
/// Return `null` on error, can be obtained via [libimage_get_err]
///
/// ## Safety
/// `buffer` must be a valid pointer and `len` must correctly represent its length.
/// Object is no longer valid after the source buffer will be dropped.
#[no_mangle]
pub unsafe extern "C" fn libimage_open_buf_w(buffer: *mut u8, len: usize) -> *mut WriteWrap {
    let slice = std::slice::from_raw_parts_mut::<'static>(buffer, len);
    let cursor = Cursor::new(slice);
    let wrap = WriteWrap::Buf(cursor);
    Box::leak(Box::new(wrap))
}

/// Create a new expanding buffer for writing.
#[no_mangle]
pub extern "C" fn libimage_open_expanding_w() -> *mut WriteWrap {
    let cursor = Cursor::new(vec![]);
    let wrap = WriteWrap::Expanding(cursor);
    Box::leak(Box::new(wrap))
}

/// Destroy a [ReadWrap] object.
///
/// This function should be called when a ReadWrap object is no longer in use.
///
/// ## Safety
/// `read` must be a valid reference to a non-destroyed [ReadWrap] object.
#[no_mangle]
pub unsafe extern "C" fn libimage_destroy_r(read: *mut ReadWrap) {
    drop(Box::from_raw(read))
}

/// Destroy a [WriteWrap] object.
///
/// This function should be called when a WriteWrap object is no longer in use.
///
/// ## Safety
/// `write` must be a valid reference to a non-destroyed [WriteWrap] object.
#[no_mangle]
pub unsafe extern "C" fn libimage_destroy_w(write: *mut WriteWrap) {
    drop(Box::from_raw(write))
}

/// Read an image of unknown format. Returns `null` if read failed or image
/// format is not supported.
///
/// To destroy [DynamicImage] object, use the [libimage_destroy_image] function.
///
/// Return `null` on error, can be obtained via [libimage_get_err]
///
/// ## Safety
/// `read` must be a valid reference to a non-destroyed [WriteWrap] object.
#[no_mangle]
pub unsafe extern "C" fn libimage_read_guess(read: *mut ReadWrap) -> *mut DynamicImage {
    let Some(read) = read.as_mut() else {
        set_err("'read' is null".to_string());
        return std::ptr::null_mut();
    };
    match ImageReader::new(BufReader::new(read)).with_guessed_format() {
        Ok(x) => match x.decode() {
            Ok(x) => Box::leak(Box::new(x)) as *mut DynamicImage,
            Err(why) => {
                set_err(why.to_string());
                std::ptr::null_mut()
            }
        },
        Err(why) => {
            set_err(why.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Read an image. Returns `null` if read failed or image format is not supported.
///
/// To destroy [DynamicImage] object, use the [libimage_destroy_image] function.
///
/// Return `null` on error, can be obtained via [libimage_get_err]
///
/// ## Safety
/// `read` must be a valid reference to a non-destroyed [WriteWrap] object.
#[no_mangle]
pub unsafe extern "C" fn libimage_read(read: *mut ReadWrap, format: u8) -> *mut DynamicImage {
    let Some(read) = read.as_mut() else {
        set_err("'read' is null".to_string());
        return std::ptr::null_mut();
    };

    let Some(format) = u8_to_format(format) else {
        set_err(format!("Unknown image format: {format}"));
        return std::ptr::null_mut();
    };

    match ImageReader::with_format(BufReader::new(read), format).decode() {
        Ok(x) => Box::leak(Box::new(x)) as *mut DynamicImage,
        Err(why) => {
            set_err(why.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Destroy a [DynamicImage] object.
///
/// This function should be called when a WriteWrap object is no longer in use.
///
/// ## Safety
/// `image` must be a valid reference to a non-destroyed [DynamicImage] object.
#[no_mangle]
pub unsafe extern "C" fn libimage_destroy_image(image: *mut DynamicImage) {
    drop(Box::from_raw(image))
}

/// Image info.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ImageMetrics {
    bufsize: usize,
    width: u32,
    height: u32,
    color: u8,
    channels: u8,
    bytes_per_pixel: u8,
    has_alpha: bool,
    has_color: bool,
}

/// Possible image info.
///
/// Value is 0 for none variant.
#[repr(C)]
pub union ImageMetricsMaybe {
    metrics: ImageMetrics,
    none: u8,
}

/// Get image metrics.
///
/// Returns `0` on error.
///
/// ## Safety
/// `image` must point to a valid object.
#[no_mangle]
pub unsafe extern "C" fn libimage_metrics(image: *mut DynamicImage) -> ImageMetricsMaybe {
    let Some(image) = image.as_mut() else {
        set_err("'image' is null".to_string());
        return ImageMetricsMaybe { none: 0 };
    };

    ImageMetricsMaybe {
        metrics: ImageMetrics {
            width: image.width(),
            height: image.height(),
            color: match image.color() {
                ColorType::L8 => COLORTYPE_L8,
                ColorType::La8 => COLORTYPE_LA8,
                ColorType::Rgb8 => COLORTYPE_RGB8,
                ColorType::Rgba8 => COLORTYPE_RGBA8,
                ColorType::L16 => COLORTYPE_L16,
                ColorType::La16 => COLORTYPE_LA16,
                ColorType::Rgb16 => COLORTYPE_RGB16,
                ColorType::Rgba16 => COLORTYPE_RGBA16,
                ColorType::Rgb32F => COLORTYPE_RGB32F,
                ColorType::Rgba32F => COLORTYPE_RGBA32F,
                _ => {
                    set_err("Unsupported color format".to_string());
                    return ImageMetricsMaybe { none: 0 };
                }
            },
            bufsize: image.as_bytes().len(),
            has_alpha: image.color().has_alpha(),
            has_color: image.color().has_color(),
            channels: image.color().channel_count(),
            bytes_per_pixel: image.color().bytes_per_pixel(),
        },
    }
}

/// Get image bytes in native endiannes.
///
/// Returns `null` on error.
///
/// To get the length of byte array, use [libimage_metrics].
///
/// ## Safety
/// `image` must point to a valid object.
#[no_mangle]
pub unsafe extern "C" fn libimage_as_bytes(image: *mut DynamicImage) -> *const u8 {
    let Some(image) = image.as_mut() else {
        set_err("'image' is null".to_string());
        return std::ptr::null();
    };

    image.as_bytes().as_ptr()
}

/// Convert image to RGBA8888
///
/// Returns `false` on error, `true` otherwise.
///
/// To get the length of byte array, multiply width and height by 4. To get the values, use [libimage_metrics].
///
/// ## Safety
/// `image` must point to a valid object. `copy_dest` must point to an array that is of or more
/// size than `width * height * 4`.
#[no_mangle]
pub unsafe extern "C" fn libimage_into_rgba8888(
    image: *mut DynamicImage,
    copy_dest: *mut u8,
) -> bool {
    let Some(image) = image.as_mut() else {
        set_err("'image' is null".to_string());
        return false;
    };

    let rgba = image.to_rgba8();
    std::slice::from_raw_parts_mut(copy_dest, rgba.len()).copy_from_slice(&rgba);

    true
}

/// Save image.
///
/// Returns `false` on error, `true` otherwise.
///
/// ## Safety
/// `image` and `write` must point to a valid object. `format` must be a valid format.
#[no_mangle]
pub unsafe extern "C" fn libimage_write(
    image: *mut DynamicImage,
    write: *mut WriteWrap,
    format: u8,
) -> bool {
    let Some(image) = image.as_mut() else {
        set_err("'image' is null".to_string());
        return false;
    };

    let Some(write) = write.as_mut() else {
        set_err("'write' is null".to_string());
        return false;
    };

    let Some(format) = u8_to_format(format) else {
        set_err(format!("Unknown image format: {format}"));
        return false;
    };

    if let Err(why) = image.write_to(write, format) {
        set_err(why.to_string());
        return false;
    }

    true
}

/// Create an image from raw RGBA8888 bytes.
///
/// ## Safety
/// `bytes` must be at least or of size of `width * height * 4`.
#[no_mangle]
pub unsafe extern "C" fn libimage_new_rgba8888(
    bytes: *const u8,
    width: u32,
    height: u32,
) -> *mut DynamicImage {
    let mut image = DynamicImage::new_rgba8(width, height);
    image
        .as_mut_rgba8()
        .unwrap()
        .copy_from_slice(std::slice::from_raw_parts(
            bytes,
            width as usize * height as usize * 4,
        ));
    Box::leak(Box::new(image)) as *mut DynamicImage
}

/// Convert a writer into a reader if it can be done trivially.
///
/// ## Safety
/// `write` must point to a valid object.
#[no_mangle]
pub unsafe extern "C" fn libimage_w_into_r(write: *mut WriteWrap) -> *mut ReadWrap {
    if write.is_null() {
        set_err("'write' is null".to_string());
        return std::ptr::null_mut();
    };

    match *Box::from_raw(write) {
        WriteWrap::File(_) => {
            set_err("Cannot cheaply convert files".into());
            std::ptr::null_mut()
        }
        WriteWrap::Buf(mut x) => {
            x.seek(std::io::SeekFrom::Start(0)).unwrap();
            Box::leak(Box::new(ReadWrap::Buf(transmute::<
                Cursor<&mut [u8]>,
                Cursor<&[u8]>,
            >(x)))) as *mut ReadWrap
        }
        WriteWrap::Expanding(mut x) => {
            x.seek(std::io::SeekFrom::Start(0)).unwrap();
            Box::leak(Box::new(ReadWrap::Vec(x))) as *mut ReadWrap
        }
    }
}

/// Pipe reader into a writer.
///
/// ## Safety
/// `write` and `read` must point to valid objects.
#[no_mangle]
pub unsafe extern "C" fn libimage_pipe(write: *mut WriteWrap, read: *mut ReadWrap) -> bool {
    let Some(write) = write.as_mut() else {
        set_err("'write' is null".to_string());
        return false;
    };

    let Some(read) = read.as_mut() else {
        set_err("'read' is null".to_string());
        return false;
    };

    let mut buf = vec![0; 8192];

    loop {
        let len = match read.read(&mut buf) {
            Ok(x) => x,
            Err(why) => {
                set_err(why.to_string());
                return false;
            }
        };

        if len == 0 {
            break true;
        }

        if let Err(why) = write.write_all(&buf) {
            set_err(why.to_string());
            break false;
        }
    }
}

/// Poll bytes from a reader. Returns 0 when done or error.
///
/// ## Safety
/// `read` must point to a valid object. `buf` must not be null.
#[no_mangle]
pub unsafe extern "C" fn libimage_poll(read: *mut ReadWrap, buf: *mut u8, len: usize) -> usize {
    let Some(read) = read.as_mut() else {
        set_err("'read' is null".to_string());
        return 0;
    };

    match read.read(std::slice::from_raw_parts_mut(buf, len)) {
        Ok(x) => x,
        Err(why) => {
            set_err(why.to_string());
            0
        }
    }
}
