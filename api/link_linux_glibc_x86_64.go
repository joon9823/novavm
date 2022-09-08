//go:build linux && !muslc && amd64 && !sys_kernelproc

package api

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lkernelproc.x86_64
import "C"
