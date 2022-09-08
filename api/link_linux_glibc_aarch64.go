//go:build linux && !muslc && arm64 && !sys_kernelproc

package api

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lkernelproc.aarch64
import "C"
