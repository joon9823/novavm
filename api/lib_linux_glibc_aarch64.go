//go:build linux && !muslc && arm64 && !sys_kernelvm

package api

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lkernelvm.aarch64
import "C"
