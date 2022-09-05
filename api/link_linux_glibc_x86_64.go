//go:build linux && !muslc && amd64 && !sys_kernelvm

package api

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lkernelvm.x86_64
import "C"
