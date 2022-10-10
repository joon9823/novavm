package api

/*
#include "bindings.h"
#include <stdio.h>

// imports (db)
GoError cSet(db_t *ptr, U8SliceView key, U8SliceView val, UnmanagedVector *errOut);
GoError cGet(db_t *ptr, U8SliceView key, UnmanagedVector *val, UnmanagedVector *errOut);
GoError cDelete(db_t *ptr, U8SliceView key, UnmanagedVector *errOut);
// imports (api)
GoError cGetBlockInfo(api_t *ptr, uint64_t *height, uint64_t *timestamp, UnmanagedVector *errOut);

// Gateway functions (db)
GoError cGet_cgo(db_t *ptr, U8SliceView key, UnmanagedVector *val, UnmanagedVector *errOut) {
	return cGet(ptr, key, val, errOut);
}
GoError cSet_cgo(db_t *ptr, U8SliceView key, U8SliceView val, UnmanagedVector *errOut) {
	return cSet(ptr, key, val, errOut);
}
GoError cDelete_cgo(db_t *ptr, U8SliceView key, UnmanagedVector *errOut) {
	return cDelete(ptr, key, errOut);
}

// Gateway functions (api)
GoError cGetBlockInfo_cgo(api_t *ptr, uint64_t *height, uint64_t *timestamp, UnmanagedVector *errOut) {
    return cGetBlockInfo(ptr, height, timestamp, errOut);
}
*/
import "C"

// We need these gateway functions to allow calling back to a go function from the c code.
// At least I didn't discover a cleaner way.
// Also, this needs to be in a different file than `callbacks.go`, as we cannot create functions
// in the same file that has //export directives. Only import header types
