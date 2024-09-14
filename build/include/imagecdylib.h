/* Generated with cbindgen:0.27.0 and then manually edited cuz zoly shit */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#ifndef IMAGECDYLIB_LIBIMAGE_H

#define LIBIMAGE_COLORTYPE_L8 1
#define LIBIMAGE_COLORTYPE_LA8 2
#define LIBIMAGE_COLORTYPE_RGB8 3
#define LIBIMAGE_COLORTYPE_RGBA8 4
#define LIBIMAGE_COLORTYPE_L16 5
#define LIBIMAGE_COLORTYPE_LA16 6
#define LIBIMAGE_COLORTYPE_RGB16 7
#define LIBIMAGE_COLORTYPE_RGBA16 8
#define LIBIMAGE_COLORTYPE_RGB32F 9
#define LIBIMAGE_COLORTYPE_RGBA32F 10

#define LIBIMAGE_IMAGEFORMAT_PNG 1
#define LIBIMAGE_IMAGEFORMAT_JPEG 2
#define LIBIMAGE_IMAGEFORMAT_GIF 3
#define LIBIMAGE_IMAGEFORMAT_WEBP 4
#define LIBIMAGE_IMAGEFORMAT_PNM 5
#define LIBIMAGE_IMAGEFORMAT_TIFF 6
#define LIBIMAGE_IMAGEFORMAT_TGA 7
#define LIBIMAGE_IMAGEFORMAT_DDS 8
#define LIBIMAGE_IMAGEFORMAT_BMP 9
#define LIBIMAGE_IMAGEFORMAT_ICO 10
#define LIBIMAGE_IMAGEFORMAT_HDR 11
#define LIBIMAGE_IMAGEFORMAT_OPENEXR 12
#define LIBIMAGE_IMAGEFORMAT_FARBFELD 13
#define LIBIMAGE_IMAGEFORMAT_AVIF 14
#define LIBIMAGE_IMAGEFORMAT_QOI 15

/**
 * A wrapper for supported [DynamicImage] type.
 */
typedef struct libimage_dynamic_image libimage_dynamic_image;

/**
 * A wrapper for supported [Read] types.
 */
typedef struct libimage_read_wrap libimage_read_wrap;

/**
 * A wrapper for supported [Write] types.
 */
typedef struct libimage_write_wrap libimage_write_wrap;

/**
 * Image info.
 */
typedef struct libimage_image_metrics {
    uintptr_t bufsize;
    uint32_t width;
    uint32_t height;
    uint8_t color;
    uint8_t channels;
    uint8_t bytes_per_pixel;
    bool has_alpha;
    bool has_color;
} libimage_image_metrics;

/**
 * Possible image info.
 *
 * Value is 0 for none variant.
 */
typedef union libimage_image_metrics_maybe {
    struct libimage_image_metrics metrics;
    uint8_t none;
} libimage_image_metrics_maybe;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Get `libimage` error.
 *
 * Must be executed at the same thread as the function that caused the error.
 *
 * ## Safety
 * Pointer must be dropped before calling [libimage_reset_err]. To prevent
 * memory leaks, [libimage_reset_err] must be called at the end.
 */
const char *libimage_get_err(void);

/**
 * Clear `libimage` error.
 *
 * Must be called after other libimage calls to prevent memory leaks.
 */
void libimage_reset_err(void);

/**
 * Open a file for reading.
 *
 * Return `null` on error, can be obtained via [libimage_get_err]
 *
 * ## Safety
 * `path` must be a valid path.
 */
libimage_read_wrap *libimage_open_file_r(const char *path);

/**
 * Open a file for writing.
 *
 * Return `null` on error, can be obtained via [libimage_get_err]
 *
 * ## Safety
 * `path` must be a valid path.
 */
libimage_write_wrap *libimage_open_file_w(const char *path);

/**
 * Use a file descriptor for reading.
 *
 * ## Safety
 * `fd` must be a valid descriptor with reading allowed.
 */
libimage_read_wrap *libimage_fd_r(int fd);

/**
 * Use a file descriptor for writing.
 *
 * ## Safety
 * `fd` must be a valid descriptor with writing allowed.
 */
libimage_write_wrap *libimage_fd_w(int fd);

/**
 * Open a static buffer for reading.
 *
 * Return `null` on error, can be obtained via [libimage_get_err]
 *
 * ## Safety
 * `buffer` must be a valid pointer and `len` must correctly represent its
 * length. Object is no longer valid after the source buffer will be dropped.
 */
libimage_read_wrap *libimage_open_buf_r(char *buffer, size_t len);

/**
 * Open a static buffer for writing.
 *
 * Return `null` on error, can be obtained via [libimage_get_err]
 *
 * ## Safety
 * `buffer` must be a valid pointer and `len` must correctly represent its
 * length. Object is no longer valid after the source buffer will be dropped.
 */
libimage_write_wrap *libimage_open_buf_w(char *buffer, size_t len);

/**
 * Create a new expanding buffer for writing.
 */
libimage_write_wrap *libimage_open_expanding_w(void);

/**
 * Destroy a [ReadWrap] object.
 *
 * This function should be called when a ReadWrap object is no longer in use.
 *
 * ## Safety
 * `read` must be a valid reference to a non-destroyed [ReadWrap] object.
 */
void libimage_destroy_r(libimage_read_wrap *read);

/**
 * Destroy a [WriteWrap] object.
 *
 * This function should be called when a WriteWrap object is no longer in use.
 *
 * ## Safety
 * `write` must be a valid reference to a non-destroyed [WriteWrap] object.
 */
void libimage_destroy_w(libimage_write_wrap *write);

/**
 * Read an image of unknown format. Returns `null` if read failed or image
 * format is not supported.
 *
 * To destroy [DynamicImage] object, use the [libimage_destroy_image] function.
 *
 * Return `null` on error, can be obtained via [libimage_get_err]
 *
 * ## Safety
 * `read` must be a valid reference to a non-destroyed [WriteWrap] object.
 */
libimage_dynamic_image *libimage_read_guess(libimage_read_wrap *read);

/**
 * Read an image. Returns `null` if read failed or image format is not
 * supported.
 *
 * To destroy [DynamicImage] object, use the [libimage_destroy_image] function.
 *
 * Return `null` on error, can be obtained via [libimage_get_err]
 *
 * ## Safety
 * `read` must be a valid reference to a non-destroyed [WriteWrap] object.
 */
libimage_dynamic_image *libimage_read(libimage_read_wrap *read, uint8_t format);

/**
 * Destroy a [DynamicImage] object.
 *
 * This function should be called when a WriteWrap object is no longer in use.
 *
 * ## Safety
 * `image` must be a valid reference to a non-destroyed [DynamicImage] object.
 */
void libimage_destroy_image(libimage_dynamic_image *image);

/**
 * Get image metrics.
 *
 * Returns `0` on error.
 *
 * ## Safety
 * `image` must point to a valid object.
 */
libimage_image_metrics_maybe libimage_metrics(libimage_dynamic_image *image);

/**
 * Get image bytes in native endiannes.
 *
 * Returns `null` on error.
 *
 * To get the length of byte array, use [libimage_metrics].
 *
 * ## Safety
 * `image` must point to a valid object.
 */
const uint8_t *libimage_as_bytes(libimage_dynamic_image *image);

/**
 * Convert image to RGBA8888
 *
 * Returns `false` on error, `true` otherwise.
 *
 * To get the length of byte array, multiply width and height by 4. To get the
 * values, use [libimage_metrics].
 *
 * ## Safety
 * `image` must point to a valid object. `copy_dest` must point to an array that
 * is of or more size than `width * height * 4`.
 */
bool libimage_into_rgba8888(libimage_dynamic_image *image, uint8_t *copy_dest);

/**
 * Save image.
 *
 * Returns `false` on error, `true` otherwise.
 *
 * ## Safety
 * `image` and `write` must point to a valid object. `format` must be a valid
 * format.
 */
bool libimage_write(libimage_dynamic_image *image, libimage_write_wrap *write,
                    uint8_t format);

/**
 * Create an image from raw RGBA8888 bytes.
 *
 * ## Safety
 * `bytes` must be at least or of size of `width * height * 4`.
 */
libimage_dynamic_image *libimage_new_rgba8888(const char *bytes, uint32_t width,
                                              uint32_t height);

/**
 * Convert a writer into a reader if it can be done trivially.
 *
 * ## Safety
 * `write` must point to a valid object.
 */
libimage_read_wrap *libimage_w_into_r(libimage_write_wrap *write);

/**
 * Pipe reader into a writer.
 *
 * ## Safety
 * `write` and `read` must point to valid objects.
 */
bool libimage_pipe(libimage_write_wrap *write, libimage_read_wrap *read);

/**
 * Poll bytes from a reader. Returns 0 when done or error.
 *
 * ## Safety
 * `read` must point to a valid object. `buf` must not be null.
 */
size_t libimage_poll(libimage_read_wrap *read, char *buf, size_t len);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif
