//go:build linux && muslc && !sys_kernelvm

package api

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lkernelvm_muslc
import "C"
