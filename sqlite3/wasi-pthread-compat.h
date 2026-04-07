/*
** Name:        wasi-pthread-compat.h
** Purpose:     WASI pthread compatibility declarations
** Author:      Adapted for WASI
** Created:     2026-04-07
** License:     MIT
*/

#ifndef WASI_PTHREAD_COMPAT_H
#define WASI_PTHREAD_COMPAT_H

#if defined(_WASI_EMULATED_PTHREAD) && (defined(__wasi__) || defined(__WASI__) || defined(__WASM__))

/* pthread_exit is missing from WASI's emulated pthread header
** Provide the declaration here. Implementation is in wasi-helpers.c
*/
#ifdef __cplusplus
extern "C" {
#endif

void pthread_exit(void *retval);

#ifdef __cplusplus
}
#endif

#endif /* _WASI_EMULATED_PTHREAD */

#endif /* WASI_PTHREAD_COMPAT_H */
