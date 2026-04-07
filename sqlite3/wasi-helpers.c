/*
** Name:        wasi-helpers.c
** Purpose:     WASI helper functions for SQLite3MultipleCiphers
** Author:      Adapted for WASI
** Created:     2026-04-07
** License:     MIT
*/

#if defined(__wasi__) || defined(__WASI__) || defined(__WASM__)

#include <stddef.h>
#include <stdint.h>
#include <errno.h>
#include <wasi/api.h>

/*
** getentropy() for WASI
** This is used by SQLite3MultipleCiphers' ChaCha20Poly1305 cipher
** when compiled with __WASM__ defined
**
** Returns: 0 on success, -1 on error
*/
int getentropy(void *buf, size_t buflen) {
    if (buf == NULL || buflen == 0) {
        errno = EINVAL;
        return -1;
    }

    /* WASI limits getentropy to 256 bytes per call (like OpenBSD) */
    if (buflen > 256) {
        errno = EIO;
        return -1;
    }

    /* Use WASI's cryptographically secure random number generator */
    __wasi_errno_t result = __wasi_random_get((uint8_t*)buf, buflen);

    if (result != __WASI_ERRNO_SUCCESS) {
        /* WASI random_get failed */
        errno = EIO;
        return -1;
    }

    /* Success */
    return 0;
}

/*
** pthread_exit stub for WASI
** argon2 calls this, but with emulated pthread it may not be available
*/
void pthread_exit(void *retval) {
    /* In single-threaded WASI, just return */
    (void)retval;
    return;
}

#endif /* __wasi__ || __WASI__ || __WASM__ */
