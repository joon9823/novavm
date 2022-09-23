/* (c) 2022 KernelLabs. Licensed under Apache-2.0 */

#ifndef __LIBNOVAVM__
#define __LIBNOVAVM__

/* Generated with cbindgen:0.24.3 */

/* Warning, this file is autogenerated by cbindgen. Don't modify this manually. */

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>


enum ErrnoValue {
  ErrnoValue_Success = 0,
  ErrnoValue_Other = 1,
  ErrnoValue_OutOfGas = 2,
};
typedef int32_t ErrnoValue;

/**
 * This enum gives names to the status codes returned from Go callbacks to Rust.
 * The Go code will return one of these variants when returning.
 *
 * 0 means no error, all the other cases are some sort of error.
 *
 */
enum GoError {
  GoError_None = 0,
  /**
   * Go panicked for an unexpected reason.
   */
  GoError_Panic = 1,
  /**
   * Go received a bad argument from Rust
   */
  GoError_BadArgument = 2,
  /**
   * Ran out of gas while using the SDK (e.g. storage). This can come from the Cosmos SDK gas meter
   * (https://github.com/cosmos/cosmos-sdk/blob/v0.45.4/store/types/gas.go#L29-L32).
   */
  GoError_OutOfGas = 3,
  /**
   * Error while trying to serialize data in Go code (typically json.Marshal)
   */
  GoError_CannotSerialize = 4,
  /**
   * An error happened during normal operation of a Go callback, which should be fed back to the contract
   */
  GoError_User = 5,
  /**
   * Unimplemented
   */
  GoError_Unimplemented = 6,
  /**
   * An error type that should never be created by us. It only serves as a fallback for the i32 to GoError conversion.
   */
  GoError_Other = -1,
};
typedef int32_t GoError;

/**
 * An optional Vector type that requires explicit creation and destruction
 * and can be sent via FFI.
 * It can be created from `Option<Vec<u8>>` and be converted into `Option<Vec<u8>>`.
 *
 * This type is always created in Rust and always dropped in Rust.
 * If Go code want to create it, it must instruct Rust to do so via the
 * [`new_unmanaged_vector`] FFI export. If Go code wants to consume its data,
 * it must create a copy and instruct Rust to destroy it via the
 * [`destroy_unmanaged_vector`] FFI export.
 *
 * An UnmanagedVector is immutable.
 *
 * ## Ownership
 *
 * Ownership is the right and the obligation to destroy an `UnmanagedVector`
 * exactly once. Both Rust and Go can create an `UnmanagedVector`, which gives
 * then ownership. Sometimes it is necessary to transfer ownership.
 *
 * ### Transfer ownership from Rust to Go
 *
 * When an `UnmanagedVector` was created in Rust using [`UnmanagedVector::new`], [`UnmanagedVector::default`]
 * or [`new_unmanaged_vector`], it can be passted to Go as a return value (see e.g. [load_wasm][crate::load_wasm]).
 * Rust then has no chance to destroy the vector anymore, so ownership is transferred to Go.
 * In Go, the data has to be copied to a garbage collected `[]byte`. Then the vector must be destroyed
 * using [`destroy_unmanaged_vector`].
 *
 * ### Transfer ownership from Go to Rust
 *
 * When Rust code calls into Go (using the vtable methods), return data or error messages must be created
 * in Go. This is done by calling [`new_unmanaged_vector`] from Go, which copies data into a newly created
 * `UnmanagedVector`. Since Go created it, it owns it. The ownership is then passed to Rust via the
 * mutable return value pointers. On the Rust side, the vector is destroyed using [`UnmanagedVector::consume`].
 *
 * ## Examples
 *
 * Transferring ownership from Rust to Go using return values of FFI calls:
 *
 * ```
 * # use crate::{cache_t, ByteSliceView, UnmanagedVector};
 * #[no_mangle]
 * pub extern "C" fn save_wasm_to_cache(
 *     cache: *mut cache_t,
 *     wasm: ByteSliceView,
 *     error_msg: Option<&mut UnmanagedVector>,
 * ) -> UnmanagedVector {
 *     # let checksum: Vec<u8> = Default::default();
 *     // some operation producing a `let checksum: Vec<u8>`
 *
 *     UnmanagedVector::new(Some(checksum)) // this unmanaged vector is owned by the caller
 * }
 * ```
 *
 * Transferring ownership from Go to Rust using return value pointers:
 *
 * ```rust
 * # use novavm::BackendResult;
 * # use crate::{Db, GoError, U8SliceView, UnmanagedVector};
 * fn db_read(db: &Db, key: &[u8]) -> BackendResult<Option<Vec<u8>>> {
 *
 *     // Create a None vector in order to reserve memory for the result
 *     let mut output = UnmanagedVector::default();
 *
 *     // …
 *     # let mut error_msg = UnmanagedVector::default();
 *
 *     let go_error: GoError = (db.vtable.read_db)(
 *         db.state,
 *         U8SliceView::new(Some(key)),
 *         // Go will create a new UnmanagedVector and override this address
 *         &mut output as *mut UnmanagedVector,
 *         &mut error_msg as *mut UnmanagedVector,
 *     )
 *     .into();
 *
 *     // We now own the new UnmanagedVector written to the pointer and must destroy it
 *     let value = output.consume();
 *
 *     Ok(value)
 * }
 * ```
 *
 *
 * If you want to mutate data, you need to comsume the vector and create a new one:
 *
 * ```rust
 * # use crate::{UnmanagedVector};
 * # let input = UnmanagedVector::new(Some(vec![0xAA]));
 * let mut mutable: Vec<u8> = input.consume().unwrap_or_default();
 * assert_eq!(mutable, vec![0xAA]);
 *
 * // `input` is now gone and we cam do everything we want to `mutable`,
 * // including operations that reallocate the underylying data.
 *
 * mutable.push(0xBB);
 * mutable.push(0xCC);
 *
 * assert_eq!(mutable, vec![0xAA, 0xBB, 0xCC]);
 *
 * let output = UnmanagedVector::new(Some(mutable));
 *
 * // `output` is ready to be passed around
 * ```
 */
typedef struct {
  /**
   * True if and only if this is None. If this is true, the other fields must be ignored.
   */
  bool is_none;
  uint8_t *ptr;
  size_t len;
  size_t cap;
} UnmanagedVector;

/**
 * A view into an externally owned byte slice (Go `[]byte`).
 * Use this for the current call only. A view cannot be copied for safety reasons.
 * If you need a copy, use [`ByteSliceView::to_owned`].
 *
 * Go's nil value is fully supported, such that we can differentiate between nil and an empty slice.
 */
typedef struct {
  /**
   * True if and only if the byte slice is nil in Go. If this is true, the other fields must be ignored.
   */
  bool is_nil;
  const uint8_t *ptr;
  size_t len;
} ByteSliceView;

typedef struct {
  uint8_t _private[0];
} db_t;

/**
 * A view into a `Option<&[u8]>`, created and maintained by Rust.
 *
 * This can be copied into a []byte in Go.
 */
typedef struct {
  /**
   * True if and only if this is None. If this is true, the other fields must be ignored.
   */
  bool is_none;
  const uint8_t *ptr;
  size_t len;
} U8SliceView;

typedef struct {
  int32_t (*read_db)(db_t*, U8SliceView, UnmanagedVector*, UnmanagedVector*);
  int32_t (*write_db)(db_t*, U8SliceView, U8SliceView, UnmanagedVector*);
  int32_t (*remove_db)(db_t*, U8SliceView, UnmanagedVector*);
} Db_vtable;

typedef struct {
  db_t *state;
  Db_vtable vtable;
} Db;

typedef struct {
  uint8_t _private[0];
} api_t;

typedef struct {
  int32_t (*bank_transfer)(const api_t*, U8SliceView, U8SliceView, U8SliceView, UnmanagedVector*);
} GoApi_vtable;

typedef struct {
  const api_t *state;
  GoApi_vtable vtable;
} GoApi;

typedef struct {
  uint8_t _private[0];
} querier_t;

typedef struct {
  int32_t (*query_external)(const querier_t*, U8SliceView, UnmanagedVector*, UnmanagedVector*);
} Querier_vtable;

typedef struct {
  const querier_t *state;
  Querier_vtable vtable;
} GoQuerier;

UnmanagedVector decode_module_bytes(UnmanagedVector *errmsg, ByteSliceView module_bytes);

UnmanagedVector decode_move_resource(Db db,
                                     UnmanagedVector *errmsg,
                                     ByteSliceView struct_tag,
                                     ByteSliceView resource_bytes);

UnmanagedVector decode_script_bytes(UnmanagedVector *errmsg, ByteSliceView script_bytes);

void destroy_unmanaged_vector(UnmanagedVector v);

UnmanagedVector execute_contract(Db db,
                                 GoApi _api,
                                 GoQuerier _querier,
                                 bool _is_verbose,
                                 uint64_t gas_limit,
                                 UnmanagedVector *errmsg,
                                 ByteSliceView session_id,
                                 ByteSliceView sender,
                                 ByteSliceView message);

UnmanagedVector execute_script(Db db,
                               GoApi _api,
                               GoQuerier _querier,
                               bool _is_verbose,
                               uint64_t gas_limit,
                               UnmanagedVector *errmsg,
                               ByteSliceView session_id,
                               ByteSliceView sender,
                               ByteSliceView message);

UnmanagedVector initialize(Db db,
                           bool _is_verbose,
                           UnmanagedVector *errmsg,
                           ByteSliceView module_bundle);

UnmanagedVector new_unmanaged_vector(bool nil, const uint8_t *ptr, size_t length);

/**
 * exported function to publish a module
 */
UnmanagedVector publish_module(Db db,
                               bool _is_verbose,
                               uint64_t gas_limit,
                               UnmanagedVector *errmsg,
                               ByteSliceView sender,
                               ByteSliceView module_bytes);

UnmanagedVector query_contract(Db db,
                               GoApi _api,
                               GoQuerier _querier,
                               bool _is_verbose,
                               uint64_t gas_limit,
                               UnmanagedVector *errmsg,
                               ByteSliceView message);

/**
 * Returns a version number of this library as a C string.
 *
 * The string is owned by libnovaproc and must not be mutated or destroyed by the caller.
 */
const char *version_str(void);

#endif /* __LIBNOVAVM__ */
