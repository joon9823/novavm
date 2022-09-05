//go:build darwin && !sys_kernelvm

package api

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lkernelvm
import "C"
