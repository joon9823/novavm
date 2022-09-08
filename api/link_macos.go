//go:build darwin && !sys_kernelproc

package api

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lkernelproc
import "C"
