//go:build linux && muslc && !sys_kernelproc

package api

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lkernelproc_muslc
import "C"
